//! `saves::save_best_times` (0x4AB236): writes the best times and scores, `var\hst.dat`.
//!
//! In:  nothing
//! Out: every register preserved; carry = not written
//! Only when the tables' memory is taken and they are filled (`g_bestTimesAllocated` and
//! `g_bestTimesLoaded` both 1). A head of 0x10 bytes is made in the scratch block at
//! 0x93E8B0 ("R5HS", 0, 0x50004, the time); the file is opened (mode 1: made anew); the
//! head and the six tables `saves::load_best_times` reads are written, each from the
//! address and with the size its pair of variables holds; the file is closed. Whether a
//! write worked is not looked at: only a file that cannot be opened gives the carry.
//! One caller, `saves::save_game_data`.
//! Shadow mode writes the file twice (Rust, then the original) and compares the result.
//! Self-test: a redirected copy whose tables, switches and path are the test's.
//! Refuses what `resolve_path` refuses.

use crate::errors::Refused;
use crate::files::handle::{close, open_by_mode, write};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::redirect::RedirectedCopy;
use crate::saves::allocate_best_times::{VARS_RVA, pair};
use crate::service::harness::{capture, restore};
use crate::settings::TEXT_RVA;
use crate::system::packed_local_time::packed_local_time;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::save_best_times", rva(0x4AB236), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x5295E4); // "var\hst.dat"
const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const WRITTEN: [usize; 6] = [1, 2, 3, 4, 7, 8];

// ---- Rust version ----

/// What the function is given: the game's own in the hook, the test's in the self-test.
pub struct Places<'a> {
    pub path: *const u8,
    /// `g_bestTimesAllocated` and `g_bestTimesLoaded`.
    pub switches: [u32; 2],
    /// The address and the size of the `n`th table of the file.
    pub table: &'a dyn Fn(usize) -> (u32, u32),
}

/// True if the file was opened (and so, as far as the function knows, written).
pub unsafe fn save_best_times(places: &Places) -> Result<bool, Refused> {
    unsafe {
        if places.switches != [1, 1] {
            return Ok(false);
        }
        // Converted: `rc2k_core::saves::kept::head`.
        let head =
            rc2k_core::saves::kept::head(rc2k_core::saves::kept::BEST_TIMES, packed_local_time());
        game_write(TEXT_RVA, head);
        let Some(file) = open_by_mode(places.path, 1)? else {
            return Ok(false);
        };
        write(file, game_addr(TEXT_RVA) as *const u8, HEAD_LEN);
        for n in 0..WRITTEN.len() {
            let (from, count) = (places.table)(n);
            write(file, from as *const u8, count);
        }
        close(file);
        Ok(true)
    }
}

// ---- hook ----

fn game_table(n: usize) -> (u32, u32) {
    unsafe {
        let pair = pair(game_addr(VARS_RVA) as *mut u32, WRITTEN[n]);
        (pair.read_unaligned(), pair.add(1).read_unaligned())
    }
}

/// `save_best_times` on the game's own file and tables.
pub fn in_game() -> Result<bool, Refused> {
    unsafe {
        save_best_times(&Places {
            path: game_addr(PATH_RVA) as *const u8,
            switches: [game_read(VARS_RVA), game_read(VARS_RVA + 4)],
            table: &game_table,
        })
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match in_game() {
        Ok(written) => written,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        r.set_carry(!mine);
        if mine {
            r.set_zero(false);
        }
        return;
    }
    let before = *r;
    unsafe { call_original(r) };
    if r.carry() == mine || !r.preserved_except(&before, 0) {
        HOOK.mismatch(|| format!("original carry={}; rust written={mine}", r.carry()));
    }
}

// ---- self-test ----

const LEN: usize = 0xE8;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0x10, 0x76, 0x74, 0x00, 0x01];
/// `zero_memory`, `packed_local_time`, `open_by_mode`, seven `files::write`, `close`.
const CALLS: [usize; 11] = [
    0x26, 0x38, 0x4B, 0x63, 0x76, 0x89, 0x9C, 0xAF, 0xC2, 0xD5, 0xDC,
];
const SWITCH_OPERANDS: [usize; 2] = [0x03, 0x10];
const PATH_OPERAND: usize = 0x42;
/// The operands of each table's `mov ebx, <address>` and `mov ecx, <size>`.
const TABLE_OPERANDS: [(usize, usize); 6] = [
    (0x6C, 0x72),
    (0x7F, 0x85),
    (0x92, 0x98),
    (0xA5, 0xAB),
    (0xB8, 0xBE),
    (0xCB, 0xD1),
];
const TEST_FILES: [&str; 2] = ["var\\rc2k_hook_hst_a.dat", "var\\rc2k_hook_hst_b.dat"];
const TEST_PATHS: [&[u8]; 2] = [b"var\\rc2k_hook_hst_a.dat\0", b"var\\rc2k_hook_hst_b.dat\0"];
const SIZES: [usize; 6] = [0x120, 0x618, 0x40, 0x2345, 1, 0];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x4AB236)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut switches = [0u32; 2];
    let mut tables: Vec<Vec<u8>> = SIZES
        .iter()
        .enumerate()
        .map(|(n, &size)| (0..size).map(|at| (at * 7 + n * 31) as u8).collect())
        .collect();
    let mut pairs: Vec<u32> = tables
        .iter_mut()
        .flat_map(|table| [table.as_mut_ptr() as u32, table.len() as u32])
        .collect();
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named =
            |operand: usize| u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
        let vars = game_addr(VARS_RVA) as *mut u32;
        let expected = TABLE_OPERANDS
            .iter()
            .zip(WRITTEN)
            .all(|(&(from, size), n)| {
                named(from) == pair(vars, n) as u32 && named(size) == pair(vars, n).add(1) as u32
            });
        if !expected
            || named(PATH_OPERAND) != game_addr(PATH_RVA) as u32
            || named(SWITCH_OPERANDS[0]) != vars as u32
            || named(SWITCH_OPERANDS[1]) != vars.add(1) as u32
        {
            return t.skip("the original is not what the test was made from");
        }
        copy.patch_u32(PATH_OPERAND, TEST_PATHS[0].as_ptr() as u32);
        for (n, operand) in SWITCH_OPERANDS.into_iter().enumerate() {
            copy.patch_u32(operand, switches.as_mut_ptr().add(n) as u32);
        }
        for (n, (from, size)) in TABLE_OPERANDS.into_iter().enumerate() {
            copy.patch_u32(from, pairs.as_mut_ptr().add(2 * n) as u32);
            copy.patch_u32(size, pairs.as_mut_ptr().add(2 * n + 1) as u32);
        }
    }
    let head = [(game_addr(TEXT_RVA) as u32, HEAD_LEN as usize)];
    let saved = capture(&head);
    let last = crate::files::stream::LastPath::save();
    for given in [[1u32, 1], [0, 1], [1, 0], [2, 1], [0, 0]] {
        for file in TEST_FILES {
            let _ = std::fs::remove_file(file);
        }
        // The copy reads them through their address.
        unsafe { std::ptr::write_volatile(&raw mut switches, given) };
        let before = Regs::SENTINEL;
        restore(&head, &vec![0xAA; saved.len()]);
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = !ro.carry() && ro.preserved_except(&before, 0);
        let theirs_ok = ro.preserved_except(&before, 0);
        restore(&head, &vec![0xAA; saved.len()]);
        let pairs = &pairs;
        let mine = unsafe {
            save_best_times(&Places {
                path: TEST_PATHS[1].as_ptr(),
                switches: given,
                table: &|n| (pairs[2 * n], pairs[2 * n + 1]),
            })
        };
        let written = TEST_FILES.map(|file| {
            std::fs::read(file).ok().map(|mut bytes| {
                if bytes.len() >= HEAD_LEN as usize {
                    bytes[0xC..0x10].fill(0);
                }
                bytes
            })
        });
        let expected: usize = HEAD_LEN as usize + SIZES.iter().sum::<usize>();
        let right = if given == [1, 1] {
            written[0]
                .as_ref()
                .is_some_and(|file| file.len() == expected)
        } else {
            written[0].is_none()
        };
        t.check(
            theirs_ok && matches!(mine, Ok(m) if m == theirs) && written[0] == written[1] && right,
            || {
                format!(
                    "switches {given:?}: original written={theirs} file of {:?} bytes; rust {:?} file of {:?} bytes; files the same={}",
                    written[0].as_ref().map(Vec::len),
                    mine.as_ref().map_err(|_| "refused"),
                    written[1].as_ref().map(Vec::len),
                    written[0] == written[1],
                )
            },
        );
    }
    restore(&head, &saved);
    last.restore();
    for file in TEST_FILES {
        let _ = std::fs::remove_file(file);
    }
    t.finish()
}
