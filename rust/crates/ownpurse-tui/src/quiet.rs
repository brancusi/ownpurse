//! The terminal backend, quieter (spec §8, V13). ratatui shows and
//! moves the cursor on every draw even when no cell changed; this wrapper
//! - writes nothing at all for a frame with no changed cells (idle writes 0 bytes),
//! - skips cursor show/hide/move when the cursor is already in that state,
//! - wraps a frame that drew cells in synchronized output (`CSI ?2026h` … `l`) when the terminal has it,
//!   so WezTerm, Ghostty, iTerm2 and kitty show each frame whole.
//!
//! Generic over the writer so tests count bytes (`Counting`).

use crossterm::{queue, terminal};
use ratatui::backend::{Backend, ClearType, CrosstermBackend, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use std::io::Write;

pub struct Quiet<W: Write> {
    inner: CrosstermBackend<W>,
    /// The terminal reports synchronized output (DECRQM 2026); else frames go out as one buffered write.
    sync: bool,
    drew: bool,
    in_sync: bool,
    cursor: Option<Position>,
    visible: Option<bool>,
    /// A fixed size for tests (the real backend asks the terminal).
    fixed: Option<Size>,
}

impl<W: Write> Quiet<W> {
    pub fn new(out: W, sync: bool) -> Quiet<W> {
        Quiet {
            inner: CrosstermBackend::new(out),
            sync,
            drew: false,
            in_sync: false,
            cursor: None,
            visible: None,
            fixed: None,
        }
    }

    /// A backend that never asks a terminal for its size (tests, benches).
    pub fn fixed(out: W, sync: bool, width: u16, height: u16) -> Quiet<W> {
        let mut q = Quiet::new(out, sync);
        q.fixed = Some(Size { width, height });
        q
    }
}

impl<W: Write> Backend for Quiet<W> {
    type Error = std::io::Error;

    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let mut content = content.peekable();
        if content.peek().is_none() {
            return Ok(());
        }
        if self.sync && !self.in_sync {
            queue!(self.inner, terminal::BeginSynchronizedUpdate)?;
            self.in_sync = true;
        }
        self.drew = true;
        self.cursor = None;
        self.inner.draw(content)
    }

    fn append_lines(&mut self, n: u16) -> std::io::Result<()> {
        self.cursor = None;
        self.inner.append_lines(n)
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        if self.visible == Some(false) {
            return Ok(());
        }
        self.visible = Some(false);
        self.inner.hide_cursor()
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        if self.visible == Some(true) {
            return Ok(());
        }
        self.visible = Some(true);
        self.inner.show_cursor()
    }

    fn get_cursor_position(&mut self) -> std::io::Result<Position> {
        match self.cursor {
            Some(p) => Ok(p),
            None if self.fixed.is_some() => Ok(Position::ORIGIN),
            None => self.inner.get_cursor_position(),
        }
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> std::io::Result<()> {
        let p = position.into();
        if self.cursor == Some(p) && !self.drew {
            return Ok(());
        }
        self.cursor = Some(p);
        if !self.drew && self.visible != Some(true) {
            // Nothing drawn and the cursor is hidden: moving it is invisible churn.
            return Ok(());
        }
        self.inner.set_cursor_position(p)
    }

    fn clear(&mut self) -> std::io::Result<()> {
        self.cursor = None;
        self.drew = true;
        self.inner.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> std::io::Result<()> {
        self.cursor = None;
        self.drew = true;
        self.inner.clear_region(clear_type)
    }

    fn size(&self) -> std::io::Result<Size> {
        match self.fixed {
            Some(s) => Ok(s),
            None => self.inner.size(),
        }
    }

    fn window_size(&mut self) -> std::io::Result<WindowSize> {
        match self.fixed {
            Some(s) => Ok(WindowSize {
                columns_rows: s,
                pixels: Size::ZERO,
            }),
            None => self.inner.window_size(),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.in_sync {
            queue!(self.inner, terminal::EndSynchronizedUpdate)?;
            self.in_sync = false;
        }
        let drew = std::mem::take(&mut self.drew);
        if !drew {
            return Ok(());
        }
        Backend::flush(&mut self.inner)
    }
}

/// A writer that counts bytes (V13: idle writes 0 bytes). Clones share the count.
#[derive(Clone, Default, Debug)]
pub struct Counting(pub std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Counting {
    pub fn bytes(&self) -> usize {
        self.0.lock().map(|d| d.len()).unwrap_or(0)
    }
    pub fn data(&self) -> Vec<u8> {
        self.0.lock().map(|d| d.clone()).unwrap_or_default()
    }
}

impl Write for Counting {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut d) = self.0.lock() {
            d.extend_from_slice(buf);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
