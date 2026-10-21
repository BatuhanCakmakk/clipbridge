#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clip;
mod config;
mod i18n;
mod mdns;
mod multipart;
mod rtf;
mod server;
mod update;
mod util;
mod wic;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::{CreateMutexW, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, PostThreadMessageW, TranslateMessage, MSG, WM_APP,
};

use config::Config;
use server::App;
use util::log;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Posted to the main thread when the update check found a newer release.
const WM_UPDATE_FOUND: u32 = WM_APP + 1;

fn main() {
    // Per-monitor DPI awareness: a sharp tray icon and menu instead of a blurry upscaled one.
    unsafe {
        windows_sys::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows_sys::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
    let (config, first_run) = Config::load_or_create();
    i18n::init(&config.language);
    let tx = i18n::ui();
    let port = config.port;
    unsafe {
        // One instance per port (a second copy with its own config may run on another port).
        CreateMutexW(std::ptr::null(), 0, util::wide(&format!("Local\\ClipBridge-{port}")).as_ptr());
        if GetLastError() == ERROR_ALREADY_EXISTS {
            util::message_box(tx.already_running);
            return;
        }
    }
    let local_url = config.local_url();
    let _mdns = mdns::announce(&config.hostname, port);
    let notifications = Arc::new(AtomicBool::new(config.notifications));
    let app = Arc::new(App::new(config));

    let server = match tiny_http::Server::http(("0.0.0.0", port)) {
        Ok(s) => s,
        Err(e) => {
            log(&format!("cannot listen on port {port}: {e}"));
            util::message_box(&i18n::fill(tx.port_failed, &[&port, &e]));
            return;
        }
    };
    let ips = util::lan_ips();
    let addresses: Vec<String> = ips.iter().map(|ip| format!("http://{ip}:{port}")).collect();
    log(&format!("v{} listening on {local_url}, {}", env!("CARGO_PKG_VERSION"), addresses.join(", ")));
    {
        let app = app.clone();
        std::thread::spawn(move || server::run(server, app));
    }

    // ----- tray menu -----
    let menu = Menu::new();
    let title = MenuItem::new(tx.menu_running, false, None);
    let mut addr_items = vec![MenuItem::new(i18n::fill(tx.menu_address, &[&local_url]), false, None)];
    addr_items.extend(addresses.iter().map(|a| MenuItem::new(format!("IP: {a}"), false, None)));
    let show_info = MenuItem::new(tx.menu_setup, true, None);
    let copy_addr = MenuItem::new(tx.menu_copy_address, true, None);
    let copy_token = MenuItem::new(tx.menu_copy_token, true, None);
    let open_dir = MenuItem::new(tx.menu_open_folder, true, None);
    let open_log = MenuItem::new(tx.menu_open_log, true, None);
    let notif_item = CheckMenuItem::new(tx.menu_notifications, true, notifications.load(Ordering::Relaxed), None);
    let links_item = CheckMenuItem::new(tx.menu_open_links, true, app.config.lock().unwrap().open_links, None);
    let autostart_on = autostart_enabled();
    let autostart = Arc::new(AtomicBool::new(autostart_on));
    let autostart_item = CheckMenuItem::new(tx.menu_autostart, true, autostart_on, None);
    let update_item = MenuItem::new(i18n::fill(tx.menu_check_updates, &[&env!("CARGO_PKG_VERSION")]), true, None);
    let quit = MenuItem::new(tx.menu_quit, true, None);

    menu.append(&title).unwrap();
    for item in &addr_items {
        menu.append(item).unwrap();
    }
    menu.append_items(&[
        &show_info,
        &copy_addr,
        &copy_token,
        &PredefinedMenuItem::separator(),
        &open_dir,
        &open_log,
        &notif_item,
        &links_item,
        &autostart_item,
        &PredefinedMenuItem::separator(),
        &update_item,
        &quit,
    ])
    .unwrap();

    let tooltip = format!("ClipBridge — {local_url}");
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip)
        .with_icon(make_icon())
        .build()
        .expect("tray icon");

    let ids = (
        show_info.id().clone(),
        copy_addr.id().clone(),
        copy_token.id().clone(),
        open_dir.id().clone(),
        open_log.id().clone(),
        notif_item.id().clone(),
        autostart_item.id().clone(),
        links_item.id().clone(),
        update_item.id().clone(),
        quit.id().clone(),
    );
    let main_thread = unsafe { GetCurrentThreadId() };
    let release: Arc<Mutex<Option<update::Release>>> = Arc::default();
    spawn_update_checks(app.clone(), release.clone(), main_thread);
    if first_run {
        show_connection_page(port);
    }

    let menu_release = release.clone();
    MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
        let (show_info, copy_addr, copy_token, open_dir, open_log, notif, auto, links, update, quit) = &ids;
        let id = &ev.id;
        if id == show_info {
            show_connection_page(port);
        } else if id == copy_addr {
            let _ = clipboard_win::set_clipboard_string(&local_url);
        } else if id == copy_token {
            let token = app.config.lock().unwrap().token.clone();
            let _ = clipboard_win::set_clipboard_string(&token);
        } else if id == open_dir {
            let _ = std::process::Command::new("explorer").arg(&app.received_dir).spawn();
        } else if id == open_log {
            let _ = std::process::Command::new("notepad").arg(util::app_dir().join("clipbridge.log")).spawn();
        } else if id == notif {
            let on = !notifications.load(Ordering::Relaxed);
            notifications.store(on, Ordering::Relaxed);
            let mut cfg = app.config.lock().unwrap();
            cfg.notifications = on;
            cfg.save();
        } else if id == auto {
            let on = !autostart.load(Ordering::Relaxed);
            autostart.store(on, Ordering::Relaxed);
            set_autostart(on);
        } else if id == links {
            let mut cfg = app.config.lock().unwrap();
            cfg.open_links = !cfg.open_links;
            cfg.save();
        } else if id == update {
            let known = menu_release.lock().unwrap().clone();
            match known {
                Some(r) => util::open_url(&r.url),
                None => check_now(menu_release.clone(), main_thread),
            }
        } else if id == quit {
            unsafe { PostQuitMessage(0) };
        }
    }));

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.hwnd.is_null() && msg.message == WM_UPDATE_FOUND {
                if let Some(r) = release.lock().unwrap().as_ref() {
                    update_item.set_text(i18n::fill(tx.menu_update_available, &[&r.version]));
                }
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    log("exit");
}

/// Opens the connection page served by our own server (only reachable from this PC).
fn show_connection_page(port: u16) {
    let _ = std::process::Command::new("explorer").arg(format!("http://127.0.0.1:{port}/setup")).spawn();
}

/// Checks GitHub shortly after start and then once a day (unless `update_check` is off).
fn spawn_update_checks(app: Arc<App>, release: Arc<Mutex<Option<update::Release>>>, main_thread: u32) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(30));
        loop {
            if app.config.lock().unwrap().update_check && release.lock().unwrap().is_none() {
                match update::check() {
                    Ok(Some(r)) => found_update(r, &release, main_thread),
                    Ok(None) => {}
                    Err(e) => log(&format!("update check failed: {e}")),
                }
            }
            std::thread::sleep(Duration::from_secs(24 * 60 * 60));
        }
    });
}

/// The tray menu item was clicked: check right away and always say what came out.
fn check_now(release: Arc<Mutex<Option<update::Release>>>, main_thread: u32) {
    std::thread::spawn(move || match update::check() {
        Ok(Some(r)) => found_update(r, &release, main_thread),
        Ok(None) => util::notify(true, i18n::fill(i18n::ui().up_to_date, &[&env!("CARGO_PKG_VERSION")])),
        Err(e) => {
            log(&format!("update check failed: {e}"));
            util::notify(true, i18n::ui().update_failed.into());
        }
    });
}

fn found_update(r: update::Release, release: &Mutex<Option<update::Release>>, main_thread: u32) {
    log(&format!("update available: v{}", r.version));
    util::notify(true, i18n::fill(i18n::ui().update_found, &[&r.version]));
    *release.lock().unwrap() = Some(r);
    unsafe { PostThreadMessageW(main_thread, WM_UPDATE_FOUND, 0, 0) };
}

fn autostart_enabled() -> bool {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    key.open_subkey(RUN_KEY).and_then(|k| k.get_value::<String, _>("ClipBridge")).is_ok()
}

fn set_autostart(on: bool) {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let Ok((key, _)) = hkcu.create_subkey(RUN_KEY) else { return };
    let result = if on {
        let exe = std::env::current_exe().unwrap_or_default();
        key.set_value("ClipBridge", &format!("\"{}\"", exe.display()))
    } else {
        key.delete_value("ClipBridge")
    };
    if let Err(e) = result {
        log(&format!("autostart change failed: {e}"));
    }
}

/// Tray icon: Iconoir "send-diagonal" (MIT) in blue, rasterized at the exact small-icon size for
/// the screen's scaling (16 px at 100%, 24 px at 150%...) so it stays crisp.
fn make_icon() -> Icon {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

    let size = unsafe { GetSystemMetrics(SM_CXSMICON) }.clamp(16, 64) as u32;
    Icon::from_rgba(render_icon(size), size, size).expect("icon")
}

fn render_icon(size: u32) -> Vec<u8> {
    use resvg::{tiny_skia, usvg};
    const SVG: &str = include_str!("../assets/tray.svg");
    let tree = usvg::Tree::from_str(SVG, &usvg::Options::default()).expect("tray.svg");
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("pixmap");
    let scale = size as f32 / tree.size().width();
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn export_icon_previews() {
        let dir = std::env::var("ICON_OUT").unwrap();
        for size in [16u32, 24, 32] {
            let img = image::RgbaImage::from_raw(size, size, super::render_icon(size)).unwrap();
            img.save(format!("{dir}/tray-{size}.png")).unwrap();
        }
    }
}
