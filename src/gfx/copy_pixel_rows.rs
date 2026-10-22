//! `gfx::copy_pixel_rows` (0x407540): the picture converter of the pixel formats that need
//! no conversion (numbers 5 and 6 in the converter table): copies rows of dwords.
//!
//! In:  esi = from, edi = to, ecx = dwords in a row, edx = rows, ebp = what is taken off
//!      edi after each row (so the rows on the destination side lie `4 * ecx - ebp` apart)
//! Out: with esi equal to edi nothing happens and nothing changes. Otherwise ebx = ecx as
//!      it came, edx = 0, esi and edi moved on past the rows; eax, ecx, ebp preserved. The
//!      flags are not reproduced.
//! Reached only through the converter table `dword_4F5288`: `gfx::convert_picture` calls
//! it with esi = edi (the picture is already what the surface wants: on the owner's 32-bit
//! screen that is every picture), `sub_41A828` with a locked texture as the destination.
//! Manual patch with a private copy (0x10 bytes, nothing to relocate): a conditional jump
//! sits in the first bytes.
//! Refuses a row count of 0 when something would be copied (the original counts through
//! all 32 bits). The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, EDI, EDX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels, pixels_entry};
use crate::screens::harness::Left;
use crate::service::harness::Region;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub const COPY_PIXEL_ROWS: usize = rva(0x407540);

pub static HOOK: RawHook = RawHook::new("gfx::copy_pixel_rows", COPY_PIXEL_ROWS, thunk, &ORIGINAL)
    .private_copy(0x10)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Where the two sides end up: (esi, edi). None if nothing was to be done.
pub unsafe fn copy_pixel_rows(
    from: u32,
    to: u32,
    row_dwords: u32,
    rows: u32,
    back: u32,
) -> Result<Option<(u32, u32)>, BadInput> {
    if from == to {
        return Ok(None);
    }
    if rows == 0 {
        return Err(BadInput("no rows to copy"));
    }
    let (mut from, mut to) = (from, to);
    for _ in 0..rows {
        // Dword by dword and forwards, as `rep movsd` goes, whatever overlaps.
        for _ in 0..row_dwords {
            unsafe {
                (to as *mut u32).write_unaligned((from as *const u32).read_unaligned());
            }
            from = from.wrapping_add(4);
            to = to.wrapping_add(4);
        }
        to = to.wrapping_sub(back);
    }
    Ok(Some((from, to)))
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(
        match unsafe { copy_pixel_rows(r.esi, r.edi, r.ecx, r.edx, r.ebp) }? {
            None => Left::nothing(),
            Some((from, to)) => Left::nothing()
                .with(EBX, r.ecx)
                .with(EDX, 0)
                .with(ESI, from)
                .with(EDI, to),
        },
    )
}

/// The destination's rows, from the first byte written to the last, for comparing. Empty
/// when nothing is copied or the numbers make no sense as a block of memory.
pub fn written(r: &Regs, bytes_per_dword: u32) -> Vec<Region> {
    if r.esi == r.edi || r.edx == 0 || r.ecx == 0 {
        return Vec::new();
    }
    let row_bytes = r.ecx as i64 * bytes_per_dword as i64;
    let step = row_bytes - r.ebp as i32 as i64;
    let last = step * (r.edx as i64 - 1);
    let (low, high) = (last.min(0), last.max(0) + row_bytes);
    if high - low > 0x400_0000 {
        return Vec::new();
    }
    vec![((r.edi as i64 + low) as u32, (high - low) as usize)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_pixels(&HOOK, r, call_original, &written(r, 4), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let from: Vec<u32> = (0..64u32)
        .map(|n| n.wrapping_mul(0x0101_0305) ^ 0x8000_0001)
        .collect();
    // (dwords in a row, rows, back): rows side by side, apart, and going backwards.
    for (row_dwords, rows, back) in [
        (1u32, 1u32, 0u32),
        (5, 3, 0),
        (4, 4, (-8i32) as u32),
        (3, 4, 24),
        (0, 3, (-4i32) as u32),
        (6, 1, 100),
    ] {
        let to = std::cell::RefCell::new(vec![0xAAAA_AAAAu32; 96]);
        let start = to.borrow_mut().as_mut_ptr() as u32 + 32 * 4;
        let before = Regs {
            esi: from.as_ptr() as u32,
            edi: start,
            ecx: row_dwords,
            edx: rows,
            ebp: back,
            ..Regs::SENTINEL
        };
        let whole = [(to.borrow().as_ptr() as u32, 96 * 4)];
        check_pixels(
            &mut t,
            &format!("row_dwords={row_dwords} rows={rows} back={}", back as i32),
            original,
            before,
            &|| to.borrow_mut().fill(0xAAAA_AAAA),
            &whole,
            &rust,
        );
    }
    // From and to the same place: nothing at all.
    let mut same = [1u32, 2, 3, 4];
    let before = Regs {
        esi: same.as_mut_ptr() as u32,
        edi: same.as_mut_ptr() as u32,
        ecx: 2,
        edx: 2,
        ebp: 0,
        ..Regs::SENTINEL
    };
    check_pixels(
        &mut t,
        "from and to the same place",
        original,
        before,
        &|| {},
        &[(same.as_ptr() as u32, 16)],
        &rust,
    );
    t.finish()
}
