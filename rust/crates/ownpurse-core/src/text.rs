//! Text as Xero returns it, and as this tool writes it.

/// Xero stores some text (history notes) with non-ASCII characters as HTML entities (`·` comes back as
/// `&#183;`): decode the numeric entities and the five named ones for display and comparison.
pub fn html_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let Some(end) = tail.find(';').filter(|e| *e <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let ent = &tail[1..end];
        let ch = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16)
                .ok()
                .and_then(char::from_u32),
            e if e.starts_with('#') => e[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Text folded to printable ASCII for what this tool writes into Xero's free text (notes): the separators
/// it uses become ASCII (`·` → `-`, `→` → `->`, `…` → `...`, dashes and quotes plain), anything else
/// non-ASCII becomes `?`. Xero then stores exactly what was sent.
pub fn ascii_fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '·' | '•' | '–' | '—' | '−' => out.push('-'),
            '→' => out.push_str("->"),
            '←' => out.push_str("<-"),
            '…' => out.push_str("..."),
            '‘' | '’' => out.push('\''),
            '“' | '”' => out.push('"'),
            '\u{a0}' => out.push(' '),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            '\n' | '\t' => out.push(' '),
            _ => out.push('?'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_decode_and_notes_fold_to_ascii() {
        assert_eq!(
            html_unescape("a &#183; b &amp; c &#x2192; d"),
            "a · b & c → d"
        );
        assert_eq!(html_unescape("5 & 6; &bogus; &"), "5 & 6; &bogus; &");
        assert_eq!(
            ascii_fold("Owner-paid · recode 6520 → 6310 … “x”"),
            "Owner-paid - recode 6520 -> 6310 ... \"x\""
        );
        assert_eq!(ascii_fold("café"), "caf?");
    }
}
