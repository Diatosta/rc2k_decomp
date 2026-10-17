//! The race's shell (`sub_4843EC`, `single_rally`'s race, and `sub_443F0B`, IDA
//! 2026-10-17): what runs round a stage's loading and its run. Here its small leaves.

use crate::framework::{Regs, call_game, game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at};

pub const CONTROLS_RVA: usize = rva(0x4F7DE4); // dword_4F7DE4 (`create_controls` 3)
/// Each car block's +0xB0 (dword_70F49C ..): the player absent.
pub const ABSENCES: [usize; 8] = [
    0x70F49C, 0x71047C, 0x71145C, 0x71243C, 0x71341C, 0x7143FC, 0x7153DC, 0x7163BC,
];
pub const KIND_ORDER_RVA: usize = rva(0x6051E0); // dword_6051E0: 0x22 kinds
pub const KIND_ORDER_LEN: u32 = 0x22;
/// The kinds `g_carKindTable`-like tables hold (0x17).
pub const KINDS: u32 = 0x17;
pub const SCREEN_MODE_RVA: usize = rva(0x4F4948); // g_raceScreenMode
pub const SET_SCREEN_MODE_RVA: usize = rva(0x404B63); // gfx::set_screen_mode (platform)

/// `race::release_controls` (0x409425): `dword_4F7DE4` 2. Every register kept.
pub unsafe fn release_controls() {
    unsafe { game_write(CONTROLS_RVA, 2u32) };
}

/// `race::clear_player_absences` (0x449D44): the eight blocks' +0xB0 0. Every register
/// kept.
pub unsafe fn clear_player_absences() {
    for a in ABSENCES {
        unsafe { game_write(rva(a), 0u32) };
    }
}

pub fn absence_regions() -> Vec<Region> {
    ABSENCES.iter().map(|&a| at(rva(a))).collect()
}

/// `race::kind_past_table` (0x4651AC): carry for a kind (eax) of 0x17 or more; zero as
/// `cmp eax, 17h` leaves it. (carry, zero).
pub fn kind_past_table(eax: u32) -> (bool, bool) {
    (eax >= KINDS, eax == KINDS)
}

/// `race::kind_unusable` (0x449D34): carry for a kind past the table or 6; zero as the
/// last `cmp` leaves it (against 0x17 when past, else against 6). (carry, zero).
pub fn kind_unusable(eax: u32) -> (bool, bool) {
    let (past, zero) = kind_past_table(eax);
    if past {
        return (true, zero);
    }
    (eax == 6, eax == 6)
}

/// `race::index_of_kind` (0x437720): the place of kind eax in `dword_6051E0` (the first
/// of equals), 0 for none. Leaves ecx the place, 0x22 for none.
pub unsafe fn index_of_kind(entry: &Regs) -> Regs {
    let mut g = *entry;
    for n in 0..KIND_ORDER_LEN {
        if unsafe { game_read::<u32>(KIND_ORDER_RVA + 4 * n as usize) } == entry.eax {
            g.ecx = n;
            g.eax = n;
            return g;
        }
    }
    g.ecx = KIND_ORDER_LEN;
    g.eax = 0;
    g
}

/// `race::set_race_screen_mode` (0x404830): `gfx::set_screen_mode` (the platform's, in the
/// game) with eax `g_raceScreenMode`; every register kept (`pusha` / `popa`).
pub unsafe fn set_race_screen_mode(entry: &Regs, part: &mut dyn FnMut(&mut Regs)) {
    let mut r = Regs {
        eax: unsafe { game_read(SCREEN_MODE_RVA) },
        ..*entry
    };
    part(&mut r);
}

/// `set_race_screen_mode`'s call in the game.
pub fn game_screen_mode(r: &mut Regs) {
    unsafe { call_game(r, game_addr(SET_SCREEN_MODE_RVA)) };
}

// ---- batch 12 ----

use crate::errors::{BadInput, Refused};

pub const RELEASE_TEXTURES_RVA: usize = rva(0x4051D3); // gfx::release_all_textures (platform)

/// `race::release_stage_textures` (0x44E79F): `gfx::release_all_textures` (the platform's,
/// in the game; a `call` and a `retn`). Leaves the registers as it does.
pub unsafe fn release_stage_textures(entry: &Regs, part: &mut dyn FnMut(&mut Regs)) -> Regs {
    let mut g = *entry;
    part(&mut g);
    g
}

/// `release_stage_textures`' call in the game.
pub fn game_release_textures(r: &mut Regs) {
    unsafe { call_game(r, game_addr(RELEASE_TEXTURES_RVA)) };
}

const ABSENT_0_RVA: usize = rva(0x70F49C); // the first block's +0xB0
const TIME_0_RVA: usize = rva(0x70F474); // the first block's +0x88
const SEAL_KEY_RVA: usize = rva(0x4F66F9); // dword_4F66F9 (unaligned)
pub const SEAL_RVA: usize = rva(0x5BECCC); // the dword after "RCIB"

/// `race::seal_stage_time` (0x41F29D): the first block's time (+0x88, held to
/// 0x22550FF unsigned; its +0xB0 if that is negative) `xor` `dword_4F66F9` and turned
/// right by its low five bits, into the dword at 0x5BECCC. Every register kept.
pub unsafe fn seal_stage_time() {
    unsafe {
        let mut v: u32 = game_read(ABSENT_0_RVA);
        if (v as i32) >= 0 {
            v = game_read(TIME_0_RVA);
            if v > 0x22550FF {
                v = 0x22550FF;
            }
        }
        let key = (game_addr(SEAL_KEY_RVA) as *const u32).read_unaligned();
        (game_addr(SEAL_RVA) as *mut u32).write_unaligned((v ^ key).rotate_right(key & 0x1F));
    }
}

/// `race::permute_bits` (0x402848): the bits of edx:eax picked by the table at ebx (a
/// count byte n, then n + 1 bytes: bit 5 set picks bit (b & 0x1F) of eax, else of edx),
/// gathered into esi:edi from the top down (`adc edi, edi` / `adc esi, esi`). Leaves esi
/// and edi the result, ebx past the table; the rest kept.
pub unsafe fn permute_bits(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let mut at = g.ebx;
        let count = *(at as *const u8) as u32;
        at += 1;
        let mut bits = 0u64;
        for _ in 0..=count {
            let b = *(at as *const u8) as u32;
            at += 1;
            let from = if b & 0x20 != 0 { g.eax } else { g.edx };
            bits = (bits << 1) | ((from >> (b & 0x1F)) & 1) as u64;
        }
        g.edi = bits as u32;
        g.esi = (bits >> 32) as u32;
        g.ebx = at;
    }
    g
}

const LEAGUE_SOURCE_RVA: usize = rva(0x75ACD0); // dword_75ACD0: 10 records of 0x50, or 0
const LEAGUE_A_RVA: usize = rva(0x60410C); // dword_60410C: 10 dwords
const LEAGUE_B_RVA: usize = rva(0x604134); // unk_604134: 16 rows of 10 dwords (0x28)

/// `race::copy_league_table` (0x4650B7): with a source (`dword_75ACD0`), its ten records'
/// +8 into `dword_60410C` and their sixteen dwords from +0x10 into column n of the
/// sixteen rows at 0x604134. Leaves esi past the records, ecx 10, edx 16, edi past the
/// last column, eax the last dword (without a source: esi 0).
pub unsafe fn copy_league_table(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.esi = game_read(LEAGUE_SOURCE_RVA);
        if g.esi == 0 {
            return g;
        }
        for n in 0..10u32 {
            let rec = g.esi;
            g.eax = *((rec + 8) as *const u32);
            game_write(LEAGUE_A_RVA + 4 * n as usize, g.eax);
            g.edi = game_addr(LEAGUE_B_RVA) as u32 + 4 * n;
            for k in 0..16u32 {
                g.eax = ((rec + 0x10 + 4 * k) as *const u32).read_unaligned();
                (g.edi as *mut u32).write_unaligned(g.eax);
                g.edi += 0x28;
            }
            g.edx = 16;
            g.esi += 0x50;
        }
        g.ecx = 10;
    }
    g
}

pub fn league_regions() -> Vec<Region> {
    vec![
        (game_addr(LEAGUE_A_RVA) as u32, 0x28),
        (game_addr(LEAGUE_B_RVA) as u32, 0x280),
    ]
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
unsafe fn start_value(kind: u32, g: &mut Regs) -> u32 {
    let k = kind.min(0x21);
    g.ebx = k * 0xB0;
    unsafe {
        if game_read::<u32>(CAR_RECORDS_LIMIT_RVA + g.ebx as usize) > 0x1B58 {
            0
        } else {
            game_read(CAR_RECORDS_VALUE_RVA + g.ebx as usize)
        }
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
        if matches!(p.read_unaligned(), 2 | 3) {
            p.write_unaligned(4);
        }
    }
}

/// `race::set_start_states` (0x446D2E): `dword_6D14CA` / `CE` the start values of the
/// first car's kind (the chosen car's, `dword_52A920`, with `dword_52A910` 1) and of
/// `dword_62385C`; then with `dword_52A910` 1: `dword_6238A4` 0 and, unless the chosen
/// car's kind is flagged, its block's +0x684 2 or 3 made 4; else `dword_6238A4` 0 (one
/// player) or held to 1 .. 2 (two), and, when the first car's kind is not flagged or
/// there are two players, the first two blocks' +0x684 2 or 3 made 4. Leaves eax the
/// second value, ebx the last record offset (with `dword_52A910` 1: the chosen block).
/// Refuses, with `dword_52A910` 1, a chosen car past 7 (the original reads past the
/// tables), before anything.
pub unsafe fn set_start_states(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
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
        let v = start_value(first_kind, &mut g);
        (game_addr(START_VALUES_RVA[0]) as *mut u32).write_unaligned(v);
        let v = start_value(game_read(SECOND_KIND_RVA), &mut g);
        (game_addr(START_VALUES_RVA[1]) as *mut u32).write_unaligned(v);
        g.eax = v;
        if one {
            g.ebx = game_read(crate::race::objects::CAR_BLOCKS_RVA + 4 * chosen as usize);
            game_write(GEAR_CHOICE_RVA, 0u32);
            if !flagged(*((g.ebx + 0x24) as *const u32)) {
                settle_state(g.ebx + 0x684);
            }
            return Ok(g);
        }
        let two = game_read::<u32>(PLAYERS_RVA) == 2;
        let c: i32 = game_read(GEAR_CHOICE_RVA);
        // The original's two sign tests (`cmp`, `jns` / `js`), wrapping as they do.
        let held = if !two {
            0
        } else {
            let mut h = c;
            if h.wrapping_sub(1) < 0 {
                h = 1;
            }
            if h.wrapping_sub(2) >= 0 {
                h = 2;
            }
            h
        };
        game_write(GEAR_CHOICE_RVA, held);
        let kind_0: u32 = game_read(rva(0x70F410));
        if !flagged(kind_0) || game_read::<u32>(PLAYERS_RVA) >= 2 {
            settle_state(game_addr(STATE_0_RVA) as u32);
            settle_state(game_addr(STATE_1_RVA) as u32);
        }
    }
    Ok(g)
}

pub fn start_state_regions() -> Vec<Region> {
    let mut r: Vec<Region> = START_VALUES_RVA
        .iter()
        .map(|&a| (game_addr(a) as u32, 4))
        .collect();
    r.push(at(GEAR_CHOICE_RVA));
    for n in 0..8u32 {
        let b: u32 = unsafe { game_read(crate::race::objects::CAR_BLOCKS_RVA + 4 * n as usize) };
        r.push((b + 0x684, 4));
    }
    r
}

/// `race::store_results_by_mode` (0x449957): by `dword_62389C`: 2 the first block's +0x8C
/// to `dword_75AFFC`, its +0x654 to `dword_75B000` and `dword_6051BC`, its +0x88 to
/// `dword_75B01C`; 0 the first two blocks' +0x88 to `dword_75B01C` / `dword_75B08C` and
/// `g_stageCarTotal` x 100 less each, held to 0 .. 300000, to `dword_75B020` /
/// `dword_75B090` (ebx the stage held to 0 .. 0x4C); 1 and 3 the two +0x88s; others
/// nothing. Leaves eax the last value.
pub unsafe fn store_results_by_mode(entry: &Regs) -> Regs {
    let mut g = *entry;
    let put = |a: usize, v: u32| unsafe { game_write(rva(a), v) };
    let get = |a: usize| -> u32 { unsafe { game_read(rva(a)) } };
    match get(0x62389C) {
        2 => {
            g.eax = get(0x70F478);
            put(0x75AFFC, g.eax);
            g.eax = get(0x70FA40);
            put(0x75B000, g.eax);
            put(0x6051BC, g.eax);
            g.eax = get(0x70F474);
            put(0x75B01C, g.eax);
        }
        0 => {
            g.eax = get(0x70F474);
            put(0x75B01C, g.eax);
            g.eax = get(0x710454);
            put(0x75B08C, g.eax);
            let mut stage = get(0x623850) as i32;
            if stage < 0 {
                stage = 0;
            }
            if stage.wrapping_sub(0x4C) >= 0 {
                stage = 0x4C;
            }
            g.ebx = stage as u32;
            let total = get(0x601C4C).wrapping_mul(100);
            let held = |t: u32| -> u32 {
                let mut v = t as i32;
                if v < 0 {
                    v = 0;
                }
                if v.wrapping_sub(300_000) >= 0 {
                    v = 300_000;
                }
                v as u32
            };
            g.eax = held(total.wrapping_sub(get(0x70F474)));
            put(0x75B020, g.eax);
            g.eax = held(total.wrapping_sub(get(0x710454)));
            put(0x75B090, g.eax);
        }
        1 | 3 => {
            g.eax = get(0x70F474);
            put(0x75B01C, g.eax);
            g.eax = get(0x710454);
            put(0x75B08C, g.eax);
        }
        _ => {}
    }
    g
}

pub fn result_regions() -> Vec<Region> {
    [
        0x75AFFC, 0x75B000, 0x6051BC, 0x75B01C, 0x75B08C, 0x75B020, 0x75B090,
    ]
    .iter()
    .map(|&a| at(rva(a)))
    .collect()
}
