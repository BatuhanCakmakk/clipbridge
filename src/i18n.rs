//! User-facing text. The PC side (tray menu, notifications, setup page) follows the Windows display
//! language or the `language` setting; replies to the phone follow the phone's `Accept-Language`.
//!
//! To add a language: add a `Lang` variant, a `Texts` table, and match arms in `code` and `texts`.
//! `{}` placeholders are filled in order by `fill`.

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Tr,
}

pub struct Texts {
    pub html_lang: &'static str,
    pub readme_setup: &'static str,

    pub already_running: &'static str,
    pub port_failed: &'static str,

    pub menu_running: &'static str,
    pub menu_address: &'static str,
    pub menu_setup: &'static str,
    pub menu_copy_address: &'static str,
    pub menu_copy_token: &'static str,
    pub menu_open_folder: &'static str,
    pub menu_open_log: &'static str,
    pub menu_notifications: &'static str,
    pub menu_open_links: &'static str,
    pub menu_autostart: &'static str,
    pub menu_check_updates: &'static str,
    pub menu_update_available: &'static str,
    pub menu_donate: &'static str,
    pub menu_quit: &'static str,

    pub up_to_date: &'static str,
    pub update_failed: &'static str,
    pub update_found: &'static str,

    pub got_rich: &'static str,
    pub got_text: &'static str,
    pub got_link: &'static str,
    pub got_image: &'static str,
    pub got_file: &'static str,
    pub got_files: &'static str,

    pub page_only_local: &'static str,
    pub bad_token: &'static str,
    pub unknown_path: &'static str,
    pub error: &'static str,
    pub read_failed: &'static str,
    pub too_large: &'static str,
    pub no_form_field: &'static str,
    pub empty_body: &'static str,
    pub write_failed: &'static str,
    pub sent_rich: &'static str,
    pub sent_text: &'static str,
    pub sent_link: &'static str,
    pub sent_image: &'static str,
    pub sent_file: &'static str,

    pub page_title: &'static str,
    pub page_step1: &'static str,
    pub page_step2: &'static str,
    pub page_step3: &'static str,
    pub page_qr_warning: &'static str,
    pub page_address: &'static str,
    pub page_ip_fallback: &'static str,
    pub page_token: &'static str,
    pub page_donate: &'static str,
}

const EN: Texts = Texts {
    html_lang: "en",
    readme_setup: "#setup",

    already_running: "ClipBridge is already running. Look for its icon next to the clock.",
    port_failed: "Could not open port {}: {}\n\nAnother program may be using it. You can change the port in config.json.",

    menu_running: "ClipBridge is running",
    menu_address: "Address: {}",
    menu_setup: "Setup / connection info (QR)",
    menu_copy_address: "Copy address",
    menu_copy_token: "Copy token",
    menu_open_folder: "Open received files folder",
    menu_open_log: "Open log",
    menu_notifications: "Show notifications",
    menu_open_links: "Open received links in the browser",
    menu_autostart: "Start with Windows",
    menu_check_updates: "Check for updates (v{})",
    menu_update_available: "New version v{} — download",
    menu_donate: "☕ Buy me a coffee",
    menu_quit: "Quit",

    up_to_date: "ClipBridge is up to date (v{}).",
    update_failed: "Could not check for updates. Check your internet connection.",
    update_found: "Version v{} is available. Download it from the tray menu.",

    got_rich: "Rich text received ({} characters) — Ctrl+V",
    got_text: "Text received ({} characters) — Ctrl+V",
    got_link: "Link opened: {}",
    got_image: "Image received: {} — Ctrl+V",
    got_file: "File received: {} — Ctrl+V",
    got_files: "{} files received — Ctrl+V",

    page_only_local: "This page can only be opened on the PC itself.",
    bad_token: "Wrong token. Run the ClipBridge Setup shortcut and scan the QR code again.",
    unknown_path: "Unknown path",
    error: "Error: {}",
    read_failed: "Could not read the request: {}",
    too_large: "The content is over the 1 GB limit",
    no_form_field: "No file field in the form (the key must be \"file\")",
    empty_body: "Nothing was received (the clipboard may be empty)",
    write_failed: "Could not save the file: {}",
    sent_rich: "Rich text sent ({} characters)",
    sent_text: "Text sent ({} characters)",
    sent_link: "Link opened on the PC",
    sent_image: "Image sent",
    sent_file: "File sent",

    page_title: "ClipBridge setup",
    page_step1: "Add the three shortcuts to your iPhone: {}",
    page_step2: "Run the <b>ClipBridge Setup</b> shortcut.",
    page_step3: "Point the camera at this QR code. Done.",
    page_qr_warning: "The QR code is shown only on this PC. It contains the address and the token, so don't share a screenshot of it.",
    page_address: "Address",
    page_ip_fallback: "IP address, if the network name doesn't work (the router may change it)",
    page_token: "Token (X-Token header; grouped by four for reading, the dashes are optional)",
    page_donate: "Find ClipBridge useful? You can {} ☕",
};

const TR: Texts = Texts {
    html_lang: "tr",
    readme_setup: "/blob/main/README.tr.md#kurulum",

    already_running: "ClipBridge zaten çalışıyor. Saatin yanındaki simgeye bak.",
    port_failed: "{} portu açılamadı: {}\n\nBaşka bir program kullanıyor olabilir. config.json içinden portu değiştirebilirsin.",

    menu_running: "ClipBridge çalışıyor",
    menu_address: "Adres: {}",
    menu_setup: "Kurulum / bağlantı bilgileri (QR)",
    menu_copy_address: "Adresi kopyala",
    menu_copy_token: "Token'ı kopyala",
    menu_open_folder: "Gelen dosyalar klasörünü aç",
    menu_open_log: "Günlüğü aç",
    menu_notifications: "Bildirim göster",
    menu_open_links: "Gelen linkleri tarayıcıda aç",
    menu_autostart: "Windows ile başlat",
    menu_check_updates: "Güncellemeleri denetle (v{})",
    menu_update_available: "Yeni sürüm v{} — indir",
    menu_donate: "☕ Bana kahve ısmarla",
    menu_quit: "Çıkış",

    up_to_date: "ClipBridge güncel (v{}).",
    update_failed: "Güncelleme denetlenemedi. İnternet bağlantısını kontrol et.",
    update_found: "Yeni sürüm var: v{}. Tepsi menüsünden indirebilirsin.",

    got_rich: "Biçimli metin alındı ({} karakter) — Ctrl+V",
    got_text: "Metin alındı ({} karakter) — Ctrl+V",
    got_link: "Link açıldı: {}",
    got_image: "Resim alındı: {} — Ctrl+V",
    got_file: "Dosya alındı: {} — Ctrl+V",
    got_files: "{} dosya alındı — Ctrl+V",

    page_only_local: "Bu sayfa yalnızca PC'nin kendisinden açılabilir.",
    bad_token: "Token hatalı. ClipBridge Setup kestirmesini çalıştırıp QR kodu yeniden okut.",
    unknown_path: "Bilinmeyen adres",
    error: "Hata: {}",
    read_failed: "İstek okunamadı: {}",
    too_large: "İçerik 1 GB sınırını aşıyor",
    no_form_field: "Formda dosya alanı yok (anahtar \"file\" olmalı)",
    empty_body: "Boş içerik geldi (pano boş olabilir)",
    write_failed: "Dosya kaydedilemedi: {}",
    sent_rich: "Biçimli metin gönderildi ({} karakter)",
    sent_text: "Metin gönderildi ({} karakter)",
    sent_link: "Link PC'de açıldı",
    sent_image: "Resim gönderildi",
    sent_file: "Dosya gönderildi",

    page_title: "ClipBridge kurulum",
    page_step1: "iPhone'a üç kestirmeyi kur: {}",
    page_step2: "<b>ClipBridge Setup</b> kestirmesini çalıştır.",
    page_step3: "Açılan kamerayı bu QR koda tut. Bitti.",
    page_qr_warning: "QR kodu yalnızca bu bilgisayarda gösterilir. İçinde adres ve token var, ekran görüntüsünü paylaşma.",
    page_address: "Adres",
    page_ip_fallback: "Ağ adı çalışmazsa IP adresi (modem değiştirebilir)",
    page_token: "Token (X-Token başlığı; 4'erli gruplar okumayı kolaylaştırmak için, tireler isteğe bağlı)",
    page_donate: "ClipBridge işine yaradıysa {} ☕",
};

pub fn texts(lang: Lang) -> &'static Texts {
    match lang {
        Lang::En => &EN,
        Lang::Tr => &TR,
    }
}

/// `"tr"`, `"tr-TR"`, `"EN-us"`... Unknown languages give `None`.
fn code(tag: &str) -> Option<Lang> {
    let primary = tag.trim().split(['-', '_']).next()?.to_ascii_lowercase();
    match primary.as_str() {
        "en" => Some(Lang::En),
        "tr" => Some(Lang::Tr),
        _ => None,
    }
}

static UI: OnceLock<Lang> = OnceLock::new();

/// `setting` is the config value: `"auto"` (Windows display language) or a language code.
pub fn init(setting: &str) {
    let lang = code(setting).unwrap_or_else(windows_language);
    let _ = UI.set(lang);
}

/// Language of the PC side.
pub fn ui() -> &'static Texts {
    texts(*UI.get().unwrap_or(&Lang::En))
}

fn windows_language() -> Lang {
    const LANG_TURKISH: u16 = 0x1f;
    let langid = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
    if langid & 0x3ff == LANG_TURKISH {
        Lang::Tr
    } else {
        Lang::En
    }
}

/// The first supported language in an `Accept-Language` header (iOS sends the phone's language
/// first, e.g. `tr-TR,tr;q=0.9`); English when there is none.
pub fn from_accept_language(header: Option<&str>) -> &'static Texts {
    let lang = header.unwrap_or("").split(',').filter_map(|part| code(part.split(';').next().unwrap_or(""))).next();
    texts(lang.unwrap_or(Lang::En))
}

/// Replaces each `{}` in `template` with the next argument.
pub fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut args = args.iter();
    let mut rest = template;
    while let Some(i) = rest.find("{}") {
        out.push_str(&rest[..i]);
        if let Some(a) = args.next() {
            out.push_str(&a.to_string());
        }
        rest = &rest[i + 2..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_language() {
        assert_eq!(from_accept_language(Some("tr-TR,tr;q=0.9")).html_lang, "tr");
        assert_eq!(from_accept_language(Some("de-DE,en;q=0.8,tr;q=0.5")).html_lang, "en");
        assert_eq!(from_accept_language(Some("de-DE")).html_lang, "en");
        assert_eq!(from_accept_language(None).html_lang, "en");
    }

    #[test]
    fn fill_in_order() {
        assert_eq!(fill("{} portu: {}", &[&8787, &"x"]), "8787 portu: x");
        assert_eq!(fill("no args", &[]), "no args");
    }
}
