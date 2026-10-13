//! `scene::prepare_menu_scene` (0x46F7FD): the menus' camera and light.
//!
//! In:  nothing
//! Out: as `scene::set_view_matrices` leaves them (eax = ebx = ecx = edx = 0, esi, edi,
//!      ebp three matrices)
//!
//! The view's place and its three angles = 0; the window 800 x 600 (`dword_62408C`,
//! `dword_624090`); `scene::set_view_window`; the light's lowest 0x50, highest 0xFF,
//! level 0xB4 and strength 0xB4, its direction's angles `dword_635094` = 0x6E and
//! `dword_635098` = 0x1E; `scene::set_view_matrices`.
//! 29 callers (the screen loops' set-up).
//! Self-test: 20 settings by lot on the game's variables.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::scene::set_view_window::{self, SIZE_RVA, set_view_window};
use crate::scene::view_matrix::{VIEW_ANGLES_RVA, VIEW_PLACE_RVA, set_view_matrices, view_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::prepare_menu_scene", rva(0x46F7FD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LIGHT_RVA: usize = rva(0x635084); // g_lightLowest, g_lightHighest, g_lightLevel, g_lightStrength
const LIGHT_ANGLES_RVA: usize = rva(0x635094); // dword_635094, dword_635098

// ---- Rust version ----

pub unsafe fn prepare_menu_scene() -> Left {
    unsafe {
        game_write(VIEW_PLACE_RVA, [0u32; 3]);
        game_write(VIEW_ANGLES_RVA, [0u32; 3]);
        game_write(SIZE_RVA, [0x320u32, 0x258]);
        set_view_window();
        game_write(LIGHT_RVA, [0x50u32, 0xFF, 0xB4, 0xB4]);
        game_write(LIGHT_ANGLES_RVA, [0x6Eu32, 0x1E]);
        set_view_matrices()
    }
}

// ---- hook ----

/// Everything it writes.
fn regions() -> Vec<Region> {
    let block = |rva: usize, len: usize| (game_addr(rva) as u32, len);
    let mut regions = view_regions();
    regions.extend(set_view_window::regions());
    regions.extend([
        block(VIEW_PLACE_RVA, 12),
        block(VIEW_ANGLES_RVA, 12),
        block(LIGHT_RVA, 0x18),
        block(SIZE_RVA, 8),
    ]);
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { prepare_menu_scene() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (regions, inputs) = (regions(), set_view_window::inputs());
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x27D4_EB2Fu32;
    for case in 0..20 {
        unsafe { set_view_window::set_inputs(&mut seed, case) };
        let set = capture(&inputs);
        let fill = [0xAAu8, 0x00, 0x3F][case % 3];
        check_state(
            &mut t,
            &format!("case {case}, filled with {fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&regions, &vec![fill; saved.0.len()]);
                restore(&inputs, &set);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
