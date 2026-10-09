//! Run a program in a pseudo-terminal: it gets a controlling terminal of a given size (so crossterm's raw
//! mode, size queries and SIGWINCH all work as in a real terminal), its output is fed into a [`Term`], and
//! keys are written to it as a user would type them.

use super::term::Term;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Pty {
    pub child: Child,
    master: File,
    pub term: Arc<Mutex<Term>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

fn winsize(cols: u16, rows: u16) -> libc::winsize {
    libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}

impl Pty {
    pub fn spawn(
        program: &Path,
        args: &[String],
        envs: &[(String, String)],
        cwd: &Path,
        cols: u16,
        rows: u16,
    ) -> std::io::Result<Pty> {
        let mut master_fd: libc::c_int = -1;
        let mut slave_fd: libc::c_int = -1;
        let mut ws = winsize(cols, rows);
        // SAFETY: valid out-pointers; no name buffer; the winsize is a valid struct.
        let rc = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut ws,
            )
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: openpty returned two fresh descriptors we now own.
        let (master, slave) = unsafe {
            (
                OwnedFd::from_raw_fd(master_fd),
                OwnedFd::from_raw_fd(slave_fd),
            )
        };
        let slave_file = File::from(slave);
        let mut cmd = Command::new(program);
        cmd.args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .stdin(Stdio::from(slave_file.try_clone()?))
            .stdout(Stdio::from(slave_file.try_clone()?))
            .stderr(Stdio::from(slave_file));
        // SAFETY: only async-signal-safe calls between fork and exec: setsid and ioctl.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = cmd.spawn()?;
        let master = File::from(master);
        let term = Arc::new(Mutex::new(Term::new(cols as usize, rows as usize)));
        let mut rd = master.try_clone()?;
        let mut wr = master.try_clone()?;
        let t2 = term.clone();
        let reader = std::thread::spawn(move || {
            let mut buf = [0u8; 65536];
            loop {
                match rd.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let replies = {
                            let mut t = t2.lock().unwrap_or_else(|e| e.into_inner());
                            t.feed(&buf[..n]);
                            std::mem::take(&mut t.replies)
                        };
                        if !replies.is_empty() {
                            let _ = wr.write_all(&replies);
                        }
                    }
                }
            }
        });
        Ok(Pty {
            child,
            master,
            term,
            reader: Some(reader),
        })
    }

    /// Type bytes as the user.
    pub fn write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.master.write_all(bytes)?;
        self.master.flush()
    }

    /// Resize the terminal (the program gets SIGWINCH from the kernel).
    pub fn resize(&mut self, cols: u16, rows: u16) -> std::io::Result<()> {
        let ws = winsize(cols, rows);
        self.term
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .resize(cols as usize, rows as usize);
        // SAFETY: a valid descriptor and a valid winsize.
        if unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ as _, &ws) } < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Term {
        self.term.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Bytes received so far.
    pub fn bytes(&self) -> u64 {
        self.term.lock().unwrap_or_else(|e| e.into_inner()).bytes
    }

    /// Wait until no output has arrived for `quiet` (the frame is complete), at most `limit`.
    pub fn settle(&self, quiet: Duration, limit: Duration) -> bool {
        let start = Instant::now();
        let mut last = self.bytes();
        let mut since = Instant::now();
        while start.elapsed() < limit {
            std::thread::sleep(Duration::from_millis(5));
            let b = self.bytes();
            if b != last {
                last = b;
                since = Instant::now();
            } else if since.elapsed() >= quiet {
                return true;
            }
        }
        false
    }

    /// Wait until the screen satisfies `pred`, at most `limit`.
    pub fn wait_for(&self, limit: Duration, pred: impl Fn(&Term) -> bool) -> bool {
        let start = Instant::now();
        while start.elapsed() < limit {
            if pred(&self.term.lock().unwrap_or_else(|e| e.into_inner())) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    }

    pub fn kill(&mut self, sig: libc::c_int) {
        // SAFETY: signalling our own child by pid.
        unsafe {
            libc::kill(self.child.id() as libc::pid_t, sig);
        }
    }

    /// Wait for the child to exit (at most `limit`); its exit status.
    pub fn wait_exit(&mut self, limit: Duration) -> Option<std::process::ExitStatus> {
        let start = Instant::now();
        while start.elapsed() < limit {
            if let Ok(Some(st)) = self.child.try_wait() {
                return Some(st);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            self.kill(libc::SIGKILL);
            let _ = self.child.wait();
        }
        if let Some(r) = self.reader.take() {
            // the reader ends once the slave side is closed by the child's exit
            let _ = r.join();
        }
    }
}
