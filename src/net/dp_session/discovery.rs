//! Finding sessions: a UDP probe and the answers heard to it. With no address given the probe goes
//! to the broadcast, to this machine, to the peers of a Tailscale network, to the last host
//! joined and to every session Bonjour has heard of (`net::bonjour`).

use super::*;

struct Found {
    desc: Desc,
    at: SocketAddr,
    heard: Instant,
}

/// The most sessions kept: a LAN has a few, and the game's list takes no more than 0x100.
const FOUND_MOST: usize = 64;

/// The sessions heard, by their instance: a session heard again replaces its entry, and the list
/// never holds more than `FOUND_MOST` (the longest silent goes first).
#[derive(Default)]
struct Heard {
    by_instance: HashMap<[u8; 16], Found>,
}

impl Heard {
    fn hear(&mut self, found: Found) {
        let instance = found.desc.instance;
        if !self.by_instance.contains_key(&instance) && self.by_instance.len() >= FOUND_MOST {
            let oldest = self
                .by_instance
                .iter()
                .min_by_key(|(_, f)| f.heard)
                .map(|(instance, _)| *instance);
            if let Some(oldest) = oldest {
                self.by_instance.remove(&oldest);
            }
        }
        self.by_instance.insert(instance, found);
    }

    /// The sessions heard in the last 4 s of `application`, the first heard first; the others
    /// forgotten.
    fn recent(&mut self, application: [u8; 16]) -> Vec<(Desc, SocketAddr)> {
        self.by_instance
            .retain(|_, x| x.heard.elapsed() < Duration::from_secs(4));
        let mut list: Vec<&Found> = self
            .by_instance
            .values()
            .filter(|x| x.desc.application == application)
            .collect();
        list.sort_by_key(|x| x.heard);
        list.into_iter().map(|x| (x.desc.clone(), x.at)).collect()
    }
}

struct Finder {
    socket: UdpSocket,
    found: Arc<Mutex<Heard>>,
}

/// How long the reader waits after a failed receive: 50 ms, doubling up to this.
const RETRY_LONGEST: Duration = Duration::from_secs(1);
/// How many failed receives in a row end the reader (the finder is made again by the next search).
const RETRIES_MOST: u32 = 12;

static FINDER: Mutex<Option<Finder>> = Mutex::new(None);

/// Where a probe goes: the address asked (a name or a number, `port` unless it says), or, when
/// none is given, the network's broadcast and this machine, the peers of a Tailscale network
/// (`tailnet`, which carries no broadcast) and the last host joined (`last`).
fn probe_targets(
    host: &str,
    port: u16,
    tailnet: &[std::net::Ipv4Addr],
    last: Option<&str>,
) -> Vec<SocketAddr> {
    let host = host.trim();
    if host.is_empty() {
        let mut targets = vec![
            SocketAddr::from(([255, 255, 255, 255], port)),
            SocketAddr::from(([127, 0, 0, 1], port)),
        ];
        targets.extend(tailnet.iter().map(|ip| SocketAddr::from((*ip, port))));
        if let Some(last) = last {
            targets.extend(resolve(last, port));
        }
        let mut seen = Vec::new();
        targets.retain(|t| {
            let new = !seen.contains(t);
            seen.push(*t);
            new
        });
        return targets;
    }
    resolve(host, port)
}

/// The IPv4 addresses `host` (a name or a number, with a port when it has one) names.
fn resolve(host: &str, port: u16) -> Vec<SocketAddr> {
    let with_port = if host.contains(':') {
        host.to_string()
    } else {
        format!("{host}:{port}")
    };
    with_port
        .to_socket_addrs()
        .map(|a| a.filter(std::net::SocketAddr::is_ipv4).collect())
        .unwrap_or_default()
}

/// What `tailscale status --json` last gave, and whether it is being asked.
struct Tailnet {
    peers: Vec<std::net::Ipv4Addr>,
    asked: Option<Instant>,
}

static TAILNET: Mutex<Tailnet> = Mutex::new(Tailnet {
    peers: Vec::new(),
    asked: None,
});

/// How long a tailnet listing is kept before `tailscale` is asked again, and how long it is given.
const TAILNET_KEPT: Duration = Duration::from_secs(30);
const TAILNET_GIVEN: Duration = Duration::from_secs(3);

/// The peers of the tailnet this machine is on, as last listed: none while the first listing is
/// still being made, and none when `tailscale` is absent or fails. A listing runs on a thread of
/// its own (the search never waits for it) and is repeated every 30 s.
fn tailnet_peers() -> Vec<std::net::Ipv4Addr> {
    let mut t = TAILNET
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if t.asked.is_none_or(|at| at.elapsed() >= TAILNET_KEPT) {
        t.asked = Some(Instant::now());
        std::thread::spawn(|| {
            let peers = list_tailnet().unwrap_or_default();
            TAILNET
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .peers = peers;
        });
    }
    t.peers.clone()
}

/// The tailnet's peers from `tailscale status --json`, given `TAILNET_GIVEN` to answer (the
/// command stopped after that).
fn list_tailnet() -> Option<Vec<std::net::Ipv4Addr>> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    let mut command = Command::new("tailscale");
    command
        .args(["status", "--json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = crate::platform::os::spawn_hidden(&mut command).ok()?;
    let mut out = child.stdout.take()?;
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = out.read_to_string(&mut text);
        let _ = send.send(text);
    });
    let text = receive.recv_timeout(TAILNET_GIVEN);
    let _ = child.kill();
    let _ = child.wait();
    Some(rc2k_core::net::tailscale::peer_addresses(&text.ok()?))
}

/// The most addresses Bonjour has heard that one search probes.
const BONJOUR_PROBES_MOST: usize = 64;

/// The sessions of application `application` heard in the last 4 s, a probe sent first (the
/// answers to it come in meanwhile, for the next call: DirectPlay's lists done
/// asynchronously).
pub fn find_sessions(host: &str, port: u16, application: [u8; 16]) -> Vec<(Desc, SocketAddr)> {
    let mut finder = FINDER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if finder.is_none() {
        let Ok(socket) = UdpSocket::bind(("0.0.0.0", 0)) else {
            return vec![];
        };
        let _ = socket.set_broadcast(true);
        let found = Arc::new(Mutex::new(Heard::default()));
        let (reader, list) = (socket.try_clone(), found.clone());
        if let Ok(reader) = reader {
            std::thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                let mut failed = 0u32;
                loop {
                    let (n, from) = match reader.recv_from(&mut buffer) {
                        Ok(got) => {
                            failed = 0;
                            got
                        }
                        // Windows reports an earlier probe's "port unreachable" (a probe to
                        // this machine with no session here) on the next receive: the socket
                        // still works, and ending here would leave every later answer unread.
                        Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => continue,
                        Err(_) => {
                            failed += 1;
                            if failed >= RETRIES_MOST {
                                // The socket is broken: the finder goes, and the next search
                                // makes a new one.
                                let mut finder = FINDER
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                                if finder
                                    .as_ref()
                                    .is_some_and(|f| Arc::ptr_eq(&f.found, &list))
                                {
                                    *finder = None;
                                }
                                return;
                            }
                            std::thread::sleep(
                                (Duration::from_millis(50) * 2u32.pow(failed - 1))
                                    .min(RETRY_LONGEST),
                            );
                            continue;
                        }
                    };
                    let mut r = R(&buffer[..n]);
                    if r.raw::<8>().as_ref() != Some(MAGIC) {
                        continue;
                    }
                    let (Some(port), Some(desc)) = (r.u32(), r.desc()) else {
                        continue;
                    };
                    // A port that is none is not a session's.
                    let Some(port) = u16::try_from(port).ok().filter(|&p| p != 0) else {
                        continue;
                    };
                    let at = SocketAddr::new(from.ip(), port);
                    list.lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .hear(Found {
                            desc,
                            at,
                            heard: Instant::now(),
                        });
                }
            });
        }
        *finder = Some(Finder { socket, found });
    }
    let Some(f) = finder.as_ref() else {
        return vec![];
    };
    let probe = W::default().raw(MAGIC).raw(&application).0;
    let (tailnet, last, bonjour) = if host.trim().is_empty() {
        (
            tailnet_peers(),
            crate::net::last_host::address(),
            heard_by_bonjour(),
        )
    } else {
        (Vec::new(), None, Vec::new())
    };
    let mut targets = probe_targets(host, port, &tailnet, last.as_deref());
    // Any machine of the network can announce a service: only so many are probed.
    for at in bonjour.into_iter().take(BONJOUR_PROBES_MOST) {
        if !targets.contains(&at) {
            targets.push(at);
        }
    }
    for target in targets {
        let _ = f.socket.send_to(&probe, target);
    }
    f.found
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .recent(application)
}

/// The sessions Bonjour has heard of (`net::bonjour`; none in the unit tests).
fn heard_by_bonjour() -> Vec<SocketAddr> {
    #[cfg(test)]
    return Vec::new();
    #[cfg(not(test))]
    crate::net::bonjour::heard()
}

/// Where the session `instance` was heard from (for joining it).
pub fn session_address(instance: &[u8; 16]) -> Option<SocketAddr> {
    let finder = FINDER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let list = finder
        .as_ref()?
        .found
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    list.by_instance.get(instance).map(|x| x.at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heard(instance: u32, heard: Instant) -> Found {
        let mut id = [0u8; 16];
        id[..4].copy_from_slice(&instance.to_le_bytes());
        Found {
            desc: Desc {
                instance: id,
                application: [1; 16],
                ..Default::default()
            },
            at: SocketAddr::from(([10, 0, 0, 1], 47000)),
            heard,
        }
    }

    #[test]
    fn the_list_of_sessions_heard_is_bounded_and_keeps_the_newest() {
        let start = Instant::now();
        let mut list = Heard::default();
        // 200 forged sessions, each a little later than the last.
        for n in 0..200u32 {
            list.hear(heard(n, start + Duration::from_micros(n as u64)));
        }
        assert_eq!(list.by_instance.len(), FOUND_MOST);
        // The last one heard is still there; the first is gone.
        assert!(
            list.by_instance
                .values()
                .any(|f| f.heard == start + Duration::from_micros(199))
        );
        assert!(list.by_instance.values().all(|f| f.heard != start));
    }

    #[test]
    fn a_session_heard_again_replaces_its_entry() {
        let now = Instant::now();
        let mut list = Heard::default();
        list.hear(heard(3, now));
        let instance = heard(3, now).desc.instance;
        let mut again = heard(3, now);
        again.at = SocketAddr::from(([10, 0, 0, 99], 1));
        list.hear(again);
        assert_eq!(list.by_instance.len(), 1);
        assert_eq!(
            list.recent([1; 16]),
            vec![(
                list.by_instance[&instance].desc.clone(),
                SocketAddr::from(([10, 0, 0, 99], 1))
            )]
        );
    }

    #[test]
    fn an_empty_address_probes_the_tailnet_and_the_last_host_too() {
        let tailnet = [std::net::Ipv4Addr::new(100, 64, 0, 2)];
        let targets = probe_targets("", 47000, &tailnet, Some("192.168.1.9"));
        assert!(targets.contains(&SocketAddr::from(([255, 255, 255, 255], 47000))));
        assert!(targets.contains(&SocketAddr::from(([100, 64, 0, 2], 47000))));
        assert!(targets.contains(&SocketAddr::from(([192, 168, 1, 9], 47000))));
        // An address asked for is the only one probed.
        assert_eq!(
            probe_targets("10.0.0.5", 47000, &tailnet, Some("192.168.1.9")),
            vec![SocketAddr::from(([10, 0, 0, 5], 47000))]
        );
    }
}
