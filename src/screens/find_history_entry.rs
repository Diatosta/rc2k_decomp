//! `screens::find_history_entry` (0x4A256C): from championship history entry edi on (0x78
//! bytes each), the first of class ebx and position eax; the entry edi points at is looked at
//! before the end test (-1); none is fatal error 0x8070A7FA ("Position in GB_ChampHST Not
//! Found"), raised by a `jmp` to `fatal::report`.
//!
//! In:  edi = entry, ebx = class, eax = position
//! Out: edi = the entry found; ZF set (no caller reads it); the rest untouched
//!
//! Caller: `screens::draw_championship_table` (0x4A502D). The loop jumps back to the first
//! byte: `.private_copy(0x18)`, the `jmp` to the fatal block relocated. Shadow mode: by state.
//! Self-test: a test table, hits on the first, a middle and the last entry (the missing case
//! is not run: the original ends the game).

use crate::errors::Refused;
use crate::framework::{EDI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::championship_table::find_history_entry;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::find_history_entry",
    rva(0x4A256C),
    thunk,
    &ORIGINAL,
)
.private_copy(0x18)
.relocate_calls(&[0x12])
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EDI, unsafe { find_history_entry(r.edi, r.ebx, r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // (class, position) of five entries, then the end.
    let entries = [(1u32, 3u32), (0, 1), (1, 1), (0, 2), (1, 2)];
    let mut table = vec![0u8; 0x78 * (entries.len() + 1)];
    for (n, &(class, position)) in entries.iter().enumerate() {
        let e = &mut table[n * 0x78..];
        e[0x10..0x14].copy_from_slice(&class.to_le_bytes());
        e[0x70..0x74].copy_from_slice(&position.to_le_bytes());
    }
    let end = entries.len() * 0x78;
    table[end..end + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    let p = table.as_ptr() as u32;
    for (class, position) in [(1u32, 3u32), (0, 1), (1, 1), (0, 2), (1, 2)] {
        check_state(
            &mut t,
            &format!("class {class} position {position}"),
            call_original,
            Regs {
                edi: p,
                ebx: class,
                eax: position,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    drop(table);
    t.finish()
}
