//! `menu::championship_standings` (0x4A9583): the championship's standings pages,
//! `dword_75AB34` 1 while they show. Standings table B from all
//! (`menu::standings_b_from_all`); outside event mode 0x13: sorted by figure and by score
//! (`menu::sort_standings_b_by_figure`, `_by_score`), numbered (`menu::number_all_b`), the
//! overall page (`menu::show_championship_standings`); in it: sorted by figure twice and
//! by points (`_by_points`), the class pages (`menu::class_championship_pass`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. The steps are their Rust versions; the pages through `part`, in
//! the game their Rust versions. Ungated (the pages wait for the player). Refuses what the
//! steps refuse. Self-test: a copy whose page calls go to stand-ins, against the Rust
//! version, in both modes, on the game's tables; put back.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::championship_standings",
    rva(0x4A9583),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FLAG_RVA: usize = rva(0x75AB34); // dword_75AB34
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// A call by place: 1 `menu::show_championship_standings`, 2 `menu::class_championship_pass`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn championship_standings(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        use crate::menu::{sort_standings_b_by_figure as fig, sort_standings_b_by_points as pts};
        g = crate::menu::standings_b_from_all::standings_b_from_all(&g)?;
        if game_read::<u32>(EVENT_MODE_RVA) != 0x13 {
            g = fig::sort_standings_b_by_figure(&g)?;
            g = crate::menu::sort_standings_b_by_score::sort_standings_b_by_score(&g)?;
            g = crate::menu::number_all_b::number_all_b(&g)?;
            game_write(FLAG_RVA, 1u32);
            part(1, &mut g)?;
        } else {
            g = fig::sort_standings_b_by_figure(&g)?;
            g = fig::sort_standings_b_by_figure(&g)?;
            g = pts::sort_standings_b_by_points(&g)?;
            game_write(FLAG_RVA, 1u32);
            part(2, &mut g)?;
        }
        game_write(FLAG_RVA, 0u32);
    }
    Ok(*entry)
}

/// In the game: the pages' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        *r = match place {
            1 => {
                use crate::menu::show_championship_standings as p;
                p::show_championship_standings(r, &mut p::game_part)
            }
            _ => {
                use crate::menu::class_championship_pass as p;
                p::class_championship_pass(r, &mut p::game_part)
            }
        }?;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { championship_standings(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4A9583,
        0x68,
        &[0x60, 0x83, 0x3D, 0xD0, 0x56, 0x75, 0x00, 0x13],
        &[0xA, 0xF, 0x14, 0x19, 0x28, 0x39, 0x3E, 0x43, 0x48, 0x57],
        &[(0x4A5A47, 1), (0x4A4A1C, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let table: u32 = unsafe { game_read(TABLE_B_RVA) };
    if table == 0 || (0..4).all(|n| player(n) == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut regions: Vec<Region> = vec![
        (table, (200 * B_ENTRY) as usize),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
        at(rva(0x7559B4)),
        at(rva(0x7559B8)),
        at(FLAG_RVA),
    ];
    regions.extend(
        (0..4u32)
            .map(|n| (player(n), 0x400))
            .filter(|&(p, _)| p != 0),
    );
    let mut vars = regions.clone();
    vars.extend([EVENT_MODE_RVA, PLAYER_COUNT_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..2u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(EVENT_MODE_RVA, if case == 1 { 0x13u32 } else { 0 });
            },
            &|r| unsafe { championship_standings(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
