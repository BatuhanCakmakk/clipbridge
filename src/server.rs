//! HTTP API used by the iOS Shortcuts.
//!
//!   GET  /ping  -> "ok" (checks address + token)
//!   POST /clip  -> body goes to the Windows clipboard (text, image or file)
//!   GET  /clip  -> current Windows clipboard (text, image as PNG, file, or several files as ZIP)
//!   GET  /setup -> setup page with the QR code (only from the PC itself, no token)
//!
//! Every request must carry the `X-Token` header. Optional `X-Filename` (URL-encoded) names files.

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::clip::{self, ClipContent, ClipImage};
use crate::config::Config;
use crate::i18n::{self, fill, Texts};
use crate::util::{self, log, notify};

const MAX_BODY: u64 = 1024 * 1024 * 1024; // 1 GB
const MAX_TEXT: usize = 10 * 1024 * 1024;
/// A file request starting within this time after the previous one finished (a multi-item share
/// loop in the Shortcut) is added to the same clipboard entry instead of replacing it.
const BATCH_WINDOW: Duration = Duration::from_secs(3);

pub struct App {
    pub config: Mutex<Config>,
    pub received_dir: PathBuf,
    batch: Mutex<(Vec<String>, Instant)>,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            config: Mutex::new(config),
            received_dir: util::received_dir(),
            batch: Mutex::new((Vec::new(), Instant::now())),
        }
    }
}

pub fn run(server: Server, app: Arc<App>) {
    for req in server.incoming_requests() {
        let app = app.clone();
        std::thread::spawn(move || handle(req, &app));
    }
}

type Resp = Response<Cursor<Vec<u8>>>;

fn header(req: &Request, name: &str) -> Option<String> {
    req.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name)).map(|h| h.value.as_str().to_string())
}

fn hdr(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn text_response(code: u16, body: &str) -> Resp {
    Response::from_string(body).with_status_code(code).with_header(hdr("Content-Type", "text/plain; charset=utf-8"))
}

fn token_ok(given: Option<String>, expected: &str) -> bool {
    let Some(given) = given else { return false };
    // The connection page shows the token in dashed groups; accept it typed either way.
    let given: String = given.chars().filter(|c| *c != '-' && !c.is_whitespace()).collect();
    let (a, b) = (given.as_bytes(), expected.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn handle(mut req: Request, app: &App) {
    let started = Instant::now();
    let peer = req.remote_addr().map(|a| a.ip().to_string()).unwrap_or_default();
    let method = req.method().clone();
    let path = req.url().split('?').next().unwrap_or("").to_string();
    let (token, notifications, open_links) = {
        let cfg = app.config.lock().unwrap();
        (cfg.token.clone(), cfg.notifications, cfg.open_links)
    };

    let from_this_pc = req.remote_addr().is_some_and(|a| a.ip().is_loopback());
    // Replies go to the phone, in the phone's language.
    let tx = i18n::from_accept_language(header(&req, "Accept-Language").as_deref());
    let resp = if method == Method::Get && (path == "/setup" || path == "/baglanti") {
        // Shows the token, so only the PC itself may open it, never another device on the network.
        if from_this_pc {
            let cfg = app.config.lock().unwrap().clone();
            Response::from_string(connection_page(&cfg)).with_header(hdr("Content-Type", "text/html; charset=utf-8"))
        } else {
            text_response(403, tx.page_only_local)
        }
    } else if !token_ok(header(&req, "X-Token"), &token) {
        log(&format!("{peer} {method} {path} -> 401 wrong or missing token"));
        text_response(401, tx.bad_token)
    } else {
        match (&method, path.as_str()) {
            (Method::Get, "/ping") => text_response(200, "ok"),
            (Method::Post, "/clip") => match receive(&mut req, app, tx, notifications, open_links, started) {
                Ok(msg) => text_response(200, &msg),
                Err(e) => {
                    log(&format!("receive failed: {e}"));
                    text_response(500, &fill(tx.error, &[&e]))
                }
            },
            (Method::Get, "/clip") => match send() {
                Ok(r) => r,
                Err(e) => {
                    log(&format!("send failed: {e}"));
                    text_response(500, &fill(tx.error, &[&e]))
                }
            },
            _ => text_response(404, tx.unknown_path),
        }
    };
    let code = resp.status_code().0;
    if code != 401 {
        log(&format!("{peer} {method} {path} -> {code} in {} ms", started.elapsed().as_millis()));
    }
    let _ = req.respond(resp);
}

/// Setup page: a QR code the "ClipBridge Setup" shortcut scans, plus the same values as text
/// (large enough to copy with the iPhone camera's Live Text) for setting up a shortcut by hand.
fn connection_page(cfg: &Config) -> String {
    let tx = i18n::ui();
    let token = &cfg.token;
    let local_url = cfg.local_url();
    let setup = serde_json::json!({ "url": local_url, "token": token }).to_string();
    let qr = qrcode::QrCode::new(setup.as_bytes())
        .map(|code| code.render::<qrcode::render::svg::Color>().min_dimensions(300, 300).quiet_zone(true).build())
        .unwrap_or_default();
    let grouped: Vec<String> = token.as_bytes().chunks(4).map(|c| String::from_utf8_lossy(c).into_owned()).collect();
    let ips: String = util::lan_ips().iter().map(|ip| format!("<div class=s>http://{ip}:{}</div>", cfg.port)).collect();
    let repo = crate::update::REPO;
    let link = format!(r#"<a href="https://github.com/{repo}{}">github.com/{repo}</a>"#, tx.readme_setup);
    format!(
        r#"<!doctype html><html lang={lang}><meta charset=utf-8><title>{title}</title>
<style>body{{font-family:Segoe UI,sans-serif;margin:40px;color:#1d1d1f;background:#fff;max-width:900px}}h1{{font-weight:500}}
.row{{display:flex;gap:48px;flex-wrap:wrap;align-items:flex-start}}.qr svg{{width:300px;height:300px}}
.l{{color:#666;margin-top:24px}}.v{{font:600 30px Consolas,monospace;letter-spacing:1px;margin:6px 0;word-break:break-all}}
.s{{font:18px Consolas,monospace;color:#444}}.n{{color:#555;line-height:1.6}}ol{{line-height:1.8}}a{{color:#0a66d8}}</style>
<h1>{title}</h1>
<div class=row><div class=qr>{qr}</div><div>
<ol><li>{step1}</li><li>{step2}</li><li>{step3}</li></ol>
<p class=n>{warning}</p>
</div></div>
<div class=l>{address}</div><div class=v>{local_url}</div>
<div class=l>{ip_fallback}</div>{ips}
<div class=l>{token_label}</div><div class=v>{grouped}</div>"#,
        lang = tx.html_lang,
        title = tx.page_title,
        step1 = fill(tx.page_step1, &[&link]),
        step2 = tx.page_step2,
        step3 = tx.page_step3,
        warning = tx.page_qr_warning,
        address = tx.page_address,
        ip_fallback = tx.page_ip_fallback,
        token_label = tx.page_token,
        grouped = grouped.join("-"),
    )
}

// ---------- phone -> PC ----------

fn receive(req: &mut Request, app: &App, tx: &Texts, notifications: bool, open_links: bool, started: Instant) -> Result<String, String> {
    let ui = i18n::ui();
    let content_type = header(req, "Content-Type").unwrap_or_default();
    let mut mime = content_type.split(';').next().unwrap_or("").trim().to_lowercase();
    let mut filename = header(req, "X-Filename")
        .map(|v| percent_decode_str(&v).decode_utf8_lossy().into_owned())
        .and_then(|v| util::sanitize_filename(&v));

    let mut body = Vec::new();
    req.as_reader().take(MAX_BODY + 1).read_to_end(&mut body).map_err(|e| fill(tx.read_failed, &[&e]))?;
    if body.len() as u64 > MAX_BODY {
        return Err(tx.too_large.into());
    }
    // Shortcuts "Form" body: the file (with its name and type) is one part of a multipart upload.
    if mime == "multipart/form-data" {
        let part = crate::multipart::first_part(&body, &content_type).ok_or(tx.no_form_field)?;
        body = part.data;
        mime = part.mime;
        filename = part.filename.as_deref().and_then(util::sanitize_filename).or(filename);
    }
    if body.is_empty() {
        return Err(tx.empty_body.into());
    }

    let ext = filename
        .as_deref()
        .and_then(|n| Path::new(n).extension())
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    let image_fmt = image::guess_format(&body).ok();

    // Rich text (Notes, Safari, Mail copies): paste as formatted where supported, plain elsewhere.
    if body.starts_with(br"{\rtf") && ext_allows_text_or_rtf(ext.as_deref()) {
        let plain = crate::rtf::to_text(&String::from_utf8_lossy(&body));
        clip::set_rich_text(&plain, &body)?;
        app.batch.lock().unwrap().0.clear();
        let chars = plain.chars().count();
        notify(notifications, fill(ui.got_rich, &[&chars]));
        return Ok(fill(tx.sent_rich, &[&chars]));
    }

    // Text: a clipboard string from the phone. Real .txt/.csv files shared from the Files app keep
    // their extension and are treated as files, unless it is .txt.
    let ext_allows_text = matches!(ext.as_deref(), None | Some("txt"));
    let binary_mime = ["image/", "video/", "audio/", "application/pdf", "application/zip"].iter().any(|p| mime.starts_with(p));
    let looks_text = !binary_mime;
    if ext_allows_text && looks_text && image_fmt.is_none() && body.len() <= MAX_TEXT && !body.contains(&0) {
        if let Ok(text) = std::str::from_utf8(&body) {
            clip::set_text(text)?;
            app.batch.lock().unwrap().0.clear();
            if let Some(url) = util::single_link(text).filter(|_| open_links) {
                util::open_url(url);
                notify(notifications, fill(ui.got_link, &[&shorten(url, 60)]));
                return Ok(tx.sent_link.into());
            }
            let chars = text.chars().count();
            notify(notifications, fill(ui.got_text, &[&chars]));
            return Ok(fill(tx.sent_text, &[&chars]));
        }
    }

    // Everything else is saved to Downloads\ClipBridge and put on the clipboard as a file.
    let ext_guess = ext
        .clone()
        .or_else(|| util::ext_for_mime(&mime).map(String::from))
        .or_else(|| image_fmt.and_then(|f| f.extensions_str().first().map(|s| s.to_string())))
        .unwrap_or_else(|| "bin".into());
    let mut name = match filename {
        Some(n) if ext.is_some() => n,
        Some(n) => format!("{n}.{ext_guess}"),
        None => format!("iphone-{}.{ext_guess}", util::timestamp(false)),
    };

    // iPhone pictures (and every image copied on iOS) arrive as HEIC, which Windows apps can't
    // paste. Decode it with Windows' own codec and keep a JPG instead.
    let is_heic = matches!(ext_guess.as_str(), "heic" | "heif") || mime == "image/heic" || mime == "image/heif" || is_heif(&body);
    let heic_img = if is_heic {
        let t = Instant::now();
        match crate::wic::decode(&body) {
            Ok(img) => {
                log(&format!("HEIC {}x{} decoded in {} ms", img.width(), img.height(), t.elapsed().as_millis()));
                Some(img)
            }
            Err(e) => {
                log(&format!("HEIC decode failed, keeping the file as is: {e}"));
                None
            }
        }
    } else {
        None
    };
    // Screenshots and graphics stay lossless (PNG); photos become JPG, since a 12 MP photo as PNG
    // is ~16 MB versus ~3 MB as a visually identical JPG.
    let photo = heic_img.as_ref().map(is_photographic).unwrap_or(false);
    if heic_img.is_some() {
        let stem = Path::new(&name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(name.clone());
        name = format!("{stem}.{}", if photo { "jpg" } else { "png" });
    }
    let path = util::unique_path(&app.received_dir, &name);
    if heic_img.is_some() {
        // Reserve the name now; the file is written after the clipboard is set (see below).
        std::fs::File::create(&path).map_err(|e| fill(tx.write_failed, &[&e]))?;
    } else {
        std::fs::write(&path, &body).map_err(|e| fill(tx.write_failed, &[&e]))?;
    }
    let path_str = path.to_string_lossy().into_owned();

    let paths = {
        let mut batch = app.batch.lock().unwrap();
        if started.saturating_duration_since(batch.1) > BATCH_WINDOW {
            batch.0.clear();
        }
        batch.0.push(path_str.clone());
        batch.1 = Instant::now();
        batch.0.clone()
    };

    // A single picture also goes on the clipboard as an image, so it pastes into Paint/Word/WhatsApp.
    let image = match (paths.len(), heic_img.as_ref()) {
        (1, Some(img)) => Some(ClipImage { dib: clip::rgba_to_dib(img), png: None }),
        (1, None) => decode_for_clipboard(&body, image_fmt),
        _ => None,
    };
    let is_image = image.is_some();
    clip::set_files(&paths, image.as_ref())?;

    // The decoded HEIC is written as a file off the request path: pasting the picture into an app
    // is ready right away; the file for Explorer follows shortly after.
    if let Some(img) = heic_img {
        let path = path.clone();
        std::thread::spawn(move || {
            let t = Instant::now();
            let encoded = if photo { encode_jpeg(&img) } else { encode_png(&img) };
            match encoded.and_then(|bytes| std::fs::write(&path, bytes).map_err(|e| e.to_string())) {
                Ok(()) => log(&format!("{} written in {} ms: {}", if photo { "JPG" } else { "PNG" }, t.elapsed().as_millis(), path.display())),
                Err(e) => log(&format!("file write failed for {}: {e}", path.display())),
            }
        });
    }

    let shown = path.file_name().unwrap().to_string_lossy().into_owned();
    let msg = match (is_image, paths.len()) {
        (true, _) => fill(ui.got_image, &[&shown]),
        (false, 1) => fill(ui.got_file, &[&shown]),
        (false, n) => fill(ui.got_files, &[&n]),
    };
    notify(notifications, msg);
    Ok(if is_image { tx.sent_image } else { tx.sent_file }.into())
}

fn shorten(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn ext_allows_text_or_rtf(ext: Option<&str>) -> bool {
    matches!(ext, None | Some("txt") | Some("rtf"))
}

/// ISO-BMFF `ftyp` box with a HEIF/HEIC brand.
fn is_heif(data: &[u8]) -> bool {
    data.len() >= 12
        && &data[4..8] == b"ftyp"
        && matches!(&data[8..12], b"heic" | b"heix" | b"hevc" | b"hevx" | b"heim" | b"heis" | b"hevm" | b"hevs" | b"mif1" | b"msf1")
}

/// iPhone screenshots are at screen resolution (at most ~3.8 MP); camera photos are 12 MP or more.
/// Smaller images that are still very colorful (a photo copied from a web page) also count.
fn is_photographic(img: &image::RgbaImage) -> bool {
    const GRID: u32 = 64;
    let (w, h) = img.dimensions();
    if w as u64 * h as u64 >= 6_000_000 {
        return true;
    }
    let mut seen = std::collections::HashSet::new();
    for gy in 0..GRID {
        for gx in 0..GRID {
            let p = img.get_pixel(gx * w / GRID, gy * h / GRID);
            seen.insert([p[0], p[1], p[2]]);
        }
    }
    seen.len() * 10 > (GRID * GRID * 3) as usize // more than 30% distinct colors
}

fn encode_jpeg(img: &image::RgbaImage) -> Result<Vec<u8>, String> {
    let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 92)
        .encode(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("JPG encoding failed: {e}"))?;
    Ok(out)
}

fn encode_png(img: &image::RgbaImage) -> Result<Vec<u8>, String> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::ImageEncoder;
    let mut out = Vec::new();
    PngEncoder::new_with_quality(&mut out, CompressionType::Fast, FilterType::Adaptive)
        .write_image(img.as_raw(), img.width(), img.height(), image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("PNG encoding failed: {e}"))?;
    Ok(out)
}

fn decode_for_clipboard(body: &[u8], fmt: Option<image::ImageFormat>) -> Option<ClipImage> {
    let fmt = fmt?;
    let img = image::load_from_memory_with_format(body, fmt).ok()?.to_rgba8();
    let png = (fmt == image::ImageFormat::Png).then(|| body.to_vec());
    Some(ClipImage { dib: clip::rgba_to_dib(&img), png })
}

// ---------- PC -> phone ----------

fn send() -> Result<Resp, String> {
    match clip::get()? {
        ClipContent::Empty => Ok(text_response(204, "")),
        ClipContent::Text(t) => Ok(text_response(200, &t)),
        ClipContent::Png(png) => Ok(file_response(png, "clipboard.png", "image/png")),
        ClipContent::Bmp(bmp) => {
            let img = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp).map_err(|e| e.to_string())?;
            let mut png = Vec::new();
            img.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png).map_err(|e| e.to_string())?;
            Ok(file_response(png, "clipboard.png", "image/png"))
        }
        ClipContent::Files(files) => {
            if files.len() == 1 && files[0].is_file() {
                let f = &files[0];
                let data = std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?;
                let name = f.file_name().unwrap().to_string_lossy().into_owned();
                let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("");
                Ok(file_response(data, &name, util::mime_for_ext(ext)))
            } else {
                let zip = zip_paths(&files)?;
                Ok(file_response(zip, &format!("clipboard-{}.zip", util::timestamp(false)), "application/zip"))
            }
        }
    }
}

fn file_response(data: Vec<u8>, name: &str, mime: &str) -> Resp {
    let ascii: String = name.chars().map(|c| if c.is_ascii_graphic() && c != '"' || c == ' ' { c } else { '_' }).collect();
    let disposition = format!(
        "attachment; filename=\"{ascii}\"; filename*=UTF-8''{}",
        utf8_percent_encode(name, NON_ALPHANUMERIC)
    );
    Response::from_data(data).with_header(hdr("Content-Type", mime)).with_header(hdr("Content-Disposition", &disposition))
}

fn zip_paths(paths: &[PathBuf]) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    fn add(zip: &mut zip::ZipWriter<Cursor<Vec<u8>>>, path: &Path, name: &str, opts: zip::write::SimpleFileOptions) -> Result<(), String> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| e.to_string())?.flatten() {
                let child = format!("{name}/{}", entry.file_name().to_string_lossy());
                add(zip, &entry.path(), &child, opts)?;
            }
        } else {
            zip.start_file(name, opts).map_err(|e| e.to_string())?;
            zip.write_all(&std::fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    for p in paths {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "file".into());
        add(&mut zip, p, &name, opts)?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

