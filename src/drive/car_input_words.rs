//! `drive::car_input_words` (0x43A27E): the current car's input words (`dword_6138C4` ..
//! `6138D0`): 0, then, for a car with a device (`dword_641680[g_currentCar]` not -1), its
//! readings (`drive::device_axes`). With a replay on (`dword_52A910`): the car's block
//! (`g_carBlocks[g_currentCar]`) +0xFD0 .. gives them, or, for the car recorded
//! (`dword_52A920`), takes them.
//!
//! In:  nothing
//! Out: eax .. edx the readings (eax the car with a replay on), esi the device or the
//!      block; the rest kept; no flags read
//!
//! Callers: `drive::car_controls` (`sub_43A1CA`), `sub_460835`, `sub_460B2F`.
//! Refuses (before anything) a current car past 7, and what `device_axes` refuses.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with a test device and a test block: device or not, replay off, playing and
//! recording.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::car_input_words", rva(0x43A27E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORDS: usize = 0x6138C4;
const CURRENT: usize = 0x71B3F8;

/// The Rust version.
pub unsafe fn car_input_words(entry: &Regs) -> Result<Left, Refused> {
    let car = g(CURRENT);
    if car > 7 {
        return Err(BadInput("a current car past 7").into());
    }
    let mut r = *entry;
    let device = g(0x641680 + 4 * car as usize);
    let axes = if device != u32::MAX {
        Some(unsafe { crate::drive::device_axes::device_axes(device) }?)
    } else {
        None
    };
    for k in 0..4 {
        p(WORDS + 4 * k, 0);
    }
    r.esi = device;
    if let Some([eax, ebx, ecx, edx]) = axes {
        (r.eax, r.ebx, r.ecx, r.edx) = (eax, ebx, ecx, edx);
        for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
            p(WORDS + 4 * k, *v);
        }
    }
    if g(0x52A910) != 0 {
        r.esi = g(0x71B3B8 + 4 * car as usize);
        r.eax = car;
        let block = r.esi;
        for k in 0..4u32 {
            let at = (block + 0xFD0 + 4 * k) as *mut u32;
            if car != g(0x52A920) {
                p(WORDS + 4 * k as usize, unsafe { at.read_unaligned() });
            } else {
                unsafe { at.write_unaligned(g(WORDS + 4 * k as usize)) };
            }
        }
    }
    Ok(Left::nothing()
        .with(EAX, r.eax)
        .with(EBX, r.ebx)
        .with(ECX, r.ecx)
        .with(EDX, r.edx)
        .with(ESI, r.esi))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { car_input_words(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let block = g(0x71B3B8 + 4 * (g(CURRENT) & 7) as usize);
    let mut all = vec![(a(WORDS), 16)];
    if block != 0 {
        all.push((block + 0xFD0, 16));
    }
    dispatch_state(&HOOK, r, call_original, &all, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(WORDS), 16),
        (a(CURRENT), 4),
        (a(0x641680), 0x20),
        (a(0x71B3B8), 0x20),
        (a(0x52A910), 0x14),
        (a(0x4F6EE8), 4),
        (a(0x4F6900), 4),
    ];
    let saved = capture(&all);
    let mut device = vec![0x1234_5678u32; 0x500 / 4];
    let mut block = vec![0u32; 0xFE0 / 4];
    let (dv, bl) = (device.as_mut_ptr() as u32, block.as_mut_ptr() as u32);
    for case in 0..6u32 {
        let set = || {
            p(CURRENT, 2);
            p(0x641680 + 8, if case % 2 == 0 { u32::MAX } else { 0 });
            p(0x4F6EE8, dv);
            p(0x4F6900, 0);
            p(0x71B3B8 + 8, bl);
            p(0x52A910, (case / 2 > 0) as u32);
            p(0x52A920, if case / 2 == 2 { 2 } else { 5 });
            for k in 0..4u32 {
                unsafe { ((bl + 0xFD0 + 4 * k) as *mut u32).write(0x100 + k) };
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &[(a(WORDS), 16), (bl + 0xFD0, 16)],
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
