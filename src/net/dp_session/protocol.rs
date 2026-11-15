//! The wire protocol: frames, the ops a machine asks of the host, the changes the host sends to
//! every machine, and how a change is applied to the session's world.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::*;

#[derive(Default)]
pub(super) struct W(pub(super) Vec<u8>);

impl W {
    pub(super) fn u8(mut self, v: u8) -> Self {
        self.0.push(v);
        self
    }
    pub(super) fn u32(mut self, v: u32) -> Self {
        self.0.extend(v.to_le_bytes());
        self
    }
    pub(super) fn raw(mut self, v: &[u8]) -> Self {
        self.0.extend(v);
        self
    }
    pub(super) fn bytes(self, v: &[u8]) -> Self {
        self.u32(v.len() as u32).raw(v)
    }
    pub(super) fn str(self, v: &str) -> Self {
        self.bytes(v.as_bytes())
    }
    pub(super) fn desc(self, d: &Desc) -> Self {
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
    pub(super) fn player(self, p: &Player) -> Self {
        self.str(&p.name).bytes(&p.data).u32(p.machine)
    }
    pub(super) fn group(self, g: &Group) -> Self {
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

pub(super) struct R<'a>(pub(super) &'a [u8]);

impl R<'_> {
    pub(super) fn u8(&mut self) -> Option<u8> {
        let (&v, rest) = self.0.split_first()?;
        self.0 = rest;
        Some(v)
    }
    pub(super) fn u32(&mut self) -> Option<u32> {
        let v = self.0.get(..4)?;
        let v = u32::from_le_bytes(v.try_into().ok()?);
        self.0 = &self.0[4..];
        Some(v)
    }
    pub(super) fn raw<const N: usize>(&mut self) -> Option<[u8; N]> {
        let v: [u8; N] = self.0.get(..N)?.try_into().ok()?;
        self.0 = &self.0[N..];
        Some(v)
    }
    pub(super) fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = self.u32()? as usize;
        let v = self.0.get(..n)?.to_vec();
        self.0 = &self.0[n..];
        Some(v)
    }
    pub(super) fn str(&mut self) -> Option<String> {
        String::from_utf8(self.bytes()?).ok()
    }
    pub(super) fn desc(&mut self) -> Option<Desc> {
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
    pub(super) fn player(&mut self) -> Option<Player> {
        Some(Player {
            name: self.str()?,
            data: self.bytes()?,
            machine: self.u32()?,
        })
    }
    pub(super) fn group(&mut self) -> Option<Group> {
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
pub(super) enum Change {
    PlayerAdded(Id, Player),
    PlayerRemoved(Id),
    GroupAdded(Id, Group),
    Joined(Id, Id),
    Left(Id, Id),
    PlayerData(Id, Vec<u8>),
    Desc(Desc),
}

impl Change {
    pub(super) fn write(&self, w: W) -> W {
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
    pub(super) fn read(r: &mut R) -> Option<Change> {
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
pub(super) enum Op {
    CreatePlayer(String, Vec<u8>),
    CreateGroup(String, Vec<u8>),
    AddToGroup(Id, Id),
    RemoveFromGroup(Id, Id),
    DestroyPlayer(Id),
    SetPlayerData(Id, Vec<u8>),
    SetDesc(Desc),
}

impl Op {
    pub(super) fn write(&self, w: W) -> W {
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
    pub(super) fn read(r: &mut R) -> Option<Op> {
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
pub(super) const JOIN: u8 = 1;
pub(super) const WELCOME: u8 = 2;
pub(super) const REFUSE: u8 = 3;
pub(super) const REQUEST: u8 = 4;
pub(super) const REPLY: u8 = 5;
pub(super) const CHANGE: u8 = 6;
pub(super) const SEND: u8 = 7;
pub(super) const DELIVER: u8 = 8;
pub(super) const BYE: u8 = 9;

/// The protocol's mark at the start of a JOIN and of the discovery's packets.
pub(super) const MAGIC: &[u8; 8] = b"RC2KNET1";

// What another machine can make this one hold (nothing the game needs comes near them).
/// The longest message a program takes (the game's buffer, `net::receive_message::ROOM`): a
/// longer one could never be taken and would hold up all behind it.
pub const MESSAGE_MOST: usize = 0x100;
/// The messages waiting for the program; others are dropped.
pub(super) const WAITING_MOST: usize = 0x1_0000;
pub(super) const PLAYERS_MOST: usize = 0x40;
pub(super) const GROUPS_MOST: usize = 0x40;
pub(super) const NAME_MOST: usize = 0x100;
pub(super) const DATA_MOST: usize = 0x80;
/// The host's connections, those not yet joined among them.
pub(super) const CONNECTIONS_MOST: usize = 0x40;
/// A machine that takes no more for this long has its connection ended (each has a thread
/// that reads whatever comes, so a machine busy loading does not stop reading).
pub(super) const WRITE_WAIT: Duration = Duration::from_secs(5);

pub(super) fn write_frame(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
    let mut frame = (bytes.len() as u32).to_le_bytes().to_vec();
    frame.extend(bytes);
    stream.write_all(&frame)
}

pub(super) fn read_frame(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut n = [0u8; 4];
    stream.read_exact(&mut n)?;
    let n = u32::from_le_bytes(n) as usize;
    if n > 1 << 20 {
        return Err(std::io::Error::other("a frame past 1 MB"));
    }
    let mut v = vec![0u8; n];
    stream.read_exact(&mut v)?;
    Ok(v)
}

// ---- system messages, as DirectPlay laid them out (its pointers 0) ----

pub(super) fn dpname() -> [u32; 4] {
    [0x10, 0, 0, 0]
}

pub(super) fn system(words: &[u32], tail: &[u8]) -> Vec<u8> {
    let mut v: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    v.extend(tail);
    v
}

/// A name and data within what a session holds.
pub(super) fn fits(name: &str, data: &[u8]) -> bool {
    name.len() <= NAME_MOST && data.len() <= DATA_MOST
}

/// Applies a change; the system messages it makes for this machine's program (of what
/// another machine did).
pub(super) fn apply(world: &mut World, change: &Change, me: u32) -> Vec<Vec<u8>> {
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
