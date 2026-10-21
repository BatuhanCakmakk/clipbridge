//! Announces the PC as `<hostname>.local` on the local network (mDNS / Bonjour), which iOS resolves
//! natively. The Shortcuts use that name instead of an IP address that the router may change.

use mdns_sd::{ServiceDaemon, ServiceInfo};

use crate::util::log;

/// Keep the returned daemon alive for as long as the name should be answered.
pub fn announce(hostname: &str, port: u16) -> Option<ServiceDaemon> {
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
