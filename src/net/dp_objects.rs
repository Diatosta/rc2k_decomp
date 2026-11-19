//! DirectPlay's objects made in Rust: an `IDirectPlay4A` and an `IDirectPlayLobby2A`, as far as
//! the game uses them, on the sessions of `net::dp_session`.
//!
//! - Connections (`EnumConnections`): TCP/IP only. Its address says where sessions are looked
//!   for: the given host, or the network's broadcast and this machine; the port is 47624 unless
//!   given.
//! - `EnumSessions`: asynchronous, as the game asks; each call sends a probe and lists what has
//!   answered, leaves out full or closed sessions when asked, then reports the time-out.
//!   `Open` hosts a session (DPOPEN_CREATE) or joins one heard (DPOPEN_JOIN).
//! - Players and groups (create, destroy, add, delete, enumerate, data), `SetSessionDesc`,
//!   `Send`, `Receive` and `Close`.
//! - The lobby object only makes compound addresses.
//!
//! The methods take and give typed values (GUIDs, names, the session's `Desc`, byte buffers)
//! and objects are held as ids; enumeration callbacks are Rust functions.

use crate::net::dp_session::{
    DEFAULT_PORT, DP_OK, Desc, Session, Signal, find_sessions, session_address,
};
use std::ffi::CString;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

const CLASS_NOT_REGISTERED: u32 = 0x8004_0154;
const DPERR_ALREADYINITIALIZED: u32 = 0x8877_0005;
const DPERR_BUFFERTOOSMALL: u32 = 0x8877_001E;
const DPERR_CANTCREATESESSION: u32 = 0x8877_0046;
const DPERR_INVALIDPARAMS: u32 = 0x8007_0057;
const DPERR_INVALIDPLAYER: u32 = 0x8877_0096;
const DPERR_NOMESSAGES: u32 = 0x8877_00BE;
const DPERR_NOSESSIONS: u32 = 0x8877_00DC;
const DPERR_UNINITIALIZED: u32 = 0x8877_0140;

const DPOPEN_JOIN: u32 = 1;
const DPOPEN_CREATE: u32 = 2;
const DPENUMSESSIONS_AVAILABLE: u32 = 1;
const DPESC_TIMEDOUT: u32 = 1;

type Guid = [u8; 16];

/// A GUID from its text's parts.
const fn guid(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> Guid {
    let a = d1.to_le_bytes();
    let b = d2.to_le_bytes();
    let c = d3.to_le_bytes();
    [
        a[0], a[1], a[2], a[3], b[0], b[1], c[0], c[1], d4[0], d4[1], d4[2], d4[3], d4[4], d4[5],
        d4[6], d4[7],
    ]
}

// The IIDs and GUIDs the game's objects are asked for.
const CLSID_DIRECTPLAY: Guid = guid(
    0xD1EB_6D20,
    0x8923,
    0x11D0,
    [0x9D, 0x97, 0x00, 0xA0, 0xC9, 0x0A, 0x43, 0xCB],
);
const CLSID_DIRECTPLAYLOBBY: Guid = guid(
    0x2FE8_F810,
    0xB2A5,
    0x11D0,
    [0xA7, 0x87, 0x00, 0x00, 0xF8, 0x03, 0xAB, 0xFC],
);
const IID_IDIRECTPLAY4A: Guid = guid(
    0x133E_FE41,
    0x32DC,
    0x11D0,
    [0x9C, 0xFB, 0x00, 0xA0, 0xC9, 0x0A, 0x43, 0xCB],
);
const IID_IDIRECTPLAYLOBBY2A: Guid = guid(
    0x1BB4_AF80,
    0xA303,
    0x11D0,
    [0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E],
);
const DPSPGUID_TCPIP: Guid = guid(
    0x36E9_5EE0,
    0x8577,
    0x11CF,
    [0x96, 0x0C, 0x00, 0x80, 0xC7, 0x53, 0x4E, 0x82],
);
const DPAID_TOTALSIZE: Guid = guid(
    0x1318_F560,
    0x912C,
    0x11D0,
    [0x9D, 0xAA, 0x00, 0xA0, 0xC9, 0x0A, 0x43, 0xCB],
);
const DPAID_SERVICEPROVIDER: Guid = guid(
    0x07D9_16C0,
    0xE0AF,
    0x11CF,
    [0x9C, 0x4E, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E],
);
const DPAID_INET: Guid = guid(
    0xC4A5_4DA0,
    0xE0AF,
    0x11CF,
    [0x9C, 0x4E, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E],
);
const DPAID_INETPORT: Guid = guid(
    0xE452_4541,
    0x8EA5,
    0x11D1,
    [0x8A, 0x96, 0x00, 0x60, 0x97, 0xB0, 0x14, 0x11],
);

// ---- the objects ----

#[derive(Default)]
struct State {
    initialized: bool,
    host: String,
    port: u16,
    /// The connection is Steam's, not TCP/IP.
    steam: bool,
    session: Option<Arc<Session>>,
}

/// An `IDirectPlay4A` or `IDirectPlayLobby2A` object.
struct Object {
    refs: AtomicU32,
    state: Mutex<State>,
}

/// The objects by id (what the game holds of each: never 0).
static OBJECTS: Mutex<Vec<(u32, Arc<Object>)>> = Mutex::new(Vec::new());
static NEXT: AtomicU32 = AtomicU32::new(1);

fn object(this: u32) -> Arc<Object> {
    let objects = OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match objects.iter().find(|(id, _)| *id == this) {
        Some((_, o)) => Arc::clone(o),
        None => {
            drop(objects);
            end(crate::errors::BadInput(
                "a DirectPlay object that is not there",
            ))
        }
    }
}

/// The object's state, for `f`.
fn with_state<R>(this: u32, f: impl FnOnce(&mut State) -> R) -> R {
    let o = object(this);
    let mut s = o
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut s)
}

/// The object's session, if it has one. An object already released has none: the receiving
/// thread ends a lost session (the host left) beside the lobby's frames, which then find the
/// object gone and get the same answer as from an object without a session.
fn session(this: u32) -> Option<Arc<Session>> {
    let found = OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|(id, _)| *id == this)
        .map(|(_, o)| Arc::clone(o))?;
    let state = found
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.session.clone()
}

fn make() -> u32 {
    let id = NEXT.fetch_add(1, Ordering::SeqCst);
    OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((
            id,
            Arc::new(Object {
                refs: AtomicU32::new(1),
                state: Mutex::new(State::default()),
            }),
        ));
    id
}

fn end(e: crate::errors::BadInput) -> ! {
    crate::fatal::raise::end("net::dp_objects", e)
}

/// A name's text up to its NUL (or its end).
fn name_text(name: &[u8]) -> String {
    let n = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    String::from_utf8_lossy(&name[..n]).into_owned()
}

/// The thing's bytes copied to `out` (as DirectPlay into the caller's buffer): the result
/// and the bytes' count (DPERR_BUFFERTOOSMALL when they do not fit).
fn give(data: &[u8], out: &mut [u8]) -> (u32, u32) {
    let size = data.len() as u32;
    match out.get_mut(..data.len()) {
        Some(room) => {
            room.copy_from_slice(data);
            (DP_OK, size)
        }
        None => (DPERR_BUFFERTOOSMALL, size),
    }
}

/// A player's event signalled (`sync`'s handle).
fn signal() -> Signal {
    Arc::new(|event: u32| {
        if event != 0 {
            crate::platform::sync::set_event(event);
        }
    })
}

/// A new instance GUID (random).
fn fresh_guid() -> Guid {
    use std::hash::{BuildHasher, Hasher};
    let mut g = [0u8; 16];
    for half in g.chunks_mut(8) {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        );
        half.copy_from_slice(&h.finish().to_le_bytes());
    }
    g
}

/// `CoCreateInstance` for DirectPlay's two classes: the object, or the error.
pub fn create(class: &Guid, interface: &Guid) -> Result<u32, u32> {
    if (*class == CLSID_DIRECTPLAY && *interface == IID_IDIRECTPLAY4A)
        || (*class == CLSID_DIRECTPLAYLOBBY && *interface == IID_IDIRECTPLAYLOBBY2A)
    {
        Ok(make())
    } else {
        Err(CLASS_NOT_REGISTERED)
    }
}

// ---- IUnknown ----

/// `Release`: the references left; at none the session is closed and the object goes.
pub fn release(this: u32) -> u32 {
    let left = object(this).refs.fetch_sub(1, Ordering::SeqCst) - 1;
    if left == 0 {
        if let Some(s) = session(this) {
            s.close();
        }
        OBJECTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(id, _)| *id != this);
    }
    left
}

// ---- IDirectPlay4A ----

pub fn add_player_to_group(this: u32, group: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.add_to_group(group, player))
}

/// The object's session taken from it (none when it has none or the object is gone).
fn take_session(this: u32) -> Option<Arc<Session>> {
    let found = OBJECTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|(id, _)| *id == this)
        .map(|(_, o)| Arc::clone(o))?;
    let mut state = found
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.session.take()
}

pub fn close(this: u32) -> u32 {
    if let Some(s) = take_session(this) {
        s.close();
    }
    DP_OK
}

/// `CreateGroup`: the group's id, or the error.
pub fn create_group(this: u32, name: &[u8], data: &[u8]) -> Result<u32, u32> {
    let Some(s) = session(this) else {
        return Err(DPERR_UNINITIALIZED);
    };
    match s.create_group(&name_text(name), data) {
        (DP_OK, made) => Ok(made),
        (e, _) => Err(e),
    }
}

/// `CreatePlayer` (its event signalled when a message comes for it): the player's id, or
/// the error.
pub fn create_player(this: u32, name: &[u8], event: u32, data: &[u8]) -> Result<u32, u32> {
    let Some(s) = session(this) else {
        return Err(DPERR_UNINITIALIZED);
    };
    match s.create_player(&name_text(name), data, event) {
        (DP_OK, made) => Ok(made),
        (e, _) => Err(e),
    }
}

pub fn delete_player_from_group(this: u32, group: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.remove_from_group(group, player))
}

/// The host puts the machine of `player` out of the session (`Session::kick`).
pub fn kick(this: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.kick(player))
}

pub fn destroy_player(this: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.destroy_player(player))
}

/// `EnumPlayers` / `EnumGroups`' callback: (the id, the kind, its short name, flags, the
/// context) -> 0 to stop.
type EnumPlayersCallback<'a> = &'a dyn Fn(u32, u32, &[u8], u32, u32) -> u32;

/// The players (kind 1) or the groups (kind 0) to `callback`, each with its name.
fn enumerate(this: u32, callback: EnumPlayersCallback<'_>, context: u32, kind: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    let world = s.world();
    let list: Vec<(u32, String)> = if kind == 1 {
        world
            .players
            .iter()
            .map(|(id, p)| (*id, p.name.clone()))
            .collect()
    } else {
        world
            .groups
            .iter()
            .map(|(id, g)| (*id, g.name.clone()))
            .collect()
    };
    for (id, name) in list {
        let name = CString::new(name).unwrap_or_default();
        if callback(id, kind, name.as_bytes_with_nul(), 0, context) == 0 {
            break;
        }
    }
    DP_OK
}

pub fn enum_groups(this: u32, callback: EnumPlayersCallback<'_>, context: u32) -> u32 {
    enumerate(this, callback, context, 0)
}

pub fn enum_players(this: u32, callback: EnumPlayersCallback<'_>, context: u32) -> u32 {
    enumerate(this, callback, context, 1)
}

/// A DPSESSIONDESC2's fields (its name's text as `name`).
pub fn desc(d: &crate::net::store::SessionDesc, name: &[u8]) -> Desc {
    let guid = |c: &[crate::memory::session::Dword; 4]| -> Guid {
        let mut g = [0u8; 16];
        for (k, cell) in c.iter().enumerate() {
            g[4 * k..4 * k + 4].copy_from_slice(&cell.get().to_le_bytes());
        }
        g
    };
    Desc {
        flags: d.flags.get(),
        instance: guid(&d.instance),
        application: guid(&d.application),
        max_players: d.max_players.get(),
        current_players: d.current_players.get(),
        name: name_text(name),
        user: crate::memory::session::read_all(&d.user),
    }
}

/// `EnumSessions`' callback: (the session, or none at the end, flags, the context) -> 0
/// to stop.
type EnumSessionsCallback<'a> = &'a dyn Fn(Option<&Desc>, u32, u32) -> u32;

/// `EnumSessions` for `application`'s sessions.
pub fn enum_sessions(
    this: u32,
    application: &Guid,
    callback: EnumSessionsCallback<'_>,
    context: u32,
    flags: u32,
) -> u32 {
    let Some((host, port, steam)) = with_state(this, |s| {
        s.initialized.then(|| (s.host.clone(), s.port, s.steam))
    }) else {
        return DPERR_UNINITIALIZED;
    };
    let heard: Vec<Desc> = if steam {
        crate::net::steam::lobbies(*application)
    } else {
        find_sessions(&host, port, *application)
            .into_iter()
            .map(|(d, _)| d)
            .collect()
    };
    for d in heard {
        let closed =
            d.flags & 0x21 != 0 || (d.max_players != 0 && d.current_players >= d.max_players);
        if flags & DPENUMSESSIONS_AVAILABLE != 0 && closed {
            continue;
        }
        if callback(Some(&d), 0, context) == 0 {
            return DP_OK;
        }
    }
    callback(None, DPESC_TIMEDOUT, context);
    DP_OK
}

/// `GetPlayerData` into `out`: the result and the data's size.
pub fn get_player_data(this: u32, player: u32, out: &mut [u8]) -> (u32, u32) {
    let Some(s) = session(this) else {
        return (DPERR_UNINITIALIZED, 0);
    };
    match s.world().players.get(&player) {
        Some(p) => give(&p.data, out),
        None => (DPERR_INVALIDPLAYER, 0),
    }
}

#[cfg(test)]
pub fn open(this: u32, d: &Desc, flags: u32) -> u32 {
    open_with_password(this, d, flags, b"")
}

/// `open`, with the password a created session is locked by or a joined one is answered with
/// (empty: none).
pub fn open_with_password(this: u32, d: &Desc, flags: u32, password: &[u8]) -> u32 {
    let (port, steam) = match with_state(this, |s| {
        if !s.initialized {
            Err(DPERR_UNINITIALIZED)
        } else if s.session.is_some() {
            Err(DPERR_ALREADYINITIALIZED)
        } else {
            Ok((s.port, s.steam))
        }
    }) {
        Ok(got) => got,
        Err(e) => return e,
    };
    let made = if steam {
        open_steam(d, flags, password)
    } else if flags & DPOPEN_CREATE != 0 {
        let d = Desc {
            instance: fresh_guid(),
            ..d.clone()
        };
        Session::host_locked(d, port, signal(), password).map_err(|e| {
            // The usual cause is another program (often a second copy of this game) already
            // listening on the port.
            crate::program_log::log(&format!(
                "net: the session could not be hosted on port {port}: {e} (is another game or program using the port?)"
            ));
            if e.kind() == std::io::ErrorKind::AddrInUse {
                PORT_IN_USE.store(u32::from(port), std::sync::atomic::Ordering::Relaxed);
            }
            DPERR_CANTCREATESESSION
        })
    } else if flags & DPOPEN_JOIN != 0 {
        match session_address(&d.instance) {
            Some(at) => {
                let joined = Session::join_with(at, signal(), password);
                if joined.is_ok() {
                    crate::net::last_host::remember(at.ip());
                }
                joined
            }
            None => {
                crate::program_log::log("net: the session to join is not in the list found");
                Err(DPERR_NOSESSIONS)
            }
        }
    } else {
        Err(DPERR_INVALIDPARAMS)
    };
    match made {
        Ok(s) => {
            with_state(this, |state| state.session = Some(s));
            DP_OK
        }
        Err(e) => e,
    }
}

/// `open` over Steam: a created session gets a lobby, a joined one connects to its lobby's
/// owner through Steam's relay.
fn open_steam(d: &Desc, flags: u32, password: &[u8]) -> Result<Arc<Session>, u32> {
    if flags & DPOPEN_CREATE != 0 {
        let d = Desc {
            instance: fresh_guid(),
            ..d.clone()
        };
        let session = Session::host_over(d.clone(), 0, signal(), password);
        crate::net::steam::host(&session, &d, !password.is_empty()).map_err(|e| {
            crate::program_log::log(&format!("net: the Steam session could not be hosted: {e}"));
            session.close();
            DPERR_CANTCREATESESSION
        })?;
        Ok(session)
    } else if flags & DPOPEN_JOIN != 0 {
        let wire = crate::net::steam::join(&d.instance)?;
        Session::join_over(wire, 0, signal(), password)
    } else {
        Err(DPERR_INVALIDPARAMS)
    }
}

/// What `Receive` gave: its result, the sender and addressee when a message was taken,
/// the size when one was there.
pub struct Received {
    pub result: u32,
    pub from_to: Option<(u32, u32)>,
    pub size: Option<u32>,
}

/// `Receive` into `out`.
pub fn receive(this: u32, out: &mut [u8]) -> Received {
    let none = |result| Received {
        result,
        from_to: None,
        size: None,
    };
    let Some(s) = session(this) else {
        return none(DPERR_UNINITIALIZED);
    };
    match s.receive(out.len()) {
        None => none(DPERR_NOMESSAGES),
        Some(Err(n)) => Received {
            result: DPERR_BUFFERTOOSMALL,
            from_to: None,
            size: Some(n as u32),
        },
        Some(Ok(m)) => {
            let (result, size) = give(&m.data, out);
            Received {
                result,
                from_to: Some((m.from, m.to)),
                size: Some(size),
            }
        }
    }
}

/// The session's host's player (none without a session or a player of the host's).
pub fn host_player(this: u32) -> Option<u32> {
    session(this).and_then(|s| s.host_player())
}

/// The round trip in ms to the host of the machine `player` is on (`Session::ping_of`).
pub fn ping_of(this: u32, player: u32) -> Option<u32> {
    session(this).and_then(|s| s.ping_of(player))
}

pub fn send(this: u32, from: u32, to: u32, data: &[u8]) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.send(from, to, data))
}

pub fn set_player_data(this: u32, player: u32, data: &[u8]) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.set_player_data(player, data))
}

pub fn set_session_desc(this: u32, desc: &Desc) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    s.set_desc(desc.clone())
}

/// `EnumConnections`' callback: (the service provider's GUID, the connection's address,
/// its name, flags, the context) -> 0 to stop.
type EnumConnectionsCallback<'a> = &'a dyn Fn(&Guid, &[u8], &[u8], u32, u32) -> u32;

/// `EnumConnections`: TCP/IP, then Steam when it is there.
pub fn enum_connections(callback: EnumConnectionsCallback<'_>, context: u32) -> u32 {
    let address = compound(&[(DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec())]);
    let name = c"Internet TCP/IP Connection For DirectPlay";
    callback(
        &DPSPGUID_TCPIP,
        &address,
        name.to_bytes_with_nul(),
        0,
        context,
    );
    // Steam is one more connection when its client runs.
    if crate::net::steam::available() {
        let address = compound(&[(DPAID_SERVICEPROVIDER, crate::net::steam::PROVIDER.to_vec())]);
        callback(
            &crate::net::steam::PROVIDER,
            &address,
            crate::net::steam::CONNECTION_NAME,
            0,
            context,
        );
    }
    DP_OK
}

/// A compound address's elements (each its type and data).
fn elements(address: &[u8]) -> Vec<(Guid, Vec<u8>)> {
    let mut out = vec![];
    let mut at = 0;
    while at + 20 <= address.len() {
        let kind: Guid = address[at..at + 16].try_into().unwrap();
        let n = u32::from_le_bytes(address[at + 16..at + 20].try_into().unwrap()) as usize;
        let Some(data) = address.get(at + 20..at + 20 + n) else {
            break;
        };
        out.push((kind, data.to_vec()));
        at += 20 + n;
    }
    out
}

/// A compound address as DirectPlay lays one out: its total size first, then the elements.
fn compound(parts: &[(Guid, Vec<u8>)]) -> Vec<u8> {
    let total = 24 + parts.iter().map(|(_, d)| 20 + d.len()).sum::<usize>();
    let mut v = Vec::with_capacity(total);
    for (kind, data) in [(DPAID_TOTALSIZE, (total as u32).to_le_bytes().to_vec())]
        .iter()
        .chain(parts)
    {
        v.extend(kind);
        v.extend((data.len() as u32).to_le_bytes());
        v.extend(data);
    }
    v
}

/// DirectPlay's `InitializeConnection` on an address Rust holds (`None`: a null one).
pub fn initialize_connection_with(this: u32, address: Option<&[u8]>) -> u32 {
    if with_state(this, |s| s.initialized) {
        return DPERR_ALREADYINITIALIZED;
    }
    let Some(raw) = address else {
        return DPERR_INVALIDPARAMS;
    };
    let total = raw
        .get(20..24)
        .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize);
    let raw = &raw[..total.clamp(24, 4096).min(raw.len())];
    let parts = elements(raw);
    with_state(this, |s| initialize(s, parts))
}

/// The port the last attempt to host found held by another program (0: none).
static PORT_IN_USE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The port a failed attempt to host found in use, once.
pub fn take_port_in_use() -> Option<u16> {
    match PORT_IN_USE.swap(0, std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        port => u16::try_from(port).ok(),
    }
}

/// The connection's host and port from its address's parts.
fn initialize(s: &mut State, parts: Vec<(Guid, Vec<u8>)>) -> u32 {
    s.host = String::new();
    s.port = DEFAULT_PORT;
    s.steam = false;
    for (kind, data) in parts {
        if kind == DPAID_SERVICEPROVIDER && data.starts_with(&crate::net::steam::PROVIDER) {
            s.steam = true;
        } else if kind == DPAID_INET {
            s.host = String::from_utf8_lossy(&data)
                .trim_end_matches('\0')
                .to_string();
        } else if kind == DPAID_INETPORT && data.len() >= 2 {
            let port = u16::from_le_bytes([data[0], data[1]]);
            if port != 0 {
                s.port = port;
            }
        }
    }
    s.initialized = true;
    if s.steam {
        crate::program_log::log("net: Steam connection");
    } else {
        crate::program_log::log(&format!(
            "net: TCP/IP connection, host \"{}\", port {}",
            s.host, s.port
        ));
    }
    DP_OK
}

// ---- IDirectPlayLobby2A ----

/// An address element: its type's GUID and its data (DPCOMPOUNDADDRESSELEMENT).
pub type Element = (Guid, Vec<u8>);

/// `CreateCompoundAddress` sized for `room` bytes: the result and the address's size.
pub fn create_compound_address(parts: &[Element], room: u32) -> (u32, u32) {
    let size = compound(parts).len() as u32;
    if room < size {
        (DPERR_BUFFERTOOSMALL, size)
    } else {
        (DP_OK, size)
    }
}

/// A DirectPlay object hosting a session on a free loopback port with one player, for the
/// tests that send through the game's own code: the object, the player, the port. Release
/// the object with `release`.
#[cfg(test)]
pub fn hosted_for_tests() -> (u32, u32, u16) {
    let first = 48_200 + (std::process::id() % 100) as u16;
    for port in first..first + 50 {
        let this = create(&CLSID_DIRECTPLAY, &IID_IDIRECTPLAY4A).expect("the object");
        let address = compound(&[
            (DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec()),
            (DPAID_INET, b"127.0.0.1\0".to_vec()),
            (DPAID_INETPORT, port.to_le_bytes().to_vec()),
        ]);
        assert_eq!(initialize_connection_with(this, Some(&address)), DP_OK);
        let d = Desc {
            application: [1; 16],
            max_players: 4,
            name: "test".into(),
            ..Desc::default()
        };
        if open(this, &d, DPOPEN_CREATE) == DP_OK {
            let player = create_player(this, b"me\0", 0, &[]).expect("a player");
            return (this, player, port);
        }
        release(this);
    }
    panic!("a free port to host on");
}

/// `CreateCompoundAddress` into Rust's memory: the address's bytes.
pub fn compound_address(parts: &[Element]) -> Vec<u8> {
    compound(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    static FOUND: Mutex<Vec<Guid>> = Mutex::new(Vec::new());

    /// Only this test's sessions (named "test"): another test's session of the same application
    /// may still be listed.
    fn session_found(desc: Option<&Desc>, flags: u32, _context: u32) -> u32 {
        if let (0, Some(d)) = (flags & DPESC_TIMEDOUT, desc)
            && d.name == "test"
        {
            FOUND.lock().unwrap().push(d.instance);
        }
        1
    }

    const APPLICATION: Guid = guid(0x1234_5678, 0x9ABC, 0xDEF0, [1, 2, 3, 4, 5, 6, 7, 8]);

    fn play_object(port: u16) -> u32 {
        let this = create(&CLSID_DIRECTPLAY, &IID_IDIRECTPLAY4A).expect("the object");
        let address = compound(&[
            (DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec()),
            (DPAID_INET, b"127.0.0.1\0".to_vec()),
            (DPAID_INETPORT, port.to_le_bytes().to_vec()),
        ]);
        assert_eq!(initialize_connection_with(this, Some(&address)), DP_OK);
        this
    }

    fn desc(instance: Guid) -> Desc {
        Desc {
            flags: 0,
            instance,
            application: APPLICATION,
            max_players: 4,
            current_players: 0,
            name: "test".into(),
            user: [0; 4],
        }
    }

    #[test]
    fn a_session_is_hosted_found_joined_and_talked_to() {
        let _lock = crate::test_support::test_lock();
        // A port another program holds (a network game on this machine): the next ones.
        let first = 47_700 + (std::process::id() % 200) as u16;
        let (port, host) = (first..first + 20)
            .find_map(|port| {
                let host = play_object(port);
                if open(host, &desc([0; 16]), DPOPEN_CREATE) == DP_OK {
                    return Some((port, host));
                }
                release(host);
                None
            })
            .expect("a free port to host on");
        let host_player = create_player(host, b"host\0", 0, &[]).unwrap();

        let guest = play_object(port);
        let mut found = None;
        // Up to 10 s (it stops when found): a full parallel test run under Wine is slow.
        for _ in 0..500 {
            assert_eq!(
                enum_sessions(guest, &APPLICATION, &session_found, 0, 0),
                DP_OK
            );
            found = FOUND.lock().unwrap().first().copied();
            if found.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let instance = found.expect("the hosted session is found");
        assert_eq!(open(guest, &desc(instance), DPOPEN_JOIN), DP_OK);
        let guest_player = create_player(guest, b"guest\0", 0, &[]).unwrap();

        assert_eq!(send(guest, guest_player, host_player, b"hello"), DP_OK);
        let mut data = [0u8; 64];
        let mut got = None;
        for _ in 0..1000 {
            let r = receive(host, &mut data);
            if r.result == DP_OK {
                let (from, to) = r.from_to.unwrap();
                if from == guest_player {
                    got = Some((data[..r.size.unwrap() as usize].to_vec(), to));
                    break;
                }
                // a system message (from 0): the guest's player made
                continue;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let (bytes, to) = got.expect("the message");
        assert_eq!(bytes, b"hello");
        assert_eq!(to, host_player);
        assert_eq!(release(guest), 0);
        assert_eq!(release(host), 0);
    }

    /// Guards the compound address's bytes: the total-size element first (its GUID, 4, the
    /// total), then each element as its data type's GUID, its size and its data; and that the
    /// session's host and port are read back from such an address.
    #[test]
    fn a_compound_address_has_its_bytes() {
        let tcp_ip: Vec<u8> = vec![
            0xE0, 0x5E, 0xE9, 0x36, 0x77, 0x85, 0xCF, 0x11, 0x96, 0x0C, 0x00, 0x80, 0xC7, 0x53,
            0x4E, 0x82,
        ];
        assert_eq!(tcp_ip, DPSPGUID_TCPIP);
        let address = compound_address(&[
            (DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec()),
            (DPAID_INET, b"127.0.0.1\0".to_vec()),
            (DPAID_INETPORT, 47624u16.to_le_bytes().to_vec()),
        ]);
        let element = |kind: [u8; 16], data: &[u8]| {
            [
                kind.to_vec(),
                (data.len() as u32).to_le_bytes().to_vec(),
                data.to_vec(),
            ]
            .concat()
        };
        let total = 24 + (20 + 16) + (20 + 10) + (20 + 2);
        let expected = [
            // DPAID_TotalSize
            element(
                [
                    0x60, 0xF5, 0x18, 0x13, 0x2C, 0x91, 0xD0, 0x11, 0x9D, 0xAA, 0x00, 0xA0, 0xC9,
                    0x0A, 0x43, 0xCB,
                ],
                &(total as u32).to_le_bytes(),
            ),
            // DPAID_ServiceProvider
            element(
                [
                    0xC0, 0x16, 0xD9, 0x07, 0xAF, 0xE0, 0xCF, 0x11, 0x9C, 0x4E, 0x00, 0xA0, 0xC9,
                    0x05, 0x42, 0x5E,
                ],
                &tcp_ip,
            ),
            // DPAID_INet
            element(
                [
                    0xA0, 0x4D, 0xA5, 0xC4, 0xAF, 0xE0, 0xCF, 0x11, 0x9C, 0x4E, 0x00, 0xA0, 0xC9,
                    0x05, 0x42, 0x5E,
                ],
                b"127.0.0.1\0",
            ),
            // DPAID_INetPort
            element(
                [
                    0x41, 0x45, 0x52, 0xE4, 0xA5, 0x8E, 0xD1, 0x11, 0x8A, 0x96, 0x00, 0x60, 0x97,
                    0xB0, 0x14, 0x11,
                ],
                &[0x08, 0xBA],
            ),
        ]
        .concat();
        assert_eq!(address, expected);
        let parts = elements(&address);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[2].1, b"127.0.0.1\0");
    }

    /// The sessions of this test's name (an earlier test's, still listed, are not it).
    static FOUND_NAMED: Mutex<Vec<Guid>> = Mutex::new(Vec::new());

    fn named(instance: Guid) -> Desc {
        Desc {
            name: "frozen".into(),
            ..desc(instance)
        }
    }

    fn named_found(desc: Option<&Desc>, flags: u32, _context: u32) -> u32 {
        if let (0, Some(d)) = (flags & DPESC_TIMEDOUT, desc)
            && d.name == "frozen"
        {
            FOUND_NAMED.lock().unwrap().push(d.instance);
        }
        1
    }

    /// Guards the game's messages over a real session as the bytes the other machine gets and
    /// as `net::take_messages` reads them: a host's call (0x40, 0x50: the round) is answered with
    /// 0x41 / 0x51 and the round, and sets its flag; the answers (0x41) are noted on the host
    /// only for the round it is in; a race message (top bit) is stored in the sender's inbox with
    /// its sequence byte doubled; one longer than 0xFE bytes, and one from a stranger, are
    /// dropped.
    #[test]
    fn the_games_messages_cross_a_session_as_the_same_bytes() {
        use crate::net::owner::Net;
        let _lock = crate::test_support::test_lock();
        let first = 47_900 + (std::process::id() % 100) as u16;
        let (port, host) = (first..first + 20)
            .find_map(|port| {
                let host = play_object(port);
                if open(host, &named([0; 16]), DPOPEN_CREATE) == DP_OK {
                    return Some((port, host));
                }
                release(host);
                None
            })
            .expect("a free port to host on");
        let host_player = create_player(host, b"host\0", 0, &[]).unwrap();
        let guest = play_object(port);
        FOUND_NAMED.lock().unwrap().clear();
        for _ in 0..500 {
            enum_sessions(guest, &APPLICATION, &named_found, 0, 0);
            if !FOUND_NAMED.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let instance = FOUND_NAMED
            .lock()
            .unwrap()
            .first()
            .copied()
            .expect("the session");
        let joined = open(guest, &named(instance), DPOPEN_JOIN);
        assert_eq!(joined, DP_OK, "port {port}, instance {instance:?}");
        let guest_player = create_player(guest, b"guest\0", 0, &[]).unwrap();

        // The program's side: this machine is the host object's player, with the guest as
        // player 0 of its table (a box: the game's own state stays as it is).
        let net: Box<Net> = Box::new(Net::new());
        let vars = &net.vars;
        net.link.set_dplay(host);
        net.link.set_player_id(host_player);
        net.race_slot.set_players(1);
        vars.receiver.sync.set(7);
        vars.received[0].id.set(0xDEAD);
        let lines = &crate::race::viewer_screen_store::NET_TEXTS;
        let take = || {
            assert!(
                crate::net::take_messages::take_messages(
                    &net,
                    lines,
                    &crate::clock::tick_count::PictureClock::new()
                )
                .is_ok()
            );
        };
        // Takes messages until `done` or ten seconds.
        let until = |done: &dyn Fn() -> bool| {
            for _ in 0..1000 {
                take();
                if done() {
                    return true;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            false
        };
        // What the guest is sent by the program, skipping the system messages.
        let from_host = || {
            let mut data = [0u8; 256];
            for _ in 0..1000 {
                let r = receive(guest, &mut data);
                if r.result == DP_OK && r.from_to.is_some_and(|(from, _)| from == host_player) {
                    return data[..r.size.unwrap() as usize].to_vec();
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            panic!("a message from the program");
        };
        let say = |bytes: &[u8]| assert_eq!(send(guest, guest_player, host_player, bytes), DP_OK);

        // Takes messages for a while: what is not to happen has had its time.
        let settle = || {
            for _ in 0..40 {
                take();
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        // A stranger's message is dropped.
        say(&[0x40, 7]);
        settle();
        assert_eq!(vars.all_raced.get(), 0);

        // The guest is a known player now: the call of this round is answered, the flag set.
        vars.received[0].id.set(guest_player);
        // A player that is not ready does not answer the call for everyone ready.
        say(&[0x40, 7]);
        settle();
        assert_eq!(vars.all_raced.get(), 0);
        vars.own.set_state(rc2k_core::net::players::State::READY);
        say(&[0x40, 7]);
        assert!(until(&|| vars.all_raced.get() == 1));
        assert_eq!(from_host(), [0x41, 7]);
        assert_eq!(vars.receiver.sync_answer.get(), 0x0741);
        // The call of another round is not.
        say(&[0x50, 8]);
        settle();
        assert_eq!(vars.race_over.get(), 0);
        say(&[0x50, 7]);
        assert!(until(&|| vars.race_over.get() == 1));
        assert_eq!(from_host(), [0x51, 7]);
        assert_eq!(vars.receiver.sync_answer.get(), 0x0751);

        // On the game's host an answer of this round is noted, one of another is not, and a
        // call is not answered.
        vars.own.set_state(rc2k_core::net::players::State::HOST);
        say(&[0x41, 8]);
        settle();
        assert!(!vars.synced(0).unwrap());
        say(&[0x41, 7]);
        assert!(until(&|| vars.synced(0).unwrap()));
        vars.all_raced.set(0);
        say(&[0x40, 7]);
        settle();
        assert_eq!(vars.all_raced.get(), 0);

        // A race message: stored with the sequence byte doubled, its length and bytes.
        say(&[0x85, 1, 2, 3, 4]);
        assert!(until(&|| net.lock_received()[0].count == 1));
        let inbox = net.lock_received()[0].clone();
        assert_eq!(inbox.records[..6], [0x0A, 4, 1, 2, 3, 4]);
        assert_eq!((inbox.oldest, inbox.newest), (0, 0x0A));
        // One longer than 0xFE bytes after the first is dropped; the next one is stored.
        say(&[vec![0x86], vec![0xEE; 0xFF]].concat());
        say(&[0x87, 9]);
        assert!(until(&|| net.lock_received()[0].count == 2));
        let inbox = net.lock_received()[0].clone();
        assert_eq!(inbox.records[0x100..0x103], [0x0E, 1, 9]);
        assert_eq!(release(guest), 0);
        assert_eq!(net.link.release_dplay(), Some(0));
    }
}
