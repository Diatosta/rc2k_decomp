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
//! The methods the game does not call answer E_NOTIMPL (DPERR_UNSUPPORTED), each with its
//! own number of arguments (`stdcall`: the callee takes them off the stack).

use crate::net::dp_session::{
    DEFAULT_PORT, DP_OK, Desc, Session, Signal, find_sessions, session_address,
};
use crate::net::types::{DpAddressElement, DpName, DpSessionDesc};
use std::ffi::CString;
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

const E_NOTIMPL: u32 = 0x8000_4001;
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

#[repr(C)]
struct Object {
    table: *const usize,
    refs: AtomicU32,
    state: Mutex<State>,
}

fn object<'a>(this: u32) -> &'a Object {
    unsafe { &*(this as *const Object) }
}

fn state(this: u32) -> std::sync::MutexGuard<'static, State> {
    object(this).state.lock().unwrap_or_else(|e| e.into_inner())
}

fn session(this: u32) -> Option<Arc<Session>> {
    state(this).session.clone()
}

fn make(table: *const usize) -> u32 {
    let at = Box::into_raw(Box::new(Object {
        table,
        refs: AtomicU32::new(1),
        state: Mutex::new(State::default()),
    })) as u32;
    // A region until its last release: the game's code reads its table by address.
    crate::pointers::register(at, size_of::<Object>() as u32);
    at
}

unsafe fn read_guid(at: u32) -> Option<Guid> {
    (at != 0).then(|| unsafe { (at as *const Guid).read_unaligned() })
}

unsafe fn put(at: u32, v: u32) {
    if at != 0 {
        unsafe { (at as *mut u32).write_unaligned(v) };
    }
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// A NUL-ended text at `at` (none: empty).
unsafe fn text(at: u32) -> String {
    if at == 0 {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(at as *const std::ffi::c_char) }
        .to_string_lossy()
        .into_owned()
}

/// A DPNAME's short name.
unsafe fn short_name(name: u32) -> String {
    if name == 0 {
        return String::new();
    }
    unsafe { text((name as *const DpName).read_unaligned().short_name) }
}

unsafe fn bytes(at: u32, n: u32) -> Vec<u8> {
    if at == 0 || n == 0 {
        return vec![];
    }
    unsafe { std::slice::from_raw_parts(at as *const u8, n as usize).to_vec() }
}

/// The thing's bytes copied to `at`, `size` (in: the room, out: the bytes' count) set:
/// DPERR_BUFFERTOOSMALL when they do not fit.
unsafe fn give(data: &[u8], at: u32, size: u32) -> u32 {
    unsafe {
        if size == 0 {
            return DPERR_INVALIDPARAMS;
        }
        let room = get(size) as usize;
        put(size, data.len() as u32);
        if at == 0 || room < data.len() {
            return DPERR_BUFFERTOOSMALL;
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), at as *mut u8, data.len());
        DP_OK
    }
}

/// A player's event signalled (a Windows event's handle).
fn signal() -> Signal {
    Arc::new(|event: u32| {
        if event != 0 {
            unsafe { windows_sys::Win32::System::Threading::SetEvent(event as _) };
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
pub unsafe fn create(class: u32, interface: u32, out: u32) -> i32 {
    unsafe {
        let (Some(class), Some(interface)) = (read_guid(class), read_guid(interface)) else {
            return DPERR_INVALIDPARAMS as i32;
        };
        let made = if class == CLSID_DIRECTPLAY && interface == IID_IDIRECTPLAY4A {
            make(play_table())
        } else if class == CLSID_DIRECTPLAYLOBBY && interface == IID_IDIRECTPLAYLOBBY2A {
            make(lobby_table())
        } else {
            put(out, 0);
            return CLASS_NOT_REGISTERED as i32;
        };
        put(out, made);
        DP_OK as i32
    }
}

// ---- IUnknown ----

unsafe extern "system" fn query_play(this: u32, iid: u32, out: u32) -> u32 {
    unsafe { query(this, iid, out, IID_IDIRECTPLAY4A) }
}

unsafe extern "system" fn query_lobby(this: u32, iid: u32, out: u32) -> u32 {
    unsafe { query(this, iid, out, IID_IDIRECTPLAYLOBBY2A) }
}

unsafe fn query(this: u32, iid: u32, out: u32, own: Guid) -> u32 {
    unsafe {
        if read_guid(iid) == Some(own) {
            object(this).refs.fetch_add(1, Ordering::SeqCst);
            put(out, this);
            DP_OK
        } else {
            put(out, 0);
            E_NOINTERFACE
        }
    }
}

unsafe extern "system" fn add_ref(this: u32) -> u32 {
    object(this).refs.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn release(this: u32) -> u32 {
    let left = object(this).refs.fetch_sub(1, Ordering::SeqCst) - 1;
    if left == 0 {
        if let Some(s) = session(this) {
            s.close();
        }
        crate::pointers::unregister(this);
        drop(unsafe { Box::from_raw(this as *mut Object) });
    }
    left
}

// ---- IDirectPlay4A ----

unsafe extern "system" fn add_player_to_group(this: u32, group: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.add_to_group(group, player))
}

unsafe extern "system" fn close(this: u32) -> u32 {
    if let Some(s) = state(this).session.take() {
        s.close();
    }
    DP_OK
}

unsafe extern "system" fn create_group(
    this: u32,
    id: u32,
    name: u32,
    data: u32,
    size: u32,
    _flags: u32,
) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    let (result, made) = unsafe { s.create_group(&short_name(name), &bytes(data, size)) };
    if result == DP_OK {
        unsafe { put(id, made) };
    }
    result
}

unsafe extern "system" fn create_player(
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
    let (result, made) = unsafe { s.create_player(&short_name(name), &bytes(data, size), event) };
    if result == DP_OK {
        unsafe { put(id, made) };
    }
    result
}

unsafe extern "system" fn delete_player_from_group(this: u32, group: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.remove_from_group(group, player))
}

unsafe extern "system" fn destroy_player(this: u32, player: u32) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| s.destroy_player(player))
}

type EnumPlayersCallback = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> u32;

/// The players (kind 1) or the groups (kind 0) to `callback`, each with a DPNAME.
unsafe fn enumerate(this: u32, callback: u32, context: u32, kind: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    if callback == 0 {
        return DPERR_INVALIDPARAMS;
    }
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
    let callback: EnumPlayersCallback = unsafe { std::mem::transmute(callback as usize) };
    for (id, name) in list {
        let name = CString::new(name).unwrap_or_default();
        // Lent to the callback, which reads it as the game's memory.
        let _lent = crate::pointers::Lent::new(name.as_bytes_with_nul());
        let dpname = dp_name(&name);
        if unsafe { callback(id, kind, &dpname as *const DpName as u32, 0, context) } == 0 {
            break;
        }
    }
    DP_OK
}

unsafe extern "system" fn enum_groups(
    this: u32,
    _guid: u32,
    callback: u32,
    context: u32,
    _flags: u32,
) -> u32 {
    unsafe { enumerate(this, callback, context, 0) }
}

unsafe extern "system" fn enum_players(
    this: u32,
    _guid: u32,
    callback: u32,
    context: u32,
    _flags: u32,
) -> u32 {
    unsafe { enumerate(this, callback, context, 1) }
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
        name: name.as_ptr() as u32,
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
        short_name: name.as_ptr() as u32,
        long_name: 0,
    }
}

/// A DPSESSIONDESC2 read (its instance, application, name, flags, most players, user data).
unsafe fn read_desc(at: u32) -> Desc {
    unsafe {
        let d = (at as *const DpSessionDesc).read_unaligned();
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

type EnumSessionsCallback = unsafe extern "system" fn(u32, u32, u32, u32) -> u32;

unsafe extern "system" fn enum_sessions(
    this: u32,
    desc: u32,
    timeout: u32,
    callback: u32,
    context: u32,
    flags: u32,
) -> u32 {
    let (host, port) = {
        let s = state(this);
        if !s.initialized {
            return DPERR_UNINITIALIZED;
        }
        (s.host.clone(), s.port)
    };
    if desc == 0 || callback == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let application = unsafe { read_desc(desc).application };
    let callback: EnumSessionsCallback = unsafe { std::mem::transmute(callback as usize) };
    let mut time = timeout;
    let time_at = &mut time as *mut u32 as u32;
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
        if unsafe { callback(&desc as *const DpSessionDesc as u32, time_at, 0, context) } == 0 {
            return DP_OK;
        }
    }
    unsafe { callback(0, time_at, DPESC_TIMEDOUT, context) };
    DP_OK
}

unsafe extern "system" fn get_player_data(
    this: u32,
    player: u32,
    data: u32,
    size: u32,
    _flags: u32,
) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    match s.world().players.get(&player) {
        Some(p) => unsafe { give(&p.data, data, size) },
        None => DPERR_INVALIDPLAYER,
    }
}

unsafe extern "system" fn open(this: u32, desc: u32, flags: u32) -> u32 {
    if desc == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let port = {
        let s = state(this);
        if !s.initialized {
            return DPERR_UNINITIALIZED;
        }
        if s.session.is_some() {
            return DPERR_ALREADYINITIALIZED;
        }
        s.port
    };
    let d = unsafe { read_desc(desc) };
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
            state(this).session = Some(s);
            DP_OK
        }
        Err(e) => e,
    }
}

unsafe extern "system" fn receive(
    this: u32,
    from: u32,
    to: u32,
    _flags: u32,
    data: u32,
    size: u32,
) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    if size == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let room = unsafe { if data == 0 { 0 } else { get(size) as usize } };
    match s.receive(room) {
        None => DPERR_NOMESSAGES,
        Some(Err(n)) => {
            unsafe { put(size, n as u32) };
            DPERR_BUFFERTOOSMALL
        }
        Some(Ok(m)) => unsafe {
            put(from, m.from);
            put(to, m.to);
            give(&m.data, data, size)
        },
    }
}

unsafe extern "system" fn send(
    this: u32,
    from: u32,
    to: u32,
    _flags: u32,
    data: u32,
    size: u32,
) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| unsafe {
        s.send(from, to, &bytes(data, size))
    })
}

unsafe extern "system" fn set_player_data(
    this: u32,
    player: u32,
    data: u32,
    size: u32,
    _flags: u32,
) -> u32 {
    session(this).map_or(DPERR_UNINITIALIZED, |s| unsafe {
        s.set_player_data(player, &bytes(data, size))
    })
}

unsafe extern "system" fn set_session_desc(this: u32, desc: u32, _flags: u32) -> u32 {
    let Some(s) = session(this) else {
        return DPERR_UNINITIALIZED;
    };
    if desc == 0 {
        return DPERR_INVALIDPARAMS;
    }
    s.set_desc(unsafe { read_desc(desc) })
}

type EnumConnectionsCallback = unsafe extern "system" fn(u32, u32, u32, u32, u32, u32) -> u32;

unsafe extern "system" fn enum_connections(
    this: u32,
    _app: u32,
    callback: u32,
    context: u32,
    _flags: u32,
) -> u32 {
    let _ = this;
    if callback == 0 {
        return DPERR_INVALIDPARAMS;
    }
    let address = compound(&[(DPAID_SERVICEPROVIDER, DPSPGUID_TCPIP.to_vec())]);
    // Lent to the callback, which reads it as the game's memory.
    let _lent = crate::pointers::Lent::new(&address);
    let name = c"Internet TCP/IP Connection For DirectPlay";
    let dpname = dp_name(name);
    let callback: EnumConnectionsCallback = unsafe { std::mem::transmute(callback as usize) };
    unsafe {
        callback(
            DPSPGUID_TCPIP.as_ptr() as u32,
            address.as_ptr() as u32,
            address.len() as u32,
            &dpname as *const DpName as u32,
            0,
            context,
        )
    };
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

unsafe extern "system" fn initialize_connection(this: u32, address: u32, _flags: u32) -> u32 {
    let mut s = state(this);
    if s.initialized {
        return DPERR_ALREADYINITIALIZED;
    }
    if address == 0 {
        return DPERR_INVALIDPARAMS;
    }
    // the total size is the first element's data
    let total = unsafe { get(address + 20) } as usize;
    let raw = unsafe { bytes(address, total.clamp(24, 4096) as u32) };
    let parts = elements(&raw);
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

unsafe extern "system" fn not_lobbied4(_this: u32, _a: u32, _b: u32, _c: u32) -> u32 {
    DPERR_NOTLOBBIED
}

unsafe extern "system" fn create_compound_address(
    _this: u32,
    list: u32,
    count: u32,
    address: u32,
    size: u32,
) -> u32 {
    if (list == 0 && count != 0) || size == 0 {
        return DPERR_INVALIDPARAMS;
    }
    // DPCOMPOUNDADDRESSELEMENT: the type's GUID, the data's size, the data
    let parts: Vec<(Guid, Vec<u8>)> = (0..count)
        .map(|n| unsafe {
            let e = (list as *const DpAddressElement)
                .add(n as usize)
                .read_unaligned();
            (e.data_type, bytes(e.data, e.data_size))
        })
        .collect();
    unsafe { give(&compound(&parts), address, size) }
}

// ---- the tables ----

macro_rules! unsupported {
    ($name:ident, $($arg:ident),*) => {
        unsafe extern "system" fn $name(_this: u32, $($arg: u32),*) -> u32 {
            $(let _ = $arg;)*
            E_NOTIMPL
        }
    };
}

unsupported!(no2, a);
unsupported!(no3, a, b);
unsupported!(no4, a, b, c);
unsupported!(no5, a, b, c, d);
unsupported!(no6, a, b, c, d, e);
unsupported!(no7, a, b, c, d, e, f);
unsupported!(no10, a, b, c, d, e, f, g, h, i);

macro_rules! table {
    ($($f:expr),* $(,)?) => {
        [$($f as *const () as usize),*]
    };
}

fn play_table() -> *const usize {
    static TABLE: OnceLock<[usize; 53]> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            table![
                query_play,               // 0x00 QueryInterface
                add_ref,                  // 0x04 AddRef
                release,                  // 0x08 Release
                add_player_to_group,      // 0x0C AddPlayerToGroup
                close,                    // 0x10 Close
                create_group,             // 0x14 CreateGroup
                create_player,            // 0x18 CreatePlayer
                delete_player_from_group, // 0x1C DeletePlayerFromGroup
                no2,                      // 0x20 DestroyGroup
                destroy_player,           // 0x24 DestroyPlayer
                no6,                      // 0x28 EnumGroupPlayers
                enum_groups,              // 0x2C EnumGroups
                enum_players,             // 0x30 EnumPlayers
                enum_sessions,            // 0x34 EnumSessions
                no3,                      // 0x38 GetCaps
                no5,                      // 0x3C GetGroupData
                no4,                      // 0x40 GetGroupName
                no3,                      // 0x44 GetMessageCount
                no4,                      // 0x48 GetPlayerAddress
                no4,                      // 0x4C GetPlayerCaps
                get_player_data,          // 0x50 GetPlayerData
                no4,                      // 0x54 GetPlayerName
                no3,                      // 0x58 GetSessionDesc
                no2,                      // 0x5C Initialize
                open,                     // 0x60 Open
                receive,                  // 0x64 Receive
                send,                     // 0x68 Send
                no5,                      // 0x6C SetGroupData
                no4,                      // 0x70 SetGroupName
                set_player_data,          // 0x74 SetPlayerData
                no4,                      // 0x78 SetPlayerName
                set_session_desc,         // 0x7C SetSessionDesc
                no3,                      // 0x80 AddGroupToGroup
                no7,                      // 0x84 CreateGroupInGroup
                no3,                      // 0x88 DeleteGroupFromGroup
                enum_connections,         // 0x8C EnumConnections
                no6,                      // 0x90 EnumGroupsInGroup
                no5,                      // 0x94 GetGroupConnectionSettings
                initialize_connection,    // 0x98 InitializeConnection
                no5,                      // 0x9C SecureOpen
                no5,                      // 0xA0 SendChatMessage
                no4,                      // 0xA4 SetGroupConnectionSettings
                no3,                      // 0xA8 StartSession
                no3,                      // 0xAC GetGroupFlags
                no3,                      // 0xB0 GetGroupParent
                no5,                      // 0xB4 GetPlayerAccount
                no3,                      // 0xB8 GetPlayerFlags
                no3,                      // 0xBC GetGroupOwner
                no3,                      // 0xC0 SetGroupOwner
                no10,                     // 0xC4 SendEx
                no6,                      // 0xC8 GetMessageQueue
                no3,                      // 0xCC CancelMessage
                no4,                      // 0xD0 CancelPriority
            ]
        })
        .as_ptr()
}

fn lobby_table() -> *const usize {
    static TABLE: OnceLock<[usize; 15]> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            table![
                query_lobby,             // 0x00 QueryInterface
                add_ref,                 // 0x04 AddRef
                release,                 // 0x08 Release
                not_lobbied4,            // 0x0C Connect
                no7,                     // 0x10 CreateAddress
                no5,                     // 0x14 EnumAddress
                no5,                     // 0x18 EnumAddressTypes
                no4,                     // 0x1C EnumLocalApplications
                not_lobbied4,            // 0x20 GetConnectionSettings
                no6,                     // 0x24 ReceiveLobbyMessage
                no5,                     // 0x28 RunApplication
                no5,                     // 0x2C SendLobbyMessage
                not_lobbied4,            // 0x30 SetConnectionSettings
                no4,                     // 0x34 SetLobbyMessageEvent
                create_compound_address, // 0x38 CreateCompoundAddress
            ]
        })
        .as_ptr()
}
