//! `sound::channels_in_use` (0x409BA1): `cmp g_channelsUsed, 0`: the zero flag set when
//! no channel is in use.
//!
//! In:  nothing
//! Out: the flags of the compare (carry clear, zero as said); every register preserved
//! One caller, `sound::send_command_7`, whose `popa` / `retn` do not look at them.
//! Shadow mode: both, the two flags compared.
//! Self-test: a copy whose variable is the test's, against the Rust version.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::channels_in_use", rva(0x409BA1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// g_channelsUsed.
const CHANNELS_RVA: usize = rva(0x506350);
const ENTRY: [u8; 7] = [0x83, 0x3D, 0x50, 0x63, 0x50, 0x00, 0x00];

// ---- Rust version ----

/// The zero flag: no channel in use.
pub fn channels_in_use(channels: u32) -> bool {
    channels == 0
}

fn zero(r: &Regs) -> bool {
    r.eflags & 0x40 != 0
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let channels: u32 = unsafe { game_read(CHANNELS_RVA) };
    if shadow() {
        let entry = *r;
        unsafe { call_original(r) };
        let mine = channels_in_use(channels);
        if (zero(r), r.carry()) != (mine, false) || !r.preserved_except(&entry, 0) {
            HOOK.mismatch(|| {
                format!(
                    "channels {channels:#x}: original zero {}, rust {mine}",
                    zero(r)
                )
            });
        }
        return;
    }
    r.set_zero(channels_in_use(channels));
    r.set_carry(false);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe { RedirectedCopy::new(game_addr(rva(0x409BA1)), 8, &ENTRY, &[], &|_| None) };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut var = 0u32;
    unsafe { copy.patch_u32(2, &raw mut var as u32) };
    for channels in [0u32, 1, 8, 0x8000_0000] {
        let mut o = Regs::SENTINEL;
        unsafe {
            (&raw mut var).write_volatile(channels);
            call_game(&mut o, copy.addr());
        }
        let mine = channels_in_use(channels);
        t.check(
            zero(&o) == mine && !o.carry() && o.preserved_except(&Regs::SENTINEL, 0),
            || {
                format!(
                    "channels {channels:#x}: original zero {}, carry {}; rust {mine}",
                    zero(&o),
                    o.carry()
                )
            },
        );
    }
    t.finish()
}
