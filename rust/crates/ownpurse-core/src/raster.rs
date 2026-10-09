//! `ownpurse ui screen --format png`: the TUI's frame as a PNG. The TUI sends its cell grid (placed runs
//! with resolved colours, `render::grid`); this rasterises it on a fixed cell grid with an embedded
//! monospace font (Hack, MIT + Bitstream Vera licence, `crates/ownpurse-core/assets/`), falling back to the system's fonts for
//! characters Hack lacks (CJK, some symbols), then encodes an 8-bit RGB PNG. Integer arithmetic only.

use serde_json::Value;
use std::sync::OnceLock;

/// One cell in pixels, and the font size: Hack's advance at 16 px is just under 10 px; 20 px rows.
pub const CELL_W: u32 = 10;
pub const CELL_H: u32 = 20;
const PX: f32 = 16.0;
/// Baseline from the top of a cell.
const BASELINE: i32 = 15;

const HACK: &[u8] = include_bytes!("../assets/Hack-Regular.ttf");

/// Fonts tried, in order, for a character Hack doesn't have. Missing files are skipped.
const FALLBACKS: [(&str, u32); 9] = [
    ("/System/Library/Fonts/Menlo.ttc", 0),
    ("/System/Library/Fonts/Apple Symbols.ttf", 0),
    ("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", 0),
    ("/System/Library/Fonts/Supplemental/Arial Unicode.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
    (
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        0,
    ),
];

fn font(bytes: &[u8], index: u32) -> Option<fontdue::Font> {
    fontdue::Font::from_bytes(
        bytes,
        fontdue::FontSettings {
            collection_index: index,
            ..fontdue::FontSettings::default()
        },
    )
    .ok()
}

fn hack() -> &'static fontdue::Font {
    static F: OnceLock<fontdue::Font> = OnceLock::new();
    F.get_or_init(|| font(HACK, 0).expect("the embedded Hack font parses"))
}

/// The fallbacks, each parsed on first need (a CJK collection is large; most frames never need one).
fn fallback(i: usize) -> Option<&'static fontdue::Font> {
    static F: [OnceLock<Option<fontdue::Font>>; FALLBACKS.len()] =
        [const { OnceLock::new() }; FALLBACKS.len()];
    F[i].get_or_init(|| {
        let (p, idx) = FALLBACKS[i];
        std::fs::read(p).ok().and_then(|b| font(&b, idx))
    })
    .as_ref()
}

/// The font that has `c`: Hack, else the first system fallback that does.
fn font_for(c: char) -> Option<&'static fontdue::Font> {
    let h = hack();
    if h.lookup_glyph_index(c) != 0 {
        return Some(h);
    }
    (0..FALLBACKS.len())
        .filter(|&i| std::path::Path::new(FALLBACKS[i].0).exists())
        .filter_map(fallback)
        .find(|f| f.lookup_glyph_index(c) != 0)
}

fn color(v: &Value) -> Option<(u8, u8, u8)> {
    let h = v.as_str()?.strip_prefix('#')?;
    let n = u32::from_str_radix(h, 16).ok()?;
    Some(((n >> 16) as u8, (n >> 8) as u8, n as u8))
}

/// `a` over `b` with coverage `cov` (0–255).
fn blend(a: (u8, u8, u8), b: (u8, u8, u8), cov: u32) -> (u8, u8, u8) {
    let mix = |x: u8, y: u8| ((u32::from(x) * cov + u32::from(y) * (255 - cov)) / 255) as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

struct Canvas {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

impl Canvas {
    fn get(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let i = ((y * self.w + x) * 3) as usize;
        (self.px[i], self.px[i + 1], self.px[i + 2])
    }
    fn put(&mut self, x: i32, y: i32, c: (u8, u8, u8), cov: u32) {
        if x < 0 || y < 0 || x as u32 >= self.w || y as u32 >= self.h || cov == 0 {
            return;
        }
        let (x, y) = (x as u32, y as u32);
        let out = if cov >= 255 {
            c
        } else {
            blend(c, self.get(x, y), cov)
        };
        let i = ((y * self.w + x) * 3) as usize;
        self.px[i] = out.0;
        self.px[i + 1] = out.1;
        self.px[i + 2] = out.2;
    }
    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, c: (u8, u8, u8)) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.put(xx as i32, yy as i32, c, 255);
            }
        }
    }
}

/// Box and block characters drawn as rectangles, so lines join across cells whatever the font's height.
fn draw_block(cv: &mut Canvas, ch: char, x: u32, y: u32, w: u32, fg: (u8, u8, u8)) -> bool {
    let mid_y = y + CELL_H / 2;
    let mid_x = x + w / 2;
    match ch {
        '─' => cv.rect(x, mid_y, w, 1, fg),
        '━' => cv.rect(x, mid_y - 1, w, 2, fg),
        '│' => cv.rect(mid_x, y, 1, CELL_H, fg),
        '▌' => cv.rect(x, y, w / 2, CELL_H, fg),
        '█' => cv.rect(x, y, w, CELL_H, fg),
        _ => return false,
    }
    true
}

fn draw_char(cv: &mut Canvas, ch: char, x: u32, y: u32, w: u32, fg: (u8, u8, u8), bold: bool) {
    if ch == ' ' || draw_block(cv, ch, x, y, w, fg) {
        return;
    }
    let Some(f) = font_for(ch) else { return };
    let (m, bitmap) = f.rasterize(ch, PX);
    let advance = m.advance_width as i32;
    // centre the glyph's advance in its cells (wide characters, narrow fallbacks)
    let dx = (w as i32 - advance).max(0) / 2;
    let x0 = x as i32 + dx + m.xmin;
    let y0 = y as i32 + BASELINE - m.ymin - m.height as i32;
    for gy in 0..m.height {
        for gx in 0..m.width {
            let cov = u32::from(bitmap[gy * m.width + gx]);
            cv.put(x0 + gx as i32, y0 + gy as i32, fg, cov);
            if bold {
                cv.put(x0 + gx as i32 + 1, y0 + gy as i32, fg, cov);
            }
        }
    }
}

/// The grid (`render::grid` JSON) as PNG bytes: `cols × CELL_W` by `rows × CELL_H` pixels.
pub fn png(grid: &Value) -> Result<Vec<u8>, String> {
    let cols = grid["cols"].as_u64().ok_or("grid without cols")? as u32;
    let rows = grid["rows"].as_u64().ok_or("grid without rows")? as u32;
    let page_bg = color(&grid["bg"]).unwrap_or((0, 0, 0));
    let mut cv = Canvas {
        w: cols * CELL_W,
        h: rows * CELL_H,
        px: Vec::new(),
    };
    cv.px = std::iter::repeat_n([page_bg.0, page_bg.1, page_bg.2], (cv.w * cv.h) as usize)
        .flatten()
        .collect();
    for (row, runs) in grid["lines"]
        .as_array()
        .ok_or("grid without lines")?
        .iter()
        .enumerate()
    {
        let y = row as u32 * CELL_H;
        for r in runs.as_array().into_iter().flatten() {
            let x = r["x"].as_u64().unwrap_or(0) as u32 * CELL_W;
            let cells = r["w"].as_u64().unwrap_or(1) as u32;
            let bg = color(&r["bg"]);
            if let Some(b) = bg {
                cv.rect(x, y, cells * CELL_W, CELL_H, b);
            }
            let under = bg.unwrap_or(page_bg);
            let mut fg = color(&r["fg"]).unwrap_or((255, 255, 255));
            if r["d"] == true {
                fg = blend(fg, under, 153); // dim: 60 %
            }
            let bold = r["b"] == true;
            let syms = r["s"].as_array().cloned().unwrap_or_default();
            let widths = r["sw"].as_array().cloned().unwrap_or_default();
            let mut cx = x;
            for (i, sym) in syms.iter().enumerate() {
                let w = widths.get(i).and_then(Value::as_u64).unwrap_or(1) as u32 * CELL_W;
                for ch in sym.as_str().unwrap_or("").chars() {
                    draw_char(&mut cv, ch, cx, y, w, fg, bold);
                }
                cx += w;
            }
            if r["u"] == true {
                cv.rect(x, y + BASELINE as u32 + 2, cells * CELL_W, 1, fg);
            }
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = ::png::Encoder::new(&mut out, cv.w, cv.h);
        enc.set_color(::png::ColorType::Rgb);
        enc.set_depth(::png::BitDepth::Eight);
        let mut w = enc.write_header().map_err(|e| e.to_string())?;
        w.write_image_data(&cv.px).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_grid_becomes_a_png_of_the_right_size() {
        let g = json!({
            "cols": 6, "rows": 2, "bg": "#141a1c", "fg": "#e2e8ea",
            "lines": [
                [{"x": 0, "w": 2, "t": "ab", "s": ["a", "b"], "sw": [1, 1], "fg": "#ff0000", "bg": null, "b": true, "d": false, "i": false, "u": true},
                 {"x": 2, "w": 2, "t": "東", "s": ["東"], "sw": [2], "fg": "#e2e8ea", "bg": "#2b353b", "b": false, "d": false, "i": false, "u": false}],
                [{"x": 0, "w": 6, "t": "──────", "s": ["─","─","─","─","─","─"], "sw": [1,1,1,1,1,1], "fg": "#94a2a9", "bg": null, "b": false, "d": true, "i": false, "u": false}]
            ]
        });
        let bytes = png(&g).unwrap();
        let dec = ::png::Decoder::new(std::io::Cursor::new(bytes));
        let mut r = dec.read_info().unwrap();
        let info = r.info();
        assert_eq!((info.width, info.height), (60, 40));
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        let frame = r.next_frame(&mut buf).unwrap();
        let px = |x: usize, y: usize| {
            let i = (y * frame.width as usize + x) * 3;
            (buf[i], buf[i + 1], buf[i + 2])
        };
        assert_eq!(px(59, 0), (0x14, 0x1a, 0x1c), "page background");
        assert_eq!(px(25, 1), (0x2b, 0x35, 0x3b), "the wide cell's background");
        // some red ink where "a" is drawn
        assert!((0..10).any(|x| (0..20).any(|y| px(x, y).0 > 0xc0 && px(x, y).1 < 0x40)));
        // the rule row is one pixel line across the whole width
        assert_ne!(px(30, 30), (0x14, 0x1a, 0x1c));
    }
}
