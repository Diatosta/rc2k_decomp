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
const SERVICE: &str = "_rc2k._udp";

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
    crate::logfile::program_log::log(text);
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
                if heard.get(service.get_fullname()) != Some(&at) {
                    log(&format!(
                        "net: Bonjour: {} heard at {at:?}",
                        service.get_fullname()
                    ));
                }
                heard.insert(service.get_fullname().to_owned(), at);
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
