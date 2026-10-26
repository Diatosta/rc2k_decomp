//! `scene::draw_fan` (0x44EB32, reached by the jump at 0x44EA65): the polygon in
//! `g_polygonVertices` drawn as a fan by the device (`sub_41AD7F`, the platform's), unless
//! `dword_5CDA8C` is 0; a failure (eax not 0) logged as "drawtri" and its code
//! (`sub_401427`, `sub_401622`, the logfile's level-1 writers).
//!
//! In:  nothing
//! Out: as the device's call leaves the registers (eax its result); ZF set when nothing
//!      was drawn or the draw worked (the original's last `cmp`; after a logged failure
//!      the original leaves the log's flags)
//!
//! 24 callers through 0x44EA65 (none reads the flags but through a `retn`; the flags are
//! set as the original's `cmp` leaves them). The device and the log are called in the
//! game (section 15). Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose three calls go to stand-ins (the draw's result given), against the Rust version
//! on the same stand-ins, the switch 0 and 1, the draw failing or not.

use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_fan", rva(0x44EB32), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const DEVICE_RVA: usize = rva(0x41AD7F); // the device's fan draw
const LOG_TEXT_RVA: usize = rva(0x401427); // logfile: a text, level 1
const LOG_HEX8_RVA: usize = rva(0x401622); // logfile: eight hex digits, level 1
const DRAWTRI_RVA: usize = rva(0x4DD9AD); // aDrawtri_0

/// The Rust version (see the header); `part` makes the calls by place: 1 the device, 2
/// the text, 3 the code.
pub unsafe fn draw_fan(r: &mut Regs, part: &mut dyn FnMut(u32, &mut Regs)) {
    if unsafe { game_read::<u32>(SWITCH_RVA) } == 0 {
        r.set_zero(true);
        r.set_carry(false);
        return;
    }
    part(1, r);
    let failed = r.eax != 0;
    r.set_zero(!failed);
    r.set_carry(false);
    if failed {
        let mut log = Regs {
            eax: game_addr(DRAWTRI_RVA) as u32,
            ..*r
        };
        part(2, &mut log);
        let mut log = *r;
        part(3, &mut log);
    }
}

/// In the game: the device and the log through their addresses.
pub fn game_part(place: u32, r: &mut Regs) {
    let at = match place {
        1 => DEVICE_RVA,
        2 => LOG_TEXT_RVA,
        _ => LOG_HEX8_RVA,
    };
    crate::calls::call(at, r);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { draw_fan(r, &mut game_part) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_fan(r, &mut game_part) };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_5CDA8C, 0
    let entry: &[u8] = &[0x83, 0x3D, 0x8C, 0xDA, 0x5C, 0x00, 0x00];
    let copy = match copy(
        0x44EB32,
        0x28,
        entry,
        &[0x9, 0x1A, 0x21],
        &[(0x41AD7F, 1), (0x401427, 2), (0x401622, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = [at(SWITCH_RVA)];
    let saved = capture(&vars);
    for on in [0u32, 1] {
        run_both(
            &mut t,
            &format!("switch {on}"),
            &copy,
            &vars,
            &|| unsafe { game_write(SWITCH_RVA, on) },
            &|r| {
                let mut g = *r;
                unsafe {
                    draw_fan(&mut g, &mut |p, r| {
                        let _ = test_part(p, r);
                    })
                };
                Ok(g)
            },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
