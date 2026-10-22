//! `gfx::components_to_4444_rows` (0x40750D): the plain picture converter for pixel format
//! number 4, two bytes a pixel with four bits for each component.
//!
//! In:  esi = from (four bytes a pixel), edi = to, ecx = pixels in a row, edx = rows, ebp =
//!      what is taken off edi after each row
//! Out: ebx = ecx as it came, edx = 0, esi and edi moved on past the rows, ax = what the
//!      last pixel left there (al its second byte, ah the top four bits of its fourth
//!      component); the rest of eax, ecx and ebp preserved. The flags are not reproduced.
//! A pixel's first byte is the top four bits of its first component in the low half and
//! of its second in the high half; its second byte the same of the third and fourth.
//! Reached only through the converter table `dword_4F5288`, where `gfx::install_converters`
//! puts it when the processor has no MMX; with MMX `sub_407276` stands there instead (not
//! ported). So on a machine of today it is never called, and its self-test is all it has.
//! Manual patch with a private copy (0x33 bytes, nothing to relocate): the loop comes back
//! to the third byte, inside what the patch overwrites.
//! Refuses a row or a row count of 0 (the original counts through all 32 bits). The name
//! is a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, EDI, EDX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::copy_pixel_rows::written;
use crate::gfx::harness::{check_pixels, dispatch_pixels, pixels_entry};
use crate::screens::harness::Left;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub const COMPONENTS_TO_4444_ROWS: usize = rva(0x40750D);

pub static HOOK: RawHook = RawHook::new(
    "gfx::components_to_4444_rows",
    COMPONENTS_TO_4444_ROWS,
    thunk,
    &ORIGINAL,
)
.private_copy(0x33)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Where the two sides end up and what the last pixel leaves in ax: (esi, edi, ax).
pub unsafe fn components_to_4444_rows(
    from: u32,
    to: u32,
    row_pixels: u32,
    rows: u32,
    back: u32,
) -> Result<(u32, u32, u16), BadInput> {
    if row_pixels == 0 || rows == 0 {
        return Err(BadInput("no pixels to convert"));
    }
    let (mut from, mut to, mut ax) = (from, to, 0u16);
    for _ in 0..rows {
        for _ in 0..row_pixels {
            unsafe {
                // Converted: `rc2k_core::gfx::pixels::to_4444`.
                let components = (from as *const [u8; 4]).read_unaligned();
                let [low, high] = rc2k_core::gfx::pixels::to_4444(components);
                (to as *mut u8).write(low);
                (to.wrapping_add(1) as *mut u8).write(high);
                ax = u16::from_le_bytes([high, components[3] & 0xF0]);
            }
            from = from.wrapping_add(4);
            to = to.wrapping_add(2);
        }
        to = to.wrapping_sub(back);
    }
    Ok((from, to, ax))
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (from, to, ax) = unsafe { components_to_4444_rows(r.esi, r.edi, r.ecx, r.edx, r.ebp) }?;
    Ok(Left::nothing()
        .with(EAX, r.eax & 0xFFFF_0000 | ax as u32)
        .with(EBX, r.ecx)
        .with(EDX, 0)
        .with(ESI, from)
        .with(EDI, to))
}

extern "C" fn dispatch(r: &mut Regs) {
    // `written` leaves the case of esi = edi out, which here converts in place.
    let regions = if r.esi == r.edi && r.ecx != 0 && r.edx != 0 && r.ebp == 0 {
        vec![(
            r.edi,
            (r.ecx as usize)
                .saturating_mul(r.edx as usize)
                .saturating_mul(2)
                .min(0x400_0000),
        )]
    } else {
        written(r, 2)
    };
    dispatch_pixels(&HOOK, r, call_original, &regions, &rust);
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
    let from: Vec<u8> = (0..256u32).map(|n| (n * 37 + 11) as u8).collect();
    for (row_pixels, rows, back) in [
        (1u32, 1u32, 0u32),
        (7, 3, 0),
        (4, 4, (-6i32) as u32),
        (3, 4, 12),
        (16, 1, 50),
    ] {
        let to = std::cell::RefCell::new(vec![0xAAu8; 256]);
        let start = to.borrow_mut().as_mut_ptr() as u32 + 96;
        let before = Regs {
            esi: from.as_ptr() as u32,
            edi: start,
            ecx: row_pixels,
            edx: rows,
            ebp: back,
            ..Regs::SENTINEL
        };
        let whole = [(to.borrow().as_ptr() as u32, 256)];
        check_pixels(
            &mut t,
            &format!("row_pixels={row_pixels} rows={rows} back={}", back as i32),
            original,
            before,
            &|| to.borrow_mut().fill(0xAA),
            &whole,
            &rust,
        );
    }
    // In place, as `convert_picture` calls it.
    let picture = std::cell::RefCell::new(from.clone());
    let at = picture.borrow_mut().as_mut_ptr() as u32;
    let before = Regs {
        esi: at,
        edi: at,
        ecx: 60,
        edx: 1,
        ebp: 0,
        ..Regs::SENTINEL
    };
    check_pixels(
        &mut t,
        "60 pixels in place",
        original,
        before,
        &|| picture.borrow_mut().copy_from_slice(&from),
        &[(at, 256)],
        &rust,
    );
    t.finish()
}
