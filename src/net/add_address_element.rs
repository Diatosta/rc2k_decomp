//! `net::add_address_element` (0x41896E): one more element of the compound address being
//! built (`unk_52AC48`, 0x18 bytes each, `dword_52AC3C` of them: a DPCOMPOUNDADDRESSELEMENT,
//! the data type's GUID, the data's size, the data).
//!
//! In:  eax = the data type's GUID, ecx = the data's size, ebx = the data
//! Out: edx = the element's offset (number * 0x18); the rest kept; the flags of the count's
//!      `inc` (no caller reads them)
//! No limit on the count. Caller: `net::build_address` (five times; three at most in one
//! go). Shadow mode: both run, the element and the count compared. Self-test: a copy
//! against the Rust version, on counts 0, 1 and 2.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    ELEMENT_TABLE, ELEMENTS, Part, Saved, a, dword, fill, g, p, region, run_both,
};
use rc2k_core::net::address;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::add_address_element", rva(0x41896E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves.
pub fn add_address_element(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::address` (`element_at`, `element`).
    r.edx = address::element_at(g(ELEMENTS));
    p(ELEMENTS, g(ELEMENTS).wrapping_add(1));
    let at = a(ELEMENT_TABLE).wrapping_add(r.edx);
    let data_type = unsafe { (r.eax as *const [u8; 16]).read_unaligned() };
    let bytes = address::element(&data_type, r.ecx, r.ebx);
    unsafe { std::ptr::copy(bytes.as_ptr(), at as *mut u8, bytes.len()) };
    Ok(r)
}

/// What it writes: the count and the next element.
fn regions() -> Vec<crate::service::harness::Region> {
    let n = g(ELEMENTS) as usize;
    vec![
        dword(ELEMENTS),
        region(
            ELEMENT_TABLE + n.wrapping_mul(address::ELEMENT_LEN as usize),
            0x18,
        ),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &regions(), 0, &|r| {
        add_address_element(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // imul edx, dword_52AC3C, 18h
    let copy = match crate::net::lobby::copy(
        0x41896E,
        0x2B,
        &[0x6B, 0x15, 0x3C, 0xAC, 0x52, 0x00, 0x18],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(ELEMENTS), region(ELEMENT_TABLE, 3 * 0x18)];
    let saved = Saved::new(&all);
    let guid: [u32; 4] = [0x0102_0304, 0x0506_0708, 0x090A_0B0C, 0x0D0E_0F10];
    for count in [0u32, 1, 2] {
        run_both(
            &mut t,
            &format!("count {count}"),
            copy.addr(),
            Regs {
                eax: guid.as_ptr() as u32,
                ebx: 0x0BAD_DA7A,
                ecx: 0x10,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                fill(ELEMENT_TABLE, 3 * 0x18, 9);
                p(ELEMENTS, count);
            },
            &add_address_element,
        );
    }
    drop(saved);
    t.finish()
}
