//! Just enough multipart/form-data parsing for the iOS Shortcuts "Form" request body.

use memchr::memmem;
use percent_encoding::percent_decode_str;

pub struct Part {
    pub data: Vec<u8>,
    pub mime: String,
    pub filename: Option<String>,
}

/// Returns the first uploaded file part (or a field named `file` / `text`); failing that, the first
/// non-empty field whatever its name, since the key is easy to get wrong in the Shortcuts editor.
pub fn first_part(body: &[u8], content_type: &str) -> Option<Part> {
    let mut fallback = None;
    let boundary = content_type
        .split(';')
        .filter_map(|p| p.trim().strip_prefix("boundary="))
        .next()?
        .trim_matches('"');
    let delim = format!("--{boundary}");
    let starts: Vec<usize> = memmem::find_iter(body, delim.as_bytes()).collect();
    for w in starts.windows(2) {
        let part = &body[w[0] + delim.len()..w[1]];
        let part = part.strip_prefix(b"\r\n").unwrap_or(part);
        let part = part.strip_suffix(b"\r\n").unwrap_or(part);
        let Some(sep) = memmem::find(part, b"\r\n\r\n") else { continue };
        let head = String::from_utf8_lossy(&part[..sep]);
        let (mut name, mut filename, mut mime) = (None, None, String::new());
        for line in head.split("\r\n") {
            let Some((key, value)) = line.split_once(':') else { continue };
            match key.trim().to_ascii_lowercase().as_str() {
                "content-type" => mime = value.split(';').next().unwrap_or("").trim().to_lowercase(),
                "content-disposition" => {
                    for param in value.split(';').map(str::trim) {
                        if let Some(v) = param.strip_prefix("name=") {
                            name = Some(v.trim_matches('"').to_string());
                        } else if let Some(v) = param.strip_prefix("filename*=") {
                            let v = v.split("''").last().unwrap_or(v);
                            filename = Some(percent_decode_str(v).decode_utf8_lossy().into_owned());
                        } else if let Some(v) = param.strip_prefix("filename=") {
                            filename.get_or_insert_with(|| v.trim_matches('"').to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        let data = &part[sep + 4..];
        if filename.is_some() || matches!(name.as_deref(), Some("file") | Some("text")) {
            return Some(Part { data: data.to_vec(), mime, filename });
        }
        if fallback.is_none() && !data.is_empty() {
            fallback = Some(Part { data: data.to_vec(), mime, filename: None });
        }
    }
    fallback
}

/// The `file` (or `text`) field of an `application/x-www-form-urlencoded` body, else the first
/// non-empty field.
pub fn urlencoded_field(body: &[u8]) -> Option<Vec<u8>> {
    let decode = |s: &[u8]| -> Vec<u8> {
        let plus_as_space: Vec<u8> = s.iter().map(|&b| if b == b'+' { b' ' } else { b }).collect();
        percent_encoding::percent_decode(&plus_as_space).collect()
    };
    let mut fallback = None;
    for pair in body.split(|&b| b == b'&') {
        let (key, value) = match pair.iter().position(|&b| b == b'=') {
            Some(i) => (&pair[..i], &pair[i + 1..]),
            None => (pair, &b""[..]),
        };
        let (key, value) = (decode(key), decode(value));
        if key == b"file" || key == b"text" {
            return Some(value);
        }
        if fallback.is_none() && !value.is_empty() {
            fallback = Some(value);
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    #[test]
    fn urlencoded_link() {
        let v = super::urlencoded_field(b"file=https%3A%2F%2Fwww.google.com%2Fsearch%3Fq%3Da%2Bb").unwrap();
        assert_eq!(v, b"https://www.google.com/search?q=a+b");
        assert_eq!(super::urlencoded_field(b"x=merhaba+d%C3%BCnya").unwrap(), "merhaba dünya".as_bytes());
    }

    #[test]
    fn shortcuts_style_form() {
        let body = b"--XyZ\r\nContent-Disposition: form-data; name=\"file\"; filename=\"IMG 0001.HEIC\"\r\nContent-Type: image/heic\r\n\r\n\x00\x01data\r\n--XyZ--\r\n";
        let p = super::first_part(body, "multipart/form-data; boundary=XyZ").unwrap();
        assert_eq!(p.filename.as_deref(), Some("IMG 0001.HEIC"));
        assert_eq!(p.mime, "image/heic");
        assert_eq!(p.data, b"\x00\x01data");
    }
}
