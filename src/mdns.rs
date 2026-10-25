//! Announces the PC as `<hostname>.local` on the local network (mDNS / Bonjour), which iOS resolves
//! natively. The Shortcuts use that name instead of an IP address that the router may change.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use mdns_sd::{ServiceDaemon, ServiceInfo};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    NotifyIpInterfaceChange, NotifyUnicastIpAddressChange, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE,
    MIB_UNICASTIPADDRESS_ROW,
};
use windows_sys::Win32::Networking::WinSock::AF_UNSPEC;

use crate::util::log;

/// Wait this long after the last network event before starting over, so a burst of changes
/// (link up, DHCP lease, IPv6 addresses) restarts the daemon once.
const SETTLE: Duration = Duration::from_secs(3);
/// How often the sleep detector wakes up, and how far the clock may jump before it counts as
/// the PC having been asleep.
const TICK: Duration = Duration::from_secs(15);
const SLEPT: Duration = Duration::from_secs(45);

static CHANGED: OnceLock<Sender<()>> = OnceLock::new();

/// Keeps the name announced for the life of the process, starting the daemon over whenever the
/// network changes or the PC wakes up.
///
/// mdns-sd polls the interfaces every few seconds, but during sleep (modern standby) its thread is
/// frozen too. If the router restarts meanwhile, Windows drops the multicast membership with the
/// link, mdns-sd never sees the interface go away, and the name stops answering until a restart.
pub fn spawn(hostname: &str, port: u16) {
    let (tx, rx) = mpsc::channel();
    let _ = CHANGED.set(tx.clone());
    watch_network();
    watch_sleep(tx);
    let hostname = hostname.to_string();
    std::thread::spawn(move || run(&hostname, port, rx));
}

fn run(hostname: &str, port: u16, changed: Receiver<()>) {
    let mut daemon = announce(hostname, port);
    while changed.recv().is_ok() {
        // Let the burst settle.
        loop {
            match changed.recv_timeout(SETTLE) {
                Ok(()) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if let Some(old) = daemon.take() {
            if let Ok(done) = old.shutdown() {
                let _ = done.recv_timeout(Duration::from_secs(2));
            }
        }
        log("mDNS: network changed, announcing again");
        daemon = announce(hostname, port);
    }
}

fn announce(hostname: &str, port: u16) -> Option<ServiceDaemon> {
    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log(&format!("mDNS unavailable: {e}"));
            return None;
        }
    };
    let instance = std::env::var("COMPUTERNAME").unwrap_or_else(|_| hostname.to_string());
    let host = format!("{hostname}.local.");
    // No fixed addresses: each network interface answers with its own current address, so a new
    // DHCP lease or a switch between Wi-Fi and Ethernet is picked up without a restart.
    let info = ServiceInfo::new("_clipbridge._tcp.local.", &instance, &host, (), port, None)
        .map(ServiceInfo::enable_addr_auto);
    match info.and_then(|info| daemon.register(info)) {
        Ok(()) => {
            log(&format!("mDNS: announcing {host}"));
            Some(daemon)
        }
        Err(e) => {
            log(&format!("mDNS registration failed: {e}"));
            None
        }
    }
}

fn signal() {
    if let Some(tx) = CHANGED.get() {
        let _ = tx.send(());
    }
}

unsafe extern "system" fn on_interface(
    _: *const core::ffi::c_void,
    _: *const MIB_IPINTERFACE_ROW,
    _: MIB_NOTIFICATION_TYPE,
) {
    signal();
}

unsafe extern "system" fn on_address(
    _: *const core::ffi::c_void,
    _: *const MIB_UNICASTIPADDRESS_ROW,
    _: MIB_NOTIFICATION_TYPE,
) {
    signal();
}

/// Link up/down and address changes. The registrations live as long as the process.
fn watch_network() {
    let (mut intf, mut addr): (HANDLE, HANDLE) = (std::ptr::null_mut(), std::ptr::null_mut());
    unsafe {
        let r = NotifyIpInterfaceChange(AF_UNSPEC, Some(on_interface), std::ptr::null(), 0, &mut intf);
        if r != 0 {
            log(&format!("mDNS: interface notifications unavailable ({r})"));
        }
        let r = NotifyUnicastIpAddressChange(AF_UNSPEC, Some(on_address), std::ptr::null(), 0, &mut addr);
        if r != 0 {
            log(&format!("mDNS: address notifications unavailable ({r})"));
        }
    }
}

/// Waking from sleep does not always bring a network event (the link may come back exactly as it
/// was), so a jump of the wall clock counts as one too.
fn watch_sleep(tx: Sender<()>) {
    std::thread::spawn(move || {
        let mut last = SystemTime::now();
        loop {
            std::thread::sleep(TICK);
            let now = SystemTime::now();
            let gap = now.duration_since(last).unwrap_or_default();
            last = now;
            if gap > SLEPT && tx.send(()).is_err() {
                return;
            }
        }
    });
}
