//! `scene::raise_texture_needs` (0x423F97): the textures of one tag are needed at a
//! level at least.
//!
//! In:  eax = the level, ebx = the tag
//! Out: ecx = 0x400, edx = 1 if any entry changed, else 0; zero set if any changed,
//!      carry set if none; everything else preserved
//!
//! For every texture whose tag (`dword_5CDAE0`) is ebx: its level in each of the two
//! tables (`dword_5CEAE0`, `dword_5CFAE0`) becomes eax if eax is above it, unsigned
//! (so -1, "not needed", is never raised).
//! 57 callers.
//! Self-test: 40 cases by lot on the game's tables, put back afterwards.

use crate::errors::Refused;
use crate::framework::{ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::texture_needs::{
    SYSTEM_LEVELS_RVA, TAGS_RVA, TEXTURES, VIDEO_LEVELS_RVA, levels,
};
use crate::scene::textures::{test_table, write_table};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::raise_texture_needs",
    rva(0x423F97),
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

// ---- Rust version ----

/// Whether any entry changed.
pub unsafe fn raise_texture_needs(level: u32, tag: u32) -> bool {
    // Converted: `rc2k_core::scene::textures::Needs::raise`.
    use crate::scene::textures::slice;
    let n = TEXTURES as u32;
    unsafe {
        rc2k_core::scene::textures::Needs {
            tags: slice(TAGS_RVA, n),
            system: slice(SYSTEM_LEVELS_RVA, n),
            video: slice(VIDEO_LEVELS_RVA, n),
        }
        .raise(level, tag)
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let changed = unsafe { raise_texture_needs(r.eax, r.ebx) };
    Ok(Left::carry(!changed)
        .with(ECX, TEXTURES as u32)
        .with(EDX, changed as u32))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [levels(SYSTEM_LEVELS_RVA), levels(VIDEO_LEVELS_RVA)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [levels(SYSTEM_LEVELS_RVA), levels(VIDEO_LEVELS_RVA)];
    let inputs = [levels(TAGS_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0007u32;
    for case in 0..40u32 {
        let tags = test_table(&mut seed, TEXTURES as u32, 1, 4);
        let system = test_table(&mut seed, TEXTURES as u32, 2, 6);
        let video = test_table(&mut seed, TEXTURES as u32, 3, 6);
        let level = [0u32, 1, 3, 5, 6, 0xFFFF_FFFF, 0x8000_0000, 2][case as usize % 8];
        let tag = [0u32, 1, 2, 3, 9][case as usize % 5];
        check_state(
            &mut t,
            &format!("case {case}: level {level:#x}, tag {tag}"),
            call_original,
            Regs {
                eax: level,
                ebx: tag,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                write_table(TAGS_RVA, &tags);
                write_table(SYSTEM_LEVELS_RVA, &system);
                write_table(VIDEO_LEVELS_RVA, &video);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
