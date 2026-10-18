//! `race::split_colours` (0x45B16A): a colour per split for the panel: `dword_601830`
//! dwords at edi set to 0xC, then for each of the block esi's +0x90 values at +0x140, 8 when
//! it is not above the one before (0x63 before the first), else 4.
//!
//! In:  esi = a view block, edi = the colours (`dword_6D160C` / `dword_6D170C`, 0x40 dwords)
//! Out: ecx = +0x90; eax the last colour and edx the last value when there was one (flags
//!      not read: `mov` / `retn` follow)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45B147` (0x45B153, 0x45B164), once per view.
//! The values are `CarBlock` +0x140, a dword per checkpoint (written by `sub_436833`),
//! `check_count` (+0x90) of them; what they hold (places at each split) is a reading. Refuses
//! (before anything) a count past the buffers' 0x40 dwords. Shadow mode: by state (the 0x40
//! dwords at edi). Self-test: a test block and buffer (0xAA), `dword_601830` set and put
//! back, counts 0, negative, 1, 5, 0x40 with rising, falling, equal and far values.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::split_colours", rva(0x45B16A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { split_colours(r.esi, r.edi, game_read(MARK_COUNT_RVA)) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(r.edi, 4 * MARK_BUFFER_DWORDS as usize)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(MARK_COUNT_RVA) };
    let mut block = vec![0u32; 0x400];
    let mut colours = vec![0u32; 0x48];
    let b = block.as_mut_ptr() as u32;
    let c = colours.as_mut_ptr() as u32;
    let regions: [Region; 2] = [(b, 0x1000), (c, 0x120)];
    let lists: [&[u32]; 6] = [
        &[],
        &[5],
        &[0x63, 0x64, 0x64, 3, 0x8000_0000, 7],
        &[1, 2, 3, 4, 5],
        &[0x7FFF_FFFF, 0x8000_0001, 0, u32::MAX, 0x63],
        &[9; 0x40],
    ];
    for (fill, count, values) in [
        (0u32, 0u32, lists[0]),
        (0x40, 0, lists[0]),
        (0x20, 0xFFFF_FFFF, lists[0]),
        (0x10, 1, lists[1]),
        (0x40, 6, lists[2]),
        (3, 5, lists[3]),
        (0x40, 5, lists[4]),
        (0x40, 0x40, lists[5]),
    ] {
        check_state(
            &mut t,
            &format!("fill {fill:#x} count {count:#x} values {values:x?}"),
            call_original,
            Regs {
                esi: b,
                edi: c,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                std::ptr::write_bytes(b as *mut u8, 0xAA, 0x1000);
                std::ptr::write_bytes(c as *mut u8, 0xAA, 0x120);
                ((b + 0x90) as *mut u32).write_unaligned(count);
                for (k, &v) in values.iter().enumerate() {
                    ((b + 0x140 + 4 * k as u32) as *mut u32).write_unaligned(v);
                }
                game_write(MARK_COUNT_RVA, fill);
            },
            &regions,
            &rust,
        );
    }
    unsafe { game_write(MARK_COUNT_RVA, saved) };
    let refused = unsafe { split_colours(b, c, 0x41) }.is_err();
    t.check(refused, || "fill 0x41 not refused".into());
    drop((block, colours));
    t.finish()
}
