//! The network sessions under DirectPlay's objects made in Rust (`net::dp_objects`), over a
//! protocol of its own.
//!
//! - A session's host keeps it: its description, the players (the host is machine 1, each joined
//!   machine the next number) and the groups. Every change is the host's to make: the others ask
//!   it, and it sends each change to every machine. Messages go to the host, which hands them to
//!   their addressees (a player, a group's players, or all; never back to the sender).
//! - Machines talk over TCP to the host's port; a session is found by a UDP probe to the same
//!   port (to the address asked, or broadcast and to this machine), which the host answers with
//!   its description.
//! - Others' doings come as DirectPlay system messages (a player or group made or gone, a player
//!   added to or removed from a group, a player's data set); the session is lost when the host's
//!   connection ends (the host does not pass to another machine).
//!
//! Nothing here knows the game: it is tested on its own over loopback.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub type Id = u32;

/// DirectPlay's TCP/IP port, when the game gives none.
pub const DEFAULT_PORT: u16 = 47624;

// DirectPlay's results (MAKE_DPHRESULT: 0x88770000 + the number).
pub const DP_OK: u32 = 0;
pub const DPERR_ACCESSDENIED: u32 = 0x8877_000A;
pub const DPERR_CANTADDPLAYER: u32 = 0x8877_0028;
pub const DPERR_INVALIDPLAYER: u32 = 0x8877_0096;
pub const DPERR_INVALIDGROUP: u32 = 0x8877_009B;
pub const DPERR_NOCONNECTION: u32 = 0x8877_00AA;
pub const DPERR_TIMEOUT: u32 = 0x8877_00F0;
pub const DPERR_SESSIONLOST: u32 = 0x8877_0136;
pub const DPERR_NONEWPLAYERS: u32 = 0x8877_014A;

// The session's flags the host looks at.
pub const DPSESSION_NEWPLAYERSDISABLED: u32 = 0x1;
pub const DPSESSION_JOINDISABLED: u32 = 0x20;

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

/// A message for this machine's players: from a player or 0 (a system message), to the
/// player it is for.
#[derive(Clone, Debug, PartialEq)]
pub struct Incoming {
    pub from: Id,
    pub to: Id,
    pub data: Vec<u8>,
}

/// Signals a player's event (a Windows event's handle, in the standalone program).
pub type Signal = Arc<dyn Fn(u32) + Send + Sync>;

// ---- the wire: a frame is its length (4 bytes) and its bytes ----

#[derive(Default)]
struct W(Vec<u8>);

impl W {
    fn u8(mut self, v: u8) -> Self {
        self.0.push(v);
        self
    }
    fn u32(mut self, v: u32) -> Self {
        self.0.extend(v.to_le_bytes());
        self
    }
    fn raw(mut self, v: &[u8]) -> Self {
        self.0.extend(v);
        self
    }
    fn bytes(self, v: &[u8]) -> Self {
        self.u32(v.len() as u32).raw(v)
    }
    fn str(self, v: &str) -> Self {
        self.bytes(v.as_bytes())
    }
    fn desc(self, d: &Desc) -> Self {
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
    fn player(self, p: &Player) -> Self {
        self.str(&p.name).bytes(&p.data).u32(p.machine)
    }
    fn group(self, g: &Group) -> Self {
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

struct R<'a>(&'a [u8]);

impl R<'_> {
    fn u8(&mut self) -> Option<u8> {
        let (&v, rest) = self.0.split_first()?;
        self.0 = rest;
        Some(v)
    }
    fn u32(&mut self) -> Option<u32> {
        let v = self.0.get(..4)?;
        let v = u32::from_le_bytes(v.try_into().ok()?);
        self.0 = &self.0[4..];
        Some(v)
    }
    fn raw<const N: usize>(&mut self) -> Option<[u8; N]> {
        let v: [u8; N] = self.0.get(..N)?.try_into().ok()?;
        self.0 = &self.0[N..];
        Some(v)
    }
    fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = self.u32()? as usize;
        let v = self.0.get(..n)?.to_vec();
        self.0 = &self.0[n..];
        Some(v)
    }
    fn str(&mut self) -> Option<String> {
        String::from_utf8(self.bytes()?).ok()
    }
    fn desc(&mut self) -> Option<Desc> {
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
    fn player(&mut self) -> Option<Player> {
        Some(Player {
            name: self.str()?,
            data: self.bytes()?,
            machine: self.u32()?,
        })
    }
    fn group(&mut self) -> Option<Group> {
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
enum Change {
    PlayerAdded(Id, Player),
    PlayerRemoved(Id),
    GroupAdded(Id, Group),
    Joined(Id, Id),
    Left(Id, Id),
    PlayerData(Id, Vec<u8>),
    Desc(Desc),
}

impl Change {
    fn write(&self, w: W) -> W {
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
    fn read(r: &mut R) -> Option<Change> {
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
enum Op {
    CreatePlayer(String, Vec<u8>),
    CreateGroup(String, Vec<u8>),
    AddToGroup(Id, Id),
    RemoveFromGroup(Id, Id),
    DestroyPlayer(Id),
    SetPlayerData(Id, Vec<u8>),
    SetDesc(Desc),
}

impl Op {
    fn write(&self, w: W) -> W {
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
    fn read(r: &mut R) -> Option<Op> {
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
const JOIN: u8 = 1;
const WELCOME: u8 = 2;
const REFUSE: u8 = 3;
const REQUEST: u8 = 4;
const REPLY: u8 = 5;
const CHANGE: u8 = 6;
const SEND: u8 = 7;
const DELIVER: u8 = 8;
const BYE: u8 = 9;

/// The protocol's mark at the start of a JOIN and of the discovery's packets.
const MAGIC: &[u8; 8] = b"RC2KNET1";

// What another machine can make this one hold (nothing the game needs comes near them).
/// The longest message a program takes (the game's buffer, `net::receive_message::ROOM`): a
/// longer one could never be taken and would hold up all behind it.
pub const MESSAGE_MOST: usize = 0x100;
/// The messages waiting for the program; others are dropped.
const WAITING_MOST: usize = 0x1_0000;
const PLAYERS_MOST: usize = 0x40;
const GROUPS_MOST: usize = 0x40;
const NAME_MOST: usize = 0x100;
const DATA_MOST: usize = 0x80;
/// The host's connections, those not yet joined among them.
const CONNECTIONS_MOST: usize = 0x40;
/// A machine that takes no more for this long has its connection ended (each has a thread
/// that reads whatever comes, so a machine busy loading does not stop reading).
const WRITE_WAIT: Duration = Duration::from_secs(5);

fn write_frame(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
    let mut frame = (bytes.len() as u32).to_le_bytes().to_vec();
    frame.extend(bytes);
    stream.write_all(&frame)
}

fn read_frame(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
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

fn dpname() -> [u32; 4] {
    [0x10, 0, 0, 0]
}

fn system(words: &[u32], tail: &[u8]) -> Vec<u8> {
    let mut v: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    v.extend(tail);
    v
}

/// A name and data within what a session holds.
fn fits(name: &str, data: &[u8]) -> bool {
    name.len() <= NAME_MOST && data.len() <= DATA_MOST
}

/// Applies a change; the system messages it makes for this machine's program (of what
/// another machine did).
fn apply(world: &mut World, change: &Change, me: u32) -> Vec<Vec<u8>> {
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

// ---- a session ----

struct Shared {
    world: World,
    queue: VecDeque<Incoming>,
    /// This machine's players' events.
    events: HashMap<Id, u32>,
    replies: HashMap<u32, (u32, Id)>,
    next_request: u32,
    next_id: Id,
    next_machine: u32,
    lost: bool,
    /// A message has been dropped (told once in the log).
    dropped: bool,
}

impl Shared {
    /// A message for the program, unless too long to take or too many wait.
    fn wait(&mut self, m: Incoming) {
        if m.data.len() <= MESSAGE_MOST && self.queue.len() < WAITING_MOST {
            self.queue.push_back(m);
        } else if !self.dropped {
            self.dropped = true;
            crate::framework::log(&format!(
                "net: dropping messages from another machine ({} bytes, {} waiting)",
                m.data.len(),
                self.queue.len()
            ));
        }
    }
}

pub struct Session {
    me: u32,
    shared: Mutex<Shared>,
    answered: Condvar,
    signal: Signal,
    stop: AtomicBool,
    /// The host: each joined machine's connection. A joined machine: the host's (as 1).
    links: Mutex<HashMap<u32, TcpStream>>,
    port: u16,
    /// The host's connections open.
    connections: AtomicUsize,
}

impl Session {
    fn new(me: u32, world: World, signal: Signal, port: u16) -> Arc<Session> {
        Arc::new(Session {
            me,
            shared: Mutex::new(Shared {
                world,
                queue: VecDeque::new(),
                events: HashMap::new(),
                replies: HashMap::new(),
                next_request: 1,
                next_id: 0x0010_0001,
                next_machine: 2,
                lost: false,
                dropped: false,
            }),
            answered: Condvar::new(),
            signal,
            stop: AtomicBool::new(false),
            links: Mutex::new(HashMap::new()),
            connections: AtomicUsize::new(0),
            port,
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn is_host(&self) -> bool {
        self.me == 1
    }

    /// The host's session made, at `port` (its TCP connections and its UDP answers).
    pub fn host(mut desc: Desc, port: u16, signal: Signal) -> std::io::Result<Arc<Session>> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        let udp = UdpSocket::bind(("0.0.0.0", port))?;
        udp.set_read_timeout(Some(Duration::from_millis(300)))?;
        desc.current_players = 0;
        let session = Session::new(
            1,
            World {
                desc,
                ..Default::default()
            },
            signal,
            port,
        );
        let s = session.clone();
        std::thread::spawn(move || s.accept(listener));
        let s = session.clone();
        std::thread::spawn(move || s.answer_probes(udp));
        Ok(session)
    }

    /// Joins the session whose host is at `at`: the session, or DirectPlay's result.
    pub fn join(at: SocketAddr, signal: Signal) -> Result<Arc<Session>, u32> {
        let mut stream = TcpStream::connect_timeout(&at, Duration::from_secs(5))
            .map_err(|_| DPERR_NOCONNECTION)?;
        let _ = stream.set_nodelay(true);
        write_frame(&mut stream, &W::default().u8(JOIN).raw(MAGIC).0)
            .map_err(|_| DPERR_NOCONNECTION)?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let answer = read_frame(&mut stream).map_err(|_| DPERR_TIMEOUT)?;
        let _ = stream.set_read_timeout(None);
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
        let session = Session::new(me, world, signal, at.port());
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

    // ---- the host's side ----

    fn accept(self: Arc<Self>, listener: TcpListener) {
        let _ = listener.set_nonblocking(true);
        while !self.stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if self.connections.fetch_add(1, Ordering::SeqCst) >= CONNECTIONS_MOST {
                        self.connections.fetch_sub(1, Ordering::SeqCst);
                        continue;
                    }
                    let _ = stream.set_nonblocking(false);
                    let s = self.clone();
                    std::thread::spawn(move || {
                        s.clone().serve(stream);
                        s.connections.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(_) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }

    fn answer_probes(self: Arc<Self>, udp: UdpSocket) {
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
    fn serve(self: Arc<Self>, mut stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let _ = stream.set_write_timeout(Some(WRITE_WAIT));
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let Ok(first) = read_frame(&mut stream) else {
            return;
        };
        let _ = stream.set_read_timeout(None);
        if first.first() != Some(&JOIN) || first.get(1..9) != Some(MAGIC) {
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

    fn to_machine(&self, machine: u32, bytes: &[u8]) {
        let mut links = self
            .links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(link) = links.get_mut(&machine)
            && write_frame(link, bytes).is_err()
        {
            let _ = link.shutdown(std::net::Shutdown::Both);
        }
    }

    /// The host applies a change and sends it to every machine.
    fn host_change(&self, change: Change) {
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
                    let _ = link.shutdown(std::net::Shutdown::Both);
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
    fn host_op(&self, op: Op, machine: u32) -> (u32, Id) {
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
    fn route(&self, from: Id, to: Id, data: Vec<u8>) {
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

    // ---- a joined machine's side ----

    fn follow_host(self: Arc<Self>, mut stream: TcpStream) {
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
    fn lose(&self) {
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
    fn ask(&self, op: Op) -> (u32, Id) {
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

    // ---- what DirectPlay's methods do ----

    pub fn create_player(&self, name: &str, data: &[u8], event: u32) -> (u32, Id) {
        let (result, id) = self.ask(Op::CreatePlayer(name.into(), data.into()));
        if result == DP_OK {
            self.lock().events.insert(id, event);
        }
        (result, id)
    }

    pub fn create_group(&self, name: &str, data: &[u8]) -> (u32, Id) {
        self.ask(Op::CreateGroup(name.into(), data.into()))
    }

    pub fn add_to_group(&self, group: Id, player: Id) -> u32 {
        self.ask(Op::AddToGroup(group, player)).0
    }

    pub fn remove_from_group(&self, group: Id, player: Id) -> u32 {
        self.ask(Op::RemoveFromGroup(group, player)).0
    }

    pub fn destroy_player(&self, player: Id) -> u32 {
        let result = self.ask(Op::DestroyPlayer(player)).0;
        if result == DP_OK {
            self.lock().events.remove(&player);
        }
        result
    }

    pub fn set_player_data(&self, player: Id, data: &[u8]) -> u32 {
        self.ask(Op::SetPlayerData(player, data.into())).0
    }

    pub fn set_desc(&self, desc: Desc) -> u32 {
        self.ask(Op::SetDesc(desc)).0
    }

    pub fn world(&self) -> World {
        self.lock().world.clone()
    }

    /// A message from this machine's player `from`.
    pub fn send(&self, from: Id, to: Id, data: &[u8]) -> u32 {
        {
            let s = self.lock();
            if s.lost {
                return DPERR_SESSIONLOST;
            }
            if s.world
                .players
                .get(&from)
                .is_none_or(|p| p.machine != self.me)
            {
                return DPERR_INVALIDPLAYER;
            }
        }
        #[cfg(feature = "remote")]
        let data = &crate::remote::mangled(data);
        if self.is_host() {
            self.route(from, to, data.to_vec());
        } else {
            self.to_machine(1, &W::default().u8(SEND).u32(from).u32(to).bytes(data).0);
        }
        DP_OK
    }

    /// The first message waiting, taken when it fits in `room` bytes (else its size: it
    /// stays).
    pub fn receive(&self, room: usize) -> Option<Result<Incoming, usize>> {
        let mut s = self.lock();
        let len = s.queue.front()?.data.len();
        if len > room {
            return Some(Err(len));
        }
        s.queue.pop_front().map(Ok)
    }

    /// Leaves the session (the host: ends it for all).
    pub fn close(&self) {
        self.stop.store(true, Ordering::SeqCst);
        let mut links = self
            .links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for link in links.values_mut() {
            let _ = write_frame(link, &[BYE]);
            let _ = link.shutdown(std::net::Shutdown::Both);
        }
        links.clear();
    }
}

// ---- finding sessions ----

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

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet(_: u32) {}

    fn session() -> Arc<Session> {
        Session::new(2, World::default(), Arc::new(quiet), 0)
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
}
