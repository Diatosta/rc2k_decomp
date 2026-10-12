//! `service::harness::dispatch_state` for functions that write whole pictures: the same
//! steps, but a mismatch line names the first byte that differs and does not print the
//! regions, which run to megabytes.

use crate::framework::{RawHook, Regs, SelfTest, call_game, shadow};
use crate::screens::harness::CallOriginal;
use crate::service::harness::{Region, Rust, capture, restore};

fn first_difference(a: &[u8], b: &[u8]) -> Option<usize> {
    a.iter().zip(b).position(|(x, y)| x != y)
}

/// The whole `dispatch` of such a hook. `regions` is everything the function may write; the
/// Rust version must not have written anything when it returns an error.
pub fn dispatch_pixels(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    rust: Rust,
) {
    hook.count_call();
    if !shadow() {
        match rust(r) {
            Ok(left) => left.apply(r),
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }
    let before = capture(regions);
    let left = match rust(r) {
        Ok(left) => left,
        Err(e) => {
            restore(regions, &before);
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let mine = capture(regions);
    restore(regions, &before);

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = capture(regions);
    if theirs != mine || !left.matches(r, &o) {
        hook.mismatch(|| {
            format!(
                "eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x} ebp={:#x}: first different byte {:?} of {}; original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x}; rust {left:x?}",
                r.eax,
                r.ebx,
                r.ecx,
                r.edx,
                r.esi,
                r.edi,
                r.ebp,
                first_difference(&theirs, &mine),
                theirs.len(),
                o.eax,
                o.ebx,
                o.ecx,
                o.edx,
                o.esi,
                o.edi
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// One self-test case for such a function: `prepare` puts the inputs in place, the code at
/// `original` (the hook's original, or a copy of it) runs, `prepare` runs again, the Rust
/// version runs, and what the two left in `regions` and the registers is compared.
pub fn check_pixels(
    t: &mut SelfTest,
    what: &str,
    original: usize,
    before: Regs,
    prepare: &dyn Fn(),
    regions: &[Region],
    rust: Rust,
) {
    prepare();
    let mut ro = before;
    unsafe { call_game(&mut ro, original) };
    let theirs = capture(regions);
    prepare();
    let mine = rust(&before);
    let state = capture(regions);
    let regs_ok = mine.as_ref().is_ok_and(|left| left.matches(&before, &ro));
    t.check(regs_ok && theirs == state, || {
        format!(
            "{what}: regs_ok={regs_ok} first different byte {:?} of {}; original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x}; rust {mine:x?}",
            first_difference(&theirs, &state),
            theirs.len(),
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.edx,
            ro.esi,
            ro.edi
        )
    });
}
