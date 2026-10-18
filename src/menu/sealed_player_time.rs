//! `menu::sealed_player_time` (0x4A3D38): the current player's (`dword_75598C`) time
//! (+0x88) sealed (`records::seal_time`) with bl its +8, cl `dword_75ACD8` and dl eax's low
//! byte; the result also into `dword_7559EC` and `hhmm`.
//!
//! In:  eax (its low byte sealed with the time)
//! Out: eax, ebx the sealed block; the rest kept
//!
//! Callers: `sub_489F76`, `sub_48C8B6`, `menu::enter_record_entry`. The seal is its Rust
//! version. Refuses a current player past 3 or missing, before anything. Self-test: the
//! original and the Rust version on the game's player, times by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::sealed_player_time", rva(0x4A3D38), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const SLOT_RVA: usize = rva(0x7559EC); // dword_7559EC
const HHMM_RVA: usize = rva(0x7559F0); // hhmm
const SEAL_BYTE_RVA: usize = rva(0x75ACD8); // dword_75ACD8
const EXTRA: [usize; 1] = [CURRENT_RVA];
const CASES: u32 = 4;

fn test_entry(case: u32) -> Regs {
    Regs {
        eax: 0x40 + case,
        ..Regs::SENTINEL
    }
}

fn prepare(case: u32) {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n < 4 && player(n) != 0 {
        put(
            player(n) + 0x88,
            [0u32, 0x12345, 0x2255100, 0xFFFF_FFFF][case as usize],
        );
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn sealed_player_time(entry: &Regs) -> Result<Regs, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let p = player(n);
    let sealed = unsafe {
        crate::records::seal_time::seal_time(&Regs {
            eax: get(p + 0x88),
            ebx: get(p + 8),
            ecx: game_read(SEAL_BYTE_RVA),
            edx: entry.eax,
            edi: p,
            ..*entry
        })
    };
    unsafe {
        game_write(SLOT_RVA, sealed.eax);
        game_write(HHMM_RVA, sealed.ebx);
    }
    Ok(Regs {
        eax: sealed.eax,
        ebx: sealed.ebx,
        ..*entry
    })
}

fn regions() -> Vec<Region> {
    let mut v = crate::race::save_seal::seal_regions();
    v.push((game_addr(crate::race::save_seal::BLOCK_RVA) as u32, 8));
    v.extend([SLOT_RVA, HHMM_RVA].map(at));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { sealed_player_time(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    let refused = unsafe { sealed_player_time(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(case),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
