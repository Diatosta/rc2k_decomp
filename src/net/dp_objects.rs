//! DirectPlay's objects made in Rust for the standalone program (`net::dplay::create_instance`
//! hands them out there; the game keeps Windows' DirectPlay): an `IDirectPlay4A` and an
//! `IDirectPlayLobby2A`, as far as the game uses them, on the sessions of `net::dp_session`.
//!
//! - The connections listed (`EnumConnections`): TCP/IP alone (IPX, modems and serial
//!   links are gone); its address (`InitializeConnection`, made by the lobby object's
//!   `CreateCompoundAddress` as DirectPlay's: `DPAID_TotalSize`, the service provider, the
//!   host's address `DPAID_INet` and port `DPAID_INetPort`) says where sessions are looked
//!   for: that host, or with none the network's broadcast and this machine; the port
//!   DirectPlay's 47624 unless given.
//! - `EnumSessions`: the sessions heard (asynchronously, as the game asks: each call sends a
//!   probe and lists what has answered), the full or closed ones left out when asked
//!   (DPENUMSESSIONS_AVAILABLE), then the time-out (DPESC_TIMEDOUT). `Open`: a session hosted
//!   (DPOPEN_CREATE, a new instance GUID) or one heard joined (DPOPEN_JOIN, by its instance).
//! - Players and groups (`CreatePlayer`, its event signalled when a message comes for it;
//!   `DestroyPlayer`, `CreateGroup`, `AddPlayerToGroup`, `DeletePlayerFromGroup`,
//!   `EnumPlayers`, `EnumGroups`, `GetPlayerData`, `SetPlayerData`), `SetSessionDesc`, `Send`,
//!   `Receive` (DPERR_NOMESSAGES, DPERR_BUFFERTOOSMALL as DirectPlay), `Close`.
//! - The lobby object: `CreateCompoundAddress` (the original never takes a lobby's launch:
//!   `net::open_lobby`).
//!
//! The game calls the methods as the functions here (the original went through
//! COM method tables), on objects held as ids; the enumerations' callbacks are Rust
//! functions. The methods the game does not call are not here. They take
//! and give typed values (GUIDs, names, the session's `Desc`, byte buffers, results
//! returned) in place of the structures' addresses; the callbacks get them the same way.

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

// The game's copies are at 0x5AB310 .. 0x5AB3F0.
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

fn session(this: u32) -> Option<Arc<Session>> {
    with_state(this, |s| s.session.clone())
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
            crate::sync::set_event(event);
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

pub fn close(this: u32) -> u32 {
    if let Some(s) = with_state(this, |s| s.session.take()) {
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

pub fn destroy_player(this: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.destroy_player(player))
}

/// `EnumPlayers` / `EnumGroups`' callback: (the id, the kind, its short name, flags, the
/// context) -> 0 to stop.
pub type EnumPlayersCallback = fn(u32, u32, &[u8], u32, u32) -> u32;

/// The players (kind 1) or the groups (kind 0) to `callback`, each with its name.
fn enumerate(this: u32, callback: EnumPlayersCallback, context: u32, kind: u32) -> u32 {
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

pub fn enum_groups(this: u32, callback: EnumPlayersCallback, context: u32) -> u32 {
    enumerate(this, callback, context, 0)
}

pub fn enum_players(this: u32, callback: EnumPlayersCallback, context: u32) -> u32 {
    enumerate(this, callback, context, 1)
}

/// A DPSESSIONDESC2's fields (its name's text as `name`).
pub fn desc(d: &crate::net::store::SessionDesc, name: &[u8]) -> Desc {
    let guid = |c: &[crate::session::Dword; 4]| -> Guid {
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
        user: crate::session::read_all(&d.user),
    }
}

/// `EnumSessions`' callback: (the session, or none at the end, flags, the context) -> 0
/// to stop.
pub type EnumSessionsCallback = fn(Option<&Desc>, u32, u32) -> u32;

/// `EnumSessions` for `application`'s sessions.
pub fn enum_sessions(
    this: u32,
    application: &Guid,
    callback: EnumSessionsCallback,
    context: u32,
    flags: u32,
) -> u32 {
    let Some((host, port)) = with_state(this, |s| s.initialized.then(|| (s.host.clone(), s.port)))
    else {
        return DPERR_UNINITIALIZED;
    };
    for (d, _) in find_sessions(&host, port, *application) {
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

pub fn open(this: u32, d: &Desc, flags: u32) -> u32 {
    let port = match with_state(this, |s| {
        if !s.initialized {
            Err(DPERR_UNINITIALIZED)
        } else if s.session.is_some() {
            Err(DPERR_ALREADYINITIALIZED)
        } else {
            Ok(s.port)
        }
    }) {
        Ok(port) => port,
        Err(e) => return e,
    };
    let made = if flags & DPOPEN_CREATE != 0 {
        let d = Desc {
            instance: fresh_guid(),
            ..d.clone()
        };
        Session::host(d, port, signal()).map_err(|_| DPERR_CANTCREATESESSION)
    } else if flags & DPOPEN_JOIN != 0 {
        match session_address(&d.instance) {
            Some(at) => Session::join(at, signal()),
            None => Err(DPERR_NOSESSIONS),
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
pub type EnumConnectionsCallback = fn(&Guid, &[u8], &[u8], u32, u32) -> u32;

/// `EnumConnections`: TCP/IP alone.
pub fn enum_connections(callback: EnumConnectionsCallback, context: u32) -> u32 {
    let address = compound(&[(DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec())]);
    let name = c"Internet TCP/IP Connection For DirectPlay";
    callback(
        &DPSPGUID_TCPIP,
        &address,
        name.to_bytes_with_nul(),
        0,
        context,
    );
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

/// The connection's host and port from its address's parts.
fn initialize(s: &mut State, parts: Vec<(Guid, Vec<u8>)>) -> u32 {
    s.host = String::new();
    s.port = DEFAULT_PORT;
    for (kind, data) in parts {
        if kind == DPAID_INET {
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
    crate::framework::log(&format!(
        "net: TCP/IP connection, host \"{}\", port {}",
        s.host, s.port
    ));
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

/// `CreateCompoundAddress` into Rust's memory: the address's bytes.
pub fn compound_address(parts: &[Element]) -> Vec<u8> {
    compound(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    static FOUND: Mutex<Vec<Guid>> = Mutex::new(Vec::new());

    fn session_found(desc: Option<&Desc>, flags: u32, _context: u32) -> u32 {
        if let (0, Some(d)) = (flags & DPESC_TIMEDOUT, desc) {
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
        let _lock = crate::store::test_lock();
        let port = 47_700 + (std::process::id() % 200) as u16;
        let host = play_object(port);
        assert_eq!(open(host, &desc([0; 16]), DPOPEN_CREATE), DP_OK);
        let host_player = create_player(host, b"host\0", 0, &[]).unwrap();

        let guest = play_object(port);
        let mut found = None;
        // Up to 10 s (it stops when found): a full parallel test run under Wine is slow.
        for _ in 0..500 {
            assert_eq!(
                enum_sessions(guest, &APPLICATION, session_found, 0, 0),
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
}
