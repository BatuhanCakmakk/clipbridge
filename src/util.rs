use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

/// Where config.json and the log live: a `ClipBridge-data` folder next to the exe (portable, and
/// what the user sees in Explorer), or %APPDATA%\ClipBridge if that folder isn't writable.
pub fn app_dir() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
            let dir = exe_dir.join("ClipBridge-data");
            // Folder name of the first, Turkish-only builds.
            let old = exe_dir.join("ClipBridge-veri");
            if !dir.exists() && old.is_dir() {
                let _ = std::fs::rename(&old, &dir);
            }
            let probe = dir.join(".writable");
            if std::fs::create_dir_all(&dir).is_ok() && std::fs::write(&probe, b"").is_ok() {
                let _ = std::fs::remove_file(probe);
                return dir;
            }
        }
        legacy_app_dir()
    })
    .clone()
}

pub fn legacy_app_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("ClipBridge");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn received_dir() -> PathBuf {
    let base = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("Downloads").join("ClipBridge");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn log(msg: &str) {
    let line = format!("[{}] {}\n", timestamp(true), msg);
    #[cfg(debug_assertions)]
    eprint!("{line}");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(app_dir().join("clipbridge.log")) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Local time as `2026-10-20 14:03:07` (pretty) or `20261020-140307` (for file names).
pub fn timestamp(pretty: bool) -> String {
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    let t = unsafe {
        let mut t = std::mem::zeroed();
        GetLocalTime(&mut t);
        t
    };
    if pretty {
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
    } else {
        format!("{:04}{:02}{:02}-{:02}{:02}{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
    }
}

/// LAN IPv4 addresses the phone can reach, skipping virtual adapters (Hyper-V, VMware, WSL...).
pub fn lan_ips() -> Vec<IpAddr> {
    const SKIP: [&str; 8] = ["vethernet", "vmware", "virtualbox", "hyper-v", "wsl", "loopback", "bluetooth", "tailscale"];
    let mut ips: Vec<IpAddr> = local_ip_address::list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter(|(name, ip)| {
            let name = name.to_lowercase();
            match ip {
                IpAddr::V4(v4) => !v4.is_loopback() && !v4.is_link_local() && !SKIP.iter().any(|s| name.contains(s)),
                IpAddr::V6(_) => false,
            }
        })
        .map(|(_, ip)| ip)
        .collect();
    if ips.is_empty() {
        if let Ok(ip) = local_ip_address::local_ip() {
            ips.push(ip);
        }
    }
    ips.dedup();
    ips
}

/// Keeps only a safe, bare file name.
pub fn sanitize_filename(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("");
    let cleaned: String = base
        .chars()
        .map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { '_' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    if cleaned.is_empty() {
        return None;
    }
    Some(cleaned.chars().take(150).collect())
}

/// `dir/name`, or `dir/name (2).ext` etc. if it already exists.
pub fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let p = Path::new(name);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ext = p.extension().and_then(|s| s.to_str());
    for i in 2.. {
        let n = match ext {
            Some(e) => format!("{stem} ({i}).{e}"),
            None => format!("{stem} ({i})"),
        };
        let c = dir.join(n);
        if !c.exists() {
            return c;
        }
    }
    unreachable!()
}

pub fn ext_for_mime(mime: &str) -> Option<&'static str> {
    Some(match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/heic" => "heic",
        "image/heif" => "heif",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/tiff" => "tiff",
        "image/bmp" => "bmp",
        "application/pdf" => "pdf",
        "application/zip" => "zip",
        "video/quicktime" => "mov",
        "video/mp4" => "mp4",
        "audio/mpeg" => "mp3",
        "audio/x-m4a" | "audio/mp4" => "m4a",
        "text/plain" => "txt",
        "text/html" => "html",
        "text/csv" => "csv",
        "application/json" => "json",
        _ => return None,
    })
}

pub fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "heic" => "image/heic",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "txt" => "text/plain; charset=utf-8",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "json" => "application/json",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

/// The text if it is exactly one http(s) link (what Safari's share sheet or a copied link sends).
pub fn single_link(text: &str) -> Option<&str> {
    let t = text.trim();
    let lower = t.to_ascii_lowercase();
    let is_link = (lower.starts_with("https://") || lower.starts_with("http://"))
        && t.len() > "https://".len()
        && t.len() <= 4096
        && !t.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"');
    is_link.then_some(t)
}

/// Opens a web address in the default browser.
pub fn open_url(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    unsafe {
        ShellExecuteW(std::ptr::null_mut(), wide("open").as_ptr(), wide(url).as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn single_link() {
        use super::single_link;
        assert_eq!(single_link(" https://example.com/a?b=c \n"), Some("https://example.com/a?b=c"));
        assert_eq!(single_link("HTTP://Example.com"), Some("HTTP://Example.com"));
        assert_eq!(single_link("bak: https://example.com"), None);
        assert_eq!(single_link("https://a.com https://b.com"), None);
        assert_eq!(single_link("https://"), None);
        assert_eq!(single_link("ftp://example.com"), None);
    }
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn message_box(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};
    unsafe {
        MessageBoxW(std::ptr::null_mut(), wide(text).as_ptr(), wide("ClipBridge").as_ptr(), MB_OK | MB_ICONWARNING);
    }
}

pub fn notify(enabled: bool, text: String) {
    if !enabled {
        return;
    }
    std::thread::spawn(move || {
        use tauri_winrt_notification::{Duration, Toast};
        let _ = Toast::new(Toast::POWERSHELL_APP_ID)
            .title("ClipBridge")
            .text1(&text)
            .sound(None)
            .duration(Duration::Short)
            .show();
    });
}
