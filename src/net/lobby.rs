//! The network game's set-up and lobby (agent G, read 2026-10-19; a reading, not seen in
//! play): the variables the lobby's functions share, the calls they make by place (the
//! callee's IDA address), and the self-tests' tools.
//!
//! The way through (`net::start_link`, 0x416F50, eax 1 host, 2 join, 3 launched by a lobby):
//! the player's record set up (`net::take_player_settings`, `net::fill_own_player`), the
//! DirectPlay connections listed (`net::find_connections`: the lobby object, DirectPlay,
//! `EnumConnections` into the 0x54-byte records at `dword_52AC38`), one chosen
//! (`net::choose_connection`, a list screen) unless the one saved (`unk_52AC94`) is there,
//! the connection made (`net::init_connection`), then hosting (`net::host_session`) or the
//! sessions listed (`net::choose_session`, `EnumSessions` into the 0x68-byte records at
//! `dword_52ACF8`) and one joined (`net::join_session`). Then the lobby screen
//! (`net::lobby_loop`: chat, car, stage, ready) and the race (`net::lets_rally`) until a
//! player leaves. The settings screen before all that is `net::settings_loop` (0x4168CF).
//! Each player's record is 16 bytes of DirectPlay player data (`byte_52AD64` for this
//! machine: +0 the state bits, 1 ready, 2 waiting, 4 racing, 0x10 the host; +1 the car, +2
//! its colours, +3 the nationality, +4 the stage, +5 its direction, +6 the race number,
//! +7 a random number, +8 the race's time, +0xC the total), read into each player's block
//! of `g_netReceived` (+0x20 the DirectPlay id, +0x24 the record).
//!
//! The calls by place: in the game the Rust versions, the logging, DirectPlay's `Send`
//! through the game (`net::game_part`), the screens' frames and the platform through the
//! game; DirectPlay's methods are called from Rust (`net::dplay`). In the tests: stand-ins
//! that note each call with its seven registers and answer what the test says, and test
//! COM objects whose methods note their arguments.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};

pub use crate::net::{Part, a, g, p};

pub const SIGN_FLAG: u32 = 1 << 7;
// ---- game data (IDA addresses) ----

pub const SESSION: usize = 0x52A910; // g_netSession: 1 while a network game runs
pub const PLAYER: usize = 0x52A920; // g_netPlayer: this machine's player
pub const PLAYERS: usize = 0x52AD98; // g_netPlayers
pub const RECEIVED: usize = 0x52AD9C; // g_netReceived: 0x804C bytes a player
pub const BLOCK: u32 = std::mem::size_of::<crate::net::types::NetPlayerBlock>() as u32;
pub const MODE: usize = 0x52A908; // dword_52A908: 0 the settings, 1 host, 2 join, 3 lobby
pub const SYNC: usize = 0x52A914; // byte_52A914: counted up at each lobby
pub const SEQUENCE: usize = 0x52A915; // g_netSequence
pub const FIRST_HOST: usize = 0x52A924; // dword_52A924: the first player not the host
pub const OWN: usize = 0x52AD64; // byte_52AD64: this machine's player record, 16 bytes
pub const OWN_TIME: usize = 0x52AD6C; // dword_52AD6C
pub const OWN_TOTAL: usize = 0x52AD70; // dword_52AD70
pub const ALL_RACED: usize = 0x52AD78; // dword_52AD78: every player is ready
pub const RACE_OVER: usize = 0x52AD7C; // dword_52AD7C: every player has finished
pub const BACK_FROM_RACE: usize = 0x52AD80; // dword_52AD80
pub const RACING: usize = 0x52AD84; // dword_52AD84: the players are not listed again
pub const SYNC_CLOCK: usize = 0x52AD88; // dword_52AD88
pub const SYNC_FIRST: usize = 0x52AD8C; // dword_52AD8C: the sync table not cleared yet
pub const SYNCED: usize = 0x52AD90; // byte_52AD90: 8 bytes, 1 a player answered
pub const CLOCK_A: usize = 0x52A4C4; // dword_52A4C4: the players' last listing
pub const RECEIVE_DONE: usize = 0x52A4B0; // g_netReceiveDone
pub const CHAT: usize = 0x52A508; // byte_52A508: the chat's lines
pub const CHAT_LINE: usize = 0x52A808; // byte_52A808: a line being cut
pub const CHAT_TEXT: usize = 0x52A888; // word_52A888: "name> text"
pub const CHAT_INPUT: usize = 0x52A4C8; // byte_52A4C8: what this player types
pub const EDITED: usize = 0x52A4B8; // dword_52A4B8: the item edited (-1: none)
pub const EDIT_TEXT: usize = 0x52A4B4; // dword_52A4B4: its text
pub const EDIT_WIDTH: usize = 0x52A4BC; // dword_52A4BC: the widest it may be
pub const EDIT_FONT: usize = 0x52A4C0; // dword_52A4C0: the font it is measured with
pub const EDIT_SAVED: usize = 0x95EAF8; // byte_95EAF8: the text before editing, 64 bytes
pub const NUMBER_TEXT: usize = 0x95E8F8; // byte_95E8F8: a number edited as text
pub const MACHINE_NAME: usize = 0x52AB90; // g_machineName, 64 bytes
pub const HOST_ADDRESS: usize = 0x52ABE4; // unk_52ABE4: the host's address (TCP/IP), 64 bytes
pub const MOST_PLAYERS: usize = 0x52ABD0; // dword_52ABD0
pub const SESSION_PASSWORD: usize = 0x52ABD4; // unk_52ABD4, 16 bytes
pub const PLAYER_NAME: usize = 0x52AB70; // byte_52AB70, 0x20 bytes
pub const PORT: usize = 0x52AC24; // dword_52AC24
pub const CAR_ITEM: usize = 0x52AC28; // dword_52AC28: the car item can be chosen
pub const STAGE_ITEM: usize = 0x52AC2C; // dword_52AC2C: the stage item can be chosen
pub const SHOW_GAPS: usize = 0x52AC30; // dword_52AC30: times shown as gaps
pub const CONNECTIONS: usize = 0x52AC34; // dword_52AC34: connections listed
pub const CONNECTION_TABLE: usize = 0x52AC38; // dword_52AC38: 0x54 bytes each
pub const ELEMENTS: usize = 0x52AC3C; // dword_52AC3C: address elements
pub const ADDRESS: usize = 0x52AC40; // dword_52AC40: the compound address
pub const ADDRESS_SIZE: usize = 0x52AC44; // bytes
pub const ELEMENT_TABLE: usize = 0x52AC48; // unk_52AC48: 0x18 bytes each
pub const SAVED_CONNECTION: usize = 0x52AC94; // unk_52AC94: its guid, 16 bytes
pub const SAVED_NAME: usize = 0x52ACA4; // unk_52ACA4: its name, 64 bytes
pub const POSTING: usize = 0x52ACE4; // g_netPosting
pub const POSTED: usize = 0x52ACE8; // g_netPostedCall
pub const POSTED_RESULT: usize = 0x52ACEC; // g_netPostedResult
pub const POSTED_ANSWERED: usize = 0x52ACF0; // g_netPostedAnswered
pub const SESSIONS: usize = 0x52ACF4; // dword_52ACF4: sessions listed
pub const SESSION_TABLE: usize = 0x52ACF8; // dword_52ACF8: 0x68 bytes each
pub const SESSION_OPEN: usize = 0x52ACFC; // dword_52ACFC
pub const JOINED: usize = 0x52AD00; // unk_52AD00: the session's guid, 16 bytes
pub const JOINED_NAME: usize = 0x52AD10; // unk_52AD10, 64 bytes
pub const JOINED_MOST: usize = 0x52AD50; // dword_52AD50
pub const JOINED_PASSWORD: usize = 0x52AD54; // unk_52AD54, 16 bytes
pub const SORTED: usize = 0x56AFFC; // unk_56AFFC: the players sorted by time
pub const SEND_BUFFER: usize = 0x52AA68; // g_netSendBuffer: 0x100 bytes
pub const NET_MEMORY: usize = 0x52A928; // g_netMemory
pub const PLAYER_ID: usize = 0x5AB270; // g_netPlayerId
pub const SEND_TO: usize = 0x5AB274; // g_netSendTo: the group
pub const DPLAY_2: usize = 0x5AB284; // dword_5AB284
pub const DPLAY: usize = 0x5AB288; // g_dplay
pub const LOBBY: usize = 0x5AB28C; // g_dplayLobby
pub const LOBBY_LAUNCHED: usize = 0x5AB290; // g_lobbyLaunched
pub const SETTINGS: usize = 0x5AB294; // dword_5AB294: the connection settings
pub const SETTINGS_SIZE: usize = 0x5AB298; // dword_5AB298
pub const DP_NAME: usize = 0x5AB29C; // dword_5AB29C: a DPNAME, 0x10 bytes
pub const SESSION_DESC: usize = 0x5AB2AC; // dword_5AB2AC: a DPSESSIONDESC2, 0x50 bytes
pub const APP_GUID: usize = 0x5AB300; // dword_5AB300: the game's guid
pub const QUITTING: usize = 0x51837C; // g_quitting
pub const SCREENSHOTS_ON: usize = 0x4F52B4; // g_screenshotsOn
pub const TEXT: usize = 0x93E430; // text: the game's text buffer
pub const NOTICE_KIND: usize = 0x4E07A0; // g_noticeKind (a byte)
pub const SPRITES: usize = 0x755581; // g_spriteTable
pub const PALETTE: usize = 0x51E624; // palette: the measuring font
pub const FONT: usize = 0x51E6A4; // font

// ---- the list screens' variables (`menu::list_*`) ----

pub use crate::race::viewer::{
    CHOICE, CHOICE_BEFORE, CLOCK, FRAME_CLOCK, KEY_CLOCK, MOVE_CLOCK, gb, pb,
};

// ---- the calls by place ----

pub const LOG_L2: u32 = 0x401432; // logfile::log_write_l2
pub const LOG_L2_NL: u32 = 0x401439; // logfile::log_write_l2_nl
pub const LOG_L1: u32 = 0x401427; // logfile::log_write_l1
pub const LOG_L1_NL: u32 = 0x40142B; // logfile::log_write_l1_nl
pub const LOG_INT_L1_NL: u32 = 0x401476; // logfile::log_int_l1_nl
pub const LOG_INT_L2_NL: u32 = 0x401484; // logfile::log_int_l2_nl
pub const LOG_HEX_L2: u32 = 0x40162D; // logfile::log_hex8_l2
pub const LOG_HEX_L2_NL: u32 = 0x401634; // logfile::log_hex8_l2_nl
pub const LOG_NEWLINE_L2: u32 = 0x4013F1; // logfile::log_newline_l2
pub const SAME_GUID: u32 = 0x4016F9; // system::same_guid
pub const COPY_PRINTABLE: u32 = 0x4017F2; // strings::copy_printable
pub const ZERO_MEMORY: u32 = 0x401829; // system::zero_memory
pub const NEXT_RANDOM: u32 = 0x4018B4; // math::next_random
pub const APPEND_TIME: u32 = 0x401A39; // strings::append_time
pub const SAME_TEXT: u32 = 0x401EDE; // strings::same_text
pub const STR_APPEND: u32 = 0x401F7D; // strings::str_append
pub const APPEND_STRING: u32 = 0x401FA3; // strings::append_string_by_id
pub const FORMAT_U32: u32 = 0x40204A; // strings::format_u32
pub const TEXT_LENGTH: u32 = 0x4022B8; // strings::text_length
pub const DRAW_KEEP: u32 = 0x402303; // text::draw_text_keep_regs
pub const MEASURE_TEXT: u32 = 0x40230B; // text::measure_text
pub const MEASURE_STRING: u32 = 0x40232C; // text::measure_string_by_id
pub const DRAW_STRING: u32 = 0x40234A; // text::draw_string_by_id
pub const DRAW_TIME: u32 = 0x402395; // text::draw_time_text
pub const DRAW_TEXT: u32 = 0x4023D5; // text::draw_text
pub const ALLOCATE: u32 = 0x4028A7; // memory::allocate
pub const MAYBE_LEXICON: u32 = 0x403555; // strings::maybe_compile_lexicon
pub const SET_MENU_MODE: u32 = 0x404823; // gfx::set_menu_screen_mode
pub const SET_RACE_MODE: u32 = 0x404830; // race::set_race_screen_mode
pub const SHOW_FRAME: u32 = 0x4057D1; // gfx::show_frame
pub const LOCK: u32 = 0x4058AD; // gfx::lock_surface
pub const UNLOCK: u32 = 0x405949; // gfx::unlock_surface
pub const CLEAR_FRONT: u32 = 0x4059F4; // gfx::clear_front_surface
pub const RESTORE_WINDOW: u32 = 0x405A33; // system::restore_window
pub const RUN_FRAME: u32 = 0x405A71; // screens::run_frame
pub const PICTURE_AS_SURFACE: u32 = 0x405FE7; // gfx::picture_as_surface
pub const COPY_PICTURE: u32 = 0x4060E3; // gfx::copy_picture_to_surface
pub const CONVERT_PICTURE: u32 = 0x406126; // gfx::convert_picture
pub const DRAW_HLINE: u32 = 0x406710; // gfx::draw_hline
pub const BLIT: u32 = 0x4069B2; // gfx::blit_glyph
pub const SCREENSHOT: u32 = 0x407897; // gfx::take_screenshot
pub const KEY_TO_CHAR: u32 = 0x407E43; // input::key_to_char
pub const TAKE_KEY: u32 = 0x407F95; // input::take_key
pub const APPLY_SOUND: u32 = 0x40EFF3; // sound::apply_settings
pub const RACE_SOUNDS: u32 = 0x40F02E; // sound::set_up_race_sounds
pub const TUNE_3: u32 = 0x40F780; // sound::play_tune_3
pub const TUNE_4: u32 = 0x40F792; // sound::play_tune_4
pub const STOP_MUSIC: u32 = 0x40F826; // sound::stop_music
pub const PUMP: u32 = 0x410535; // system::pump_messages
pub const SLEEP: u32 = 0x41100C; // system::sleep
pub const TICK: u32 = 0x411024; // system::tick_count
pub const PARSE_NUMBER: u32 = 0x411623; // settings::parse_number
pub const LOAD_PICTURE: u32 = 0x411863; // gfx::load_picture
pub const LIST_TAKE_KEY: u32 = 0x41469A; // menu::list_take_key
pub const CLEAR_RACE_SOUNDS: u32 = 0x415C6D; // race::clear_race_sound_objects
pub const SET_UP_RACE_SOUNDS: u32 = 0x415C93; // race::set_up_race_sound_objects
pub const COPY_RECEIVED: u32 = 0x4173A0; // net::copy_received
pub const STOP: u32 = 0x41838C; // net::stop
pub const LOCK_RECEIVED: u32 = 0x418474; // net::lock_received
pub const UNLOCK_RECEIVED: u32 = 0x418481; // net::unlock_received
pub const POST_CALL: u32 = 0x4184F8; // net::post_call
pub const SEND: u32 = 0x4196FF; // net::dplay_send (DirectPlay's Send)
pub const SET_UP_ENGINE: u32 = 0x442AF6; // scene::set_up_engine
pub const RELEASE_TEXTURES: u32 = 0x443396; // race::release_textures
pub const RENDER3D_TIDY: u32 = 0x4433A7; // race::render3d_tidy
pub const OBJECTS_TIDY: u32 = 0x44346B; // race::objects_tidy
pub const NULLSUB_6: u32 = 0x44347E; // nullsub_6
pub const LOAD_STAGE: u32 = 0x444040; // race::load_stage
pub const TIDY_EVERYTHING: u32 = 0x444AC7; // race::tidy_everything
pub const PLAY_RACE: u32 = 0x444CEE; // drive::play_race
pub const ARCADE_CARS: u32 = 0x4468FB; // race::set_up_arcade_cars
pub const START_STATES: u32 = 0x446D2E; // race::set_start_states
pub const FRONT_END_SCENE: u32 = 0x474E04; // scene::set_up_front_end_scene
pub const SELECT_CAR: u32 = 0x47AC0C; // menu::select_car_loop
pub const TAKE_SHOWN_CAR: u32 = 0x47B29D; // menu::take_shown_car
pub const LOADING_PICTURE: u32 = 0x47CEA8; // menu::show_loading_picture
pub const SELECT_STAGE: u32 = 0x482099; // menu::select_stage_loop
pub const START_FRONT_END: u32 = 0x4154CD; // sub_4154CD (agent F's)
/// `CoCreateInstance` (its import thunk): eax the class, ebx the interface, ecx where.
pub const CO_CREATE: u32 = 0x4D7CCE;

// ---- small pieces ----

/// The flags of `test eax, eax`.
pub fn test_eax(r: &mut Regs) {
    r.set_carry(false);
    r.set_zero(r.eax == 0);
    set_sign(r, r.eax as i32);
}

/// The flags of `cmp a, b`: carry, zero, sign.
pub fn cmp_flags(r: &mut Regs, a: u32, b: u32) {
    r.set_carry(a < b);
    r.set_zero(a == b);
    set_sign(r, a.wrapping_sub(b) as i32);
}

pub fn set_sign(r: &mut Regs, v: i32) {
    if v < 0 {
        r.eflags |= SIGN_FLAG;
    } else {
        r.eflags &= !SIGN_FLAG;
    }
}

/// The address of a player's block of `g_netReceived`.
pub fn block(player: u32) -> u32 {
    a(RECEIVED).wrapping_add(player.wrapping_mul(BLOCK))
}

/// `rep movsb` of `n` bytes, forwards (addresses).
pub fn movsb(to: u32, from: u32, n: u32) {
    for k in 0..n {
        unsafe { ((to + k) as *mut u8).write(((from + k) as *const u8).read()) };
    }
}

/// A logged text: `pusha; lea eax, text; call place; popa`.
pub fn log_text(place: u32, text: u32, r: &Regs, part: Part) -> Result<(), Refused> {
    let mut l = Regs { eax: text, ..*r };
    part(place, &mut l)
}

/// A fatal error with no message.
pub const fn fatal(code: u32) -> crate::errors::FatalError {
    crate::errors::FatalError { code, message: "" }
}

pub use rc2k_core::net::lists::{NO_CONNECTION, NO_CONNECTIONS, NO_SESSION, NO_SESSIONS};
pub const CONNECTION_LEN: u32 = std::mem::size_of::<crate::net::types::NetConnection>() as u32;
pub const SESSION_LEN: u32 = std::mem::size_of::<crate::net::types::NetSession>() as u32;

/// Record `index` (unsigned) of a table of `len`-byte records at `table` holding `count`:
/// `jnb` the first fatal error, a null table the second.
pub fn record_of(
    index: u32,
    count: usize,
    table: usize,
    len: u32,
    codes: [u32; 2],
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::lists::record`.
    rc2k_core::net::lists::record(index, g(count), || g(table), len, codes)
        .map_err(|missing| fatal(missing.0).into())
}

/// A connection's record (`net::connection_entry`).
pub fn connection(index: u32) -> Result<u32, Refused> {
    record_of(
        index,
        CONNECTIONS,
        CONNECTION_TABLE,
        CONNECTION_LEN,
        [NO_CONNECTION, NO_CONNECTIONS],
    )
}

/// A session's record (`net::session_entry`).
pub fn session(index: u32) -> Result<u32, Refused> {
    record_of(
        index,
        SESSIONS,
        SESSION_TABLE,
        SESSION_LEN,
        [NO_SESSION, NO_SESSIONS],
    )
}

// ---- the calls in the game ----

/// The calls by place in the game: the Rust versions; the frames' platform through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::net::lobby_calls::call(place, r)
}

/// `dispatch_lobby` without its hook (typed tables).
pub fn lobby_entry(
    r: &mut Regs,
    rust: fn(&Regs, Part) -> Result<Regs, Refused>,
) -> Result<(), Refused> {
    let left = rust(r, &mut game_part)?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// The address of a function or label of the game.
pub fn game(x: u32) -> u32 {
    game_addr(rva(x as usize)) as u32
}

/// A call into the game through its address (the hook's Rust version where there is one).
pub fn in_game(place: u32, r: &mut Regs) {
    crate::calls::call(rva(place as usize), r)
}

pub fn not_known(_: u32) -> Refused {
    BadInput("a lobby call with no version").into()
}

// ---- DirectPlay's callbacks (`stdcall`, their arguments on the stack) ----

/// Calls a `stdcall` function of up to six dword arguments at `code`: its eax.
pub unsafe fn call_stdcall(code: usize, args: &[u32]) -> u32 {
    type F4 = unsafe extern "system" fn(u32, u32, u32, u32) -> u32;
    type F5 = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> u32;
    type F6 = unsafe extern "system" fn(u32, u32, u32, u32, u32, u32) -> u32;
    unsafe {
        match *args {
            [a, b, c, d] => std::mem::transmute::<usize, F4>(code)(a, b, c, d),
            [a, b, c, d, e] => std::mem::transmute::<usize, F5>(code)(a, b, c, d, e),
            [a, b, c, d, e, f] => std::mem::transmute::<usize, F6>(code)(a, b, c, d, e, f),
            _ => unreachable!("the callbacks take four to six arguments"),
        }
    }
}

/// A callback's Rust version: its arguments in, its eax out.
pub type Callback = fn(&[u32], Part) -> Result<u32, Refused>;

thread_local!(static CALLBACK_ARGS: std::cell::RefCell<(usize, Vec<u32>)> =
    const { std::cell::RefCell::new((0, Vec::new())) });

/// A callback's Rust version where DirectPlay calls it (no hook; 2026-10-23): shadow mode
/// runs the game's callback at `ral` (an IDA address); a refused input runs it too (logged
/// once, as `menu::table_calls` does), a fatal error is raised.
pub fn callback_entry(name: &str, ral: u32, args: &[u32], rust: Callback) -> u32 {
    let result = rust(args, &mut game_part);
    match result {
        Ok(eax) => eax,
        Err(Refused::BadInput(why)) => {
            static SEEN: std::sync::Mutex<Vec<(u32, &'static str)>> =
                std::sync::Mutex::new(Vec::new());
            let mut seen = SEEN.lock().unwrap();
            if !seen.contains(&(ral, why.0)) {
                seen.push((ral, why.0));
                crate::framework::log(&format!(
                    "{name}: refused (first time, the game's runs): {}",
                    why.0
                ));
            }
            drop(seen);
            unsafe { call_stdcall(game(ral) as usize, args) }
        }
        Err(Refused::Fatal(error)) => {
            crate::framework::log(&format!(
                "{name}: the Rust version raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            let r = Regs::default();
            unsafe { crate::fatal::raise::raise(error, &r) };
            r.eax
        }
    }
}
