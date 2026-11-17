//! A joined machine's side of a session: joining, following the host's frames, asking it for
//! requests and losing the session.

use super::*;

impl Session {
    /// Joins the session whose host is at `at`, answering a locked host with `password`.
    pub fn join_with(at: SocketAddr, signal: Signal, password: &[u8]) -> Result<Arc<Session>, u32> {
        let stream = TcpStream::connect_timeout(&at, Duration::from_secs(5))
            .map_err(|_| DPERR_NOCONNECTION)?;
        let _ = stream.set_nodelay(true);
        Session::join_over(Box::new(stream), at.port(), signal, password)
    }

    /// Joins over a connection already made to the host (`port` is what the session reports).
    pub fn join_over(
        mut stream: Box<dyn Wire>,
        port: u16,
        signal: Signal,
        password: &[u8],
    ) -> Result<Arc<Session>, u32> {
        write_frame(&mut stream, &W::default().u8(JOIN).raw(MAGIC).0)
            .map_err(|_| DPERR_NOCONNECTION)?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let answer = read_frame(&mut stream).map_err(|e| match e.kind() {
            // A host that ends the connection on the first frame is of an older version.
            std::io::ErrorKind::UnexpectedEof
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted => DPERR_VERSION,
            _ => DPERR_TIMEOUT,
        })?;
        let _ = stream.set_read_timeout(None);
        let mut answer = answer;
        if answer.first() == Some(&CHALLENGE) {
            // A locked session: the proof of the password for this challenge.
            let mut r = R(&answer[1..]);
            let (Some(salt), Some(challenge)) = (r.raw(), r.raw()) else {
                return Err(DPERR_NOCONNECTION);
            };
            let key = rc2k_core::net::password::key(&salt, password);
            let proof = rc2k_core::net::password::proof(&challenge, &key);
            write_frame(&mut stream, &W::default().u8(PROOF).raw(&proof).0)
                .map_err(|_| DPERR_NOCONNECTION)?;
            let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
            answer = read_frame(&mut stream).map_err(|_| DPERR_TIMEOUT)?;
            let _ = stream.set_read_timeout(None);
        }
        let mut r = R(&answer);
        match r.u8() {
            Some(WELCOME) => {}
            Some(REFUSE) => return Err(r.u32().unwrap_or(DPERR_NONEWPLAYERS)),
            _ => return Err(DPERR_NOCONNECTION),
        }
        let bad = DPERR_NOCONNECTION;
        let me = r.u32().ok_or(bad)?;
        let desc = r.desc().ok_or(bad)?;
        let mut world = World {
            desc,
            ..Default::default()
        };
        for _ in 0..r.u32().ok_or(bad)? {
            let id = r.u32().ok_or(bad)?;
            world.players.insert(id, r.player().ok_or(bad)?);
        }
        for _ in 0..r.u32().ok_or(bad)? {
            let id = r.u32().ok_or(bad)?;
            world.groups.insert(id, r.group().ok_or(bad)?);
        }
        let _ = stream.set_write_timeout(Some(WRITE_WAIT));
        let session = Session::new(me, world, signal, port, None);
        let reader = stream.try_clone().map_err(|_| bad)?;
        session
            .links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(1, stream);
        let s = session.clone();
        std::thread::spawn(move || s.follow_host(reader));
        Ok(session)
    }

    pub(super) fn follow_host(self: Arc<Self>, mut stream: Box<dyn Wire>) {
        while let Ok(frame) = read_frame(&mut stream) {
            let mut r = R(&frame);
            match r.u8() {
                Some(CHANGE) => {
                    let Some(change) = Change::read(&mut r) else {
                        break;
                    };
                    let events = {
                        let mut s = self.lock();
                        let messages = apply(&mut s.world, &change, self.me);
                        let to = s.events.keys().next().copied().unwrap_or(0);
                        for m in &messages {
                            s.wait(Incoming {
                                from: 0,
                                to,
                                data: m.clone(),
                            });
                        }
                        if messages.is_empty() {
                            vec![]
                        } else {
                            s.events.values().copied().collect::<Vec<_>>()
                        }
                    };
                    for e in events {
                        (self.signal)(e);
                    }
                }
                Some(KICKED) => {
                    // Put out by the host: the session is lost for this machine, with a line
                    // for the log saying why.
                    crate::logfile::program_log::log(
                        "net: the host put this machine out of the session",
                    );
                    break;
                }
                Some(DELIVER) => {
                    let (Some(from), Some(to), Some(data)) = (r.u32(), r.u32(), r.bytes()) else {
                        break;
                    };
                    let event = {
                        let mut s = self.lock();
                        s.wait(Incoming { from, to, data });
                        s.events.get(&to).copied()
                    };
                    if let Some(e) = event {
                        (self.signal)(e);
                    }
                }
                Some(REPLY) => {
                    let (Some(request), Some(result), Some(id)) = (r.u32(), r.u32(), r.u32())
                    else {
                        break;
                    };
                    let mut s = self.lock();
                    // only what was asked: a host cannot fill the map
                    if request < s.next_request {
                        s.replies.insert(request, (result, id));
                    }
                    drop(s);
                    self.answered.notify_all();
                }
                _ => break,
            }
        }
        self.lose();
    }

    /// The host's connection ended: the session lost.
    pub(super) fn lose(&self) {
        let events = {
            let mut s = self.lock();
            if s.lost {
                return;
            }
            s.lost = true;
            let to = s.events.keys().next().copied().unwrap_or(0);
            s.queue.push_back(Incoming {
                from: 0,
                to,
                data: system(&[DPSYS_SESSIONLOST], &[]),
            });
            s.events.values().copied().collect::<Vec<_>>()
        };
        self.answered.notify_all();
        if !self.stop.load(Ordering::SeqCst) {
            for e in events {
                (self.signal)(e);
            }
        }
    }

    /// A request done by the host (here, or asked of it and waited for).
    pub(super) fn ask(&self, op: Op) -> (u32, Id) {
        if self.is_host() {
            return self.host_op(op, 1);
        }
        let request = {
            let mut s = self.lock();
            if s.lost {
                return (DPERR_SESSIONLOST, 0);
            }
            let r = s.next_request;
            s.next_request += 1;
            r
        };
        self.to_machine(1, &op.write(W::default().u8(REQUEST).u32(request)).0);
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut s = self.lock();
        loop {
            if let Some(answer) = s.replies.remove(&request) {
                return answer;
            }
            if s.lost {
                return (DPERR_SESSIONLOST, 0);
            }
            let now = Instant::now();
            if now >= deadline {
                return (DPERR_TIMEOUT, 0);
            }
            s = self
                .answered
                .wait_timeout(s, deadline - now)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
    }
}
