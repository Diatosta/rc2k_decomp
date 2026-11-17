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
    // A free port: bound and let go.
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
    std::thread::sleep(Duration::from_millis(200));
    let desc = Desc {
        application,
        instance: [0x33; 16],
        name: "found".into(),
        ..Default::default()
    };
    // A port free a moment ago can be taken before the host binds it (other tests and
    // programs on the machine): another port then, a few times.
    let (session, port) = (0..10)
        .find_map(|_| {
            let port = free("the session");
            match Session::host(desc.clone(), port, Arc::new(quiet)) {
                Ok(session) => Some((session, port)),
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => None,
                Err(e) => panic!("a hosted session: {e}"),
            }
        })
        .expect("a hosted session on one of ten free ports");
    let mut found = Vec::new();
    for _ in 0..20 {
        found = find_sessions("127.0.0.1", port, application);
        if !found.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
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

#[test]
fn a_host_cannot_fill_the_lists() {
    let mut world = World::default();
    let player = |name: &str, data: &[u8]| Player {
        name: name.into(),
        data: data.to_vec(),
        machine: 1,
    };
    for id in 0..PLAYERS_MOST as u32 + 5 {
        apply(&mut world, &Change::PlayerAdded(id, player("p", &[])), 2);
    }
    assert_eq!(world.players.len(), PLAYERS_MOST);
    world.players.clear();
    let long = "n".repeat(NAME_MOST + 1);
    apply(&mut world, &Change::PlayerAdded(1, player(&long, &[])), 2);
    apply(
        &mut world,
        &Change::PlayerAdded(2, player("p", &[0; DATA_MOST + 1])),
        2,
    );
    assert!(world.players.is_empty());
    apply(&mut world, &Change::PlayerAdded(3, player("p", &[1])), 2);
    apply(
        &mut world,
        &Change::PlayerData(3, vec![0; DATA_MOST + 1]),
        2,
    );
    assert_eq!(world.players[&3].data, [1]);
}

#[test]
fn every_system_message_can_be_taken() {
    let mut world = World::default();
    let p = Player {
        name: "n".repeat(NAME_MOST),
        data: vec![0; DATA_MOST],
        machine: 1,
    };
    let out = apply(&mut world, &Change::PlayerAdded(1, p), 2);
    let data = apply(&mut world, &Change::PlayerData(1, vec![0; DATA_MOST]), 2);
    assert!(out.iter().chain(&data).all(|m| m.len() <= MESSAGE_MOST));
    assert!(!out.is_empty() && !data.is_empty());
}

// ---- the wire, frozen: bytes a build of this program sends to another ----

/// Dwords as little-endian bytes.
fn le(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// A free port: bound and let go.
fn free_port() -> u16 {
    UdpSocket::bind(("127.0.0.1", 0))
        .and_then(|u| u.local_addr())
        .map(|a| a.port())
        .expect("a free port")
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

/// A host on a free port (a port taken in between: another one), its port.
fn wire_host() -> (Arc<Session>, u16) {
    (0..10)
        .find_map(|_| {
            let port = free_port();
            Session::host(wire_desc(), port, Arc::new(quiet))
                .ok()
                .map(|s| (s, port))
        })
        .expect("a hosted session on one of ten free ports")
}

fn connect(port: u16) -> TcpStream {
    let stream = TcpStream::connect(("127.0.0.1", port)).expect("the host");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("a read timeout");
    stream
}

/// Guards the frame: its length as a little-endian dword, then the bytes; a longer one than
/// 1 MB is refused when read.
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
        .write_all(&((1u32 << 20) + 1).to_le_bytes())
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
    assert!(read_frame(&mut wrong).is_err());

    let mut peer = connect(port);
    write_frame(&mut peer, &join).unwrap();
    let welcome = read_frame(&mut peer).expect("the welcome");
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
        read_frame(&mut peer).unwrap(),
        read_frame(&mut peer).unwrap(),
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
    let delivered = read_frame(&mut peer).unwrap();
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
    let welcome = read_frame(&mut second).unwrap();
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
    let (host, port) = (0..10)
        .find_map(|_| {
            let port = free_port();
            Session::host(desc.clone(), port, Arc::new(quiet))
                .ok()
                .map(|s| (s, port))
        })
        .expect("a hosted session");
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    let answer = read_frame(&mut peer).unwrap();
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
    udp.set_read_timeout(Some(Duration::from_millis(700)))
        .unwrap();
    assert!(udp.recv_from(&mut buffer).is_err());
    host.close();
}

/// Guards each change's bytes and that they read back: the order of fields and the tags.
#[test]
fn every_change_has_its_bytes() {
    let player = Player {
        name: "p".into(),
        data: vec![9],
        machine: 2,
    };
    let group = Group {
        name: "g".into(),
        data: vec![],
        machine: 1,
        members: vec![5, 6],
    };
    let desc = wire_desc();
    let cases: Vec<(Change, Vec<u8>)> = vec![
        (
            Change::PlayerAdded(7, player),
            [
                vec![0],
                le(&[7, 1]),
                b"p".to_vec(),
                le(&[1]),
                vec![9],
                le(&[2]),
            ]
            .concat(),
        ),
        (Change::PlayerRemoved(7), [vec![1], le(&[7])].concat()),
        (
            Change::GroupAdded(8, group),
            [vec![2], le(&[8, 1]), b"g".to_vec(), le(&[0, 1, 2, 5, 6])].concat(),
        ),
        (Change::Joined(8, 7), [vec![4], le(&[8, 7])].concat()),
        (Change::Left(8, 7), [vec![5], le(&[8, 7])].concat()),
        (
            Change::PlayerData(7, vec![1, 2]),
            [vec![6], le(&[7, 2]), vec![1, 2]].concat(),
        ),
        (Change::Desc(desc), [vec![7], wire_desc_bytes(0)].concat()),
    ];
    for (change, bytes) in cases {
        assert_eq!(change.write(W::default()).0, bytes, "{change:?}");
        let read = Change::read(&mut R(&bytes)).expect("a change");
        assert_eq!(read, change);
    }
    assert!(Change::read(&mut R(&[3])).is_none());
}

/// Guards each request's bytes and that they read back.
#[test]
fn every_request_has_its_bytes() {
    let cases: Vec<(Op, Vec<u8>)> = vec![
        (
            Op::CreatePlayer("p".into(), vec![9]),
            [vec![0], le(&[1]), b"p".to_vec(), le(&[1]), vec![9]].concat(),
        ),
        (
            Op::CreateGroup("g".into(), vec![]),
            [vec![1], le(&[1]), b"g".to_vec(), le(&[0])].concat(),
        ),
        (Op::AddToGroup(8, 7), [vec![2], le(&[8, 7])].concat()),
        (Op::RemoveFromGroup(8, 7), [vec![3], le(&[8, 7])].concat()),
        (Op::DestroyPlayer(7), [vec![4], le(&[7])].concat()),
        (
            Op::SetPlayerData(7, vec![1, 2]),
            [vec![5], le(&[7, 2]), vec![1, 2]].concat(),
        ),
        (
            Op::SetDesc(wire_desc()),
            [vec![6], wire_desc_bytes(0)].concat(),
        ),
    ];
    for (op, bytes) in cases {
        assert_eq!(op.write(W::default()).0, bytes, "{op:?}");
        let read = Op::read(&mut R(&bytes)).expect("a request");
        assert_eq!(format!("{read:?}"), format!("{op:?}"));
    }
    assert!(Op::read(&mut R(&[7])).is_none());
}

/// Guards the system messages a program is given of another machine's doings: DirectPlay's
/// layouts, dword by dword, with the data after them.
#[test]
fn the_system_messages_keep_directplay_layout() {
    let mut world = World::default();
    let player = Player {
        name: "p".into(),
        data: vec![9, 8],
        machine: 2,
    };
    let name = [0x10, 0, 0, 0];
    let made = apply(&mut world, &Change::PlayerAdded(7, player.clone()), 1);
    let expected = [
        le(&[DPSYS_CREATEPLAYERORGROUP, 1, 7, 1, 0, 2]),
        le(&name),
        le(&[0, 0]),
        vec![9, 8],
    ]
    .concat();
    assert_eq!(made, [expected]);
    // A player of this machine's own is no message.
    assert!(
        apply(
            &mut world,
            &Change::PlayerAdded(
                8,
                Player {
                    machine: 1,
                    ..player
                }
            ),
            1
        )
        .is_empty()
    );
    let group = Group {
        name: "g".into(),
        data: vec![],
        machine: 2,
        members: vec![],
    };
    let made = apply(&mut world, &Change::GroupAdded(9, group), 1);
    let expected = [
        le(&[DPSYS_CREATEPLAYERORGROUP, 0, 9, 2, 0, 0]),
        le(&name),
        le(&[0, 0]),
    ]
    .concat();
    assert_eq!(made, [expected]);
    let made = apply(&mut world, &Change::Joined(9, 7), 1);
    assert_eq!(made, [le(&[DPSYS_ADDPLAYERTOGROUP, 9, 7])]);
    let made = apply(&mut world, &Change::PlayerData(7, vec![1]), 1);
    assert_eq!(
        made,
        [[le(&[DPSYS_SETPLAYERORGROUPDATA, 1, 7, 0, 1]), vec![1]].concat()]
    );
    let made = apply(&mut world, &Change::Left(9, 7), 1);
    assert_eq!(made, [le(&[DPSYS_DELETEPLAYERFROMGROUP, 9, 7])]);
    let made = apply(&mut world, &Change::PlayerRemoved(7), 1);
    let expected = [
        le(&[DPSYS_DESTROYPLAYERORGROUP, 1, 7, 0, 0, 0, 0]),
        le(&name),
        le(&[0, 0]),
    ]
    .concat();
    assert_eq!(made, [expected]);
}

/// Guards the frame a host sends each joined machine when it closes: BYE alone.
#[test]
fn the_hosts_closing_frame_is_frozen() {
    let (host, port) = wire_host();
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    read_frame(&mut peer).expect("the welcome");
    host.close();
    assert_eq!(read_frame(&mut peer).expect("the bye"), vec![BYE]);
}

/// A joiner of another version of the protocol is refused with `DPERR_VERSION` (the first
/// version's mark `RC2KNET1` is one), not dropped silently.
#[test]
fn another_version_is_refused_with_its_result() {
    let (host, port) = wire_host();
    let mut old = connect(port);
    write_frame(&mut old, &[vec![JOIN], b"RC2KNET1".to_vec()].concat()).unwrap();
    assert_eq!(
        read_frame(&mut old).expect("the refusal"),
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
    read_frame(&mut peer).expect("the welcome");
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
        let frame = read_frame(&mut peer).expect("a frame");
        if frame[0] == REPLY {
            player = Some(u32::from_le_bytes(frame[9..13].try_into().unwrap()));
            break;
        }
    }
    let player = player.expect("the reply with the player");
    assert_eq!(host.kick(0x7777), DPERR_INVALIDPLAYER);
    assert_eq!(host.kick(player), DP_OK);
    let mut kicked = false;
    while let Ok(frame) = read_frame(&mut peer) {
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
    let (host, port) = (0..10)
        .find_map(|_| {
            let port = free_port();
            Session::host_locked(wire_desc(), port, Arc::new(quiet), b"rally")
                .ok()
                .map(|s| (s, port))
        })
        .expect("a locked session on one of ten free ports");
    assert_ne!(host.world().desc.flags & DPSESSION_PASSWORDREQUIRED, 0);
    let at = SocketAddr::from(([127, 0, 0, 1], port));
    // A bare join is asked for a proof, and the plain password is nowhere in what is sent.
    let mut peer = connect(port);
    write_frame(&mut peer, &[vec![JOIN], MAGIC.to_vec()].concat()).unwrap();
    let asked = read_frame(&mut peer).expect("the challenge");
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
