//! Finding sessions: a UDP probe and the answers heard to it.

use super::*;

struct Found {
    desc: Desc,
    at: SocketAddr,
    heard: Instant,
}

struct Finder {
    socket: UdpSocket,
    found: Arc<Mutex<Vec<Found>>>,
}

static FINDER: Mutex<Option<Finder>> = Mutex::new(None);

/// Where a probe goes: the address asked (a name or a number, `port` unless it says), or
/// the network's broadcast and this machine.
fn probe_targets(host: &str, port: u16) -> Vec<SocketAddr> {
    let host = host.trim();
    if host.is_empty() {
        return vec![
            SocketAddr::from(([255, 255, 255, 255], port)),
            SocketAddr::from(([127, 0, 0, 1], port)),
        ];
    }
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
        let found = Arc::new(Mutex::new(Vec::<Found>::new()));
        let (reader, list) = (socket.try_clone(), found.clone());
        if let Ok(reader) = reader {
            std::thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    let (n, from) = match reader.recv_from(&mut buffer) {
                        Ok(got) => got,
                        // Windows reports an earlier probe's "port unreachable" (a probe to
                        // this machine with no session here) on the next receive: the socket
                        // still works, and ending here would leave every later answer unread.
                        Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => continue,
                        Err(_) => {
                            std::thread::sleep(Duration::from_millis(50));
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
                    let at = SocketAddr::new(from.ip(), port as u16);
                    let mut list = list
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    list.retain(|f| f.desc.instance != desc.instance);
                    list.push(Found {
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
    for target in probe_targets(host, port) {
        let _ = f.socket.send_to(&probe, target);
    }
    let mut list = f
        .found
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    list.retain(|x| x.heard.elapsed() < Duration::from_secs(4));
    list.iter()
        .filter(|x| x.desc.application == application)
        .map(|x| (x.desc.clone(), x.at))
        .collect()
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
    list.iter()
        .find(|x| &x.desc.instance == instance)
        .map(|x| x.at)
}
