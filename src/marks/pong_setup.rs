//! `marks::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `marks::squares_setup` with the texture places from `dword_769222` .. (the original
//! also loads `dword_76921E` into eax and drops it).
//!
//! In:  nothing
//! Out: eax .. edx the four places, edi `g_polygonVertices`; the rest as the device calls
//!      leave them
//!
//! Caller: `marks::run_pong`. The device calls through `part` as `squares_setup`'s.
//! Hooked through a private copy (a call at the entry). Shadow mode runs the original
//! alone (it tells the device). Self-test: a copy whose calls go to stand-ins, against
//! the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::marks::squares_setup::{CALLS, Part, game_part, setup_with};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::pong_setup", rva(0x4B88CB), thunk, &ORIGINAL)
    .private_copy(0x140)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLACES_RVA: usize = rva(0x769222); // dword_769222 .. dword_76922E
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pong_setup(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { setup_with(entry, PLACES_RVA, part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { pong_setup(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call scene::depth_buffer_on (its first bytes as the original's, not the hook's)
    let first: [u8; 5] = {
        let rel = (0x44E41Fi64 - (0x4B88CB + 5)) as i32;
        let b = rel.to_le_bytes();
        [0xE8, b[0], b[1], b[2], b[3]]
    };
    let copy = match copy(
        0x4B88CB,
        0x140,
        &first,
        &CALLS,
        &[(0x44E41F, 1), (0x44E77E, 2), (0x44EB72, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(VERTICES_RVA) as u32, 0x80),
        at(rva(0x6240F8)),
        at(rva(0x62400C)),
        at(rva(0x89CEEA)),
    ];
    let mut vars = regions.clone();
    vars.extend([rva(0x5D1E10), rva(0x6241C8)].map(at));
    vars.push((game_addr(PLACES_RVA) as u32, 16));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                for n in 0..4 {
                    game_write(
                        PLACES_RVA + 4 * n,
                        ((n as u32 + case) as f32 * 0.5).to_bits(),
                    );
                }
                game_write(rva(0x6241C8), 2500.0f32.to_bits());
                game_write(rva(0x5D1E10), 0x41 + case);
                std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
            },
            &|r| unsafe { pong_setup(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
