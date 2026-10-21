use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::util::{app_dir, log};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub token: String,
    #[serde(default = "default_true")]
    pub notifications: bool,
    /// Announced on the network as `<hostname>.local`, so the Shortcuts keep working when the
    /// router hands the PC a new IP address.
    #[serde(default = "default_hostname")]
    pub hostname: String,
    /// A single web link sent from the phone opens in the default browser.
    #[serde(default = "default_true")]
    pub open_links: bool,
    /// Once a day, ask GitHub whether a newer release exists.
    #[serde(default = "default_true")]
    pub update_check: bool,
}

fn default_true() -> bool {
    true
}

/// `clipbridge-<computer name>`, lowercased and reduced to what a DNS label allows.
fn default_hostname() -> String {
    let pc = std::env::var("COMPUTERNAME").unwrap_or_default().to_lowercase();
    let pc: String = pc.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let pc = pc.trim_matches('-');
    if pc.is_empty() {
        "clipbridge".into()
    } else {
        format!("clipbridge-{pc}").chars().take(63).collect()
    }
}

impl Config {
    /// The second value is true when no config existed yet (first start on this PC).
    pub fn load_or_create() -> (Config, bool) {
        let path = app_dir().join("config.json");
        // Carry over the token from the older %APPDATA% location, so existing Shortcuts keep working.
        let legacy = crate::util::legacy_app_dir().join("config.json");
        if !path.exists() && legacy != path && legacy.exists() {
            let _ = std::fs::copy(&legacy, &path);
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<Config>(&text) {
                Ok(cfg) if !cfg.token.is_empty() => {
                    cfg.save(); // writes fields added by newer versions, so they can be edited
                    return (cfg, false);
                }
                Ok(_) => log("config.json has an empty token, regenerating"),
                Err(e) => log(&format!("config.json unreadable ({e}), regenerating")),
            }
        }
        let cfg = Config {
            port: 8787,
            token: new_token(),
            notifications: true,
            hostname: default_hostname(),
            open_links: true,
            update_check: true,
        };
        cfg.save();
        (cfg, true)
    }

    /// Address for the Shortcuts, e.g. `http://clipbridge-desktop.local:8787`.
    pub fn local_url(&self) -> String {
        format!("http://{}.local:{}", self.hostname, self.port)
    }

    pub fn save(&self) {
        let path = app_dir().join("config.json");
        if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(self).unwrap()) {
            log(&format!("could not save config: {e}"));
        }
    }
}

/// 20 chars from an alphabet that is easy to type on a phone (no 0/o, 1/l/i lookalikes): ~99 bits.
fn new_token() -> String {
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let mut rng = rand::thread_rng();
    (0..20).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect()
}
