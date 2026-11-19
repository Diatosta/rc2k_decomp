//! The network sessions' wire protocol: frames, the ops a machine asks of the host, the changes
//! the host sends to every machine, how a change is applied to the session's world, and the
//! limits no machine's message may pass. No socket, thread or clock is in it: the sessions in
//! `net::dp_session` carry these bytes.

use std::collections::BTreeMap;
use std::io::{Read, Write};

/// A player's or group's number in a session.
pub type Id = u32;

// DirectPlay's system messages (their first dword).
pub const DPSYS_CREATEPLAYERORGROUP: u32 = 0x3;
pub const DPSYS_DESTROYPLAYERORGROUP: u32 = 0x5;
pub const DPSYS_ADDPLAYERTOGROUP: u32 = 0x7;
pub const DPSYS_DELETEPLAYERFROMGROUP: u32 = 0x21;
pub const DPSYS_SESSIONLOST: u32 = 0x31;
pub const DPSYS_SETPLAYERORGROUPDATA: u32 = 0x102;

/// A session's description (DirectPlay's `DPSESSIONDESC2` but the password).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Desc {
    pub flags: u32,
    pub instance: [u8; 16],
    pub application: [u8; 16],
    pub max_players: u32,
    pub current_players: u32,
    pub name: String,
    pub user: [u32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub name: String,
    pub data: Vec<u8>,
    pub machine: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub name: String,
    pub data: Vec<u8>,
    pub machine: u32,
    pub members: Vec<Id>,
}

/// What every machine holds of the session.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct World {
    pub desc: Desc,
    pub players: BTreeMap<Id, Player>,
    pub groups: BTreeMap<Id, Group>,
}

#[derive(Default)]
pub struct W(pub Vec<u8>);

impl W {
    pub fn u8(mut self, v: u8) -> Self {
        self.0.push(v);
        self
    }
    pub fn u32(mut self, v: u32) -> Self {
        self.0.extend(v.to_le_bytes());
        self
    }
    pub fn raw(mut self, v: &[u8]) -> Self {
        self.0.extend(v);
        self
    }
    pub fn bytes(self, v: &[u8]) -> Self {
        self.u32(v.len() as u32).raw(v)
    }
    pub fn str(self, v: &str) -> Self {
        self.bytes(v.as_bytes())
    }
    pub fn desc(self, d: &Desc) -> Self {
        let mut w = self
            .u32(d.flags)
            .raw(&d.instance)
            .raw(&d.application)
            .u32(d.max_players)
            .u32(d.current_players)
            .str(&d.name);
        for u in d.user {
            w = w.u32(u);
        }
        w
    }
    pub fn player(self, p: &Player) -> Self {
        self.str(&p.name).bytes(&p.data).u32(p.machine)
    }
    pub fn group(self, g: &Group) -> Self {
        let mut w = self
            .str(&g.name)
            .bytes(&g.data)
            .u32(g.machine)
            .u32(g.members.len() as u32);
        for &m in &g.members {
            w = w.u32(m);
        }
        w
    }
}

pub struct R<'a>(pub &'a [u8]);

impl R<'_> {
    pub fn u8(&mut self) -> Option<u8> {
        let (&v, rest) = self.0.split_first()?;
        self.0 = rest;
        Some(v)
    }
    pub fn u32(&mut self) -> Option<u32> {
        let v = self.0.get(..4)?;
        let v = u32::from_le_bytes(v.try_into().ok()?);
        self.0 = &self.0[4..];
        Some(v)
    }
    pub fn raw<const N: usize>(&mut self) -> Option<[u8; N]> {
        let v: [u8; N] = self.0.get(..N)?.try_into().ok()?;
        self.0 = &self.0[N..];
        Some(v)
    }
    pub fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = self.u32()? as usize;
        let v = self.0.get(..n)?.to_vec();
        self.0 = &self.0[n..];
        Some(v)
    }
    pub fn str(&mut self) -> Option<String> {
        String::from_utf8(self.bytes()?).ok()
    }
    pub fn desc(&mut self) -> Option<Desc> {
        Some(Desc {
            flags: self.u32()?,
            instance: self.raw()?,
            application: self.raw()?,
            max_players: self.u32()?,
            current_players: self.u32()?,
            name: self.str()?,
            user: [self.u32()?, self.u32()?, self.u32()?, self.u32()?],
        })
    }
    pub fn player(&mut self) -> Option<Player> {
        Some(Player {
            name: self.str()?,
            data: self.bytes()?,
            machine: self.u32()?,
        })
    }
    pub fn group(&mut self) -> Option<Group> {
        let (name, data, machine) = (self.str()?, self.bytes()?, self.u32()?);
        let n = self.u32()? as usize;
        let mut members = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            members.push(self.u32()?);
        }
        Some(Group {
            name,
            data,
            machine,
            members,
        })
    }
}

/// A change to the session, which the host makes and every machine applies.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    PlayerAdded(Id, Player),
    PlayerRemoved(Id),
    GroupAdded(Id, Group),
    Joined(Id, Id),
    Left(Id, Id),
    PlayerData(Id, Vec<u8>),
    Desc(Desc),
}

impl Change {
    pub fn write(&self, w: W) -> W {
        match self {
            Change::PlayerAdded(id, p) => w.u8(0).u32(*id).player(p),
            Change::PlayerRemoved(id) => w.u8(1).u32(*id),
            Change::GroupAdded(id, g) => w.u8(2).u32(*id).group(g),
            Change::Joined(g, p) => w.u8(4).u32(*g).u32(*p),
            Change::Left(g, p) => w.u8(5).u32(*g).u32(*p),
            Change::PlayerData(id, d) => w.u8(6).u32(*id).bytes(d),
            Change::Desc(d) => w.u8(7).desc(d),
        }
    }
    pub fn read(r: &mut R) -> Option<Change> {
        Some(match r.u8()? {
            0 => Change::PlayerAdded(r.u32()?, r.player()?),
            1 => Change::PlayerRemoved(r.u32()?),
            2 => Change::GroupAdded(r.u32()?, r.group()?),
            4 => Change::Joined(r.u32()?, r.u32()?),
            5 => Change::Left(r.u32()?, r.u32()?),
            6 => Change::PlayerData(r.u32()?, r.bytes()?),
            7 => Change::Desc(r.desc()?),
            _ => return None,
        })
    }
}

/// What a machine asks the host to do.
#[derive(Clone, Debug)]
pub enum Op {
    CreatePlayer(String, Vec<u8>),
    CreateGroup(String, Vec<u8>),
    AddToGroup(Id, Id),
    RemoveFromGroup(Id, Id),
    DestroyPlayer(Id),
    SetPlayerData(Id, Vec<u8>),
    SetDesc(Desc),
}

impl Op {
    pub fn write(&self, w: W) -> W {
        match self {
            Op::CreatePlayer(n, d) => w.u8(0).str(n).bytes(d),
            Op::CreateGroup(n, d) => w.u8(1).str(n).bytes(d),
            Op::AddToGroup(g, p) => w.u8(2).u32(*g).u32(*p),
            Op::RemoveFromGroup(g, p) => w.u8(3).u32(*g).u32(*p),
            Op::DestroyPlayer(p) => w.u8(4).u32(*p),
            Op::SetPlayerData(p, d) => w.u8(5).u32(*p).bytes(d),
            Op::SetDesc(d) => w.u8(6).desc(d),
        }
    }
    pub fn read(r: &mut R) -> Option<Op> {
        Some(match r.u8()? {
            0 => Op::CreatePlayer(r.str()?, r.bytes()?),
            1 => Op::CreateGroup(r.str()?, r.bytes()?),
            2 => Op::AddToGroup(r.u32()?, r.u32()?),
            3 => Op::RemoveFromGroup(r.u32()?, r.u32()?),
            4 => Op::DestroyPlayer(r.u32()?),
            5 => Op::SetPlayerData(r.u32()?, r.bytes()?),
            6 => Op::SetDesc(r.desc()?),
            _ => return None,
        })
    }
}

// The frames' first bytes.
pub const JOIN: u8 = 1;
pub const WELCOME: u8 = 2;
pub const REFUSE: u8 = 3;
pub const REQUEST: u8 = 4;
pub const REPLY: u8 = 5;
pub const CHANGE: u8 = 6;
pub const SEND: u8 = 7;
pub const DELIVER: u8 = 8;
pub const BYE: u8 = 9;
/// Host to a machine: it has been put out of the session (the host ends its connection after).
pub const KICKED: u8 = 10;
/// Host to a joining machine of a locked session: the password's salt and a fresh challenge.
pub const CHALLENGE: u8 = 11;
/// A joining machine to the host: the proof for the challenge.
pub const PROOF: u8 = 12;
/// Host to a machine: a round trip's start (the host's clock, in ms); answered with `PONG`.
pub const PING: u8 = 13;
/// A machine to the host: the `PING`'s stamp given back.
pub const PONG: u8 = 14;
/// Host to every machine: each machine's round trip to the host in ms (a count, then machine
/// and ms for each).
pub const PINGS: u8 = 15;

/// The protocol's mark at the start of a JOIN and of the discovery's packets. The last
/// character is the protocol's version: 2 added the host putting a player out (`KICKED`), 3 the sessions locked by a
/// password (`CHALLENGE`, `PROOF`), 4 the host ordering the lobby chat (`messages::CHAT_RELAYED`),
/// 5 the round trips measured (`PING`, `PONG`, `PINGS`), 6 the host's messages for the bots'
/// cars (`messages::BOT_RACE`). A frame of a kind a machine does not know is passed over.
pub const MAGIC: &[u8; 8] = b"RC2KNET6";
/// The first seven characters of every version's mark: a JOIN that starts so but is not this
/// version's is answered with a refusal for its version, not dropped.
pub const MAGIC_FAMILY: &[u8; 7] = b"RC2KNET";

// What another machine can make this one hold (nothing the game needs comes near them).
/// The longest message a program takes (the game's buffer, `net::receive_message::ROOM`): a
/// longer one could never be taken and would hold up all behind it.
pub const MESSAGE_MOST: usize = 0x100;
/// The messages waiting for the program; others are dropped.
pub const WAITING_MOST: usize = 0x1_0000;
pub const PLAYERS_MOST: usize = 0x40;
pub const GROUPS_MOST: usize = 0x40;
pub const NAME_MOST: usize = 0x100;
pub const DATA_MOST: usize = 0x80;
/// The host's connections, those not yet joined among them.
pub const CONNECTIONS_MOST: usize = 0x40;

pub fn write_frame(stream: &mut (impl Write + ?Sized), bytes: &[u8]) -> std::io::Result<()> {
    let mut frame = (bytes.len() as u32).to_le_bytes().to_vec();
    frame.extend(bytes);
    stream.write_all(&frame)
}

/// The longest frame taken: above the welcome of a full session (64 players and 64 groups, each at
/// the longest names, data and members), far below what a length field can claim.
pub const FRAME_MOST: usize = 0x2_0000;

/// A frame's bytes are read as they arrive and held only as far as they have come, so a peer
/// that announces a long frame and sends nothing costs no memory.
pub fn read_frame(stream: &mut (impl Read + ?Sized)) -> std::io::Result<Vec<u8>> {
    let mut n = [0u8; 4];
    stream.read_exact(&mut n)?;
    let n = u32::from_le_bytes(n) as usize;
    if n > FRAME_MOST {
        return Err(std::io::Error::other("a frame past its limit"));
    }
    let mut v = Vec::with_capacity(n.min(0x1000));
    stream.take(n as u64).read_to_end(&mut v)?;
    if v.len() != n {
        return Err(std::io::ErrorKind::UnexpectedEof.into());
    }
    Ok(v)
}

// ---- system messages, as DirectPlay laid them out (its pointers 0) ----

pub fn dpname() -> [u32; 4] {
    [0x10, 0, 0, 0]
}

pub fn system(words: &[u32], tail: &[u8]) -> Vec<u8> {
    let mut v: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    v.extend(tail);
    v
}

/// A name and data within what a session holds.
pub fn fits(name: &str, data: &[u8]) -> bool {
    name.len() <= NAME_MOST && data.len() <= DATA_MOST
}

/// Applies a change; the system messages it makes for this machine's program (of what
/// another machine did).
pub fn apply(world: &mut World, change: &Change, me: u32) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    match change {
        Change::PlayerAdded(_, p)
            if world.players.len() >= PLAYERS_MOST || !fits(&p.name, &p.data) => {}
        Change::GroupAdded(_, g)
            if world.groups.len() >= GROUPS_MOST || !fits(&g.name, &g.data) => {}
        Change::PlayerData(_, d) if d.len() > DATA_MOST => {}
        Change::PlayerAdded(id, p) => {
            world.players.insert(*id, p.clone());
            if p.machine != me {
                let n = dpname();
                out.push(system(
                    &[
                        DPSYS_CREATEPLAYERORGROUP,
                        1,
                        *id,
                        world.players.len() as u32,
                        0,
                        p.data.len() as u32,
                        n[0],
                        n[1],
                        n[2],
                        n[3],
                        0,
                        0,
                    ],
                    &p.data,
                ));
            }
        }
        Change::PlayerRemoved(id) => {
            if let Some(p) = world.players.remove(id) {
                for g in world.groups.values_mut() {
                    g.members.retain(|m| m != id);
                }
                if p.machine != me {
                    let n = dpname();
                    out.push(system(
                        &[
                            DPSYS_DESTROYPLAYERORGROUP,
                            1,
                            *id,
                            0,
                            0,
                            0,
                            0,
                            n[0],
                            n[1],
                            n[2],
                            n[3],
                            0,
                            0,
                        ],
                        &[],
                    ));
                }
            }
        }
        Change::GroupAdded(id, g) => {
            world.groups.insert(*id, g.clone());
            if g.machine != me {
                let n = dpname();
                out.push(system(
                    &[
                        DPSYS_CREATEPLAYERORGROUP,
                        0,
                        *id,
                        world.players.len() as u32,
                        0,
                        g.data.len() as u32,
                        n[0],
                        n[1],
                        n[2],
                        n[3],
                        0,
                        0,
                    ],
                    &g.data,
                ));
            }
        }
        Change::Joined(g, p) => {
            if let Some(group) = world.groups.get_mut(g)
                && !group.members.contains(p)
                && group.members.len() < PLAYERS_MOST
            {
                group.members.push(*p);
            }
            if world.players.get(p).is_some_and(|x| x.machine != me) {
                out.push(system(&[DPSYS_ADDPLAYERTOGROUP, *g, *p], &[]));
            }
        }
        Change::Left(g, p) => {
            if let Some(group) = world.groups.get_mut(g) {
                group.members.retain(|m| m != p);
            }
            if world.players.get(p).is_some_and(|x| x.machine != me) {
                out.push(system(&[DPSYS_DELETEPLAYERFROMGROUP, *g, *p], &[]));
            }
        }
        Change::PlayerData(id, d) => {
            if let Some(p) = world.players.get_mut(id) {
                p.data = d.clone();
                if p.machine != me {
                    out.push(system(
                        &[DPSYS_SETPLAYERORGROUPDATA, 1, *id, 0, d.len() as u32],
                        d,
                    ));
                }
            }
        }
        Change::Desc(d) => world.desc = d.clone(),
    }
    world.desc.current_players = world.players.len() as u32;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dwords as little-endian bytes.
    fn le(words: &[u32]) -> Vec<u8> {
        words.iter().flat_map(|w| w.to_le_bytes()).collect()
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

    /// Guards the frame: its length as a little-endian dword, then the bytes; a longer one than
    /// `FRAME_MOST` is refused when read; the bytes are taken as they come.
    #[test]
    fn a_frame_is_its_length_and_its_bytes() {
        let mut wire = Vec::new();
        write_frame(&mut wire, b"abc").unwrap();
        assert_eq!(wire, [3, 0, 0, 0, b'a', b'b', b'c']);
        write_frame(&mut wire, &[9, 8]).unwrap();
        let mut read = std::io::Cursor::new(wire);
        assert_eq!(read_frame(&mut read).unwrap(), b"abc");
        assert_eq!(read_frame(&mut read).unwrap(), [9, 8]);
        // Nothing more: the end of the stream is an error, not an empty frame.
        assert!(read_frame(&mut read).is_err());
        let mut long = std::io::Cursor::new(((FRAME_MOST as u32) + 1).to_le_bytes().to_vec());
        assert!(read_frame(&mut long).is_err());
        // A frame announced and cut short is an error too, with only what came held.
        let mut cut =
            std::io::Cursor::new([[0x00, 0x00, 0x01, 0x00].to_vec(), vec![7; 10]].concat());
        assert_eq!(
            read_frame(&mut cut).unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
        // The longest frame is taken whole.
        let mut wire = Vec::new();
        write_frame(&mut wire, &vec![5u8; FRAME_MOST]).unwrap();
        assert_eq!(
            read_frame(&mut std::io::Cursor::new(wire)).unwrap().len(),
            FRAME_MOST
        );
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
}
