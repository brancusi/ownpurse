//! A minimal, dependency-free, deterministic PDF writer for the review export: US Letter pages of monospaced
//! text (the built-in Courier font, so nothing is embedded), a header line carrying the package hash on every
//! page and a `page n of m` footer. No timestamps and no random ids, so the same package always gives the same
//! bytes (and the export can be hashed and compared). Text outside printable ASCII is transliterated (`→` → `->`,
//! `−` → `-`, `✓` → `ok`, other characters → `?`), since the base-14 fonts have no Unicode encoding.

const PAGE_W: u32 = 612;
const PAGE_H: u32 = 792;
const MARGIN: u32 = 40;
const SIZE: u32 = 8;
const LEADING: u32 = 10;
/// Characters per line at 8 pt Courier (0.6 em wide) inside the margins.
pub const COLS: usize = ((PAGE_W - 2 * MARGIN) * 10 / (SIZE * 6)) as usize;

/// The printable-ASCII form of a line (PDF string escapes are applied later).
pub fn ascii(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' '..='~' => o.push(c),
            '\t' => o.push_str("    "),
            '→' => o.push_str("->"),
            '←' => o.push_str("<-"),
            '−' | '–' | '—' => o.push('-'),
            '·' | '•' => o.push('*'),
            '✓' => o.push_str("ok"),
            '≠' => o.push_str("!="),
            '…' => o.push_str("..."),
            '‘' | '’' => o.push('\''),
            '“' | '”' => o.push('"'),
            '⚑' => o.push('!'),
            _ => o.push('?'),
        }
    }
    o
}

fn escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '(' | ')' | '\\') {
            o.push('\\');
        }
        o.push(c);
    }
    o
}

/// Wrap long lines at `COLS` (hard wrap, continuation indented by two spaces).
fn wrap(lines: &[String]) -> Vec<String> {
    let mut out = vec![];
    for l in lines {
        let a = ascii(l);
        if a.len() <= COLS {
            out.push(a);
            continue;
        }
        let mut rest = a.as_str();
        let mut first = true;
        while !rest.is_empty() {
            let w = if first { COLS } else { COLS - 2 };
            let cut = rest.len().min(w);
            let (head, tail) = rest.split_at(cut);
            out.push(if first {
                head.to_string()
            } else {
                format!("  {head}")
            });
            rest = tail;
            first = false;
        }
    }
    out
}

/// The PDF bytes for `lines`, with `header` (e.g. `Package sha256:…`) on top of every page.
pub fn render(header: &str, lines: &[String]) -> Vec<u8> {
    let body = wrap(lines);
    let per_page = ((PAGE_H - 2 * MARGIN) / LEADING) as usize - 3;
    let pages: Vec<&[String]> = if body.is_empty() {
        vec![&[]]
    } else {
        body.chunks(per_page).collect()
    };
    let n = pages.len();
    // objects: 1 catalog, 2 pages, 3 font, then (page, content) per page
    let mut objs: Vec<String> = vec![];
    objs.push("<< /Type /Catalog /Pages 2 0 R >>".into());
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
    objs.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {n} >>",
        kids.join(" ")
    ));
    objs.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>".into(),
    );
    let head = escape(&ascii(header));
    for (i, page) in pages.iter().enumerate() {
        let mut s = String::new();
        s.push_str(&format!(
            "BT /F1 {SIZE} Tf {LEADING} TL {MARGIN} {} Td\n",
            PAGE_H - MARGIN
        ));
        s.push_str(&format!("({head}) Tj T* T*\n"));
        for l in page.iter() {
            s.push_str(&format!("({}) Tj T*\n", escape(l)));
        }
        s.push_str("ET\n");
        s.push_str(&format!(
            "BT /F1 {SIZE} Tf {MARGIN} {} Td (page {} of {n}) Tj ET\n",
            MARGIN - 16,
            i + 1
        ));
        objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objs.push(format!("<< /Length {} >>\nstream\n{s}endstream", s.len()));
    }
    let mut out: Vec<u8> = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_well_formed() {
        let lines: Vec<String> = (0..150).map(|i| format!("line {i} (a) \\ → ✓")).collect();
        let a = render("Package sha256:abc", &lines);
        let b = render("Package sha256:abc", &lines);
        assert_eq!(a, b);
        let s = String::from_utf8_lossy(&a);
        assert!(s.starts_with("%PDF-1.4"));
        assert!(s.ends_with("%%EOF\n"));
        assert!(s.contains("/Count 3"));
        assert!(s.contains("line 0 \\(a\\) \\\\ -> ok"));
        // every xref offset points at its object
        let x = s.find("xref\n").unwrap();
        for (i, l) in s[x..].lines().skip(3).take(5).enumerate() {
            let off: usize = l[..10].parse().unwrap();
            assert!(s[off..].starts_with(&format!("{} 0 obj", i + 1)));
        }
    }
}
