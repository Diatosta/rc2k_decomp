//! The address of the last host joined, kept in `var/rc2k.toml` (`last_host`): a search with no
//! address probes it as well, so that a host over a network that carries no broadcast (a VPN)
//! is found again.

use std::sync::Mutex;

static LAST: Mutex<String> = Mutex::new(String::new());

/// The last host's address as text, in the form the options keep (padded with NULs).
pub fn get() -> Vec<u8> {
    LAST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .into_bytes()
}

/// The last host's address, when one is kept.
pub fn address() -> Option<String> {
    let last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    (!last.is_empty()).then_some(last)
}

/// Keeps `text` (up to its first NUL) as the last host's address.
pub fn set(text: &[u8]) {
    let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
    *LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        String::from_utf8_lossy(&text[..end]).trim().to_string();
}

/// Notes the host joined at `ip` (the port is the settings'), unless the address is the
/// machine's own.
pub fn remember(ip: std::net::IpAddr) {
    if ip.is_loopback() {
        return;
    }
    set(ip.to_string().as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_is_kept_as_text_and_loopback_is_not() {
        let _lock = crate::test_support::test_lock();
        set(b"");
        assert_eq!(address(), None);
        remember("127.0.0.1".parse().unwrap());
        assert_eq!(address(), None);
        remember("100.64.0.7".parse().unwrap());
        assert_eq!(address().as_deref(), Some("100.64.0.7"));
        set(b"192.168.1.5\0\0\0");
        assert_eq!(get(), b"192.168.1.5".to_vec());
        set(b"");
    }
}
