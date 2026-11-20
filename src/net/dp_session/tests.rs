use std::io::{Read, Write};

use super::*;

fn quiet(_: u32) {}

fn session() -> Arc<Session> {
    Session::new(2, World::default(), Arc::new(quiet), 0, None)
}

fn message(len: usize) -> Incoming {
    Incoming {
        from: 7,
        to: 8,
        data: vec![0x55; len],
    }
}

#[test]
fn a_session_is_found_after_a_probe_nobody_answered() {
    // A port with nothing behind it on purpose: bound and let go.
    let free = |kind: &str| {
        let port = UdpSocket::bind(("127.0.0.1", 0))
            .and_then(|u| u.local_addr())
            .map(|a| a.port());
        port.unwrap_or_else(|e| panic!("a free port for {kind}: {e}"))
    };
    let application = [0x5A; 16];
    // First a probe to a port with nothing behind it (Windows then reports the port
    // unreachable on the finder's next receive).
    let nobody = free("nobody");
    let _ = find_sessions("127.0.0.1", nobody, application);
    let desc = Desc {
        application,
        instance: [0x33; 16],
        name: "found".into(),
        ..Default::default()
    };
    let (session, port) = host_anywhere(desc, b"");
    let mut found = Vec::new();
    // Asked again until it answers, for at most five seconds.
    let until = Instant::now() + Duration::from_secs(5);
    while found.is_empty() && Instant::now() < until {
        found = find_sessions("127.0.0.1", port, application);
        if found.is_empty() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    session.close();
    assert_eq!(found.first().map(|(d, _)| d.name.as_str()), Some("found"));
}

#[test]
fn a_message_too_long_to_take_holds_up_nothing() {
    let s = session();
    s.lock().wait(message(MESSAGE_MOST + 1));
    s.lock().wait(message(MESSAGE_MOST));
    let Some(Ok(taken)) = s.receive(MESSAGE_MOST) else {
        panic!("the message that fits");
    };
    assert_eq!(taken.data.len(), MESSAGE_MOST);
    assert!(s.receive(MESSAGE_MOST).is_none());
}

#[test]
fn the_waiting_messages_stop_at_their_limit() {
    let s = session();
    for _ in 0..WAITING_MOST + 10 {
        s.lock().wait(message(1));
    }
    assert_eq!(s.lock().queue.len(), WAITING_MOST);
}

#[test]
fn the_session_lost_message_comes_with_the_queue_full() {
    let s = session();
    for _ in 0..WAITING_MOST {
        s.lock().wait(message(1));
    }
    s.lose();
    assert_eq!(s.lock().queue.len(), WAITING_MOST + 1);
}

// ---- the wire, frozen: bytes a build of this program sends to another ----

/// Dwords as little-endian bytes.
fn le(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// A host on the port the system gave its listener (a clash of the UDP side with another
/// socket: another port, a few times), with the waits shortened; its port.
fn host_anywhere(desc: Desc, password: &[u8]) -> (Arc<Session>, u16) {
    (0..10)
        .find_map(|_| {
            match Session::host_anywhere(desc.clone(), Arc::new(quiet), password, Pace::quick()) {
                Ok(session) => Some((session.clone(), session.port)),
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => None,
                Err(e) => panic!("a hosted session: {e}"),
            }
        })
        .expect("a hosted session on one of ten ports")
}

fn wire_desc() -> Desc {
    Desc {
        flags: 0,
        instance: [0x33; 16],
        application: [0x5A; 16],
        max_players: 4,
        current_players: 0,
        name: "x".into(),
        user: [1, 2, 3, 4],
    }
}

/// The session description as the wire holds it: flags, instance, application, most players,
/// players now, the name (length and bytes), four user dwords.
fn wire_desc_bytes(current_players: u32) -> Vec<u8> {
    [
        le(&[0]),
        vec![0x33; 16],
        vec![0x5A; 16],
        le(&[4, current_players, 1]),
        b"x".to_vec(),
        le(&[1, 2, 3, 4]),
    ]
    .concat()
}

/// A host of `wire_desc` on a port the system gave.
fn wire_host() -> (Arc<Session>, u16) {
    host_anywhere(wire_desc(), b"")
}

/// The host's next frame with its pings left out: the test hosts ping every 20 ms
/// (`Pace::quick`), so a ping may come between any two frames a test reads.
fn host_frame(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    loop {
        let frame = read_frame(stream)?;
        if frame.first() != Some(&PING) {
            return Ok(frame);
        }
    }
}

fn connect(port: u16) -> TcpStream {
    let stream = TcpStream::connect(("127.0.0.1", port)).expect("the host");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("a read timeout");
    stream
}

/// Guards the frame: its length as a little-endian dword, then the bytes; a longer one than
/// `FRAME_MOST` is refused when read.
#[test]
fn a_frame_is_its_length_and_its_bytes() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a listener");
    let port = listener.local_addr().expect("its address").port();
    let mut sender = connect(port);
    let (mut receiver, _) = listener.accept().expect("the connection");
    receiver
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("a read timeout");
    write_frame(&mut sender, b"abc").expect("written");
    let mut raw = [0u8; 7];
    receiver.read_exact(&mut raw).expect("seven bytes");
    assert_eq!(raw, [3, 0, 0, 0, b'a', b'b', b'c']);
    write_frame(&mut sender, &[9, 8]).expect("written");
    assert_eq!(read_frame(&mut receiver).expect("a frame"), [9, 8]);
    sender
        .write_all(&((FRAME_MOST as u32) + 1).to_le_bytes())
        .expect("a length");
    assert!(read_frame(&mut receiver).is_err());
}

/// Guards the host's side of the TCP protocol with a raw peer: the JOIN's answer
/// (WELCOME, the machine number, the session, the players, the groups), a request's REPLY
/// and the CHANGE that tells every machine, a SEND routed back as a DELIVER, a refusal with
/// bad magic.
#[test]
fn the_hosts_frames_are_frozen() {
    let (host, port) = wire_host();
    let join = [vec![JOIN], MAGIC.to_vec()].concat();

    // A JOIN without the magic is dropped: the connection ends.
    let mut wrong = connect(port);
    write_frame(&mut wrong, &[vec![JOIN], b"XC2KNET0".to_vec()].concat()).unwrap();
    assert!(host_frame(&mut wrong).is_err());

    let mut peer = connect(port);
    write_frame(&mut peer, &join).unwrap();
    let welcome = host_frame(&mut peer).expect("the welcome");
    let expected = [vec![WELCOME], le(&[2]), wire_desc_bytes(0), le(&[0, 0])].concat();
    assert_eq!(welcome, expected);

    // CreatePlayer("p", [9, 9]).
    let create = [
        vec![REQUEST],
        le(&[1]),
        vec![0],
        le(&[1]),
        b"p".to_vec(),
        le(&[2]),
        vec![9, 9],
    ]
    .concat();
    write_frame(&mut peer, &create).unwrap();
    let mut frames = [
        host_frame(&mut peer).unwrap(),
        host_frame(&mut peer).unwrap(),
    ];
    frames.sort_by_key(|f| f[0]);
    // REPLY: the request, the result, the player's id.
    assert_eq!(
        frames[0],
        [vec![REPLY], le(&[1, DP_OK, 0x0010_0001])].concat()
    );
    // CHANGE: PlayerAdded (0), the id, the player (name, data, machine 2).
    let added = [
        vec![CHANGE, 0],
        le(&[0x0010_0001, 1]),
        b"p".to_vec(),
        le(&[2]),
        vec![9, 9],
        le(&[2]),
    ]
    .concat();
    assert_eq!(frames[1], added);

    // SEND from the player to itself: delivered back with the same bytes.
    let send = [
        vec![SEND],
        le(&[0x0010_0001, 0x0010_0001, 2]),
        b"hi".to_vec(),
    ]
    .concat();
    write_frame(&mut peer, &send).unwrap();
    let delivered = host_frame(&mut peer).unwrap();
    assert_eq!(
        delivered,
        [
            vec![DELIVER],
            le(&[0x0010_0001, 0x0010_0001, 2]),
            b"hi".to_vec()
        ]
        .concat()
    );

    // A second machine is told the player in its WELCOME.
    let mut second = connect(port);
    write_frame(&mut second, &join).unwrap();
    let welcome = host_frame(&mut second).unwrap();
    let expected = [
        vec![WELCOME],
        le(&[3]),
        wire_desc_bytes(1),
        le(&[1, 0x0010_0001, 1]),
        b"p".to_vec(),
        le(&[2]),
        vec![9, 9],
        le(&[2, 0]),
    ]
    .concat();
    assert_eq!(welcome, expected);
    host.close();
}

/// Guards the refusal of a joiner when joining is disabled: REFUSE and DirectPlay's result.
#[test]
fn a_closed_session_refuses_with_its_result() {
    let desc = Desc {
        flags: DPSESSION_JOINDISABLED,
        ..wire_desc()
    };
    let (host, port) = host_anywhere(desc, b"");
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    let answer = host_frame(&mut peer).unwrap();
    assert_eq!(answer, [vec![REFUSE], le(&[DPERR_NONEWPLAYERS])].concat());
    host.close();
}

/// Guards the UDP probe: the magic and the application's GUID; the answer is the magic, the
/// port and the session's description. Another application gets no answer.
#[test]
fn the_probe_and_its_answer_are_frozen() {
    let (host, port) = wire_host();
    let udp = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
    udp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let probe = [MAGIC.to_vec(), vec![0x5A; 16]].concat();
    assert_eq!(probe.len(), 24);
    udp.send_to(&probe, ("127.0.0.1", port)).unwrap();
    let mut buffer = [0u8; 256];
    let (n, _) = udp.recv_from(&mut buffer).expect("an answer");
    let expected = [MAGIC.to_vec(), le(&[port as u32]), wire_desc_bytes(0)].concat();
    assert_eq!(&buffer[..n], expected.as_slice());
    let other = [MAGIC.to_vec(), vec![0x5B; 16]].concat();
    udp.send_to(&other, ("127.0.0.1", port)).unwrap();
    udp.set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    assert!(udp.recv_from(&mut buffer).is_err());
    host.close();
}

/// Guards the frame a host sends each joined machine when it closes: BYE alone.
#[test]
fn the_hosts_closing_frame_is_frozen() {
    let (host, port) = wire_host();
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    host_frame(&mut peer).expect("the welcome");
    host.close();
    assert_eq!(host_frame(&mut peer).expect("the bye"), vec![BYE]);
}

/// A joiner of another version of the protocol is refused with `DPERR_VERSION` (the first
/// version's mark `RC2KNET1` is one), not dropped silently.
#[test]
fn another_version_is_refused_with_its_result() {
    let (host, port) = wire_host();
    let mut old = connect(port);
    write_frame(&mut old, &[vec![JOIN], b"RC2KNET1".to_vec()].concat()).unwrap();
    assert_eq!(
        host_frame(&mut old).expect("the refusal"),
        [vec![REFUSE], le(&[DPERR_VERSION])].concat()
    );
    host.close();
}

/// The host puts a joined machine's player out: the machine gets KICKED and its connection
/// ends; a player of the host's own machine, or none, is refused.
#[test]
fn the_host_puts_a_player_out() {
    let (host, port) = wire_host();
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    host_frame(&mut peer).expect("the welcome");
    // Make a player: request 0, operation 0, name "p", data 9 9.
    let make = [
        vec![REQUEST],
        le(&[0]),
        vec![0],
        le(&[1]),
        b"p".to_vec(),
        le(&[2]),
        vec![9, 9],
    ]
    .concat();
    write_frame(&mut peer, &make).unwrap();
    let mut player = None;
    for _ in 0..4 {
        let frame = host_frame(&mut peer).expect("a frame");
        if frame[0] == REPLY {
            player = Some(u32::from_le_bytes(frame[9..13].try_into().unwrap()));
            break;
        }
    }
    let player = player.expect("the reply with the player");
    assert_eq!(host.kick(0x7777), DPERR_INVALIDPLAYER);
    assert_eq!(host.kick(player), DP_OK);
    let mut kicked = false;
    while let Ok(frame) = host_frame(&mut peer) {
        kicked |= frame == vec![KICKED];
    }
    assert!(kicked);
    host.close();
}

/// A host with a password: its description says it is locked; a joiner is asked a challenge
/// and gets in only with the right password; the wrong one and none are refused with
/// `DPERR_INVALIDPASSWORD`; an open session never asks.
#[test]
fn a_locked_session_lets_in_only_the_right_password() {
    let (host, port) = host_anywhere(wire_desc(), b"rally");
    assert_ne!(host.world().desc.flags & DPSESSION_PASSWORDREQUIRED, 0);
    let at = SocketAddr::from(([127, 0, 0, 1], port));
    // A bare join is asked for a proof, and the plain password is nowhere in what is sent.
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    let asked = host_frame(&mut peer).expect("the challenge");
    assert_eq!(asked.first(), Some(&CHALLENGE));
    assert_eq!(asked.len(), 1 + 16 + 16);
    assert!(!asked.windows(5).any(|w| w == b"rally"));
    drop(peer);
    assert_eq!(
        Session::join_with(at, Arc::new(quiet), b"wrong").err(),
        Some(DPERR_INVALIDPASSWORD)
    );
    assert_eq!(
        Session::join_with(at, Arc::new(quiet), b"").err(),
        Some(DPERR_INVALIDPASSWORD)
    );
    let guest = Session::join_with(at, Arc::new(quiet), b"rally").expect("the right password");
    assert_ne!(guest.world().desc.flags & DPSESSION_PASSWORDREQUIRED, 0);
    // The host's own description changes do not unlock it.
    let mut open = wire_desc();
    open.flags = 0;
    assert_eq!(host.set_desc(open), DP_OK);
    assert_ne!(host.world().desc.flags & DPSESSION_PASSWORDREQUIRED, 0);
    guest.close();
    host.close();
}

/// A session without a password is joined with no challenge, whatever the joiner holds.
#[test]
fn an_open_session_asks_no_password() {
    let (host, port) = wire_host();
    assert_eq!(host.world().desc.flags & DPSESSION_PASSWORDREQUIRED, 0);
    let at = SocketAddr::from(([127, 0, 0, 1], port));
    let guest = Session::join_with(at, Arc::new(quiet), b"unneeded").expect("an open session");
    guest.close();
    host.close();
}

/// The whole protocol over a connection that is not TCP: a joined machine makes a player, the
/// host's player and the joined one swap messages, and the host's closing ends the session.
#[test]
fn a_session_runs_over_an_in_memory_connection() {
    let host = Session::host_over(wire_desc(), 0, Arc::new(quiet), b"");
    let (host_end, guest_end) = Pipe::pair();
    host.serve_wire(Box::new(host_end));
    let guest = Session::join_over(Box::new(guest_end), 0, Arc::new(quiet), b"").expect("joined");
    let (result, mine) = host.create_player("host", &[], 0);
    assert_eq!(result, DP_OK);
    let (result, theirs) = guest.create_player("guest", &[7], 0);
    assert_eq!(result, DP_OK);
    assert_eq!(host.world().players.len(), 2);

    let wait_for = |session: &Session, to: Id| {
        for _ in 0..200 {
            match session.receive(MESSAGE_MOST) {
                Some(Ok(m)) if m.to == to && m.from != 0 => return m,
                Some(_) => {}
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        panic!("no message came");
    };
    assert_eq!(guest.send(theirs, mine, b"to host"), DP_OK);
    let m = wait_for(&host, mine);
    assert_eq!((m.from, m.data), (theirs, b"to host".to_vec()));
    assert_eq!(host.send(mine, theirs, b"to guest"), DP_OK);
    let m = wait_for(&guest, theirs);
    assert_eq!((m.from, m.data), (mine, b"to guest".to_vec()));

    host.close();
    for _ in 0..200 {
        if guest.send(theirs, mine, b"x") == DPERR_SESSIONLOST {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the guest never lost the session");
}

/// The host measures each machine's round trip once a second and tells every machine; the host's
/// own players have none.
#[test]
fn a_machines_ping_is_measured_and_told_to_all() {
    let (host, port) = wire_host();
    let at = SocketAddr::from(([127, 0, 0, 1], port));
    let guest = Session::join_with(at, Arc::new(quiet), b"").expect("joined");
    let (_, mine) = guest.create_player("guest", &[1], 0);
    let (_, theirs) = host.create_player("host", &[2], 0);
    let mut seen = (None, None);
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        seen = (host.ping_of(mine), guest.ping_of(mine));
        if seen.0.is_some() && seen.1.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let (on_host, on_guest) = (seen.0.expect("measured"), seen.1.expect("told"));
    assert!(on_host < 1000 && on_guest < 5000);
    assert_eq!(host.ping_of(theirs), None);
    assert_eq!(guest.ping_of(theirs), None);
    guest.close();
    host.close();
}

/// A frame of a kind the host does not know is passed over: the connection goes on.
#[test]
fn a_frame_of_an_unknown_kind_is_passed_over() {
    let (host, port) = wire_host();
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    host_frame(&mut peer).expect("the welcome");
    write_frame(&mut peer, &[0xEE, 1, 2, 3]).unwrap();
    let make = [
        vec![REQUEST],
        le(&[0]),
        vec![0],
        le(&[1]),
        b"p".to_vec(),
        le(&[2]),
        vec![9, 9],
    ]
    .concat();
    write_frame(&mut peer, &make).unwrap();
    let mut answered = false;
    for _ in 0..4 {
        let frame = host_frame(&mut peer).expect("a frame");
        if frame[0] == REPLY {
            answered = true;
            break;
        }
    }
    assert!(answered);
    host.close();
}

/// A connection whose writes block until it is released or cut: a machine that reads nothing.
#[derive(Clone)]
struct Stalled {
    released: Arc<AtomicBool>,
}

impl Read for Stalled {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl Write for Stalled {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        while !self.released.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }
        Err(std::io::ErrorKind::BrokenPipe.into()).map(|()| data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Wire for Stalled {
    fn try_clone(&self) -> std::io::Result<Box<dyn Wire>> {
        Ok(Box::new(self.clone()))
    }

    fn shutdown(&self) -> std::io::Result<()> {
        self.released.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn set_read_timeout(&self, _: Option<Duration>) -> std::io::Result<()> {
        Ok(())
    }

    fn set_write_timeout(&self, _: Option<Duration>) -> std::io::Result<()> {
        Ok(())
    }
}

/// A machine that has stopped reading holds up the host's sends to the others: each machine's
/// frames go through a queue of its own, never under a lock all the sends share.
#[test]
fn a_machine_that_reads_nothing_holds_up_no_other() {
    let host = Session::host_over(wire_desc(), 0, Arc::new(quiet), b"");
    let (host_end, guest_end) = Pipe::pair();
    host.serve_wire(Box::new(host_end));
    let guest = Session::join_over(Box::new(guest_end), 0, Arc::new(quiet), b"").expect("joined");
    let stalled = Stalled {
        released: Arc::new(AtomicBool::new(false)),
    };
    let released = stalled.released.clone();
    host.links
        .lock()
        .unwrap()
        .insert(99, Link::new(Box::new(stalled)).expect("a link"));
    for _ in 0..50 {
        host.to_machine(99, &[PING, 1, 0, 0, 0]);
    }
    let (result, _) = host.create_player("host", &[], 0);
    assert_eq!(result, DP_OK);
    // (Both sends returned: a send held up by the stalled machine would not get here.)
    // The machine that reads was told all the same.
    for _ in 0..200 {
        if !guest.world().players.is_empty() {
            released.store(true, Ordering::SeqCst);
            host.close();
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    released.store(true, Ordering::SeqCst);
    panic!("the other machine was never told");
}

/// A machine that is too slow to follow (its queue full) is cut, and sending goes on.
#[test]
fn a_machine_whose_queue_fills_is_cut() {
    let stalled = Stalled {
        released: Arc::new(AtomicBool::new(false)),
    };
    let released = stalled.released.clone();
    let link = Link::new(Box::new(stalled)).expect("a link");
    for _ in 0..10_000 {
        link.send(&[PING, 1, 0, 0, 0]);
    }
    assert!(
        released.load(Ordering::SeqCst),
        "the connection was not cut"
    );
}

/// A joined machine that hears nothing from its host for the silence it allows has lost the
/// session, as when the connection ends.
#[test]
fn a_host_that_goes_silent_loses_the_session() {
    let (host_end, guest_end) = Pipe::pair();
    let session = Session::new(2, World::default(), Arc::new(quiet), 0, None);
    let reader = session.clone();
    let follow = std::thread::spawn(move || {
        reader.follow_host(Box::new(guest_end), Duration::from_millis(20));
    });
    follow.join().expect("the reader ends on its own");
    assert!(session.lock().lost);
    drop(host_end);
}

/// A frame announced long and then not sent is not held: the bytes that came are all there is.
#[test]
fn a_long_announced_frame_that_never_comes_is_an_error() {
    let (mut sender, mut receiver) = Pipe::pair();
    receiver
        .set_read_timeout(Some(Duration::from_millis(20)))
        .expect("a timeout");
    sender
        .write_all(&(FRAME_MOST as u32).to_le_bytes())
        .expect("a length");
    sender.write_all(&[1, 2, 3]).expect("a little");
    assert!(read_frame(&mut receiver).is_err());
}
