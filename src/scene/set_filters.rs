//! `scene::set_filters` (0x44E8FB): the textures' filters, by the setting.
//!
//! In:  nothing
//! Out: eax = 1, ebx = 0x31; everything else preserved
//!
//! With `dword_623898` at 0 render states 0x11 and 0x12 (the filters for a texture
//! made larger and smaller) both 2; else 2 and 6. Then state 0x31 = 1. Each through
//! `scene::set_render_state`. Five callers, `scene::begin_scene` among them.
//! Self-test: both settings, recorded on the layer, nothing drawn.

use crate::d3d::recorder::record;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::scene::set_render_state::set_render_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_filters", rva(0x44E8FB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SETTING_RVA: usize = rva(0x623898); // dword_623898

// ---- Rust version ----

// Converted: `rc2k_core::scene::render_states::filters`
pub unsafe fn set_filters(r: &mut Regs) {
    unsafe {
        for s in rc2k_core::scene::render_states::filters(game_read(SETTING_RVA)) {
            set_render_state(r, s.value, s.state);
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    unsafe { set_filters(r) };
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(SETTING_RVA) };
    for setting in [0u32, 1, 0x80] {
        unsafe { game_write(SETTING_RVA, setting) };
        let before = Regs::SENTINEL;
        let (theirs, their_calls) = record(true, || {
            let mut r = before;
            unsafe { call_original(&mut r) };
            r
        });
        let (mine, my_calls) = record(true, || {
            let mut r = before;
            unsafe { set_filters(&mut r) };
            r
        });
        t.check(
            theirs.preserved_except(&before, EAX | EBX)
                && (theirs.eax, theirs.ebx) == (mine.eax, mine.ebx)
                && mine.preserved_except(&before, EAX | EBX)
                && their_calls == my_calls,
            || {
                format!(
                    "setting {setting:#x}: eax, ebx {:#x} {:#x} / {:#x} {:#x}; calls {their_calls:x?} / {my_calls:x?}",
                    theirs.eax, theirs.ebx, mine.eax, mine.ebx
                )
            },
        );
    }
    unsafe { game_write(SETTING_RVA, saved) };
    t.finish()
}
