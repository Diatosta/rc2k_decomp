//! `race::set_up_panel` (0x45AB36): the panel's values each frame drawn:
//! `race::panel_values`, `panel_neighbours`, `panel_players`, `panel_motion`,
//! `panel_marks`, `both_countdown_lights`, `both_split_colours` (seven calls and a `retn`).
//!
//! In:  nothing
//! Out: the registers as the seven leave them (the caller `sub_45429B` goes on with a
//!      `call`)
//!
//! See `race/frame_draw.rs`. It begins with a call: private copy (0x24, its seven calls
//! relocated), manual patch. Refuses (before anything) what `panel_neighbours` or
//! `both_split_colours` would. Shadow mode: by state (the panel 0x6D13FE .. 0x6D14CA, the
//! scratch dword, the labels, the lights and their flags, the colour buffers). Self-test:
//! on the game's state with the outputs filled with 0xAA, the mode as it is and 0; all put
//! back (skipped without view blocks or when the game's state would be refused).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::set_up_panel", rva(0x45AB36), thunk, &ORIGINAL)
    .private_copy(0x24)
    .relocate_calls(&[0x00, 0x05, 0x0A, 0x0F, 0x14, 0x19, 0x1E])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MODE: usize = rva(0x62389C); // dword_62389C

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = panel_regions();
    v.extend(light_regions());
    v.extend([
        g(0x6D1838, 7),
        g(0x6D1844, 7),
        g(0x6D1830, 8),
        g(0x6D160C, 0x200),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { set_up_panel(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let views: Vec<u32> = VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect();
    if views.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(MODE));
    let saved = capture(&kept);
    let mode: u32 = unsafe { game_read(MODE) };
    for m in [mode, 0] {
        unsafe { game_write(MODE, m) };
        let refused = unsafe { set_up_panel(&Regs::SENTINEL) }.is_err();
        restore(&kept, &saved);
        if refused {
            t.check(true, String::new);
            continue;
        }
        check_state(
            &mut t,
            &format!("mode {m}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(r, len) in &regions {
                    std::ptr::write_bytes(r as *mut u8, 0xAA, len);
                }
                game_write(MODE, m);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
