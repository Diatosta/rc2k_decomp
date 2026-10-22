//! `menu::inspection_pass` (0x4A8556): the inspection after a stage. `dword_75592C` 1;
//! for each counted player (`dword_75598C` 0 .. `dword_755984`, the original's `do ..
//! while`) taking part (+0xD0 1): its figures (`menu::driver_stats`), the verdict
//! (`menu::inspection_verdict`) and the faults (`menu::inspection_faults`); one that fails
//! (+0xD0 now 2) counts in `dword_755978` and is shown the inspection: its screen
//! (`menu::inspection_screen`), a second's sleep, the fade in, the inspection page
//! (`menu::inspection_area_loop`; a yes to leaving ends it all), the fade out, 1250 ms'
//! sleep. When every player has failed (`dword_755978` the player count), `dword_75592C`
//! 0 (the event is over).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The figures, verdict and faults are their Rust versions;
//! the sleeps, the screen, the fades and the page through `part`, in the game their Rust
//! versions (the sleep `system::sleep`'s). Ungated (the page waits for the player).
//! Refuses a player count of 0 or past 4, a player missing, a taking-part player's car
//! without its table, before anything. Shadow mode runs the original alone (it logs and
//! shows). Self-test: a copy whose screen, sleep, fade and
//! page calls go to stand-ins, against the Rust version, players taking part by lot.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::inspection_pass", rva(0x4A8556), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const GOES_ON_RVA: usize = rva(0x75592C); // dword_75592C
const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FAILED_RVA: usize = rva(0x755978); // dword_755978
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

/// A call by place: 1 `system::sleep` (eax the milliseconds), 2 `menu::inspection_area_loop`,
/// 3 `menu::fade_in`, 4 `menu::fade_out`, 5 `menu::inspection_screen`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn inspection_pass(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::passes` (who is looked at, the pauses, `goes_on`).
    use rc2k_core::menu::passes;
    let players = counted_players()?;
    for &p in &players {
        if passes::takes_part(get(p + 0xD0)) {
            crate::menu::driver_stats::car_table(p)?;
        }
    }
    let mut g = *entry;
    unsafe {
        game_write(GOES_ON_RVA, 1u32);
        for (n, &p) in players.iter().enumerate() {
            g.ecx = n as u32;
            game_write(CURRENT_RVA, g.ecx);
            g.edi = p;
            if !passes::takes_part(get(p + 0xD0)) {
                continue;
            }
            crate::menu::driver_stats::driver_stats(&g)?;
            crate::menu::inspection_verdict::inspection_verdict(&g)?;
            crate::menu::inspection_faults::inspection_faults(&g)?;
            if !passes::is_out(get(p + 0xD0)) {
                continue;
            }
            game_write(FAILED_RVA, game_read::<u32>(FAILED_RVA).wrapping_add(1));
            part(5, &mut g)?;
            g.eax = passes::SCREEN_PAUSE;
            part(1, &mut g)?;
            part(3, &mut g)?;
            part(2, &mut g)?;
            if game_read::<u32>(ANSWER_RVA) == 1 {
                return Ok(*entry);
            }
            part(4, &mut g)?;
            g.eax = passes::INSPECTION_PAUSE;
            part(1, &mut g)?;
        }
        if !passes::goes_on(game_read(FAILED_RVA), game_read(PLAYER_COUNT_RVA)) {
            game_write(GOES_ON_RVA, 0u32);
        }
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::menu::fades::{fade, game_part as clock};
    unsafe {
        match place {
            1 => crate::system::sleep::sleep(r.eax),
            2 => {
                crate::menu::screen_loop::run_screen(&crate::menu::inspection_area_loop::SCREEN, r)
            }
            3 => *r = fade(r, false, &mut clock)?,
            4 => *r = fade(r, true, &mut clock)?,
            _ => crate::menu::inspection_screen::inspection_screen(
                r,
                &mut crate::menu::screen_loop::Real,
            ),
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
    match unsafe { inspection_pass(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { inspection_pass(r, &mut game_part) }?;
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
        0x4A8556,
        0xB2,
        // pusha; mov dword_75592C, 1
        &[
            0x60, 0xC7, 0x05, 0x2C, 0x59, 0x75, 0x00, 0x01, 0x00, 0x00, 0x00,
        ],
        &[0x2D, 0x32, 0x37, 0x59, 0x63, 0x68, 0x6D, 0x7B, 0x85],
        &[
            (0x41100C, 1),
            (0x47DBF3, 2),
            (0x4A1CFA, 3),
            (0x4A1DA5, 4),
            (0x4A1BBF, 5),
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
    let mut regions: Vec<Region> = vec![(game_addr(rva(0x75AB60)) as u32, 0x75AC88 - 0x75AB60)];
    regions.extend((0..4u32).map(|n| (player(n), 0x400)));
    regions.extend(
        [
            GOES_ON_RVA,
            CURRENT_RVA,
            FAILED_RVA,
            rva(0x7559EC),
            rva(0x7559F0),
            rva(0x75AC80),
        ]
        .map(at),
    );
    regions.push((game_addr(rva(0x759C76)) as u32, 13 * 4));
    regions.push((game_addr(rva(0x75ACA8)) as u32, 0x14));
    let mut vars = regions.clone();
    vars.extend([PLAYER_COUNT_RVA, ANSWER_RVA].map(at));
    let saved = capture(&vars);
    // (player count, the players taking part, the leaving answered yes)
    for (count, taking, yes) in [
        (1u32, 0b0001u32, 0u32),
        (2, 0b0011, 0),
        (4, 0b1011, 0),
        (3, 0b0111, 1),
        (4, 0, 0),
    ] {
        run_both(
            &mut t,
            &format!("players {count}, taking part {taking:#06b}, yes {yes}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(PLAYER_COUNT_RVA, count);
                game_write(ANSWER_RVA, yes);
                game_write(FAILED_RVA, 0u32);
                for n in 0..4u32 {
                    let p = player(n);
                    put(p + 0xD0, u32::from(taking & (1 << n) != 0));
                    for k in 0..14u32 {
                        put(p + 0xD4 + 4 * k, [0u32, 3, 40, 200][((k + n) % 4) as usize]);
                    }
                }
            },
            &|r| unsafe { inspection_pass(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
