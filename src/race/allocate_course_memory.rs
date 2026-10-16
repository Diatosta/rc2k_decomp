//! `race::allocate_course_memory` (0x41F8C4): the course's memory taken (`alloc coursemem`).
//!
//! In:  nothing
//! Out: eax = the memory's end, ebx = the list; the rest as `memory::allocate` leaves them
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::allocate_course_memory",
    rva(0x41F8C4),
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

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    unsafe {
        allocate_course_memory(
            r,
            v,
            crate::framework::game_addr(COURSE_LIST_RVA) as u32,
            &mut game_part,
        )
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match rust(r, &Vars::game()) {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = TestMemory::new();
    let copy = match copy_of(
        &m,
        0x41F8C4,
        0x35,
        &[0xB8, 0x00, 0x00, 0xF0, 0x00],
        &[0x1B],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    for total in [0u32, 0x1000] {
        both(
            &mut t,
            &format!("total {total:#x}"),
            &copy,
            &m,
            &|| {
                GIVES.store(0x1234_0000, std::sync::atomic::Ordering::SeqCst);
                put(v.list_total, total);
            },
            &|r| unsafe { allocate_course_memory(r, &v, m.patches()[14].1, &mut test_part) },
        );
    }
    t.finish()
}
