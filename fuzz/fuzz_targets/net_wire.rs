#![no_main]

// What another machine can send a session: frames, requests, changes, DirectPlay's description
// and the game's own messages. Whatever the bytes, reading them gives a value or nothing, and a
// change applied to a world keeps the world within its limits.
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    use rc2k_core::net::message::{Message, Payload, SystemMessage};
    use rc2k_core::net::protocol::{
        Change, GROUPS_MOST, Op, PLAYERS_MOST, R, World, apply, read_frame,
    };
    let mut stream = data;
    while let Ok(frame) = read_frame(&mut stream) {
        let mut r = R(&frame);
        let _ = Op::read(&mut r);
        let mut world = World::default();
        let mut r = R(&frame);
        if let Some(change) = Change::read(&mut r) {
            let _ = apply(&mut world, &change, 1);
            assert!(world.players.len() <= PLAYERS_MOST && world.groups.len() <= GROUPS_MOST);
        }
        let mut r = R(&frame);
        let _ = r.desc();
        let _ = R(&frame).player();
        let _ = R(&frame).group();
    }
    let message = Message::parse(data);
    // A message parsed and written again gives its bytes back.
    assert_eq!(message.encode(), data);
    let _ = Payload::parse(data);
    let _ = SystemMessage::parse(data);
});
