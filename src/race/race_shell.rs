//! The race's shell (`sub_4843EC`, `single_rally`'s race, and `sub_443F0B`, IDA
//! 2026-10-17): what runs round a stage's loading and its run. Here its small leaves.

use crate::framework::rva;
use crate::input::store::DEVICES;
use crate::store::{game_addr, game_read, game_write};

/// Each car block's +0xB0 (dword_70F49C ..): the player absent.
pub const ABSENCES: [usize; 8] = [
    0x70F49C, 0x71047C, 0x71145C, 0x71243C, 0x71341C, 0x7143FC, 0x7153DC, 0x7163BC,
];
pub const KIND_ORDER_RVA: usize = rva(0x6051E0); // dword_6051E0: 0x22 kinds
pub const KIND_ORDER_LEN: u32 = rc2k_core::race::cars::KIND_ORDER_LEN;
pub const SCREEN_MODE_RVA: usize = rva(0x4F4948); // g_raceScreenMode

/// `race::release_controls` (0x409425): `dword_4F7DE4` 2. Every register kept.
pub unsafe fn release_controls() {
    DEVICES.input_command.set(2); // g_inputCommand
}

/// `race::clear_player_absences` (0x449D44): the eight blocks' +0xB0 0. Every register
/// kept.
pub unsafe fn clear_player_absences() {
    for a in ABSENCES {
        unsafe { game_write(rva(a), 0u32) };
    }
}

/// `race::kind_unusable` (0x449D34): carry for a kind past the table or 6; zero as the
/// last `cmp` leaves it (against 0x17 when past, else against 6). (carry, zero).
pub fn kind_unusable(eax: u32) -> (bool, bool) {
    // Converted: `rc2k_core::race::cars::kind_unusable`.
    let f = rc2k_core::race::cars::kind_unusable(eax);
    (f.carry, f.zero)
}

/// `race::index_of_kind` (0x437720): the place of kind `kind` in `dword_6051E0` (the
/// first of equals), 0 for none.
pub unsafe fn index_of_kind(kind: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::index_of_kind`; the table read entry by entry.
    let order =
        (0..KIND_ORDER_LEN).map(|n| unsafe { game_read::<u32>(KIND_ORDER_RVA + 4 * n as usize) });
    rc2k_core::race::cars::index_of_kind(kind, order).unwrap_or(0)
}

/// `race::set_race_screen_mode` (0x404830): `gfx::set_screen_mode` (the platform's) with
/// `g_raceScreenMode`; every register and flag kept (`pusha` / `popa`; its carry is read
/// by nobody).
pub unsafe fn set_race_screen_mode() {
    crate::platform::set_mode_now(unsafe { game_read(SCREEN_MODE_RVA) });
}

// ---- batch 12 ----

use crate::errors::{BadInput, Refused};

/// `race::release_stage_textures` (0x44E79F): `gfx::release_all_textures` (the
/// platform's; a `call` and a `retn`).
pub fn release_stage_textures() {
    crate::platform_3d::release_textures();
}

const ABSENT_0_RVA: usize = rva(0x70F49C); // the first block's +0xB0
const TIME_0_RVA: usize = rva(0x70F474); // the first block's +0x88
pub const SEAL_RVA: usize = rva(0x5BECCC); // the dword after "RCIB"

/// `race::seal_stage_time` (0x41F29D): the first block's time (+0x88, held to
/// 0x22550FF unsigned; its +0xB0 if that is negative) `xor` `dword_4F66F9` and turned
/// right by its low five bits, into the dword at 0x5BECCC. Every register kept.
pub unsafe fn seal_stage_time() {
    // Converted: `rc2k_core::race::seal::seal_time`.
    unsafe {
        let key = crate::sound::store::WHEELS.seal_key.get();
        let sealed = rc2k_core::race::seal::seal_time(
            game_read(ABSENT_0_RVA),
            || game_read(TIME_0_RVA),
            key,
        );
        (game_addr(SEAL_RVA) as *mut u32).write_unaligned(sealed);
    }
}

const LEAGUE_SOURCE_RVA: usize = rva(0x75ACD0); // dword_75ACD0: 10 records of 0x50, or 0
const LEAGUE_A_RVA: usize = rva(0x60410C); // dword_60410C: 10 dwords
const LEAGUE_B_RVA: usize = rva(0x604134); // unk_604134: 16 rows of 10 dwords (0x28)

/// `race::copy_league_table` (0x4650B7): with a source (`dword_75ACD0`), its ten records'
/// +8 into `dword_60410C` and their sixteen dwords from +0x10 into column n of the
/// sixteen rows at 0x604134. The registers it leaves are read by no caller (taint at its
/// one call site, 0x4486DB; conversion phase 3, 2026-10-24).
pub unsafe fn copy_league_table() {
    // Converted: `rc2k_core::race::results::league`.
    use rc2k_core::race::results::league;
    unsafe {
        let source: u32 = game_read(LEAGUE_SOURCE_RVA);
        if source == 0 {
            return;
        }
        let l = league(|n, off| ((source + 0x50 * n as u32 + off) as *const u32).read_unaligned());
        game_write(LEAGUE_A_RVA, l.firsts);
        game_write(LEAGUE_B_RVA, l.rows);
    }
}

const ONE_PLAYER_CHOICE_RVA: usize = rva(0x52A910); // dword_52A910
const ONE_PLAYER_CAR_RVA: usize = rva(0x52A920); // dword_52A920
const CAR_KINDS_RVA: usize = rva(0x623858); // g_raceCarKinds
const SECOND_KIND_RVA: usize = rva(0x62385C); // dword_62385C
const CAR_RECORDS_LIMIT_RVA: usize = rva(0x5C7980); // dword_5C7980: per kind (0xB0 apart)
const CAR_RECORDS_VALUE_RVA: usize = rva(0x5C7924); // a00RenaultMaxiM+0x18
pub const START_VALUES_RVA: [usize; 2] = [rva(0x6D14CA), rva(0x6D14CE)]; // unaligned
pub const GEAR_CHOICE_RVA: usize = rva(0x6238A4); // dword_6238A4
const KIND_FLAGS_RVA: usize = rva(0x5C5AC0); // dword_5C5AC0: 0x22
const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94
const STATE_0_RVA: usize = rva(0x70FA70); // the first block's +0x684
const STATE_1_RVA: usize = rva(0x710A50); // the second's

/// A kind's value for `set_start_states`: the record of the kind (held to 0x21) its
/// +0x18 when its limit (`dword_5C7980`) is 0x1B58 or less, else 0.
unsafe fn start_value(kind: u32) -> u32 {
    use rc2k_core::race::cars::{start_record, start_state_value};
    let record = start_record(kind) as usize * 0xB0;
    unsafe {
        start_state_value(game_read(CAR_RECORDS_LIMIT_RVA + record), || {
            game_read(CAR_RECORDS_VALUE_RVA + record)
        })
    }
}

/// Whether a kind is one with its flag (`dword_5C5AC0`) set (below 0x22).
unsafe fn flagged(kind: u32) -> bool {
    kind < 0x22 && unsafe { game_read::<u32>(KIND_FLAGS_RVA + 4 * kind as usize) } != 0
}

/// A block's +0x684 of 2 or 3 made 4.
unsafe fn settle_state(at: u32) {
    unsafe {
        let p = at as *mut u32;
        if let Some(v) = rc2k_core::race::cars::settled_state(p.read_unaligned()) {
            p.write_unaligned(v);
        }
    }
}

/// `race::set_start_states` (0x446D2E): `dword_6D14CA` / `CE` the start values of the
/// first car's kind (the chosen car's, `dword_52A920`, with `dword_52A910` 1) and of
/// `dword_62385C`; then with `dword_52A910` 1: `dword_6238A4` 0 and, unless the chosen
/// car's kind is flagged, its block's +0x684 2 or 3 made 4; else `dword_6238A4` 0 (one
/// player) or held to 1 .. 2 (two), and, when the first car's kind is not flagged or
/// there are two players, the first two blocks' +0x684 2 or 3 made 4. The eax and ebx it
/// leaves are read by none of its four callers (taint at each site; it keeps the rest).
/// Refuses, with `dword_52A910` 1, a chosen car past 7 (the original reads past the
/// tables), before anything.
pub unsafe fn set_start_states() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::cars::{start_record, start_state_value, held_split,
    // settled_state}`.
    unsafe {
        let one = game_read::<u32>(ONE_PLAYER_CHOICE_RVA) == 1;
        let chosen: u32 = game_read(ONE_PLAYER_CAR_RVA);
        if one && chosen > 7 {
            return Err(BadInput("a chosen car past the eight").into());
        }
        let first_kind: u32 = if one {
            game_read(CAR_KINDS_RVA + 4 * chosen as usize)
        } else {
            game_read(CAR_KINDS_RVA)
        };
        let v = start_value(first_kind);
        (game_addr(START_VALUES_RVA[0]) as *mut u32).write_unaligned(v);
        let v = start_value(game_read(SECOND_KIND_RVA));
        (game_addr(START_VALUES_RVA[1]) as *mut u32).write_unaligned(v);
        if one {
            let block: u32 = game_read(crate::race::objects::CAR_BLOCKS_RVA + 4 * chosen as usize);
            game_write(GEAR_CHOICE_RVA, 0u32);
            if !flagged(*((block + 0x24) as *const u32)) {
                settle_state(block + 0x684);
            }
            return Ok(());
        }
        let two = game_read::<u32>(PLAYERS_RVA) == 2;
        let held = rc2k_core::race::cars::held_split(two, game_read(GEAR_CHOICE_RVA));
        game_write(GEAR_CHOICE_RVA, held);
        let kind_0: u32 = game_read(rva(0x70F410));
        if !flagged(kind_0) || game_read::<u32>(PLAYERS_RVA) >= 2 {
            settle_state(game_addr(STATE_0_RVA) as u32);
            settle_state(game_addr(STATE_1_RVA) as u32);
        }
    }
    Ok(())
}

/// `race::store_results_by_mode` (0x449957): by `dword_62389C`: 2 the first block's +0x8C
/// to `dword_75AFFC`, its +0x654 to `dword_75B000` and `dword_6051BC`, its +0x88 to
/// `dword_75B01C`; 0 the first two blocks' +0x88 to `dword_75B01C` / `dword_75B08C` and
/// `g_stageCarTotal` x 100 less each, held to 0 .. 300000, to `dword_75B020` /
/// `dword_75B090` (ebx the stage held to 0 .. 0x4C); 1 and 3 the two +0x88s; others
/// nothing. The registers it leaves are read by no caller (taint at 0x448C02).
pub unsafe fn store_results_by_mode() {
    // Converted: `rc2k_core::race::cars::margin`.
    use rc2k_core::race::cars::margin;
    let put = |a: usize, v: u32| unsafe { game_write(rva(a), v) };
    let get = |a: usize| -> u32 { unsafe { game_read(rva(a)) } };
    match get(0x62389C) {
        2 => {
            put(0x75AFFC, get(0x70F478));
            let v = get(0x70FA40);
            put(0x75B000, v);
            put(0x6051BC, v);
            put(0x75B01C, get(0x70F474));
        }
        0 => {
            put(0x75B01C, get(0x70F474));
            put(0x75B08C, get(0x710454));
            let total = get(0x601C4C);
            put(0x75B020, margin(total, get(0x70F474)));
            put(0x75B090, margin(total, get(0x710454)));
        }
        1 | 3 => {
            put(0x75B01C, get(0x70F474));
            put(0x75B08C, get(0x710454));
        }
        _ => {}
    }
}
