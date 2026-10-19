//! `menu::enter_time_trial_best` (0x48C8B6): the current player's time put in the tenth
//! of the chosen stage's best times and moved up to its place. `dword_7559EC` the stage's
//! +0x3C; the tenth entry's +0x14 the player's time (+0x88), +0x1C its +0x90, +8 the
//! nationality (+4), +0xC and +0x20 the name pointer (+0x18), +0x10 the car (+8), +4 1,
//! +0x40 1 (the new entry's mark), +0x50 `g_cheats`, +0x54 / +0x58 the sealed time
//! (`menu::sealed_player_time` with eax the stage's +0x3C), the 0x40 bytes at the player's
//! +0x20C at +0x5C, the name's first 31 bytes at +0x20; then `menu::rank_time_trial_best`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events, when the player qualifies. The two callees are their
//! Rust versions, `sealed_player_time` with the registers the original has there (eax the
//! stage's +0x3C, ebx `dword_7558D0`, esi the player, edi the entry). Refuses (before
//! anything) a current player past the four or missing, a stage the tables do not give,
//! or a name pointer of 0. Self-test: the game's tables for each stage choice there, the
//! player's time by case; the seal's state, `dword_7559EC`, `hhmm` and `g_keptPlayerName`
//! compared too; put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::event_flow::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::enter_time_trial_best",
    rva(0x48C8B6),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const STAGE_SLOT: usize = 0x7559EC; // dword_7559EC
const HHMM: usize = 0x7559F0; // hhmm

pub unsafe fn enter_time_trial_best(entry: &Regs) -> Result<(), Refused> {
    let record = stage_record()?;
    let tenth = stage_times()? + crate::menu::time_trial_qualifies::TENTH;
    let p = current_player()?;
    let name = dword(p + 0x18);
    if name == 0 {
        return Err(BadInput("a player's name pointer of 0").into());
    }
    let slot = dword(record + 0x3C);
    set(STAGE_SLOT, slot);
    put(tenth + 0x14, dword(p + 0x88));
    put(tenth + 0x1C, dword(p + 0x90));
    put(tenth + 8, dword(p + 4));
    put(tenth + 0xC, name);
    put(tenth + 0x10, dword(p + 8));
    put(tenth + 4, 1);
    put(tenth + 0x40, 1);
    put(tenth + 0x50, get(CHEATS));
    let sealed = unsafe {
        crate::menu::sealed_player_time::sealed_player_time(&Regs {
            eax: slot,
            ebx: get(STAGE_SUB),
            esi: p,
            edi: tenth,
            ..*entry
        })?
    };
    put(tenth + 0x54, sealed.eax);
    put(tenth + 0x58, sealed.ebx);
    put(tenth + 0x20, name);
    unsafe {
        std::ptr::copy((p + 0x20C) as *const u8, (tenth + 0x5C) as *mut u8, 0x40);
        let name = dword(tenth + 0xC);
        for k in 0..0x1F {
            *((tenth + 0x20 + k) as *mut u8) = *((name + k) as *const u8);
        }
    }
    crate::menu::rank_time_trial_best::rank_time_trial_best()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { enter_time_trial_best(r)? };
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let mut v = stage_regions().unwrap_or_default();
    v.push((
        game_addr(rva(crate::menu::rank_time_trial_best::KEPT_NAME)) as u32,
        0x1F,
    ));
    v.extend(crate::race::save_seal::seal_regions());
    v.push((game_addr(crate::race::save_seal::BLOCK_RVA) as u32, 8));
    v.extend([STAGE_SLOT, HHMM].map(|a| at(rva(a))));
    v
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(p) = current_player() else {
        return t.skip("no current player");
    };
    if dword(p + 0x18) == 0 {
        return t.skip("the current player has no name pointer");
    }
    let mut vars = [STAGE_KIND, STAGE_SUB, STAGE_NUMBER]
        .map(|v| at(rva(v)))
        .to_vec();
    vars.push((p + 0x88, 4));
    let saved_vars = capture(&vars);
    let choices = stage_choices();
    if choices.is_empty() {
        return t.skip("no stage tables");
    }
    for c in choices {
        choose_stage(c);
        let regions = regions();
        let saved = capture(&regions);
        let Ok(times) = stage_times() else { continue };
        for (case, time) in [0u32, 1, 0x12345, 0x7FFF_FFFF].into_iter().enumerate() {
            check_state(
                &mut t,
                &format!("stage {c:x?}, time {time:#x}"),
                call_original,
                Regs {
                    eax: 0x40 + case as u32,
                    ..Regs::SENTINEL
                },
                &|| {
                    restore(&regions, &saved);
                    choose_stage(c);
                    put(p + 0x88, time);
                    for k in 0..10u32 {
                        put(times + k * TIME_ENTRY + 0x14, 0x100 + k * 0x1000);
                    }
                },
                &regions,
                &rust,
            );
        }
        restore(&regions, &saved);
    }
    restore(&vars, &saved_vars);
    t.finish()
}
