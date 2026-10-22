//! `menu::service_pass` (0x4A15E3): the service area between stages. `dword_75ACC0` 0,
//! `dword_75593C` up by 900, `dword_75592C` 1; for each counted player (`dword_75598C` 0
//! .. `dword_755984`, the original's `do .. while`; ebx adds up their +0xD0) taking part
//! (+0xD0 1) when the service is due (`dword_755938` 1): the inspection's screen
//! (`menu::inspection_screen`), a second's sleep, the fade in, 250 ms' sleep,
//! `dword_75AC8C` 0, the figures (`menu::driver_stats`), `dword_75AB38` 0, the drivers'
//! condition reset (`menu::reset_driver_condition`), the repair flags cleared
//! (`service::clear_repair_flags`), six keys taken (`input::take_key`), the service page
//! (`service::service_area_loop`; a yes to leaving ends it all), the repairs
//! (`service::apply_repairs`), the entry kind (`menu::set_player_entry_kind`); a player
//! now out (+0xD0 2) gets the faults (`menu::inspection_faults`) and the inspection page
//! (`menu::inspection_area_loop`; a yes ends it all) and counts in `dword_755978`; its
//! +0x288 then `dword_75ACC0` x 100. When every player is out, `dword_75592C` 0. At the
//! end `dword_755938` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The pure steps are their Rust versions; the sleeps, the
//! screen, the fade, the keys, the pages and the repairs (its clock) through `part`, in the
//! game their Rust versions (the repairs through their hook). Ungated (the pages wait for
//! the player). Refuses a player count of 0 or past 4, a player missing, a taking-part
//! player's car without its table, before anything. Shadow mode runs the original alone.
//! Self-test: a copy whose screen, sleep, fade, key, page
//! and repair calls go to stand-ins, against the Rust version, by lot.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::service_pass", rva(0x4A15E3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const MONEY_RVA: usize = rva(0x75ACC0); // dword_75ACC0
const TIME_RVA: usize = rva(0x75593C); // dword_75593C
const GOES_ON_RVA: usize = rva(0x75592C); // dword_75592C
const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const DUE_RVA: usize = rva(0x755938); // dword_755938
const CLEARED_RVA: [usize; 2] = [rva(0x75AC8C), rva(0x75AB38)]; // dword_75AC8C, dword_75AB38
const OUT_RVA: usize = rva(0x755978); // dword_755978
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const SERVICE_LOOP: usize = 0x47284A; // service::service_area_loop

/// Its calls (the test's copy).
const CALLS: [usize; 18] = [
    0x5B, 0x65, 0x6A, 0x74, 0x83, 0x92, 0x97, 0x9C, 0xA1, 0xA6, 0xAB, 0xB0, 0xB5, 0xBA, 0xC8, 0xCD,
    0xE9, 0xEE,
];

/// A call by place: 1 `system::sleep` (eax), 2 `menu::inspection_area_loop`, 3
/// `menu::fade_in`, 5 `menu::inspection_screen`, 6 `service::service_area_loop`, 7
/// `service::apply_repairs`, 8 `input::take_key`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn service_pass(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::passes` (who is serviced, the minutes, the money, the
    // pauses, `goes_on`).
    use rc2k_core::menu::passes;
    let players = counted_players()?;
    let due = unsafe { game_read::<u32>(DUE_RVA) } == 1;
    for &p in &players {
        if due && passes::takes_part(get(p + 0xD0)) {
            crate::menu::driver_stats::car_table(p)?;
        }
    }
    let mut g = *entry;
    unsafe {
        game_write(MONEY_RVA, 0u32);
        game_write(TIME_RVA, passes::more_minutes(game_read(TIME_RVA)));
        game_write(GOES_ON_RVA, 1u32);
        g.ebx = 0;
        'all: {
            for (n, &p) in players.iter().enumerate() {
                g.ecx = n as u32;
                game_write(CURRENT_RVA, g.ecx);
                g.edi = p;
                g.eax = get(p + 0xD0);
                g.ebx = g.ebx.wrapping_add(g.eax);
                if !passes::serviced(g.eax, || game_read(DUE_RVA)) {
                    continue;
                }
                part(5, &mut g)?;
                g.eax = passes::SCREEN_PAUSE;
                part(1, &mut g)?;
                part(3, &mut g)?;
                g.eax = passes::FADE_PAUSE;
                part(1, &mut g)?;
                game_write(CLEARED_RVA[0], 0u32);
                crate::menu::driver_stats::driver_stats(&g)?;
                game_write(CLEARED_RVA[1], 0u32);
                crate::menu::reset_driver_condition::reset_driver_condition(&g)?;
                crate::service::clear_repair_flags::clear_repair_flags()?;
                for _ in 0..passes::DROPPED_KEYS {
                    part(8, &mut g)?;
                }
                part(6, &mut g)?;
                if game_read::<u32>(ANSWER_RVA) == 1 {
                    break 'all;
                }
                part(7, &mut g)?;
                g = crate::menu::set_player_entry_kind::set_player_entry_kind(&g)?;
                g.edi = p;
                if passes::is_out(get(p + 0xD0)) {
                    crate::menu::inspection_faults::inspection_faults(&g)?;
                    part(2, &mut g)?;
                    if game_read::<u32>(ANSWER_RVA) == 1 {
                        break 'all;
                    }
                    game_write(OUT_RVA, game_read::<u32>(OUT_RVA).wrapping_add(1));
                }
                g.edi = p;
                g.eax = passes::spent(game_read(MONEY_RVA));
                put(p + 0x288, g.eax);
            }
            if !passes::goes_on(game_read(OUT_RVA), game_read(PLAYER_COUNT_RVA)) {
                game_write(GOES_ON_RVA, 0u32);
            }
        }
        game_write(DUE_RVA, 0u32);
    }
    Ok(*entry)
}

/// In the game: the Rust versions (the repairs through their hook, for their clock).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            6 => {
                crate::menu::screen_loop::run_screen(&crate::service::service_area_loop::SCREEN, r)
            }
            7 => crate::service::apply_repairs::apply_repairs_in_game(r),
            8 => {
                let key = crate::input::take_key::take_key();
                r.eax = (r.eax & !0xFF) | u32::from(key);
                r.set_zero(key == 0);
            }
            _ => crate::menu::inspection_pass::game_part(place, r)?,
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
    match unsafe { service_pass(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { service_pass(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Unpaused: frames drawn meanwhile may see the tables this test changes.
    let _disturbing = crate::screens::harness::Disturbing::begin();
    let copy = match copy(
        0x4A15E3,
        0x150,
        // pusha; mov dword_75ACC0, 0
        &[
            0x60, 0xC7, 0x05, 0xC0, 0xAC, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &CALLS,
        &[
            (0x41100C, 1),
            (0x47DBF3, 2),
            (0x4A1CFA, 3),
            (0x4A1BBF, 5),
            (SERVICE_LOOP, 6),
            (0x46C14D, 7),
            (0x407F95, 8),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    if (0..4).any(|n| player(n) == 0) {
        return t.skip("the game's players are not there");
    }
    // The copy runs the original `driver_stats`, which reads each car's table unchecked.
    if (0..4).any(|n| crate::menu::driver_stats::car_table(player(n)).is_err()) {
        return t.skip("a player's car has no table (the original would read through 0)");
    }
    let mut regions: Vec<Region> = vec![
        (game_addr(rva(0x75AB60)) as u32, 0x75AC88 - 0x75AB60),
        (game_addr(rva(0x759C76)) as u32, 13 * 4),
        (game_addr(rva(0x75ACA8)) as u32, 0x14),
    ];
    let first = game_addr(DRIVERS_RVA) as u32;
    if let Ok(d) = entries(first, DRIVER) {
        regions.push((first, d.len() * DRIVER as usize));
    }
    regions.extend((0..4u32).map(|n| (player(n), 0x400)));
    regions.extend(
        [
            MONEY_RVA,
            TIME_RVA,
            GOES_ON_RVA,
            CURRENT_RVA,
            DUE_RVA,
            CLEARED_RVA[0],
            CLEARED_RVA[1],
            OUT_RVA,
            rva(0x7559EC),
            rva(0x7559F0),
        ]
        .map(at),
    );
    regions.extend(crate::service::harness::repair_regions());
    let mut vars = regions.clone();
    vars.extend([PLAYER_COUNT_RVA, ANSWER_RVA].map(at));
    let saved = capture(&vars);
    // (player count, the players taking part, due, the leaving answered yes)
    for (count, taking, due, yes) in [
        (1u32, 0b0001u32, 1u32, 0u32),
        (2, 0b0011, 1, 0),
        (4, 0b1101, 1, 0),
        (2, 0b0011, 1, 1),
        (3, 0b0111, 0, 0),
    ] {
        run_both(
            &mut t,
            &format!("players {count}, taking part {taking:#06b}, due {due}, yes {yes}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(PLAYER_COUNT_RVA, count);
                game_write(ANSWER_RVA, yes);
                game_write(DUE_RVA, due);
                game_write(OUT_RVA, 0u32);
                game_write(MONEY_RVA, 7u32);
                for n in 0..4u32 {
                    put(player(n) + 0xD0, u32::from(taking & (1 << n) != 0));
                }
            },
            &|r| unsafe { service_pass(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
