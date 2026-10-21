//! `menu::find_class_time` (0x47B4EA): the first entry (0x9C bytes each, from edi) whose
//! class (+0x1C) is eax, or the end mark's (-1) when none is: its +0x18 with ebx 1, else
//! its +0x14.
//!
//! In:  eax = the class, ebx = 1 for +0x18, edi = the list
//! Out: eax = the dword; the rest kept (edi pushed and popped)
//!
//! Caller: `screens::draw_class_times` (the first time of the class). Compared in shadow
//! mode. Refuses a list without its end in 0x400 entries (the original runs on). Manual
//! patch with a private copy (0x20 bytes, nothing to relocate): its loop comes back to the
//! second byte, and a `jz` sits in the first five. Self-test: lists of the test's, classes
//! in them and not, ebx 1 and 0.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::find_class_time", rva(0x47B4EA), thunk, &ORIGINAL)
    .private_copy(0x20)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ENTRY_LEN: u32 = 0x9C;
const LIMIT: u32 = 0x400;

fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The entry the original stops at.
pub fn class_entry(list: u32, class: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::menu::standings::search` (the end mark's entry when none).
    use rc2k_core::menu::standings::{Found, search};
    let at = |n: u32| list.wrapping_add(n.wrapping_mul(ENTRY_LEN));
    let mut last = 0;
    match search(
        |n| {
            last = n;
            get(at(n))
        },
        |n| get(at(n) + 0x1C) == class,
        LIMIT,
    ) {
        Found::At(n) => Ok(at(n)),
        Found::End => Ok(at(last)),
        Found::NoEnd => Err(BadInput("a list without its end in 0x400 entries")),
    }
}

/// The Rust version (see the header).
pub fn find_class_time(list: u32, class: u32, second: bool) -> Result<u32, BadInput> {
    let e = class_entry(list, class)?;
    Ok(get(e + if second { 0x18 } else { 0x14 }))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let v = find_class_time(r.edi, r.eax, r.ebx == 1)?;
    Ok(Left::nothing().with(EAX, v))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut list = vec![0u32; 6 * ENTRY_LEN as usize / 4];
    for (n, d) in list.iter_mut().enumerate() {
        *d = 0x1000 + n as u32;
    }
    let at = list.as_mut_ptr() as u32;
    for (k, class) in [3u32, 5, 3, 7, 9].iter().enumerate() {
        unsafe { ((at + k as u32 * ENTRY_LEN + 0x1C) as *mut u32).write_unaligned(*class) };
    }
    unsafe { ((at + 5 * ENTRY_LEN) as *mut u32).write_unaligned(u32::MAX) };
    for class in [3u32, 5, 7, 9, 4] {
        for ebx in [1u32, 0, 2] {
            check_state(
                &mut t,
                &format!("class {class}, ebx {ebx}"),
                call_original,
                Regs {
                    eax: class,
                    ebx,
                    edi: at,
                    ..Regs::SENTINEL
                },
                &|| {},
                &[(at, list.len() * 4)],
                &rust,
            );
        }
    }
    drop(list);
    t.finish()
}
