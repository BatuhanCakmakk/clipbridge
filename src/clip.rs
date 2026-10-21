//! Windows clipboard access: writes what the phone sends, reads what the phone asks for.

use std::path::PathBuf;

use clipboard_win::{options::NoClear, raw, Clipboard};
use image::RgbaImage;

const CF_BITMAP: u32 = 2;
const CF_DIB: u32 = 8;
const CF_UNICODETEXT: u32 = 13;
const CF_HDROP: u32 = 15;
const DROPEFFECT_COPY: u32 = 1;

pub enum ClipContent {
    Text(String),
    Files(Vec<PathBuf>),
    Png(Vec<u8>),
    Bmp(Vec<u8>),
    Empty,
}

/// An image ready for the clipboard: CF_DIB for every app, plus the original PNG when we have one
/// (keeps transparency in apps that read the "PNG" format).
pub struct ClipImage {
    pub dib: Vec<u8>,
    pub png: Option<Vec<u8>>,
}

fn open() -> Result<Clipboard, String> {
    Clipboard::new_attempts(20).map_err(|e| format!("clipboard busy: {e}"))
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub fn set_text(text: &str) -> Result<(), String> {
    let _c = open()?;
    raw::empty().map_err(err)?;
    raw::set_string_with(text, NoClear).map_err(err)
}

/// Rich text: plain text for every app plus the original RTF for Word, Outlook, WordPad...
pub fn set_rich_text(plain: &str, rtf: &[u8]) -> Result<(), String> {
    let _c = open()?;
    raw::empty().map_err(err)?;
    raw::set_string_with(plain, NoClear).map_err(err)?;
    if let Some(fmt) = raw::register_format("Rich Text Format") {
        raw::set_without_clear(fmt.get(), rtf).map_err(err)?;
    }
    Ok(())
}

/// Puts files on the clipboard (Ctrl+V in Explorer copies them). If `image` is given, the same
/// clipboard entry also carries the picture, so Ctrl+V in Paint/Word/WhatsApp pastes the image.
pub fn set_files(paths: &[String], image: Option<&ClipImage>) -> Result<(), String> {
    let _c = open()?;
    raw::empty().map_err(err)?;
    if let Some(img) = image {
        raw::set_without_clear(CF_DIB, &img.dib).map_err(err)?;
        if let (Some(png), Some(fmt)) = (&img.png, raw::register_format("PNG")) {
            raw::set_without_clear(fmt.get(), png).map_err(err)?;
        }
    }
    raw::set_file_list_with(paths, NoClear).map_err(err)?;
    if let Some(fmt) = raw::register_format("Preferred DropEffect") {
        raw::set_without_clear(fmt.get(), &DROPEFFECT_COPY.to_le_bytes()).map_err(err)?;
    }
    Ok(())
}

/// Reads the clipboard. Preference: files, then text, then image. Text wins over image because
/// apps like Excel put both, and the text is what people want there.
pub fn get() -> Result<ClipContent, String> {
    let _c = open()?;
    if raw::is_format_avail(CF_HDROP) {
        let mut files: Vec<String> = Vec::new();
        raw::get_file_list(&mut files).map_err(err)?;
        if !files.is_empty() {
            return Ok(ClipContent::Files(files.into_iter().map(PathBuf::from).collect()));
        }
    }
    if raw::is_format_avail(CF_UNICODETEXT) {
        let mut buf = Vec::new();
        raw::get_string(&mut buf).map_err(err)?;
        let text = String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string();
        if !text.is_empty() {
            return Ok(ClipContent::Text(text));
        }
    }
    if let Some(fmt) = raw::register_format("PNG") {
        if raw::is_format_avail(fmt.get()) {
            let mut buf = Vec::new();
            raw::get_vec(fmt.get(), &mut buf).map_err(err)?;
            if !buf.is_empty() {
                return Ok(ClipContent::Png(buf));
            }
        }
    }
    if raw::is_format_avail(CF_DIB) || raw::is_format_avail(CF_BITMAP) {
        let mut buf = Vec::new();
        raw::get_bitmap(&mut buf).map_err(err)?;
        return Ok(ClipContent::Bmp(buf));
    }
    Ok(ClipContent::Empty)
}

/// 32-bit bottom-up CF_DIB (BITMAPINFOHEADER + BGRA pixels).
pub fn rgba_to_dib(img: &RgbaImage) -> Vec<u8> {
    let (w, h) = img.dimensions();
    let size = w as usize * h as usize * 4;
    let mut out = Vec::with_capacity(40 + size);
    out.extend_from_slice(&40u32.to_le_bytes()); // biSize
    out.extend_from_slice(&(w as i32).to_le_bytes()); // biWidth
    out.extend_from_slice(&(h as i32).to_le_bytes()); // biHeight (positive = bottom-up)
    out.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    out.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    out.extend_from_slice(&0u32.to_le_bytes()); // biCompression = BI_RGB
    out.extend_from_slice(&(size as u32).to_le_bytes()); // biSizeImage
    out.extend_from_slice(&[0u8; 16]); // resolution + palette fields
    for row in img.as_raw().chunks_exact(w as usize * 4).rev() {
        for px in row.chunks_exact(4) {
            out.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
    }
    out
}
