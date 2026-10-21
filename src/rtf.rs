//! Minimal RTF -> plain text, for rich text copied in iOS apps (Notes, Safari, Mail).

pub fn to_text(rtf: &str) -> String {
    const SKIP_DESTINATIONS: [&str; 10] =
        ["fonttbl", "colortbl", "expandedcolortbl", "stylesheet", "info", "pict", "listtable", "listoverridetable", "generator", "themedata"];
    let b = rtf.as_bytes();
    let mut out = String::new();
    let mut stack: Vec<(bool, usize)> = Vec::new();
    let (mut skip, mut uc, mut pending) = (false, 1usize, 0usize);
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'{' => {
                stack.push((skip, uc));
                i += 1;
            }
            b'}' => {
                if let Some((s, u)) = stack.pop() {
                    (skip, uc) = (s, u);
                }
                i += 1;
            }
            b'\r' | b'\n' => i += 1,
            b'\\' if i + 1 < b.len() => {
                i += 1;
                let c = b[i];
                if c.is_ascii_alphabetic() {
                    let start = i;
                    while i < b.len() && b[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    let word = &rtf[start..i];
                    let num_start = i;
                    if i < b.len() && (b[i] == b'-' || b[i].is_ascii_digit()) {
                        i += 1;
                        while i < b.len() && b[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                    let num: Option<i32> = rtf[num_start..i].parse().ok();
                    if i < b.len() && b[i] == b' ' {
                        i += 1;
                    }
                    match word {
                        "par" | "line" if !skip => out.push('\n'),
                        "tab" if !skip => out.push('\t'),
                        "uc" => uc = num.unwrap_or(1).max(0) as usize,
                        "u" => {
                            if !skip {
                                let n = num.unwrap_or(0);
                                let cp = if n < 0 { n + 65536 } else { n } as u32;
                                out.extend(char::from_u32(cp));
                            }
                            pending = uc;
                        }
                        w if SKIP_DESTINATIONS.contains(&w) => skip = true,
                        _ => {}
                    }
                } else if c == b'\'' {
                    let hex = rtf.get(i + 1..i + 3).unwrap_or("");
                    i += 3;
                    if pending > 0 {
                        pending -= 1;
                    } else if !skip {
                        if let Ok(v) = u8::from_str_radix(hex, 16) {
                            out.push(cp1254(v));
                        }
                    }
                } else {
                    match c {
                        b'*' => skip = true,
                        b'{' | b'}' | b'\\' if !skip => out.push(c as char),
                        b'~' if !skip => out.push('\u{a0}'),
                        b'\n' | b'\r' if !skip => out.push('\n'),
                        _ => {}
                    }
                    i += 1;
                }
            }
            _ => {
                let ch = rtf[i..].chars().next().unwrap();
                if pending > 0 {
                    pending -= 1;
                } else if !skip {
                    out.push(ch);
                }
                i += ch.len_utf8();
            }
        }
    }
    out.trim_end().to_string()
}

/// Windows-1254 (Turkish) for `\'xx` escapes; Latin-1 elsewhere.
fn cp1254(v: u8) -> char {
    match v {
        0xD0 => 'Ğ', 0xF0 => 'ğ', 0xDD => 'İ', 0xFD => 'ı', 0xDE => 'Ş', 0xFE => 'ş',
        0x80 => '€', 0x85 => '…', 0x91 => '‘', 0x92 => '’', 0x93 => '“', 0x94 => '”',
        0x95 => '•', 0x96 => '–', 0x97 => '—',
        _ => v as char,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn apple_notes_rtf() {
        let rtf = r"{\rtf1\ansi\ansicpg1254\cocoartf2761
\cocoatextscaling0\cocoaplatform0{\fonttbl\f0\fswiss\fcharset0 Helvetica;}
{\colortbl;\red255\green255\blue255;}
{\*\expandedcolortbl;;}
\pard\tx560\pardirnatural\partightenfactor0

\f0\fs24 \cf0 Merhaba \uc0\u287 \uc0\u252 zel \b kal\'fdn\b0  \{x\}\
ikinci sat\uc0\u305 r}";
        assert_eq!(super::to_text(rtf), "Merhaba ğüzel kalın {x}\nikinci satır");
    }
}
