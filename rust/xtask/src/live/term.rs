//! A small VT100/xterm screen emulator, enough for what crossterm/ratatui write: cursor addressing, SGR
//! colours and attributes, erase, the alternate screen, DEC private modes (ignored), OSC (ignored). It turns
//! the live TUI's PTY output into a cell grid the acceptance suite compares with `ui state` and `ui screen`.
//! It also answers the terminal queries crossterm sends at start-up (cursor position, primary device
//! attributes), so the TUI never waits on a timeout.

use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Color {
    #[default]
    Default,
    Idx(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The grapheme drawn here ("" for the second column of a wide character).
    pub ch: String,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: " ".into(),
            style: Style::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Term {
    pub cols: usize,
    pub rows: usize,
    grid: Vec<Vec<Cell>>,
    alt: Option<Vec<Vec<Cell>>>,
    pub x: usize,
    pub y: usize,
    saved: (usize, usize),
    style: Style,
    state: State,
    params: String,
    utf8: Vec<u8>,
    /// Bytes the terminal must send back (query answers).
    pub replies: Vec<u8>,
    /// Number of synchronized-update frames completed (`CSI ? 2026 l`).
    pub frames: u64,
    pub bytes: u64,
    /// The text of the last completed frames, with their numbers (the harness asks "did the terminal show
    /// this frame?", not "does it show it at this instant": a read itself may move the header or Activity).
    pub history: std::collections::VecDeque<(u64, String)>,
}

/// How many completed frames the history keeps.
const HISTORY: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Ground,
    Esc,
    Csi,
    Osc,
    OscEsc,
    Charset,
}

impl Term {
    pub fn new(cols: usize, rows: usize) -> Term {
        Term {
            cols,
            rows,
            grid: vec![vec![Cell::default(); cols]; rows],
            alt: None,
            x: 0,
            y: 0,
            saved: (0, 0),
            style: Style::default(),
            state: State::Ground,
            params: String::new(),
            utf8: vec![],
            replies: vec![],
            frames: 0,
            bytes: 0,
            history: std::collections::VecDeque::new(),
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        for row in &mut self.grid {
            row.resize(cols, Cell::default());
        }
        self.grid.resize(rows, vec![Cell::default(); cols]);
        self.cols = cols;
        self.rows = rows;
        self.x = self.x.min(cols.saturating_sub(1));
        self.y = self.y.min(rows.saturating_sub(1));
    }

    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.grid[y][x]
    }

    /// Row `y` as text (wide characters once, trailing spaces kept).
    pub fn line(&self, y: usize) -> String {
        self.grid[y].iter().map(|c| c.ch.as_str()).collect()
    }

    pub fn text(&self) -> String {
        (0..self.rows)
            .map(|y| self.line(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.bytes += data.len() as u64;
        for &b in data {
            self.byte(b);
        }
    }

    fn byte(&mut self, b: u8) {
        match self.state {
            State::Ground => self.ground(b),
            State::Esc => self.esc(b),
            State::Csi => {
                if (0x40..=0x7e).contains(&b) {
                    let p = std::mem::take(&mut self.params);
                    self.state = State::Ground;
                    self.csi(&p, b as char);
                } else {
                    self.params.push(b as char);
                }
            }
            State::Osc => match b {
                0x07 => self.state = State::Ground,
                0x1b => self.state = State::OscEsc,
                _ => {}
            },
            State::OscEsc => {
                self.state = if b == b'\\' {
                    State::Ground
                } else {
                    State::Osc
                }
            }
            State::Charset => self.state = State::Ground,
        }
    }

    fn ground(&mut self, b: u8) {
        if !self.utf8.is_empty() || b >= 0x80 {
            self.utf8.push(b);
            match std::str::from_utf8(&self.utf8) {
                Ok(s) => {
                    let s = s.to_string();
                    self.utf8.clear();
                    for ch in s.chars() {
                        self.print(ch);
                    }
                }
                Err(e) if e.error_len().is_some() => self.utf8.clear(),
                Err(_) => {}
            }
            return;
        }
        match b {
            0x1b => self.state = State::Esc,
            b'\r' => self.x = 0,
            b'\n' | 0x0b | 0x0c => self.linefeed(),
            0x08 => self.x = self.x.saturating_sub(1),
            b'\t' => self.x = ((self.x / 8 + 1) * 8).min(self.cols - 1),
            0x07 | 0x00 => {}
            _ if b < 0x20 => {}
            _ => self.print(b as char),
        }
    }

    fn linefeed(&mut self) {
        if self.y + 1 >= self.rows {
            self.grid.remove(0);
            self.grid.push(vec![Cell::default(); self.cols]);
        } else {
            self.y += 1;
        }
    }

    fn print(&mut self, ch: char) {
        let w = ch.width().unwrap_or(0);
        if w == 0 {
            // combining mark: attach to the previous cell
            if self.x > 0 {
                self.grid[self.y][self.x - 1].ch.push(ch);
            }
            return;
        }
        if self.x + w > self.cols {
            self.x = 0;
            self.linefeed();
        }
        let style = self.style.clone();
        self.grid[self.y][self.x] = Cell {
            ch: ch.to_string(),
            style: style.clone(),
        };
        if w == 2 && self.x + 1 < self.cols {
            self.grid[self.y][self.x + 1] = Cell {
                ch: String::new(),
                style,
            };
        }
        self.x = (self.x + w).min(self.cols);
        if self.x >= self.cols {
            // pending wrap: stay on the last column until the next print
            self.x = self.cols;
        }
    }

    fn esc(&mut self, b: u8) {
        self.state = State::Ground;
        match b {
            b'[' => {
                self.state = State::Csi;
                self.params.clear();
            }
            b']' => self.state = State::Osc,
            b'(' | b')' | b'*' | b'+' => self.state = State::Charset,
            b'7' => self.saved = (self.x, self.y),
            b'8' => (self.x, self.y) = self.saved,
            b'c' => *self = Term::new(self.cols, self.rows),
            _ => {}
        }
    }

    fn nums(p: &str) -> Vec<u32> {
        p.split([';', ':'])
            .map(|s| s.parse().unwrap_or(0))
            .collect()
    }

    fn csi(&mut self, p: &str, f: char) {
        let private = p.starts_with(['?', '>', '<', '=']);
        let n = Self::nums(p.trim_start_matches(['?', '>', '<', '=']));
        let a = |i: usize, d: u32| -> usize {
            let v = n.get(i).copied().unwrap_or(0);
            (if v == 0 { d } else { v }) as usize
        };
        match (private, f) {
            (false, 'H' | 'f') => {
                self.y = (a(0, 1) - 1).min(self.rows - 1);
                self.x = (a(1, 1) - 1).min(self.cols - 1);
            }
            (false, 'A') => self.y = self.y.saturating_sub(a(0, 1)),
            (false, 'B') => self.y = (self.y + a(0, 1)).min(self.rows - 1),
            (false, 'C') => self.x = (self.x + a(0, 1)).min(self.cols - 1),
            (false, 'D') => self.x = self.x.saturating_sub(a(0, 1)),
            (false, 'G') => self.x = (a(0, 1) - 1).min(self.cols - 1),
            (false, 'd') => self.y = (a(0, 1) - 1).min(self.rows - 1),
            (false, 'E') => {
                self.y = (self.y + a(0, 1)).min(self.rows - 1);
                self.x = 0;
            }
            (false, 'F') => {
                self.y = self.y.saturating_sub(a(0, 1));
                self.x = 0;
            }
            (false, 'J') => self.erase_display(n.first().copied().unwrap_or(0)),
            (false, 'K') => self.erase_line(n.first().copied().unwrap_or(0)),
            (false, 'X') => {
                let x = self.x.min(self.cols);
                for i in x..(x + a(0, 1)).min(self.cols) {
                    self.grid[self.y][i] = self.blank();
                }
            }
            (false, 'm') => self.sgr(&n),
            (false, 'n') if n.first() == Some(&6) => {
                let r = format!("\x1b[{};{}R", self.y + 1, self.x.min(self.cols - 1) + 1);
                self.replies.extend_from_slice(r.as_bytes());
            }
            (false, 'c') => self.replies.extend_from_slice(b"\x1b[?62;22c"),
            (false, 's') => self.saved = (self.x, self.y),
            (false, 'u') => (self.x, self.y) = self.saved,
            (true, 'h') | (true, 'l') => {
                let on = f == 'h';
                for m in &n {
                    match m {
                        1049 | 1047 | 47 => self.alt_screen(on),
                        2026 if !on => {
                            self.frames += 1;
                            let t = self.text();
                            if self.history.back().is_none_or(|(_, last)| *last != t) {
                                self.history.push_back((self.frames, t));
                                if self.history.len() > HISTORY {
                                    self.history.pop_front();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn blank(&self) -> Cell {
        Cell {
            ch: " ".into(),
            style: Style {
                bg: self.style.bg,
                ..Style::default()
            },
        }
    }

    fn erase_display(&mut self, mode: u32) {
        let blank = self.blank();
        match mode {
            0 => {
                self.erase_line(0);
                for y in self.y + 1..self.rows {
                    self.grid[y] = vec![blank.clone(); self.cols];
                }
            }
            1 => {
                self.erase_line(1);
                for y in 0..self.y {
                    self.grid[y] = vec![blank.clone(); self.cols];
                }
            }
            _ => self.grid = vec![vec![blank; self.cols]; self.rows],
        }
    }

    fn erase_line(&mut self, mode: u32) {
        let blank = self.blank();
        let x = self.x.min(self.cols);
        let (from, to) = match mode {
            0 => (x, self.cols),
            1 => (0, (x + 1).min(self.cols)),
            _ => (0, self.cols),
        };
        for i in from..to {
            self.grid[self.y][i] = blank.clone();
        }
    }

    fn alt_screen(&mut self, on: bool) {
        if on && self.alt.is_none() {
            let main = std::mem::replace(
                &mut self.grid,
                vec![vec![Cell::default(); self.cols]; self.rows],
            );
            self.alt = Some(main);
        } else if !on && let Some(main) = self.alt.take() {
            self.grid = main;
        }
    }

    fn sgr(&mut self, n: &[u32]) {
        let mut i = 0;
        let n = if n.is_empty() { &[0][..] } else { n };
        while i < n.len() {
            match n[i] {
                0 => self.style = Style::default(),
                1 => self.style.bold = true,
                2 => self.style.dim = true,
                3 => self.style.italic = true,
                4 => self.style.underline = true,
                7 => self.style.reverse = true,
                22 => {
                    self.style.bold = false;
                    self.style.dim = false;
                }
                23 => self.style.italic = false,
                24 => self.style.underline = false,
                27 => self.style.reverse = false,
                30..=37 => self.style.fg = Color::Idx((n[i] - 30) as u8),
                39 => self.style.fg = Color::Default,
                40..=47 => self.style.bg = Color::Idx((n[i] - 40) as u8),
                49 => self.style.bg = Color::Default,
                90..=97 => self.style.fg = Color::Idx((n[i] - 90 + 8) as u8),
                100..=107 => self.style.bg = Color::Idx((n[i] - 100 + 8) as u8),
                intro @ (38 | 48) => {
                    let c = match n.get(i + 1) {
                        Some(5) => {
                            let c = Color::Idx(n.get(i + 2).copied().unwrap_or(0) as u8);
                            i += 2;
                            c
                        }
                        Some(2) => {
                            let g = |k: usize| n.get(i + k).copied().unwrap_or(0) as u8;
                            let c = Color::Rgb(g(2), g(3), g(4));
                            i += 4;
                            c
                        }
                        _ => Color::Default,
                    };
                    if intro == 38 {
                        self.style.fg = c;
                    } else {
                        self.style.bg = c;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_drawing() {
        let mut t = Term::new(10, 3);
        t.feed(b"\x1b[?1049h\x1b[2J\x1b[1;1Hab\x1b[2;3H\x1b[38;5;2;48;2;1;2;3mcd\x1b[0m\x1b[6n");
        assert_eq!(t.line(0), "ab        ");
        assert_eq!(t.line(1), "  cd      ");
        assert_eq!(t.cell(2, 1).style.fg, Color::Idx(2));
        assert_eq!(t.cell(3, 1).style.bg, Color::Rgb(1, 2, 3));
        assert_eq!(t.replies, b"\x1b[2;5R");
        t.feed("\x1b[3;1H大x".as_bytes());
        assert_eq!(t.line(2), "大x       ");
    }
}
