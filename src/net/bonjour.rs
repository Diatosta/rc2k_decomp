//! Bonjour (mDNS service discovery) beside the broadcast probe, for networks and systems where a
//! broadcast does not reach (iOS lets an app neither send nor hear one). A hosted session is
//! announced as a `_rc2k._udp` service at its probe port while it answers probes; a search with
//! no address given also probes every service heard, directly. The answers and the joining are
//! the probe's own, unchanged. iOS uses the system's own Bonjour (`platform::ffi::dns_sd`), the
//! other systems `mdns-sd`; the web build has none.

#![cfg_attr(
    test,
    allow(
        dead_code,
        reason = "the unit tests' sessions are neither announced nor searched for"
    )
)]

use std::net::SocketAddr;

/// The service type announced and listened for.
#[cfg_attr(web, allow(dead_code, reason = "the web build has no network"))]
const SERVICE: &str = "_rc2k._udp";

/// The services kept as heard, and the addresses kept of each.
#[cfg_attr(
    any(target_os = "ios", web),
    allow(dead_code, reason = "the system's own browsing keeps the list")
)]
const HEARD_MOST: usize = 64;
#[cfg_attr(
    any(target_os = "ios", web),
    allow(dead_code, reason = "the system's own browsing keeps the list")
)]
const ADDRESSES_MOST: usize = 8;

/// A session announced until this is dropped.
pub struct Announcement {
    #[cfg(target_os = "ios")]
    _service: crate::platform::ffi::dns_sd::Announcement,
    #[cfg(not(any(target_os = "ios", web)))]
    full_name: String,
}

/// The session answering probes at `port` announced; none (logged) where it cannot be.
pub fn announce(port: u16) -> Option<Announcement> {
    let made = system::announce(port);
    match &made {
        Ok(_) => log(&format!(
            "net: Bonjour: the session announced at port {port}"
        )),
        Err(e) => log(&format!("net: Bonjour: the session is not announced: {e}")),
    }
    made.ok()
}

/// The addresses of the sessions Bonjour has heard of (they are probed like any other).
pub fn heard() -> Vec<SocketAddr> {
    system::heard()
}

fn log(text: &str) {
    crate::program_log::log(text);
}

#[cfg(target_os = "ios")]
mod system {
    use super::*;
    use crate::platform::ffi::dns_sd;

    pub fn announce(port: u16) -> Result<Announcement, String> {
        dns_sd::announce(c"_rc2k._udp", port)
            .map(|service| Announcement { _service: service })
            .map_err(|code| format!("the system refused it (error {code})"))
    }

    pub fn heard() -> Vec<SocketAddr> {
        debug_assert_eq!(SERVICE, "_rc2k._udp");
        dns_sd::heard(c"_rc2k._udp")
    }
}

#[cfg(not(any(target_os = "ios", web)))]
mod system {
    use super::*;
    use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    /// The service type with its domain, as `mdns-sd` names it.
    fn kind() -> String {
        format!("{SERVICE}.local.")
    }

    /// The one daemon (its own thread) for the announcements and the search; none when it
    /// could not start (no network interface).
    fn daemon() -> Option<&'static ServiceDaemon> {
        static DAEMON: OnceLock<Option<ServiceDaemon>> = OnceLock::new();
        DAEMON
            .get_or_init(|| {
                // Many phones drop the multicast packets unless the app holds the Wi-Fi's lock.
                #[cfg(target_os = "android")]
                if crate::platform::ffi::android_java::call_with_path(
                    c"org.rc2k.rally.Multicast",
                    c"hold",
                    &std::ffi::CString::from(c"rc2k bonjour"),
                ) != Some(true)
                {
                    log("net: Bonjour: the Wi-Fi's multicast lock is not held");
                }
                match ServiceDaemon::new() {
                    Ok(daemon) => Some(daemon),
                    Err(e) => {
                        log(&format!("net: Bonjour: not started: {e}"));
                        None
                    }
                }
            })
            .as_ref()
    }

    pub fn announce(port: u16) -> Result<Announcement, String> {
        let daemon = daemon().ok_or("Bonjour did not start")?;
        let tag = std::process::id()
            ^ (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.subsec_nanos()));
        let name = format!("Rally Championship {tag:08x}");
        let host = format!("rc2k-{tag:08x}.local.");
        let info = ServiceInfo::new(
            &kind(),
            &name,
            &host,
            "",
            port,
            None::<HashMap<String, String>>,
        )
        .map_err(|e| e.to_string())?
        .enable_addr_auto();
        let full_name = info.get_fullname().to_owned();
        daemon.register(info).map_err(|e| e.to_string())?;
        Ok(Announcement { full_name })
    }

    impl Drop for Announcement {
        fn drop(&mut self) {
            if let Some(daemon) = daemon() {
                let _ = daemon.unregister(&self.full_name);
            }
        }
    }

    /// The services heard (full name to address); filled by the search's thread, begun at the
    /// first call.
    static HEARD: Mutex<Option<HashMap<String, Vec<SocketAddr>>>> = Mutex::new(None);

    pub fn heard() -> Vec<SocketAddr> {
        let mut heard = HEARD
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if heard.is_none() {
            *heard = Some(HashMap::new());
            if let Some(events) = daemon().and_then(|d| d.browse(&kind()).ok()) {
                std::thread::spawn(move || {
                    while let Ok(event) = events.recv() {
                        note(event);
                    }
                });
            }
        }
        heard
            .as_ref()
            .map(|h| h.values().flatten().copied().collect())
            .unwrap_or_default()
    }

    /// A service heard, kept with at most `ADDRESSES_MOST` of its addresses; a new service is
    /// not kept once `HEARD_MOST` are (any machine of the network can announce as many as it
    /// likes, and every one kept is probed). Whether it is new or its addresses changed.
    pub(super) fn remember(
        heard: &mut HashMap<String, Vec<SocketAddr>>,
        full_name: &str,
        mut at: Vec<SocketAddr>,
    ) -> bool {
        at.truncate(ADDRESSES_MOST);
        if !heard.contains_key(full_name) && heard.len() >= HEARD_MOST {
            return false;
        }
        heard.insert(full_name.to_owned(), at.clone()) != Some(at)
    }

    fn note(event: ServiceEvent) {
        let mut heard = HEARD
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(heard) = heard.as_mut() else {
            return;
        };
        match event {
            ServiceEvent::ServiceResolved(service) => {
                let at: Vec<SocketAddr> = service
                    .get_addresses_v4()
                    .into_iter()
                    .map(|ip| SocketAddr::from((ip, service.get_port())))
                    .collect();
                if remember(heard, service.get_fullname(), at.clone()) {
                    log(&format!(
                        "net: Bonjour: {} heard at {at:?}",
                        service.get_fullname()
                    ));
                }
            }
            ServiceEvent::ServiceRemoved(_, full_name) => {
                heard.remove(&full_name);
            }
            _ => {}
        }
    }
}

#[cfg(web)]
mod system {
    use super::*;

    pub fn announce(_port: u16) -> Result<Announcement, String> {
        Err("no network in the web build".into())
    }

    pub fn heard() -> Vec<SocketAddr> {
        Vec::new()
    }
}

#[cfg(all(test, not(any(target_os = "ios", web))))]
mod tests {
    use super::system::remember;
    use super::*;
    use std::collections::HashMap;

    fn at(n: u8) -> Vec<SocketAddr> {
        vec![SocketAddr::from(([10, 0, 0, n], 47000))]
    }

    /// Guards the list of services heard against a network that announces without end.
    #[test]
    fn the_services_heard_are_bounded() {
        let mut heard = HashMap::new();
        for n in 0..HEARD_MOST + 20 {
            remember(&mut heard, &format!("service {n}"), at(1));
        }
        assert_eq!(heard.len(), HEARD_MOST);
        // A service already kept can still change its address.
        assert!(remember(&mut heard, "service 0", at(2)));
        assert_eq!(heard["service 0"], at(2));
        assert!(!remember(&mut heard, "service 0", at(2)));
        // And a new one is not kept.
        assert!(!remember(&mut heard, "late", at(3)));
        assert!(!heard.contains_key("late"));
    }

    #[test]
    fn only_the_first_addresses_of_a_service_are_kept() {
        let mut heard = HashMap::new();
        let many: Vec<SocketAddr> = (0..40u8).flat_map(at).collect();
        remember(&mut heard, "one", many);
        assert_eq!(heard["one"].len(), ADDRESSES_MOST);
    }
}
