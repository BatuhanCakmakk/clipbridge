//! Asks GitHub for the latest release and compares it with this build's version.

use std::time::Duration;

pub const REPO: &str = "BatuhanCakmakk/clipbridge";
/// Tray menu and setup page link for supporting the project.
pub const DONATE_URL: &str = "https://buymeacoffee.com/batuhancakmak";

#[derive(Clone)]
pub struct Release {
    pub version: String,
    pub url: String,
}

/// `Ok(Some(..))` when a newer release than this build is published.
pub fn check() -> Result<Option<Release>, String> {
    use ureq::tls::{TlsConfig, TlsProvider};
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build())
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .into();
    let resp = agent
        .get(&format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .header("User-Agent", concat!("ClipBridge/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call();
    let mut resp = match resp {
        Ok(r) => r,
        Err(ureq::Error::StatusCode(404)) => return Ok(None), // nothing released yet
        Err(e) => return Err(e.to_string()),
    };
    let body = resp.body_mut().read_to_string().map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let tag = json["tag_name"].as_str().unwrap_or_default();
    if !is_newer(tag, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }
    let url = json["html_url"].as_str().map(String::from).unwrap_or_else(|| format!("https://github.com/{REPO}/releases/latest"));
    Ok(Some(Release { version: tag.trim_start_matches('v').to_string(), url }))
}

fn is_newer(tag: &str, current: &str) -> bool {
    fn parts(v: &str) -> Vec<u64> {
        v.trim().trim_start_matches('v').split(['.', '-']).map_while(|p| p.parse().ok()).collect()
    }
    let (new, cur) = (parts(tag), parts(current));
    !new.is_empty() && new > cur
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn versions() {
        assert!(is_newer("v1.1.0", "1.0.0"));
        assert!(is_newer("v1.0.10", "1.0.9"));
        assert!(!is_newer("v1.0.0", "1.0.0"));
        assert!(!is_newer("v0.9.0", "1.0.0"));
        assert!(!is_newer("", "1.0.0"));
    }
}
