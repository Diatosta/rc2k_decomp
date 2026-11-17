//! Valve's Steamworks library, loaded while the program runs and called through its "flat" C
//! interface. Nothing is linked: when the library is not beside the program (or not on the
//! system's search path), or lacks a function this file needs, Steam is simply not available.
//!
//! The layouts and interface versions below are those of the Steamworks SDK carried by
//! `steamworks-sys` 0.13.0, the one whose redistributable library `build.sh` copies beside the
//! program: `SteamMatchmaking_v009`, `SteamFriends_v018`, `SteamNetworkingSockets_SteamAPI_v012`
//! and `SteamNetworkingUtils_SteamAPI_v004`. A library of a much newer SDK keeps the flat
//! functions of older interface versions only for a while; a missing name makes Steam
//! unavailable (and says which in the program's log).
//!
//! The program drives Steam's callbacks itself (the library's "manual dispatch"): `run_frame`
//! is called from one thread, which hands each callback to its caller, and answers of
//! asynchronous calls to the handler that was registered with the call.

#![allow(unsafe_code)]

use super::callbacks::{self, id};
use sdl3_sys::everything::{
    SDL_FunctionPointer, SDL_LoadFunction, SDL_LoadObject, SDL_SharedObject,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::Mutex;

type SteamId = u64;
type Call = u64;
type Conn = u32;

/// A handler for the answer of one asynchronous call: the answer's bytes, `None` when the call
/// failed.
pub type CallHandler = Box<dyn FnOnce(Option<&[u8]>) + Send>;

/// The size of `SteamNetworkingIdentity` (a type, a size and 128 bytes of what it names).
const IDENTITY_SIZE: usize = 136;
/// The flat functions' application error message buffer (`SteamErrMsg`).
const ERR_MSG_SIZE: usize = 1024;
/// `k_nSteamNetworkingSend_ReliableNoNagle`.
const SEND_RELIABLE_NO_NAGLE: c_int = 9;
/// `k_ESteamNetConnectionEnd_App_Generic`: a normal close by the application.
const END_APP_GENERIC: c_int = 1000;
/// `k_EFriendFlagImmediate`.
const FRIEND_FLAG_IMMEDIATE: c_int = 0x04;
/// `k_ELobbyTypePublic`.
const LOBBY_PUBLIC: c_int = 2;
/// `k_ELobbyComparisonEqual`.
const COMPARE_EQUAL: c_int = 0;
/// `k_ELobbyDistanceFilterWorldwide`.
const DISTANCE_WORLDWIDE: c_int = 3;

/// The start of `SteamNetworkingMessage_t`, all this program reads.
#[repr(C)]
struct NetMessage {
    data: *mut c_void,
    size: c_int,
}

/// `CallbackMsg_t`.
#[repr(C)]
struct CallbackMsg {
    user: c_int,
    callback: c_int,
    param: *mut u8,
    size: c_int,
}

/// `FriendGameInfo_t`.
#[repr(C)]
#[derive(Default)]
struct FriendGame {
    game: u64,
    ip: u32,
    port: u16,
    query_port: u16,
    lobby: u64,
}

macro_rules! symbols {
    ($($field:ident: $ty:ty = $name:literal,)*) => {
        /// The library's functions, by the names they are looked up under.
        struct Symbols {
            $($field: $ty,)*
        }

        impl Symbols {
            /// Every name the library must carry, for the check that it does.
            #[cfg(test)]
            const NAMES: &[&str] = &[$($name),*];

            // One symbol (`dispatch_init`) has SDL's own pointer type, which makes its cast look
            // useless to the lint; the other casts are the point.
            #[allow(clippy::useless_transmute)]
            fn load(lib: *mut SDL_SharedObject) -> Result<Symbols, String> {
                Ok(Symbols {
                    $($field: {
                        // SAFETY: `lib` is a library SDL loaded; the name is NUL-ended.
                        let f: SDL_FunctionPointer = unsafe {
                            SDL_LoadFunction(lib, concat!($name, "\0").as_ptr().cast())
                        };
                        let Some(f) = f else {
                            return Err(format!("the library has no {}", $name));
                        };
                        // SAFETY: the declared type is the function's C signature in the SDK's
                        // `steam_api_flat.h` / `steam_api.h`; both sides are plain `extern "C"`
                        // function pointers.
                        unsafe { std::mem::transmute::<unsafe extern "C" fn(), $ty>(f) }
                    },)*
                })
            }
        }
    };
}

symbols! {
    init: unsafe extern "C" fn(*mut c_char) -> c_int = "SteamAPI_InitFlat",
    dispatch_init: unsafe extern "C" fn() = "SteamAPI_ManualDispatch_Init",
    get_pipe: unsafe extern "C" fn() -> c_int = "SteamAPI_GetHSteamPipe",
    run_frame: unsafe extern "C" fn(c_int) = "SteamAPI_ManualDispatch_RunFrame",
    next_callback: unsafe extern "C" fn(c_int, *mut CallbackMsg) -> bool = "SteamAPI_ManualDispatch_GetNextCallback",
    free_callback: unsafe extern "C" fn(c_int) = "SteamAPI_ManualDispatch_FreeLastCallback",
    call_result: unsafe extern "C" fn(c_int, Call, *mut c_void, c_int, c_int, *mut bool) -> bool = "SteamAPI_ManualDispatch_GetAPICallResult",
    matchmaking: unsafe extern "C" fn() -> *mut c_void = "SteamAPI_SteamMatchmaking_v009",
    friends: unsafe extern "C" fn() -> *mut c_void = "SteamAPI_SteamFriends_v018",
    sockets: unsafe extern "C" fn() -> *mut c_void = "SteamAPI_SteamNetworkingSockets_SteamAPI_v012",
    net_utils: unsafe extern "C" fn() -> *mut c_void = "SteamAPI_SteamNetworkingUtils_SteamAPI_v004",
    relay_access: unsafe extern "C" fn(*mut c_void) = "SteamAPI_ISteamNetworkingUtils_InitRelayNetworkAccess",
    create_lobby: unsafe extern "C" fn(*mut c_void, c_int, c_int) -> Call = "SteamAPI_ISteamMatchmaking_CreateLobby",
    join_lobby: unsafe extern "C" fn(*mut c_void, SteamId) -> Call = "SteamAPI_ISteamMatchmaking_JoinLobby",
    leave_lobby: unsafe extern "C" fn(*mut c_void, SteamId) = "SteamAPI_ISteamMatchmaking_LeaveLobby",
    request_list: unsafe extern "C" fn(*mut c_void) -> Call = "SteamAPI_ISteamMatchmaking_RequestLobbyList",
    string_filter: unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, c_int) = "SteamAPI_ISteamMatchmaking_AddRequestLobbyListStringFilter",
    distance_filter: unsafe extern "C" fn(*mut c_void, c_int) = "SteamAPI_ISteamMatchmaking_AddRequestLobbyListDistanceFilter",
    count_filter: unsafe extern "C" fn(*mut c_void, c_int) = "SteamAPI_ISteamMatchmaking_AddRequestLobbyListResultCountFilter",
    lobby_by_index: unsafe extern "C" fn(*mut c_void, c_int) -> SteamId = "SteamAPI_ISteamMatchmaking_GetLobbyByIndex",
    lobby_data: unsafe extern "C" fn(*mut c_void, SteamId, *const c_char) -> *const c_char = "SteamAPI_ISteamMatchmaking_GetLobbyData",
    set_lobby_data: unsafe extern "C" fn(*mut c_void, SteamId, *const c_char, *const c_char) -> bool = "SteamAPI_ISteamMatchmaking_SetLobbyData",
    member_count: unsafe extern "C" fn(*mut c_void, SteamId) -> c_int = "SteamAPI_ISteamMatchmaking_GetNumLobbyMembers",
    lobby_owner: unsafe extern "C" fn(*mut c_void, SteamId) -> SteamId = "SteamAPI_ISteamMatchmaking_GetLobbyOwner",
    set_presence: unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> bool = "SteamAPI_ISteamFriends_SetRichPresence",
    clear_presence: unsafe extern "C" fn(*mut c_void) = "SteamAPI_ISteamFriends_ClearRichPresence",
    friend_count: unsafe extern "C" fn(*mut c_void, c_int) -> c_int = "SteamAPI_ISteamFriends_GetFriendCount",
    friend_by_index: unsafe extern "C" fn(*mut c_void, c_int, c_int) -> SteamId = "SteamAPI_ISteamFriends_GetFriendByIndex",
    friend_game: unsafe extern "C" fn(*mut c_void, SteamId, *mut FriendGame) -> bool = "SteamAPI_ISteamFriends_GetFriendGamePlayed",
    listen_p2p: unsafe extern "C" fn(*mut c_void, c_int, c_int, *const c_void) -> u32 = "SteamAPI_ISteamNetworkingSockets_CreateListenSocketP2P",
    close_listen: unsafe extern "C" fn(*mut c_void, u32) -> bool = "SteamAPI_ISteamNetworkingSockets_CloseListenSocket",
    connect_p2p: unsafe extern "C" fn(*mut c_void, *const c_void, c_int, c_int, *const c_void) -> Conn = "SteamAPI_ISteamNetworkingSockets_ConnectP2P",
    accept: unsafe extern "C" fn(*mut c_void, Conn) -> c_int = "SteamAPI_ISteamNetworkingSockets_AcceptConnection",
    close_conn: unsafe extern "C" fn(*mut c_void, Conn, c_int, *const c_char, bool) -> bool = "SteamAPI_ISteamNetworkingSockets_CloseConnection",
    send: unsafe extern "C" fn(*mut c_void, Conn, *const c_void, u32, c_int, *mut i64) -> c_int = "SteamAPI_ISteamNetworkingSockets_SendMessageToConnection",
    receive: unsafe extern "C" fn(*mut c_void, Conn, *mut *mut NetMessage, c_int) -> c_int = "SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnConnection",
    conn_info: unsafe extern "C" fn(*mut c_void, Conn, *mut c_void) -> bool = "SteamAPI_ISteamNetworkingSockets_GetConnectionInfo",
    release_message: unsafe extern "C" fn(*mut NetMessage) = "SteamAPI_SteamNetworkingMessage_t_Release",
    identity_set_id: unsafe extern "C" fn(*mut c_void, u64) = "SteamAPI_SteamNetworkingIdentity_SetSteamID64",
}

/// What the library's file is called here.
const LIBRARY: &str = if cfg!(target_os = "windows") {
    if cfg!(target_pointer_width = "64") {
        "steam_api64.dll"
    } else {
        "steam_api.dll"
    }
} else if cfg!(target_os = "macos") {
    "libsteam_api.dylib"
} else {
    "libsteam_api.so"
};

/// Where the library may be, in the order tried: the file named by `RC2K_STEAM_LIBRARY`, the
/// program's own folder, then the system's search path.
fn candidates() -> Vec<String> {
    let mut paths = vec![];
    if let Some(p) = std::env::var_os("RC2K_STEAM_LIBRARY") {
        paths.push(p.to_string_lossy().into_owned());
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(std::path::Path::to_path_buf))
    {
        paths.push(dir.join(LIBRARY).to_string_lossy().into_owned());
    }
    paths.push(LIBRARY.to_string());
    paths
}

/// Loads the library from the first place that has it.
fn open_library() -> Result<*mut SDL_SharedObject, String> {
    for path in candidates() {
        let Ok(c) = CString::new(path.as_str()) else {
            continue;
        };
        // SAFETY: a NUL-ended path; SDL loads libraries without being initialised.
        let lib = unsafe { SDL_LoadObject(c.as_ptr()) };
        if !lib.is_null() {
            return Ok(lib);
        }
    }
    Err(format!("{LIBRARY} was not found"))
}

/// A started Steam client, shared by every thread: the functions and the interfaces they are
/// called on, plus the handlers waiting for answers.
pub struct Flat {
    f: Symbols,
    pipe: c_int,
    matchmaking: *mut c_void,
    friends: *mut c_void,
    sockets: *mut c_void,
    calls: Mutex<HashMap<Call, CallHandler>>,
}

// SAFETY: the library's functions and interfaces are callable from any thread (Steam's own
// rule for these interfaces); the pointers are never freed while the program runs.
unsafe impl Send for Flat {}
// SAFETY: as `Send`.
unsafe impl Sync for Flat {}

fn c(text: &str) -> CString {
    CString::new(text.replace('\0', "")).unwrap_or_default()
}

impl Flat {
    /// Loads the library and starts the client as application `app_id`.
    pub fn start(app_id: u32) -> Result<Flat, String> {
        let lib = open_library()?;
        let f = Symbols::load(lib)?;
        // The library reads the application number from the environment; set before it is
        // asked, while nothing else is changing the environment.
        // SAFETY: called once, from the first use of Steam, before any other thread reads
        // these two variables.
        unsafe {
            std::env::set_var("SteamAppId", app_id.to_string());
            std::env::set_var("SteamGameId", app_id.to_string());
        }
        let mut message = [0 as c_char; ERR_MSG_SIZE];
        // SAFETY: a buffer of the size `SteamErrMsg` has.
        let result = unsafe { (f.init)(message.as_mut_ptr()) };
        if result != 0 {
            // SAFETY: the library wrote a NUL-ended text, or left the zeroed buffer.
            let text = unsafe { CStr::from_ptr(message.as_ptr()) }.to_string_lossy();
            return Err(format!(
                "the client did not start (result {result}): {text}"
            ));
        }
        // SAFETY: after a successful init, as the SDK requires.
        let (pipe, matchmaking, friends, sockets, utils) = unsafe {
            (f.dispatch_init)();
            (
                (f.get_pipe)(),
                (f.matchmaking)(),
                (f.friends)(),
                (f.sockets)(),
                (f.net_utils)(),
            )
        };
        if matchmaking.is_null() || friends.is_null() || sockets.is_null() || utils.is_null() {
            return Err("the library gave no interfaces".into());
        }
        // SAFETY: the networking utilities interface the library just gave.
        unsafe { (f.relay_access)(utils) };
        Ok(Flat {
            f,
            pipe,
            matchmaking,
            friends,
            sockets,
            calls: Mutex::new(HashMap::new()),
        })
    }

    /// Runs Steam's pending callbacks: answers of asynchronous calls go to the handler that
    /// was registered with the call, every other callback to `callback` (its number and bytes).
    pub fn run_frame(&self, callback: &mut dyn FnMut(i32, &[u8])) {
        let mut msg = CallbackMsg {
            user: 0,
            callback: 0,
            param: std::ptr::null_mut(),
            size: 0,
        };
        // SAFETY: the dispatch functions of the started client; `msg` is filled by the library
        // and valid until `free_callback`, and its bytes are copied before that call.
        unsafe {
            (self.f.run_frame)(self.pipe);
            while (self.f.next_callback)(self.pipe, &raw mut msg) {
                let bytes: &[u8] = if msg.param.is_null() || msg.size <= 0 {
                    &[]
                } else {
                    std::slice::from_raw_parts(msg.param, msg.size as usize)
                };
                if msg.callback == id::API_CALL_COMPLETED {
                    if let Some((call, expected, size)) = callbacks::api_call_completed(bytes) {
                        self.finish_call(call, expected, size);
                    }
                } else {
                    callback(msg.callback, bytes);
                }
                (self.f.free_callback)(self.pipe);
            }
        }
    }

    /// Fetches a finished call's answer and gives it to its handler.
    fn finish_call(&self, call: Call, expected: i32, size: u32) {
        let mut buffer = vec![0u8; size as usize];
        let mut failed = false;
        // SAFETY: a buffer of the size the library announced, for the call it announced.
        let got = unsafe {
            (self.f.call_result)(
                self.pipe,
                call,
                buffer.as_mut_ptr().cast(),
                size as c_int,
                expected,
                &raw mut failed,
            )
        };
        if !got {
            return;
        }
        let handler = self
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&call);
        if let Some(handler) = handler {
            handler((!failed).then_some(buffer.as_slice()));
        }
    }

    /// Starts an asynchronous call and registers its handler before any callback can run.
    fn start_call(&self, start: impl FnOnce() -> Call, handler: CallHandler) {
        let mut calls = self
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let call = start();
        if call != 0 {
            calls.insert(call, handler);
        } else {
            drop(calls);
            handler(None);
        }
    }

    // ---- matchmaking ----

    /// Makes a public lobby for `most` members; `done` gets the answer (`LobbyCreated_t`).
    pub fn create_lobby(&self, most: u32, done: CallHandler) {
        // SAFETY: the matchmaking interface of the started client.
        self.start_call(
            || unsafe { (self.f.create_lobby)(self.matchmaking, LOBBY_PUBLIC, most as c_int) },
            done,
        );
    }

    /// Joins a lobby; `done` gets the answer (`LobbyEnter_t`).
    pub fn join_lobby(&self, lobby: u64, done: CallHandler) {
        // SAFETY: as `create_lobby`.
        self.start_call(
            || unsafe { (self.f.join_lobby)(self.matchmaking, lobby) },
            done,
        );
    }

    pub fn leave_lobby(&self, lobby: u64) {
        // SAFETY: as `create_lobby`.
        unsafe { (self.f.leave_lobby)(self.matchmaking, lobby) };
    }

    /// Asks for the public lobbies whose data `key` equals `value`, at most `most` of them;
    /// `done` gets the answer (`LobbyMatchList_t`), the lobbies by `lobby_by_index`.
    pub fn request_lobby_list(&self, key: &str, value: &str, most: u32, done: CallHandler) {
        let (key, value) = (c(key), c(value));
        // SAFETY: as `create_lobby`; the texts are NUL-ended and outlive the calls.
        unsafe {
            (self.f.string_filter)(
                self.matchmaking,
                key.as_ptr(),
                value.as_ptr(),
                COMPARE_EQUAL,
            );
            (self.f.distance_filter)(self.matchmaking, DISTANCE_WORLDWIDE);
            (self.f.count_filter)(self.matchmaking, most as c_int);
        }
        // SAFETY: as `create_lobby`.
        self.start_call(|| unsafe { (self.f.request_list)(self.matchmaking) }, done);
    }

    /// The `index`th lobby of the last search.
    pub fn lobby_by_index(&self, index: u32) -> u64 {
        // SAFETY: as `create_lobby`.
        unsafe { (self.f.lobby_by_index)(self.matchmaking, index as c_int) }
    }

    /// A lobby's data under `key`; `None` for none.
    pub fn lobby_data(&self, lobby: u64, key: &str) -> Option<String> {
        let key = c(key);
        // SAFETY: as `create_lobby`; the library answers with a NUL-ended text it owns, copied
        // here at once.
        let text = unsafe { (self.f.lobby_data)(self.matchmaking, lobby, key.as_ptr()) };
        if text.is_null() {
            return None;
        }
        // SAFETY: the text just returned.
        let text = unsafe { CStr::from_ptr(text) }
            .to_string_lossy()
            .into_owned();
        (!text.is_empty()).then_some(text)
    }

    pub fn set_lobby_data(&self, lobby: u64, key: &str, value: &str) {
        let (key, value) = (c(key), c(value));
        // SAFETY: as `create_lobby`; NUL-ended texts.
        unsafe { (self.f.set_lobby_data)(self.matchmaking, lobby, key.as_ptr(), value.as_ptr()) };
    }

    pub fn member_count(&self, lobby: u64) -> u32 {
        // SAFETY: as `create_lobby`.
        unsafe { (self.f.member_count)(self.matchmaking, lobby) }.max(0) as u32
    }

    pub fn lobby_owner(&self, lobby: u64) -> u64 {
        // SAFETY: as `create_lobby`.
        unsafe { (self.f.lobby_owner)(self.matchmaking, lobby) }
    }

    // ---- friends ----

    /// Sets one rich presence key, or clears it with `None`.
    pub fn set_rich_presence(&self, key: &str, value: Option<&str>) {
        let key = c(key);
        let value = value.map(c);
        // SAFETY: as `create_lobby`; NUL-ended texts, the value null to clear.
        unsafe {
            (self.f.set_presence)(
                self.friends,
                key.as_ptr(),
                value.as_ref().map_or(std::ptr::null(), |v| v.as_ptr()),
            )
        };
    }

    pub fn clear_rich_presence(&self) {
        // SAFETY: the friends interface of the started client.
        unsafe { (self.f.clear_presence)(self.friends) };
    }

    /// The game and the lobby each of the player's friends is in, for those in a game.
    pub fn friends_in_games(&self) -> Vec<(u64, u64)> {
        // SAFETY: the friends interface of the started client; `FriendGame` is the structure
        // the library fills.
        unsafe {
            let n = (self.f.friend_count)(self.friends, FRIEND_FLAG_IMMEDIATE);
            (0..n.max(0))
                .filter_map(|i| {
                    let friend = (self.f.friend_by_index)(self.friends, i, FRIEND_FLAG_IMMEDIATE);
                    let mut game = FriendGame::default();
                    (self.f.friend_game)(self.friends, friend, &raw mut game)
                        .then_some((game.game, game.lobby))
                })
                .collect()
        }
    }

    // ---- relay connections ----

    /// A listen socket for connections through the relay; 0 when none was made.
    pub fn listen_p2p(&self) -> u32 {
        // SAFETY: the sockets interface of the started client; no options.
        unsafe { (self.f.listen_p2p)(self.sockets, 0, 0, std::ptr::null()) }
    }

    pub fn close_listen(&self, socket: u32) {
        // SAFETY: as `listen_p2p`.
        unsafe { (self.f.close_listen)(self.sockets, socket) };
    }

    /// Connects through the relay to a Steam account's listen socket; 0 when none was made.
    pub fn connect_p2p(&self, steam_id: u64) -> Conn {
        let mut identity = [0u64; IDENTITY_SIZE / 8];
        // SAFETY: a zeroed buffer of the size `SteamNetworkingIdentity` has, set by the
        // library's own function; no options.
        unsafe {
            (self.f.identity_set_id)(identity.as_mut_ptr().cast(), steam_id);
            (self.f.connect_p2p)(
                self.sockets,
                identity.as_ptr().cast(),
                0,
                0,
                std::ptr::null(),
            )
        }
    }

    /// Accepts a connection that came in on a listen socket.
    pub fn accept(&self, conn: Conn) -> bool {
        // SAFETY: as `listen_p2p`.
        unsafe { (self.f.accept)(self.sockets, conn) == callbacks::RESULT_OK }
    }

    /// Closes a connection, letting what was sent first arrive.
    pub fn close(&self, conn: Conn) {
        let reason = c("closed");
        // SAFETY: as `listen_p2p`; a NUL-ended text.
        unsafe { (self.f.close_conn)(self.sockets, conn, END_APP_GENERIC, reason.as_ptr(), true) };
    }

    /// Sends `data` as one reliable message.
    pub fn send(&self, conn: Conn, data: &[u8]) -> Result<(), i32> {
        // SAFETY: as `listen_p2p`; the bytes are copied by the call.
        let result = unsafe {
            (self.f.send)(
                self.sockets,
                conn,
                data.as_ptr().cast(),
                data.len() as u32,
                SEND_RELIABLE_NO_NAGLE,
                std::ptr::null_mut(),
            )
        };
        if result == callbacks::RESULT_OK {
            Ok(())
        } else {
            Err(result)
        }
    }

    /// Hands every message that has arrived on the connection to `each`.
    pub fn receive(&self, conn: Conn, each: &mut dyn FnMut(&[u8])) {
        let mut batch = [std::ptr::null_mut::<NetMessage>(); 32];
        loop {
            // SAFETY: as `listen_p2p`; room for 32 message pointers.
            let n = unsafe { (self.f.receive)(self.sockets, conn, batch.as_mut_ptr(), 32) };
            if n <= 0 {
                return;
            }
            for &message in &batch[..n as usize] {
                // SAFETY: messages the library just handed over; each is read once and
                // released.
                unsafe {
                    let m = &*message;
                    if !m.data.is_null() && m.size > 0 {
                        each(std::slice::from_raw_parts(m.data.cast(), m.size as usize));
                    }
                    (self.f.release_message)(message);
                }
            }
        }
    }

    /// A connection's state (`ESteamNetworkingConnectionState`); `None` when it is unknown.
    pub fn state(&self, conn: Conn) -> Option<i32> {
        let mut info = [0u64; callbacks::INFO_SIZE / 8];
        // SAFETY: a zeroed, 8-aligned buffer of the size `SteamNetConnectionInfo_t` has.
        let got = unsafe { (self.f.conn_info)(self.sockets, conn, info.as_mut_ptr().cast()) };
        if !got {
            return None;
        }
        let mut bytes = [0u8; callbacks::INFO_SIZE];
        for (chunk, word) in bytes.chunks_mut(8).zip(info) {
            chunk.copy_from_slice(&word.to_le_bytes());
        }
        callbacks::info_state(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With Valve's library to hand (`RC2K_STEAM_LIBRARY` names the file): every function the
    /// program calls is found in it. Nothing is started, so no Steam client is touched.
    #[test]
    fn the_library_has_every_function() {
        let Some(path) = std::env::var_os("RC2K_STEAM_LIBRARY") else {
            eprintln!("RC2K_STEAM_LIBRARY is not set: skipped");
            return;
        };
        let lib = open_library().expect("the library loads");
        if let Err(e) = Symbols::load(lib) {
            panic!("{}: {e}", path.to_string_lossy());
        }
        assert!(Symbols::NAMES.len() > 30);
    }

    #[test]
    fn the_structures_have_the_sdk_sizes() {
        assert_eq!(std::mem::size_of::<FriendGame>(), 24);
        assert_eq!(IDENTITY_SIZE % 8, 0);
        assert_eq!(callbacks::INFO_SIZE % 8, 0);
    }
}
