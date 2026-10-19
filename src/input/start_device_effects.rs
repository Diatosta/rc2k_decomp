//! `input::start_device_effects` (0x409672): the six effects of the record in esi asked to
//! start (`input::set_effect` with ebx 2, eax 0 .. 5).
//!
//! In:  esi = the device's record
//! Out: eax 6, ebx 2, the rest as `set_effect` keeps them (all)
//!
//! See `input/effects.rs`. Two callers, both in `input::start_effects`. Its calls through
//! `part` (`set_effect`). Shadow mode runs the original alone. Self-test: a copy whose
//! calls go to the noting stand-ins, against the Rust version (`stop_device_effects`
//! shares it).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::{Part, SET_EFFECT, compare, dispatch_effects};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "input::start_device_effects",
    rva(0x409672),
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

/// The six effects told `what` (the loop of both functions).
pub fn all_six(r: &mut Regs, what: u32, part: Part) -> Result<(), Refused> {
    r.eax = 0;
    r.ebx = what;
    loop {
        part(SET_EFFECT, r)?;
        r.eax += 1;
        if r.eax == 6 {
            return Ok(());
        }
    }
}

/// The Rust version.
pub fn start_device_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    all_six(r, 2, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, start_device_effects);
}

/// The test of both: a copy of the original at `address` (0x13 bytes, its call at +7).
pub fn test_both(
    hook: &RawHook,
    address: usize,
    entry: &[u8],
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match copy_with(address, 0x13, entry, &[7], &[SET_EFFECT], &[], &[], 0) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for esi in [0x1234_5678u32, 0] {
        let entry = Regs {
            esi,
            ..Regs::SENTINEL
        };
        compare(
            &mut t,
            &format!("esi {esi:#x}"),
            &copy,
            entry,
            &|| {},
            &[],
            &|r, part| rust(r, part),
            false,
        );
    }
    t.finish()
}

fn self_test() -> (usize, usize) {
    // xor eax, eax; mov ebx, 2
    test_both(
        &HOOK,
        0x409672,
        &[0x33, 0xC0, 0xBB, 0x02, 0x00, 0x00, 0x00],
        start_device_effects,
    )
}
