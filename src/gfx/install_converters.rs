//! `gfx::install_converters` (0x407062): fills the table of picture converters, one for each
//! pixel format number.
//!
//! In:  nothing
//! Out: every register preserved
//! The table `dword_4F5288` has seven entries; a format without one is converted pixel by
//! pixel through `components_to_pixel`. Number 0 never has one. Numbers 5 and 6 get
//! `gfx::copy_pixel_rows`. Number 4 gets `gfx::components_to_4444_rows`, and numbers 1 to 3
//! nothing, unless the processor has MMX (`dword_5183A8` is 1): then 1 to 4 get
//! `sub_4070F9`, `sub_407171`, `sub_4071ED`, `sub_407276`, or with 3DNow as well
//! (`dword_5183B8` is 1) `sub_4072FB`, `sub_407377`, `sub_4073F7`, `sub_407484`. Those eight
//! are not ported.
//! One caller, the game's start (0x4010DF), which may well run before this hook is in
//! place: then the self-test is all that checks it.
//! The name is a reading.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::gfx::components_to_4444_rows::COMPONENTS_TO_4444_ROWS;
use crate::gfx::copy_pixel_rows::COPY_PIXEL_ROWS;
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::install_converters", rva(0x407062), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const CONVERTERS_RVA: usize = rva(0x4F5288); // dword_4F5288
pub const CONVERTER_COUNT: usize = rc2k_core::gfx::converters::CONVERTERS;
const HAS_MMX_RVA: usize = rva(0x5183A8); // dword_5183A8
const HAS_3DNOW_RVA: usize = rva(0x5183B8); // dword_5183B8

/// The converters for formats 1 to 4 with MMX, and with 3DNow as well.
const MMX: [usize; 4] = [rva(0x4070F9), rva(0x407171), rva(0x4071ED), rva(0x407276)];
const AMD: [usize; 4] = [rva(0x4072FB), rva(0x407377), rva(0x4073F7), rva(0x407484)];

// ---- Rust version ----

/// The table for a processor with or without the two instruction sets, as addresses in the
/// game; 0 = no converter.
pub fn converters(has_mmx: u32, has_3dnow: u32) -> [u32; CONVERTER_COUNT] {
    // Converted: `rc2k_core::gfx::converters::converters`.
    use rc2k_core::gfx::converters::Converter;
    rc2k_core::gfx::converters::converters(has_mmx, has_3dnow).map(|entry| {
        let at = match entry {
            Converter::None => return 0,
            Converter::Components4444 => COMPONENTS_TO_4444_ROWS,
            Converter::CopyRows => COPY_PIXEL_ROWS,
            Converter::Mmx(n) => MMX[n as usize - 1],
            Converter::ThreeDNow(n) => AMD[n as usize - 1],
        };
        game_addr(at) as u32
    })
}

pub unsafe fn install_converters() {
    let table = unsafe { converters(game_read(HAS_MMX_RVA), game_read(HAS_3DNOW_RVA)) };
    unsafe { (game_addr(CONVERTERS_RVA) as *mut [u32; CONVERTER_COUNT]).write_unaligned(table) };
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { install_converters() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let table = (game_addr(CONVERTERS_RVA) as u32, CONVERTER_COUNT * 4);
    dispatch_state(&HOOK, r, call_original, &[table], &rust);
}

// ---- self-test: a copy of the original that reads the two processor flags from test
// variables and fills a table of the test's ----

static T_MMX: AtomicU32 = AtomicU32::new(0);
static T_3DNOW: AtomicU32 = AtomicU32::new(0);
static T_TABLE: [AtomicU32; CONVERTER_COUNT] = [const { AtomicU32::new(0) }; CONVERTER_COUNT];

const LEN: usize = 0x97;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x53, 0x51, 0x8D, 0x1D, 0x88, 0x52, 0x4F];
/// `system::zero_memory`, which stays.
const CALLS: [usize; 1] = [0x0E];
/// Where the copy names the table (its start, then the entry each store goes to).
const TABLE_OPERANDS: [usize; 12] = [
    0x05, 0x17, 0x21, 0x2B, 0x3E, 0x48, 0x52, 0x5C, 0x6F, 0x79, 0x83, 0x8D,
];
const MMX_OPERAND: usize = 0x35;
const AMD_OPERAND: usize = 0x66;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe {
        RedirectedCopy::new(game_addr(rva(0x407062)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let code = unsafe { std::slice::from_raw_parts(copy.addr() as *const u8, LEN) }.to_vec();
    let real_table = game_addr(CONVERTERS_RVA) as u32;
    for operand in TABLE_OPERANDS {
        // Each operand is the table's address plus the entry's offset.
        let named = u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
        let offset = named.wrapping_sub(real_table);
        if offset as usize >= CONVERTER_COUNT * 4 {
            return t.skip("the function is not what the operand list was made from");
        }
        unsafe { copy.patch_u32(operand, T_TABLE.as_ptr() as u32 + offset) };
    }
    unsafe {
        copy.patch_u32(MMX_OPERAND, &T_MMX as *const AtomicU32 as u32);
        copy.patch_u32(AMD_OPERAND, &T_3DNOW as *const AtomicU32 as u32);
    }
    for (has_mmx, has_3dnow) in [(0u32, 0u32), (1, 0), (1, 1), (0, 1), (2, 1), (1, 2)] {
        for entry in &T_TABLE {
            entry.store(0xAAAA_AAAA, Ordering::SeqCst);
        }
        T_MMX.store(has_mmx, Ordering::SeqCst);
        T_3DNOW.store(has_3dnow, Ordering::SeqCst);
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs: Vec<u32> = T_TABLE.iter().map(|e| e.load(Ordering::SeqCst)).collect();
        let mine = converters(has_mmx, has_3dnow);
        let preserved = ro.preserved_except(&before, 0);
        t.check(theirs == mine && preserved, || {
            format!(
                "mmx={has_mmx} 3dnow={has_3dnow}: preserved={preserved} original {theirs:x?} rust {mine:x?}"
            )
        });
    }
    t.finish()
}
