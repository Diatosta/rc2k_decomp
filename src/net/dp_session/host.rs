//! The host's side of a session: accepting machines, answering probes, doing their requests,
//! sending each change to every machine and routing messages.

use super::*;

impl Session {
    /// The host's session made, at `port` (its TCP connections and its UDP answers).
    #[cfg(test)]
    pub fn host(desc: Desc, port: u16, signal: Signal) -> std::io::Result<Arc<Session>> {
        Session::host_locked(desc, port, signal, b"")
    }

    /// The host's session made, locked by `password` unless it is empty: joiners must prove they
    /// know it, and the session's description says it is locked.
    pub fn host_locked(
        desc: Desc,
        port: u16,
        signal: Signal,
        password: &[u8],
    ) -> std::io::Result<Arc<Session>> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        let udp = UdpSocket::bind(("0.0.0.0", port))?;
        udp.set_read_timeout(Some(Duration::from_millis(300)))?;
        let session = Session::host_over(desc, port, signal, password);
        let s = session.clone();
        std::thread::spawn(move || s.accept(listener));
        let s = session.clone();
        std::thread::spawn(move || s.answer_probes(udp));
        Ok(session)
    }

    /// The host's session made with no listener of its own: machines come through connections
    /// handed to `serve_wire` (a kind of connection that finds its own machines).
    pub fn host_over(mut desc: Desc, port: u16, signal: Signal, password: &[u8]) -> Arc<Session> {
        desc.current_players = 0;
        let lock = if password.is_empty() {
            desc.flags &= !DPSESSION_PASSWORDREQUIRED;
            None
        } else {
            desc.flags |= DPSESSION_PASSWORDREQUIRED;
            let salt = random_16();
            Some(Lock {
                salt,
                key: rc2k_core::net::password::key(&salt, password),
            })
        };
        Session::new(
            1,
            World {
                desc,
                ..Default::default()
            },
            signal,
            port,
            lock,
        )
    }

    /// A machine's connection taken in, on a thread of its own, until it ends.
    pub fn serve_wire(self: &Arc<Self>, wire: Box<dyn Wire>) {
        if self.connections.fetch_add(1, Ordering::SeqCst) >= CONNECTIONS_MOST {
            self.connections.fetch_sub(1, Ordering::SeqCst);
            let _ = wire.shutdown();
            return;
        }
        let s = self.clone();
        std::thread::spawn(move || {
            s.clone().serve(wire);
            s.connections.fetch_sub(1, Ordering::SeqCst);
        });
    }

    pub(super) fn accept(self: Arc<Self>, listener: TcpListener) {
        let _ = listener.set_nonblocking(true);
        while !self.stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_nodelay(true);
                    self.serve_wire(Box::new(stream));
                }
                Err(_) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }

    pub(super) fn answer_probes(self: Arc<Self>, udp: UdpSocket) {
        let mut buffer = [0u8; 64];
        while !self.stop.load(Ordering::SeqCst) {
            let Ok((n, from)) = udp.recv_from(&mut buffer) else {
                continue;
            };
            if n != 24 || &buffer[..8] != MAGIC {
                continue;
            }
            let desc = self.lock().world.desc.clone();
            if buffer[8..24] != desc.application {
                continue;
            }
            let answer = W::default().raw(MAGIC).u32(self.port as u32).desc(&desc);
            let _ = udp.send_to(&answer.0, from);
        }
    }

    /// One joined machine's connection, until it ends.
    pub(super) fn serve(self: Arc<Self>, mut stream: Box<dyn Wire>) {
        let _ = stream.set_write_timeout(Some(WRITE_WAIT));
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let Ok(first) = read_frame(&mut stream) else {
            return;
        };
        let _ = stream.set_read_timeout(None);
        if first.first() != Some(&JOIN) {
            return;
        }
        if first.get(1..9) != Some(MAGIC) {
            // Another version of this protocol is told so (the machine shows its reason);
            // anything else is not a machine of this game and is dropped.
            if first.get(1..8) == Some(MAGIC_FAMILY) {
                let _ = write_frame(&mut stream, &W::default().u8(REFUSE).u32(DPERR_VERSION).0);
            }
            return;
        }
        if let Some(lock) = &self.lock
            && !self.proves_password(&mut *stream, lock)
        {
            // A wrong guess waits before it is answered: guesses cost a second each.
            std::thread::sleep(Duration::from_secs(1));
            let _ = write_frame(
                &mut stream,
                &W::default().u8(REFUSE).u32(DPERR_INVALIDPASSWORD).0,
            );
            return;
        }
        let (machine, welcome) = {
            let mut s = self.lock();
            if s.world.desc.flags & DPSESSION_JOINDISABLED != 0 {
                let _ = write_frame(
                    &mut stream,
                    &W::default().u8(REFUSE).u32(DPERR_NONEWPLAYERS).0,
                );
                return;
            }
            let machine = s.next_machine;
            s.next_machine += 1;
            let mut w = W::default()
                .u8(WELCOME)
                .u32(machine)
                .desc(&s.world.desc)
                .u32(s.world.players.len() as u32);
            for (id, p) in &s.world.players {
                w = w.u32(*id).player(p);
            }
            w = w.u32(s.world.groups.len() as u32);
            for (id, g) in &s.world.groups {
                w = w.u32(*id).group(g);
            }
            (machine, w.0)
        };
        let Ok(writer) = stream.try_clone() else {
            return;
        };
        // the welcome under the links' lock, so that no change goes out before it
        {
            let mut links = self
                .links
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut writer = writer;
            if write_frame(&mut writer, &welcome).is_err() {
                return;
            }
            links.insert(machine, writer);
        }
        while let Ok(frame) = read_frame(&mut stream) {
            let mut r = R(&frame);
            match r.u8() {
                Some(REQUEST) => {
                    let Some(request) = r.u32() else { break };
                    let Some(op) = Op::read(&mut r) else { break };
                    let (result, id) = self.host_op(op, machine);
                    self.to_machine(
                        machine,
                        &W::default().u8(REPLY).u32(request).u32(result).u32(id).0,
                    );
                }
                Some(SEND) => {
                    let (Some(from), Some(to), Some(data)) = (r.u32(), r.u32(), r.bytes()) else {
                        break;
                    };
                    // only a machine's own players send
                    let own = self
                        .lock()
                        .world
                        .players
                        .get(&from)
                        .is_some_and(|p| p.machine == machine);
                    if own {
                        self.route(from, to, data);
                    }
                }
                _ => break,
            }
        }
        // the machine has gone: its players with it
        self.links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&machine);
        let gone: Vec<Id> = {
            let s = self.lock();
            s.world
                .players
                .iter()
                .filter(|(_, p)| p.machine == machine)
                .map(|(id, _)| *id)
                .collect()
        };
        for id in gone {
            self.host_change(Change::PlayerRemoved(id));
        }
    }

    /// Whether the machine on `stream` answers a fresh challenge with the proof of the password.
    fn proves_password(&self, stream: &mut dyn Wire, lock: &Lock) -> bool {
        use rc2k_core::net::password::{KEY_LEN, proof, same};
        let challenge = random_16();
        let asked = W::default().u8(CHALLENGE).raw(&lock.salt).raw(&challenge);
        if write_frame(stream, &asked.0).is_err() {
            return false;
        }
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let answer = read_frame(stream);
        let _ = stream.set_read_timeout(None);
        let Ok(answer) = answer else {
            return false;
        };
        let mut r = R(&answer);
        let given = (r.u8() == Some(PROOF))
            .then(|| r.raw::<KEY_LEN>())
            .flatten();
        given.is_some_and(|given| same(&given, &proof(&challenge, &lock.key)))
    }

    pub(super) fn to_machine(&self, machine: u32, bytes: &[u8]) {
        let mut links = self
            .links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(link) = links.get_mut(&machine)
            && write_frame(link, bytes).is_err()
        {
            let _ = link.shutdown();
        }
    }

    /// The host applies a change and sends it to every machine.
    pub(super) fn host_change(&self, change: Change) {
        let (messages, events) = {
            let mut s = self.lock();
            let messages = apply(&mut s.world, &change, self.me);
            let events: Vec<u32> = if messages.is_empty() {
                vec![]
            } else {
                s.events.values().copied().collect()
            };
            let to = s.events.keys().next().copied().unwrap_or(0);
            for m in &messages {
                s.wait(Incoming {
                    from: 0,
                    to,
                    data: m.clone(),
                });
            }
            (messages, events)
        };
        let bytes = change.write(W::default().u8(CHANGE)).0;
        {
            let mut links = self
                .links
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for link in links.values_mut() {
                if write_frame(link, &bytes).is_err() {
                    let _ = link.shutdown();
                }
            }
        }
        if !messages.is_empty() {
            for e in events {
                (self.signal)(e);
            }
        }
    }

    /// A request of machine `machine` done (or not): DirectPlay's result and a new id.
    pub(super) fn host_op(&self, op: Op, machine: u32) -> (u32, Id) {
        let change = {
            let mut s = self.lock();
            let w = &s.world;
            let owned = |id: &Id| w.players.get(id).map(|p| p.machine == machine);
            let result = match &op {
                Op::CreatePlayer(name, data) => {
                    let d = &w.desc;
                    if d.flags & DPSESSION_NEWPLAYERSDISABLED != 0
                        || (d.max_players != 0 && w.players.len() as u32 >= d.max_players)
                        || w.players.len() >= PLAYERS_MOST
                        || !fits(name, data)
                    {
                        Err(DPERR_CANTADDPLAYER)
                    } else {
                        Ok(())
                    }
                }
                Op::CreateGroup(name, data) => {
                    if w.groups.len() < GROUPS_MOST && fits(name, data) {
                        Ok(())
                    } else {
                        Err(DPERR_CANTADDPLAYER)
                    }
                }
                Op::AddToGroup(g, p) | Op::RemoveFromGroup(g, p) => {
                    match (w.groups.get(g), w.players.contains_key(p)) {
                        (None, _) => Err(DPERR_INVALIDGROUP),
                        (_, false) => Err(DPERR_INVALIDPLAYER),
                        (Some(group), true) => {
                            if matches!(op, Op::RemoveFromGroup(..)) && !group.members.contains(p) {
                                Err(DPERR_INVALIDPLAYER)
                            } else {
                                Ok(())
                            }
                        }
                    }
                }
                Op::SetPlayerData(_, d) if d.len() > DATA_MOST => Err(DPERR_ACCESSDENIED),
                Op::DestroyPlayer(p) | Op::SetPlayerData(p, _) => match owned(p) {
                    None => Err(DPERR_INVALIDPLAYER),
                    Some(false) => Err(DPERR_ACCESSDENIED),
                    Some(true) => Ok(()),
                },
                Op::SetDesc(_) => {
                    if machine == 1 {
                        Ok(())
                    } else {
                        Err(DPERR_ACCESSDENIED)
                    }
                }
            };
            if let Err(e) = result {
                return (e, 0);
            }
            let mut new = 0;
            let mut fresh = || {
                new = s.next_id;
                s.next_id += 1;
                new
            };
            let change = match op {
                Op::CreatePlayer(name, data) => Change::PlayerAdded(
                    fresh(),
                    Player {
                        name,
                        data,
                        machine,
                    },
                ),
                Op::CreateGroup(name, data) => Change::GroupAdded(
                    fresh(),
                    Group {
                        name,
                        data,
                        machine,
                        members: vec![],
                    },
                ),
                Op::AddToGroup(g, p) => Change::Joined(g, p),
                Op::RemoveFromGroup(g, p) => Change::Left(g, p),
                Op::DestroyPlayer(p) => Change::PlayerRemoved(p),
                Op::SetPlayerData(p, d) => Change::PlayerData(p, d),
                Op::SetDesc(mut d) => {
                    // what only the host keeps
                    d.instance = s.world.desc.instance;
                    d.application = s.world.desc.application;
                    d.flags = (d.flags & !DPSESSION_PASSWORDREQUIRED)
                        | (s.world.desc.flags & DPSESSION_PASSWORDREQUIRED);
                    Change::Desc(d)
                }
            };
            (change, new)
        };
        let (change, new) = change;
        self.host_change(change);
        (DP_OK, new)
    }

    /// A message handed to its addressees' machines.
    pub(super) fn route(&self, from: Id, to: Id, data: Vec<u8>) {
        if data.len() > MESSAGE_MOST {
            return;
        }
        let (local, remote, events) = {
            let s = self.lock();
            let w = &s.world;
            let to_ids: Vec<Id> = if to == 0 {
                w.players.keys().copied().filter(|&p| p != from).collect()
            } else if let Some(g) = w.groups.get(&to) {
                g.members.iter().copied().filter(|&p| p != from).collect()
            } else if w.players.contains_key(&to) {
                vec![to]
            } else {
                vec![]
            };
            let mut local = vec![];
            let mut remote = vec![];
            for p in to_ids {
                match w.players.get(&p).map(|x| x.machine) {
                    Some(1) => local.push(p),
                    Some(m) => remote.push((m, p)),
                    None => {}
                }
            }
            let events: Vec<u32> = local
                .iter()
                .filter_map(|p| s.events.get(p).copied())
                .collect();
            (local, remote, events)
        };
        if !local.is_empty() {
            let mut s = self.lock();
            for &p in &local {
                s.wait(Incoming {
                    from,
                    to: p,
                    data: data.clone(),
                });
            }
        }
        for e in events {
            (self.signal)(e);
        }
        for (machine, p) in remote {
            self.to_machine(
                machine,
                &W::default().u8(DELIVER).u32(from).u32(p).bytes(&data).0,
            );
        }
    }
}

/// Sixteen random bytes (a salt or a challenge): each half from a hasher the system keyed
/// at random, with the time in.
pub(super) fn random_16() -> [u8; 16] {
    use std::hash::{BuildHasher, Hasher};
    let mut out = [0u8; 16];
    for half in out.chunks_mut(8) {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        );
        half.copy_from_slice(&h.finish().to_le_bytes());
    }
    out
}
