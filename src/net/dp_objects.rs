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
//! - The lobby object: `CreateCompoundAddress`; the rest of a lobby's launch is not there
//!   (DPERR_NOTLOBBIED: the standalone program is never launched by one).
//!
//! The game calls the methods as the functions here (phase 11, 2026-10-26: before, through
//! COM method tables), on objects held as ids; the enumerations' callbacks are Rust
//! functions. The methods the game does not call are not here.

use crate::net::dp_session::{
    DEFAULT_PORT, DP_OK, Desc, Session, Signal, find_sessions, session_address,
};
use crate::net::types::{DpAddressElement, DpName, DpSessionDesc};
use std::ffi::CString;
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

const E_NOINTERFACE: u32 = 0x8000_4002;
const CLASS_NOT_REGISTERED: u32 = 0x8004_0154;
const DPERR_ALREADYINITIALIZED: u32 = 0x8877_0005;
const DPERR_BUFFERTOOSMALL: u32 = 0x8877_001E;
const DPERR_CANTCREATESESSION: u32 = 0x8877_0046;
const DPERR_INVALIDPARAMS: u32 = 0x8007_0057;
const DPERR_INVALIDPLAYER: u32 = 0x8877_0096;
const DPERR_NOMESSAGES: u32 = 0x8877_00BE;
const DPERR_NOSESSIONS: u32 = 0x8877_00DC;
const DPERR_UNINITIALIZED: u32 = 0x8877_0140;
const DPERR_NOTLOBBIED: u32 = 0x8877_042E;

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

/// Which object: `IDirectPlay4A` or `IDirectPlayLobby2A`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Play,
    Lobby,
}

struct Object {
    kind: Kind,
    refs: AtomicU32,
    state: Mutex<State>,
}

/// The objects by id (what the game holds of each: never 0).
static OBJECTS: Mutex<Vec<(u32, Arc<Object>)>> = Mutex::new(Vec::new());
static NEXT: AtomicU32 = AtomicU32::new(1);

fn object(this: u32) -> Arc<Object> {
    let objects = OBJECTS.lock().unwrap_or_else(|e| e.into_inner());
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
    let mut s = o.state.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut s)
}

fn session(this: u32) -> Option<Arc<Session>> {
    with_state(this, |s| s.session.clone())
}

fn make(kind: Kind) -> u32 {
    let id = NEXT.fetch_add(1, Ordering::SeqCst);
    OBJECTS.lock().unwrap_or_else(|e| e.into_inner()).push((
        id,
        Arc::new(Object {
            kind,
            refs: AtomicU32::new(1),
            state: Mutex::new(State::default()),
        }),
    ));
    id
}

fn read_guid(at: u32) -> Option<Guid> {
    (at != 0).then(|| crate::pointers::read_or_end(at, "a DirectPlay GUID"))
}

fn put(at: u32, v: u32) {
    if at != 0 {
        crate::pointers::write_or_end(at, v, "a DirectPlay method's result")
    }
}

fn get(at: u32) -> u32 {
    crate::pointers::read_or_end(at, "a DirectPlay method's argument")
}

fn end(e: crate::errors::BadInput) -> ! {
    crate::fatal::raise::end("net::dp_objects", e)
}

/// A NUL-ended text at `at` (none: empty).
fn text(at: u32) -> String {
    if at == 0 {
        return String::new();
    }
    let bytes = crate::files::cstring(crate::pointers::GamePtr::new(at), "a DirectPlay text")
        .unwrap_or_else(|e| end(e));
    String::from_utf8_lossy(&bytes).into_owned()
}

/// A DPNAME's short name.
fn short_name(name: u32) -> String {
    if name == 0 {
        return String::new();
    }
    text(crate::pointers::read_or_end::<DpName>(name, "a DPNAME").short_name)
}

fn bytes(at: u32, n: u32) -> Vec<u8> {
    if at == 0 || n == 0 {
        return vec![];
    }
    crate::pointers::GamePtr::new(at)
        .read_bytes(n as usize, "DirectPlay data")
        .unwrap_or_else(|e| end(e))
}

/// The thing's bytes copied to `at`, `size` (in: the room, out: the bytes' count) set:
/// DPERR_BUFFERTOOSMALL when they do not fit.
fn give(data: &[u8], at: u32, size: u32) -> u32 {
    if size == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let room = get(size) as usize;
    put(size, data.len() as u32);
    if at == 0 || room < data.len() {
        return DPERR_BUFFERTOOSMALL;
    }
    crate::pointers::GamePtr::new(at)
        .write_bytes(data, "DirectPlay data")
        .unwrap_or_else(|e| end(e));
    DP_OK
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

/// `CoCreateInstance` for DirectPlay's two classes: the object into `out`, or the error.
pub fn create(class: u32, interface: u32, out: u32) -> i32 {
    let (Some(class), Some(interface)) = (read_guid(class), read_guid(interface)) else {
        return DPERR_INVALIDPARAMS as i32;
    };
    let made = if class == CLSID_DIRECTPLAY && interface == IID_IDIRECTPLAY4A {
        make(Kind::Play)
    } else if class == CLSID_DIRECTPLAYLOBBY && interface == IID_IDIRECTPLAYLOBBY2A {
        make(Kind::Lobby)
    } else {
        put(out, 0);
        return CLASS_NOT_REGISTERED as i32;
    };
    put(out, made);
    DP_OK as i32
}

// ---- IUnknown ----

/// `QueryInterface`: the object itself for its own interface (one reference more).
pub fn query_interface(this: u32, iid: u32, out: u32) -> u32 {
    let o = object(this);
    let own = match o.kind {
        Kind::Play => IID_IDIRECTPLAY4A,
        Kind::Lobby => IID_IDIRECTPLAYLOBBY2A,
    };
    if read_guid(iid) == Some(own) {
        o.refs.fetch_add(1, Ordering::SeqCst);
        put(out, this);
        DP_OK
    } else {
        put(out, 0);
        E_NOINTERFACE
    }
}

/// `Release`: the references left; at none the session is closed and the object goes.
pub fn release(this: u32) -> u32 {
    let left = object(this).refs.fetch_sub(1, Ordering::SeqCst) - 1;
    if left == 0 {
        if let Some(s) = session(this) {
            s.close();
        }
        OBJECTS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
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

pub fn create_group(this: u32, id: u32, name: u32, data: u32, size: u32, _flags: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    let (result, made) = s.create_group(&short_name(name), &bytes(data, size));
    if result == DP_OK {
        put(id, made);
    }
    result
}

pub fn create_player(
    this: u32,
    id: u32,
    name: u32,
    event: u32,
    data: u32,
    size: u32,
    _flags: u32,
) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    let (result, made) = s.create_player(&short_name(name), &bytes(data, size), event);
    if result == DP_OK {
        put(id, made);
    }
    result
}

pub fn delete_player_from_group(this: u32, group: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.remove_from_group(group, player))
}

pub fn destroy_player(this: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.destroy_player(player))
}

/// `EnumPlayers` / `EnumGroups`' callback: (the id, the kind, its DPNAME, flags, the
/// context) -> 0 to stop.
pub type EnumPlayersCallback = fn(u32, u32, u32, u32, u32) -> u32;

/// The players (kind 1) or the groups (kind 0) to `callback`, each with a DPNAME.
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
        // Lent to the callback, which reads it as the game's memory.
        let _lent = crate::pointers::Lent::new(name.as_bytes_with_nul());
        let dpname = dp_name(&name);
        if callback(
            id,
            kind,
            crate::pointers::game_address(&dpname as *const DpName),
            0,
            context,
        ) == 0
        {
            break;
        }
    }
    DP_OK
}

pub fn enum_groups(
    this: u32,
    _guid: u32,
    callback: EnumPlayersCallback,
    context: u32,
    _flags: u32,
) -> u32 {
    enumerate(this, callback, context, 0)
}

pub fn enum_players(
    this: u32,
    _guid: u32,
    callback: EnumPlayersCallback,
    context: u32,
    _flags: u32,
) -> u32 {
    enumerate(this, callback, context, 1)
}

/// A session's DPSESSIONDESC2 (its name at `name`).
fn session_desc(d: &Desc, name: &CString) -> DpSessionDesc {
    DpSessionDesc {
        size: size_of::<DpSessionDesc>() as u32,
        flags: d.flags,
        instance: d.instance,
        application: d.application,
        max_players: d.max_players,
        current_players: d.current_players,
        name: crate::pointers::game_address(name.as_ptr()),
        password: 0,
        reserved: [0; 2],
        user: d.user,
    }
}

/// A DPNAME of a short name (kept alive by `name`).
fn dp_name(name: &std::ffi::CStr) -> DpName {
    DpName {
        size: size_of::<DpName>() as u32,
        flags: 0,
        short_name: crate::pointers::game_address(name.as_ptr()),
        long_name: 0,
    }
}

/// A DPSESSIONDESC2 read (its instance, application, name, flags, most players, user data).
fn read_desc(at: u32) -> Desc {
    {
        let d: DpSessionDesc = crate::pointers::read_or_end(at, "a DPSESSIONDESC2");
        Desc {
            flags: d.flags,
            instance: d.instance,
            application: d.application,
            max_players: d.max_players,
            current_players: d.current_players,
            name: text(d.name),
            user: d.user,
        }
    }
}

/// `EnumSessions`' callback: (its DPSESSIONDESC2 or 0 at the end, where the time-out is,
/// flags, the context) -> 0 to stop.
pub type EnumSessionsCallback = fn(u32, u32, u32, u32) -> u32;

pub fn enum_sessions(
    this: u32,
    desc: u32,
    timeout: u32,
    callback: EnumSessionsCallback,
    context: u32,
    flags: u32,
) -> u32 {
    let Some((host, port)) = with_state(this, |s| s.initialized.then(|| (s.host.clone(), s.port)))
    else {
        return DPERR_UNINITIALIZED;
    };
    if desc == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let application = read_desc(desc).application;
    let mut time = timeout;
    let time_at = crate::pointers::game_address(&mut time as *mut u32);
    for (d, _) in find_sessions(&host, port, application) {
        let closed =
            d.flags & 0x21 != 0 || (d.max_players != 0 && d.current_players >= d.max_players);
        if flags & DPENUMSESSIONS_AVAILABLE != 0 && closed {
            continue;
        }
        let name = CString::new(d.name.clone()).unwrap_or_default();
        // Lent to the callback, which reads it as the game's memory.
        let _lent = crate::pointers::Lent::new(name.as_bytes_with_nul());
        let desc = session_desc(&d, &name);
        if callback(
            crate::pointers::game_address(&desc as *const DpSessionDesc),
            time_at,
            0,
            context,
        ) == 0
        {
            return DP_OK;
        }
    }
    callback(0, time_at, DPESC_TIMEDOUT, context);
    DP_OK
}

pub fn get_player_data(this: u32, player: u32, data: u32, size: u32, _flags: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    match s.world().players.get(&player) {
        Some(p) => give(&p.data, data, size),
        None => DPERR_INVALIDPLAYER,
    }
}

pub fn open(this: u32, desc: u32, flags: u32) -> u32 {
    if desc == 0 {
        return DPERR_INVALIDPARAMS;
    }
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
    let d = read_desc(desc);
    let made = if flags & DPOPEN_CREATE != 0 {
        let d = Desc {
            instance: fresh_guid(),
            ..d
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

pub fn receive(this: u32, from: u32, to: u32, _flags: u32, data: u32, size: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    if size == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let room = { if data == 0 { 0 } else { get(size) as usize } };
    match s.receive(room) {
        None => DPERR_NOMESSAGES,
        Some(Err(n)) => {
            put(size, n as u32);
            DPERR_BUFFERTOOSMALL
        }
        Some(Ok(m)) => {
            put(from, m.from);
            put(to, m.to);
            give(&m.data, data, size)
        }
    }
}

pub fn send(this: u32, from: u32, to: u32, _flags: u32, data: u32, size: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| {
        s.send(from, to, &bytes(data, size))
    })
}

pub fn set_player_data(this: u32, player: u32, data: u32, size: u32, _flags: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| {
        s.set_player_data(player, &bytes(data, size))
    })
}

pub fn set_session_desc(this: u32, desc: u32, _flags: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    if desc == 0 {
        return DPERR_INVALIDPARAMS;
    }
    s.set_desc(read_desc(desc))
}

/// `EnumConnections`' callback: (the service provider's GUID, the connection's address,
/// its size, its DPNAME, flags, the context) -> 0 to stop.
pub type EnumConnectionsCallback = fn(u32, u32, u32, u32, u32, u32) -> u32;

pub fn enum_connections(
    this: u32,
    _app: u32,
    callback: EnumConnectionsCallback,
    context: u32,
    _flags: u32,
) -> u32 {
    let _ = this;
    let address = compound(&[(DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec())]);
    // Lent to the callback, which reads it as the game's memory.
    let _lent = crate::pointers::Lent::new(&address);
    let name = c"Internet TCP/IP Connection For DirectPlay";
    let dpname = dp_name(name);
    callback(
        crate::pointers::game_address(DPSPGUID_TCPIP.as_ptr()),
        crate::pointers::game_address(address.as_ptr()),
        address.len() as u32,
        crate::pointers::game_address(&dpname as *const DpName),
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

pub fn initialize_connection(this: u32, address: u32, _flags: u32) -> u32 {
    if with_state(this, |s| s.initialized) {
        return DPERR_ALREADYINITIALIZED;
    }
    if address == 0 {
        return DPERR_INVALIDPARAMS;
    }
    // the total size is the first element's data
    let total = { get(address + 20) } as usize;
    let raw = bytes(address, total.clamp(24, 4096) as u32);
    let parts = elements(&raw);
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

/// The lobby's `Connect`: never launched by a lobby.
pub fn connect(_this: u32, _flags: u32, _out: u32, _outer: u32) -> u32 {
    DPERR_NOTLOBBIED
}

/// The lobby's `GetConnectionSettings`: never launched by a lobby.
pub fn get_connection_settings(_this: u32, _app: u32, _data: u32, _size: u32) -> u32 {
    DPERR_NOTLOBBIED
}

/// The lobby's `SetConnectionSettings`: never launched by a lobby.
pub fn set_connection_settings(_this: u32, _flags: u32, _app: u32, _settings: u32) -> u32 {
    DPERR_NOTLOBBIED
}

pub fn create_compound_address(_this: u32, list: u32, count: u32, address: u32, size: u32) -> u32 {
    if (list == 0 && count != 0) || size == 0 {
        return DPERR_INVALIDPARAMS;
    }
    // DPCOMPOUNDADDRESSELEMENT: the type's GUID, the data's size, the data
    let parts: Vec<(Guid, Vec<u8>)> = (0..count)
        .map(|n| {
            let e: DpAddressElement = crate::pointers::read_or_end(
                list + n * size_of::<DpAddressElement>() as u32,
                "a DPCOMPOUNDADDRESSELEMENT",
            );
            (e.data_type, bytes(e.data, e.data_size))
        })
        .collect();
    give(&compound(&parts), address, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A value kept for the test's life, at a game address.
    fn kept<T: Copy + 'static>(value: T) -> u32 {
        let value: &'static T = Box::leak(Box::new(value));
        crate::pointers::lend(value as *const T, size_of::<T>())
    }

    fn kept_bytes(bytes: &[u8]) -> u32 {
        let bytes: &'static [u8] = Box::leak(bytes.to_vec().into_boxed_slice());
        crate::pointers::lend(bytes.as_ptr(), bytes.len())
    }

    static FOUND: Mutex<Vec<Guid>> = Mutex::new(Vec::new());

    fn session_found(desc: u32, _time: u32, flags: u32, _context: u32) -> u32 {
        if flags & DPESC_TIMEDOUT == 0 && desc != 0 {
            let d: DpSessionDesc = crate::pointers::read_or_end(desc, "a session found");
            FOUND.lock().unwrap().push(d.instance);
        }
        1
    }

    const APPLICATION: Guid = guid(0x1234_5678, 0x9ABC, 0xDEF0, [1, 2, 3, 4, 5, 6, 7, 8]);

    fn play_object(port: u16) -> u32 {
        let out = kept(0u32);
        let made = create(kept(CLSID_DIRECTPLAY), kept(IID_IDIRECTPLAY4A), out);
        assert_eq!(made, DP_OK as i32);
        let this: u32 = crate::pointers::read_or_end(out, "the object");
        let address = compound(&[
            (DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec()),
            (DPAID_INET, b"127.0.0.1\0".to_vec()),
            (DPAID_INETPORT, port.to_le_bytes().to_vec()),
        ]);
        assert_eq!(initialize_connection(this, kept_bytes(&address), 0), DP_OK);
        this
    }

    fn desc(instance: Guid) -> u32 {
        kept(DpSessionDesc {
            size: size_of::<DpSessionDesc>() as u32,
            flags: 0,
            instance,
            application: APPLICATION,
            max_players: 4,
            current_players: 0,
            name: kept_bytes(b"test\0"),
            password: 0,
            reserved: [0; 2],
            user: [0; 4],
        })
    }

    fn player(this: u32, name: &[u8]) -> u32 {
        let id = kept(0u32);
        let dp_name = kept(DpName {
            size: size_of::<DpName>() as u32,
            flags: 0,
            short_name: kept_bytes(name),
            long_name: 0,
        });
        assert_eq!(create_player(this, id, dp_name, 0, 0, 0, 0), DP_OK);
        crate::pointers::read_or_end(id, "a player's id")
    }

    #[test]
    fn a_session_is_hosted_found_joined_and_talked_to() {
        let port = 47_700 + (std::process::id() % 200) as u16;
        let host = play_object(port);
        assert_eq!(open(host, desc([0; 16]), DPOPEN_CREATE), DP_OK);
        let host_player = player(host, b"host\0");

        let guest = play_object(port);
        let mut found = None;
        for _ in 0..100 {
            assert_eq!(
                enum_sessions(guest, desc([0; 16]), 0, session_found, 0, 0),
                DP_OK
            );
            found = FOUND.lock().unwrap().first().copied();
            if found.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let instance = found.expect("the hosted session is found");
        assert_eq!(open(guest, desc(instance), DPOPEN_JOIN), DP_OK);
        let guest_player = player(guest, b"guest\0");

        assert_eq!(
            send(guest, guest_player, host_player, 0, kept_bytes(b"hello"), 5),
            DP_OK
        );
        let (from, to, data, size) = (kept(0u32), kept(0u32), kept([0u8; 64]), kept(0u32));
        let mut got = None;
        for _ in 0..200 {
            crate::pointers::write_or_end(size, 64u32, "the room");
            if receive(host, from, to, 0, data, size) == DP_OK {
                let sender: u32 = crate::pointers::read_or_end(from, "from");
                if sender == guest_player {
                    let n: u32 = crate::pointers::read_or_end(size, "size");
                    let bytes: [u8; 64] = crate::pointers::read_or_end(data, "data");
                    got = Some(bytes[..n as usize].to_vec());
                    break;
                }
                // a system message (from 0): the guest's player made
                continue;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(got.as_deref(), Some(&b"hello"[..]));
        assert_eq!(crate::pointers::read_or_end::<u32>(to, "to"), host_player);
        assert_eq!(release(guest), 0);
        assert_eq!(release(host), 0);
    }
}
