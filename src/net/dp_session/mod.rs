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

use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

mod discovery;
mod host;
mod peer;
#[cfg(test)]
mod tests;
pub(crate) mod wire;

pub use discovery::{find_sessions, session_address};
use rc2k_core::net::protocol::*;
pub use rc2k_core::net::protocol::{Desc, Group, Id, MESSAGE_MOST, Player, World};

/// A machine that takes no more for this long has its connection ended (each has a thread
/// that reads whatever comes, so a machine busy loading does not stop reading).
const WRITE_WAIT: Duration = Duration::from_secs(5);
#[cfg(test)]
pub use wire::Pipe;
pub use wire::Wire;

#[cfg(steam_support)]
/// The protocol's version as text (Steam lobbies carry it, so that only machines of the same
/// version are listed).
pub fn protocol_tag() -> &'static str {
    std::str::from_utf8(MAGIC).unwrap_or("")
}

/// DirectPlay's TCP/IP port, when the game gives none.
pub const DEFAULT_PORT: u16 = 47624;

// DirectPlay's results (MAKE_DPHRESULT: 0x88770000 + the number).
pub const DP_OK: u32 = 0;
pub const DPERR_ACCESSDENIED: u32 = 0x8877_000A;
pub const DPERR_CANTADDPLAYER: u32 = 0x8877_0028;
pub const DPERR_INVALIDPLAYER: u32 = 0x8877_0096;
pub const DPERR_INVALIDGROUP: u32 = 0x8877_009B;
pub const DPERR_NOCONNECTION: u32 = 0x8877_00AA;
/// The other machine runs another version of the protocol (not a DirectPlay number).
pub const DPERR_VERSION: u32 = 0x8877_00E0;
pub const DPERR_TIMEOUT: u32 = 0x8877_00F0;
pub const DPERR_SESSIONLOST: u32 = 0x8877_0136;
pub const DPERR_NONEWPLAYERS: u32 = 0x8877_014A;
/// The host wants a password and the proof of it was wrong or missing (not a DirectPlay number).
pub const DPERR_INVALIDPASSWORD: u32 = 0x8877_00E1;

// The session's flags the host looks at.
pub const DPSESSION_NEWPLAYERSDISABLED: u32 = 0x1;
pub const DPSESSION_JOINDISABLED: u32 = 0x20;
/// The session is locked by a password (DirectPlay's own flag), shown in the session list.
pub const DPSESSION_PASSWORDREQUIRED: u32 = 0x400;

/// A message for this machine's players: from a player or 0 (a system message), to the
/// player it is for.
#[derive(Clone, Debug, PartialEq)]
pub struct Incoming {
    pub from: Id,
    pub to: Id,
    pub data: Vec<u8>,
}

/// Signals a player's event (a `platform::sync` event's handle).
pub type Signal = Arc<dyn Fn(u32) + Send + Sync>;

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
    /// Each machine's round trip to the host in ms, a moving average: measured by the host,
    /// told by it to every machine.
    pings: HashMap<u32, u32>,
}

impl Shared {
    /// A message for the program, unless too long to take or too many wait.
    fn wait(&mut self, m: Incoming) {
        if m.data.len() <= MESSAGE_MOST && self.queue.len() < WAITING_MOST {
            self.queue.push_back(m);
        } else if !self.dropped {
            self.dropped = true;
            crate::logfile::program_log::log(&format!(
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
    links: Mutex<HashMap<u32, Box<dyn Wire>>>,
    port: u16,
    /// When the session began: the clock the host's pings are stamped with.
    started: Instant,
    /// The host's connections open.
    connections: AtomicUsize,
    /// The host's password, as its salt and the key made from it (never the password).
    lock: Option<Lock>,
    /// What the connection's kind keeps while the session is open (the Steam lobby and its
    /// listening socket); dropped when the session closes.
    held: Mutex<Option<Box<dyn std::any::Any + Send>>>,
}

/// What a locked session's host keeps of its password.
struct Lock {
    salt: [u8; rc2k_core::net::password::SALT_LEN],
    key: [u8; rc2k_core::net::password::KEY_LEN],
}

impl Session {
    fn new(me: u32, world: World, signal: Signal, port: u16, lock: Option<Lock>) -> Arc<Session> {
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
                pings: HashMap::new(),
            }),
            answered: Condvar::new(),
            signal,
            stop: AtomicBool::new(false),
            links: Mutex::new(HashMap::new()),
            connections: AtomicUsize::new(0),
            lock,
            held: Mutex::new(None),
            started: Instant::now(),
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

    /// The host's player (the first player the host made), when the session has one.
    pub fn host_player(&self) -> Option<Id> {
        self.lock()
            .world
            .players
            .iter()
            .find(|(_, p)| p.machine == 1)
            .map(|(&id, _)| id)
    }

    /// The round trip to the host in ms (a moving average) of the machine that `player` is on, as
    /// the host has measured it: none for the host's own players, and while none has been
    /// measured yet.
    pub fn ping_of(&self, player: Id) -> Option<u32> {
        let s = self.lock();
        let machine = s.world.players.get(&player)?.machine;
        if machine == 1 {
            return None;
        }
        s.pings.get(&machine).copied()
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
    /// The host puts the machine of `player` out of the session: it is told, its connection ends
    /// and its players go with it as when it leaves. `DPERR_ACCESSDENIED` on a machine that is
    /// not the host, `DPERR_INVALIDPLAYER` for a player that is none, or this machine's own.
    pub fn kick(&self, player: Id) -> u32 {
        if !self.is_host() {
            return DPERR_ACCESSDENIED;
        }
        let machine = self.lock().world.players.get(&player).map(|p| p.machine);
        let Some(machine) = machine.filter(|&m| m != self.me) else {
            return DPERR_INVALIDPLAYER;
        };
        self.to_machine(machine, &[KICKED]);
        let links = self
            .links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(link) = links.get(&machine) {
            // Ends the machine's reader, which removes its players.
            let _ = link.shutdown();
        }
        DP_OK
    }

    #[cfg(steam_support)]
    /// Keeps `what` until the session closes.
    pub fn hold(&self, what: Box<dyn std::any::Any + Send>) {
        *self
            .held
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(what);
    }

    pub fn close(&self) {
        self.stop.store(true, Ordering::SeqCst);
        let mut links = self
            .links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for link in links.values_mut() {
            let _ = write_frame(link, &[BYE]);
            let _ = link.shutdown();
        }
        links.clear();
        drop(links);
        self.held
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}
