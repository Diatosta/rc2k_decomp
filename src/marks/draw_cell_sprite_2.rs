//! `marks::draw_cell_sprite_2` (0x4CBA54): `marks::draw_cell_sprite`'s twin, the same
//! instructions at another place (its jumps go to its own `retn`).
//!
//! In:  as `draw_cell_sprite`
//! Out: as `draw_cell_sprite`
//!
//! Caller: the ring's kind 0xB (0x4CE7C7). The Rust version is `draw_cell_sprite`'s.
//! Thunk by `three_in_hook!`. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_cell_sprite`'s, on a copy of this one.

use crate::framework::{RawHook, Regs, rva, shadow};
use crate::marks::draw_cell_sprite::{draw_cell_sprite, game_part, test_at};
use crate::scene::vector3::{Triple80, three_in_hook};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static TRIPLE: Triple80 = Triple80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::draw_cell_sprite_2", rva(0x4CBA54), thunk, &ORIGINAL).self_test(self_test);

three_in_hook!(
    thunk,
    call_original,
    original_with_three,
    dispatch = dispatch,
    original = ORIGINAL,
    triple = TRIPLE
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { original_with_three(r) };
        return;
    }
    match unsafe { draw_cell_sprite(r, TRIPLE.inputs(), &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_three) },
    }
}

fn self_test() -> (usize, usize) {
    test_at(&HOOK, 0x4CBA54)
}
