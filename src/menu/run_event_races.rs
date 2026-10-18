//! `menu::run_event_races` (0x473126): an event's races. `dword_75592C` 1 (the event goes
//! on), `dword_75ACF0` the cheat bit 0x12, `dword_7554C8` 0; the computer drivers' times
//! (`menu::calc_cpu_times`); eight drivers picked by lot (of the first 0xB2) get 0x4844
//! plus a random number under 3000 added to their +0xC and +0x10; one to four drivers by
//! lot still running (+0x24 1) drop out (+0x24 4, +0xC and +0x10 0x36EE8A). With
//! `dword_755250` 1 the players' times are reset (`menu::reset_player_times`); otherwise
//! standings table A from the drivers (`menu::standings_from_drivers`), sorted by time,
//! the summary copied (`menu::copy_standings_summary`), `dword_75ACD0` the summary table
//! and `dword_75ACD4` the record's track pack entry's +0x34; then by the event mode
//! (`dword_755988`):
//! - 9, 0xE: one race (`race::single_rally_race`, eax 0, ebx 1); a result
//!   (`dword_75B028`) of -1 ends the event (`dword_75592C` 0).
//! - 0xA, 0xF: the same with `dword_75ACE4` `dword_7554C8` (0xA: before and after), the
//!   result and `dword_75B098` both -1 ending it.
//! - 0xB, 0x10 (two players), 0xC, 0x11 (three), 0xD, 0x12 (four): each player
//!   (`dword_75598C`) taking part (+0xD0 1) races, eax 1 when the next one takes part
//!   (`menu::player_entered`), 0 for the last; a result of -2 resets the players' times,
//!   sets the front end up again (`menu::reload_front_end`) and stops; one not taking
//!   part counts -1; all -1 ends the event (0xD also gives the fourth player +0x88
//!   0x36EE80 and +0xD0 6).
//!
//! Then the players' +0x288 cleared (`menu::clear_players_288`).
//!
//! In:  nothing
//! Out: no `pusha`: eax, ecx, edx, ebx, esi and edi as the original leaves them (see
//!      `run_event_races`)
//!
//! Callers: the event drivers. The steps are their Rust versions; the races and the front
//! end through `part`, in the game their Rust versions. Ungated (the races). Refuses fewer
//! than 0xB2 drivers or fewer than four still running among them (the drop-out loop would
//! run on), a missing player for the mode, and what the steps refuse, before anything.
//! Shadow mode runs the original alone (it runs races, and logs). Self-test:
//! a copy whose races and front-end call go to stand-ins, against the Rust version,
//! by mode and result; put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::run_event_races", rva(0x473126), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const GOES_ON_RVA: usize = rva(0x75592C); // dword_75592C
const CHEAT_RVA: usize = rva(0x75ACF0); // dword_75ACF0
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const KEPT_RVA: usize = rva(0x7554C8); // dword_7554C8
const RESET_ONLY_RVA: usize = rva(0x755250); // dword_755250
const SUMMARY_RVA: usize = rva(0x7556F4); // dword_7556F4
const RACE_INFO_RVA: usize = rva(0x75ACD0); // dword_75ACD0, dword_75ACD4
const MODE_RVA: usize = rva(0x755988); // dword_755988
const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const RESULT_RVA: usize = rva(0x75B028); // dword_75B028
const RESULT_2_RVA: usize = rva(0x75B098); // dword_75B098
const RACE_KEPT_RVA: usize = rva(0x75ACE4); // dword_75ACE4
const PICKED: u32 = 0xB2;

/// The calls of the test's copy.
const CALLS: [usize; 82] = [
    0x32, 0x47, 0x59, 0x71, 0x85, 0xBC, 0xC6, 0xCB, 0xD0, 0xF0, 0xF5, 0x123, 0x16D, 0x1E1, 0x1EB,
    0x1F9, 0x1FE, 0x245, 0x253, 0x258, 0x2C2, 0x2CC, 0x2DA, 0x2DF, 0x326, 0x330, 0x33E, 0x343,
    0x38D, 0x39B, 0x3A0, 0x40A, 0x414, 0x422, 0x427, 0x46E, 0x478, 0x486, 0x48B, 0x4D5, 0x4DF,
    0x4ED, 0x4F2, 0x53C, 0x54A, 0x54F, 0x5B8, 0x602, 0x66A, 0x674, 0x682, 0x687, 0x6CE, 0x6DC,
    0x6E1, 0x74B, 0x755, 0x763, 0x768, 0x7AF, 0x7B9, 0x7C7, 0x7CC, 0x816, 0x824, 0x829, 0x893,
    0x89D, 0x8AB, 0x8B0, 0x8F7, 0x901, 0x90F, 0x914, 0x95E, 0x968, 0x976, 0x97B, 0x9C2, 0x9D0,
    0x9D5, 0x9F5,
];
const OUT: u32 = 0x36EE8A;
const QUIT: u32 = 0xFFFF_FFFE;
const NONE: u32 = 0xFFFF_FFFF;

/// A call by place: 1 `race::single_rally_race`, 2 `menu::reload_front_end`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The players racing in an event mode, and whether the fourth's dropping out is noted.
fn players_of(mode: u32) -> Option<(u32, bool)> {
    match mode {
        0xB | 0x10 => Some((2, false)),
        0xC | 0x11 => Some((3, false)),
        0xD => Some((4, true)),
        0x12 => Some((4, false)),
        _ => None,
    }
}

/// The races of the players' modes (see the header), from the registers `g`.
unsafe fn players_races(
    g: &mut Regs,
    n: u32,
    note_fourth: bool,
    part: Part,
) -> Result<(), Refused> {
    let mut kept = 0u32;
    for i in 0..n {
        unsafe {
            game_write(CURRENT_RVA, i);
            g.edi = player(i);
            if get(g.edi + 0xD0) != 1 {
                if i == 0 {
                    g.eax = NONE;
                    kept = NONE;
                } else {
                    game_write(RESULT_RVA, NONE);
                }
            } else {
                if i + 1 < n {
                    *g = crate::menu::player_entered::player_entered(&Regs {
                        eax: 1,
                        ecx: i + 1,
                        ..*g
                    })?;
                } else {
                    g.eax = 0;
                }
                g.ebx = 1;
                part(1, g)?;
                if game_read::<u32>(RESULT_RVA) == QUIT {
                    crate::menu::reset_player_times::reset_player_times(g)?;
                    part(2, g)?;
                    if i > 0 {
                        g.eax = kept;
                    }
                    return Ok(());
                }
                if i == 0 {
                    g.eax = game_read(RESULT_RVA);
                    kept = g.eax;
                }
            }
            if i > 0 {
                g.eax = kept & game_read::<u32>(RESULT_RVA);
                kept = g.eax;
            }
        }
    }
    if kept == NONE {
        unsafe {
            if note_fourth {
                g.edi = player(3);
                put(g.edi + 0x88, 0x36EE80);
                put(g.edi + 0xD0, 6);
            }
            game_write(GOES_ON_RVA, 0u32);
        }
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_event_races(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use crate::math::random_below::random_below;
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    if drivers.len() < PICKED as usize
        || drivers[..PICKED as usize]
            .iter()
            .filter(|&&d| get(d + 0x24) == 1)
            .count()
            < 4
    {
        return Err(BadInput("fewer than 0xB2 drivers, or fewer than four still running").into());
    }
    let mode: u32 = unsafe { game_read(MODE_RVA) };
    if let Some((n, _)) = players_of(mode)
        && (0..n).any(|i| player(i) == 0)
    {
        return Err(BadInput("a player of the event's mode missing").into());
    }
    let reset_only = unsafe { game_read::<u32>(RESET_ONLY_RVA) } == 1;
    let rec = crate::menu::calc_cpu_times::record()?;
    crate::menu::calc_cpu_times::check(rec)?;
    let pack = unsafe { crate::records::trakpak_entry::record_entry(rec as *const u8) }?;
    let first = drivers[0];
    let mut g = *entry;
    unsafe {
        game_write(GOES_ON_RVA, 1u32);
        let cheat = game_read::<u32>(CHEATS_RVA) & (1 << 0x12) != 0;
        game_write(CHEAT_RVA, u32::from(cheat));
        game_write(KEPT_RVA, 0u32);
        crate::menu::calc_cpu_times::calc_cpu_times(&g)?;
        for _ in 0..8 {
            g.edi = first + random_below(PICKED)? * DRIVER;
            g.eax = random_below(3000)?.wrapping_add(0x4844);
            put(g.edi + 0xC, get(g.edi + 0xC).wrapping_add(g.eax));
            put(g.edi + 0x10, get(g.edi + 0x10).wrapping_add(g.eax));
        }
        let out = random_below(4)? + 1;
        for _ in 0..out {
            loop {
                g.eax = random_below(PICKED)? * DRIVER;
                g.edi = first + g.eax;
                if get(g.edi + 0x24) == 1 {
                    break;
                }
            }
            put(g.edi + 0x24, 4);
            put(g.edi + 0xC, OUT);
            put(g.edi + 0x10, OUT);
        }
        g.ecx = 0;
        if reset_only {
            crate::menu::reset_player_times::reset_player_times(&g)?;
        } else {
            crate::menu::standings_from_drivers::standings_from_drivers(&g)?;
            crate::menu::sort_standings_by_time::sort_standings_by_time(&g)?;
            crate::menu::copy_standings_summary::copy_standings_summary(&g)?;
            g.eax = game_read(SUMMARY_RVA);
            game_write(RACE_INFO_RVA, g.eax);
            g.eax = game_read(INDEX_RVA);
            g.ebx = game_read(SECTION_RVA);
            g.edi = pack;
            g.esi = pack;
            g.ebx = get(rec + 4).wrapping_mul(0x40);
            g.ecx = get(rec + 8);
            g.edx = crate::records::trakpak_entry::loaded_pack_id();
            g.esi = game_addr(RACE_INFO_RVA) as u32;
            g.eax = get(pack + 0x34);
            game_write(RACE_INFO_RVA + 4, g.eax);
            match mode {
                9 | 0xA | 0xE | 0xF => {
                    game_write(CURRENT_RVA, 0u32);
                    g.eax = 0;
                    g.ebx = 1;
                    let kept = mode == 0xA || mode == 0xF;
                    if kept {
                        g.edx = game_read(KEPT_RVA);
                        game_write(RACE_KEPT_RVA, g.edx);
                    }
                    part(1, &mut g)?;
                    if mode == 0xA {
                        g.edx = game_read(KEPT_RVA);
                        game_write(RACE_KEPT_RVA, g.edx);
                    }
                    let ended = if kept {
                        g.eax = game_read::<u32>(RESULT_RVA) & game_read::<u32>(RESULT_2_RVA);
                        g.eax == NONE
                    } else {
                        game_read::<u32>(RESULT_RVA) == NONE
                    };
                    if ended {
                        game_write(GOES_ON_RVA, 0u32);
                    }
                }
                _ => {
                    if let Some((n, note_fourth)) = players_of(mode) {
                        players_races(&mut g, n, note_fourth, part)?;
                    }
                }
            }
        }
        crate::menu::clear_players_288::clear_players_288(&g)?;
    }
    Ok(g)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                *r = crate::race::single_rally::single_rally_race(
                    r,
                    &mut crate::race::single_rally::game_part,
                )?
            }
            _ => {
                use crate::menu::reload_front_end as f;
                *r = f::reload_front_end(r, &mut f::game_part)?
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_event_races(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    // Unpaused (it logs): the generator to this test alone meanwhile.
    crate::math::next_random::own_random(test)
}

fn test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Unpaused: frames drawn meanwhile may see the tables this test changes.
    let _disturbing = crate::screens::harness::Disturbing::begin();
    let copy = match copy(
        0x473126,
        0x9FB,
        // mov dword_75592C, 1
        &[0xC7, 0x05, 0x2C, 0x59, 0x75, 0x00, 0x01, 0x00, 0x00, 0x00],
        &CALLS,
        &[(0x4843EC, 1), (0x48140D, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    // The copy runs the original `calc_cpu_times` (through its hook) on the game's tables:
    // where those would make it raise its fatal error (a class time of 0 between stages),
    // the test is not run.
    let checked = crate::menu::calc_cpu_times::record()
        .map_err(Refused::from)
        .and_then(crate::menu::calc_cpu_times::check);
    if checked.is_err() {
        return t.skip(
            "the computer times' check refuses the game's tables (the original would end the game)",
        );
    }
    let first = game_addr(DRIVERS_RVA) as u32;
    let (Ok(drivers), table, summary) = (
        entries(first, DRIVER),
        unsafe { game_read::<u32>(TABLE_A_RVA) },
        unsafe { game_read::<u32>(SUMMARY_RVA) },
    ) else {
        return t.skip("the game's tables are not there");
    };
    if table == 0 || summary == 0 || (0..4).any(|n| player(n) == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut regions: Vec<Region> = vec![
        (first, drivers.len() * DRIVER as usize),
        (table, (200 * A_ENTRY) as usize),
        (summary, 0x50 * 0x5A),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
        (game_addr(rva(0x75AAE8)) as u32, 0x18),
        (game_addr(rva(0x71C3D8)) as u32, 8),
        (game_addr(RACE_INFO_RVA) as u32, 8),
    ];
    regions.extend(
        [
            GOES_ON_RVA,
            CHEAT_RVA,
            KEPT_RVA,
            CURRENT_RVA,
            RESULT_RVA,
            RACE_KEPT_RVA,
            rva(0x7559EC),
            rva(0x7559F0),
            rva(0x755944),
            rva(0x755948),
            rva(0x7559B4),
            rva(0x7559B8),
            crate::math::next_random::STATE_RVA,
        ]
        .map(at),
    );
    regions.extend((0..4u32).map(|n| (player(n), 0x400)));
    let mut vars = regions.clone();
    vars.extend([MODE_RVA, RESET_ONLY_RVA, RESULT_2_RVA].map(at));
    let saved = capture(&vars);
    // (mode, reset only, result, the players taking part)
    let cases: [(u32, u32, u32, u32); 8] = [
        (9, 0, NONE, 0b1111),
        (0xA, 0, 0, 0b1111),
        (0xB, 0, NONE, 0b0011),
        (0xB, 0, QUIT, 0b0011),
        (0xC, 0, NONE, 0b0110),
        (0xD, 0, NONE, 0b1111),
        (0x12, 0, 0, 0b1010),
        (0x10, 1, NONE, 0b0001),
    ];
    for (mode, reset_only, result, taking) in cases {
        run_both(
            &mut t,
            &format!(
                "mode {mode:#x}, reset only {reset_only}, result {result:#x}, players {taking:#06b}"
            ),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(MODE_RVA, mode);
                game_write(RESET_ONLY_RVA, reset_only);
                game_write(RESULT_RVA, result);
                game_write(RESULT_2_RVA, NONE);
                game_write(crate::math::next_random::STATE_RVA, 0x0BAD_5EEDu32 ^ mode);
                for n in 0..4u32 {
                    put(player(n) + 0xD0, if taking & (1 << n) != 0 { 1 } else { 0 });
                }
                for &d in &drivers[..PICKED as usize] {
                    put(d + 0x24, 1);
                }
            },
            &|r| unsafe { run_event_races(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
