//! `saves::load_best_times` (0x4AB155): reads the best times and scores, `var\hst.dat`.
//!
//! In:  nothing
//! Out: carry = not loaded (loaded: carry and zero clear); every register preserved
//! `saves::allocate_best_times`; the file is opened (mode 0); its head of 0x10 bytes goes
//! to the scratch block at 0x93E8B0 and must start "R5HS" and have 0x50004 at +8; then six
//! of the tables are read straight from the file, each to the address and with the size
//! its pair of variables holds (tables 1, 2, 3, 4, 7 and 8 of `allocate_best_times`: 0xBB8
//! bytes, an entry of 0x618 a stage, 0x15E0, 0xDB60, 0xDB60, an entry a stage again); the
//! file is closed and `dword_747614` = 1.
//! A read that fails or gives fewer bytes than asked for (`sub_4AB21C`, inlined) closes
//! the file and ends it there, with what was read so far left in the tables. A head that
//! is not the right one ends it without the file being closed: the original's slip, kept.
//! One caller, `sub_411239`.
//! Shadow mode runs the original alone and then reads the file again into memory of its
//! own, and compares that with the tables. Manual patch with a private copy (0xE1 bytes,
//! `sub_4AB21C` with it, six calls relocated): a call starts at the fourth byte.
//! Self-test: a redirected copy whose tables, switch and path are the test's.
//! Refuses what `resolve_path` and `allocate_best_times` refuse.

use crate::errors::{BadInput, Refused};
use crate::files::handle::{close, open_by_mode, read};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::memory::MemoryList;
use crate::redirect::RedirectedCopy;
use crate::saves::allocate_best_times::{VARS_RVA, allocate_best_times, pair};
use crate::service::harness::{capture, restore};
use crate::settings::TEXT_RVA;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("saves::load_best_times", rva(0x4AB155), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x5295E4); // "var\hst.dat"
const LOADED_RVA: usize = rva(0x747614); // dword_747614
const LIST_RVA: usize = rva(0x7475D0); // stru_7475D0
const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const MAGIC: u32 = 0x5348_3552; // "R5HS"
const VERSION: u32 = 0x50004;
const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const READ: [usize; 6] = [1, 2, 3, 4, 7, 8];

// ---- Rust version ----

/// What the function is given: the game's own in the hook, the test's in the self-test.
pub struct Places<'a> {
    pub path: *const u8,
    /// `saves::allocate_best_times`.
    pub allocate: &'a mut dyn FnMut() -> Result<(), Refused>,
    /// `files::open_by_mode` with mode 0.
    pub open: &'a mut dyn FnMut(*const u8) -> Result<Option<u32>, BadInput>,
    /// The address and the size of the `n`th table of the file, as they are after
    /// `allocate`.
    pub table: &'a dyn Fn(usize) -> (u32, u32),
    pub loaded: *mut u32,
}

/// Reads exactly `count` bytes, or closes the file and says false.
unsafe fn read_or_close(file: u32, to: *mut u8, count: u32) -> bool {
    unsafe {
        let (got, ok) = read(file, to, count);
        if !ok || got != count {
            close(file);
            return false;
        }
        true
    }
}

/// True if the file was loaded.
pub unsafe fn load_best_times(places: &mut Places) -> Result<bool, Refused> {
    unsafe {
        (places.allocate)()?;
        let Some(file) = (places.open)(places.path)? else {
            return Ok(false);
        };
        if !read_or_close(file, game_addr(TEXT_RVA) as *mut u8, HEAD_LEN) {
            return Ok(false);
        }
        // Converted: `rc2k_core::saves::kept::head_ok`.
        let head = std::slice::from_raw_parts(game_addr(TEXT_RVA) as *const u8, HEAD_LEN as usize);
        if !rc2k_core::saves::kept::head_ok(head, rc2k_core::saves::kept::BEST_TIMES) {
            // The file stays open, as in the original.
            return Ok(false);
        }
        for n in 0..READ.len() {
            let (to, count) = (places.table)(n);
            if !read_or_close(file, to as *mut u8, count) {
                return Ok(false);
            }
        }
        close(file);
        places.loaded.write_unaligned(1);
        Ok(true)
    }
}

// ---- hook ----

fn game_table(n: usize) -> (u32, u32) {
    unsafe {
        let pair = pair(game_addr(VARS_RVA) as *mut u32, READ[n]);
        (pair.read_unaligned(), pair.add(1).read_unaligned())
    }
}

/// `load_best_times` on the game's own file and tables.
pub unsafe fn load_game_best_times() -> Result<bool, Refused> {
    unsafe {
        load_best_times(&mut Places {
            path: game_addr(PATH_RVA) as *const u8,
            allocate: &mut || {
                allocate_best_times(
                    game_addr(VARS_RVA) as *mut u32,
                    game_addr(LIST_RVA) as *mut MemoryList,
                    game_addr(RECORD_TABLE_RVA) as *const u8,
                )
            },
            open: &mut |path| open_by_mode(path, 0),
            table: &game_table,
            loaded: game_addr(LOADED_RVA) as *mut u32,
        })
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let path = game_addr(PATH_RVA) as *const u8;
    if !shadow() {
        match unsafe { load_game_best_times() } {
            Ok(loaded) => {
                r.set_carry(!loaded);
                if loaded {
                    r.set_zero(false);
                }
            }
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }

    // The original for real; then the file once more, into memory of our own.
    let before = *r;
    let loaded_before: u32 = unsafe { game_read(LOADED_RVA) };
    unsafe { call_original(r) };
    let theirs: Vec<(u32, u32)> = (0..READ.len()).map(game_table).collect();
    if theirs
        .iter()
        .any(|&(to, count)| to == 0 || count > 0x100_0000)
    {
        HOOK.mismatch(|| format!("the original left the tables at {theirs:x?}"));
        return;
    }
    let head = [(game_addr(TEXT_RVA) as u32, HEAD_LEN as usize)];
    let head_theirs = capture(&head);
    let last = crate::files::stream::LastPath::save();
    let mut mine: Vec<Vec<u8>> = theirs
        .iter()
        .map(|&(to, count)| unsafe {
            std::slice::from_raw_parts(to as *const u8, count as usize).to_vec()
        })
        .collect();
    let unread = mine.clone();
    let mut loaded = loaded_before;
    let places: Vec<(u32, u32)> = mine
        .iter_mut()
        .map(|table| (table.as_mut_ptr() as u32, table.len() as u32))
        .collect();
    let result = unsafe {
        load_best_times(&mut Places {
            path,
            allocate: &mut || Ok(()),
            open: &mut |path| open_by_mode(path, 0),
            table: &|n| places[n],
            loaded: &mut loaded,
        })
    };
    let head_mine = capture(&head);
    restore(&head, &head_theirs);
    last.restore();
    let ok = matches!(result, Ok(l) if l != r.carry())
        && r.preserved_except(&before, 0)
        && loaded == unsafe { game_read::<u32>(LOADED_RVA) }
        && head_mine == head_theirs
        && mine == unread;
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "original carry={} loaded={:#x}; rust {:?} loaded={loaded:#x}; same head={}; first table that differs {:?}",
                r.carry(),
                unsafe { game_read::<u32>(LOADED_RVA) },
                result.as_ref().map_err(|_| "refused"),
                head_mine == head_theirs,
                mine.iter().zip(&unread).position(|(a, b)| a != b),
            )
        });
    }
}

// ---- self-test ----

const LEN: usize = 0xE1;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0xEC, 0xE8, 0x7B, 0x13, 0xFD, 0xFF];
/// `allocate_best_times`, `files::open_by_mode`, `files::version_differs`, `files::close`,
/// and in `sub_4AB21C` `files::read` and `files::close`.
const CALLS: [usize; 6] = [0x03, 0x13, 0x41, 0xB4, 0xC9, 0xD7];
const PATH_OPERAND: usize = 0x0A;
const LOADED_OPERAND: usize = 0xBB;
/// The operands of each table's `mov ebx, <address>` and `mov ecx, <size>`.
const TABLE_OPERANDS: [(usize, usize); 6] = [
    (0x4E, 0x54),
    (0x5F, 0x65),
    (0x70, 0x76),
    (0x81, 0x87),
    (0x92, 0x98),
    (0xA3, 0xA9),
];
const ALLOCATE_RVA: usize = rva(0x47C4D8);
const OPEN_RVA: usize = rva(0x402DE8);
const TEST_FILE: &str = "var\\rc2k_hook_hst.dat";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_hst.dat\0";
/// The sizes of the test's six tables.
const SIZES: [usize; 6] = [0x120, 0x618, 0x40, 0x2345, 1, 0x618];

static ALLOCATIONS: AtomicU32 = AtomicU32::new(0);
/// The file a run opened last, or -1.
static OPENED: AtomicU32 = AtomicU32::new(u32::MAX);

/// Place 0 is `allocate_best_times`, which is only counted; place 1 is the opening of the
/// file, which is done and noted.
fn handler(place: u32, r: &mut Regs) {
    if place == 0 {
        ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        return;
    }
    let file = unsafe { open_by_mode(r.eax as *const u8, r.ebx) }
        .ok()
        .flatten();
    OPENED.store(file.unwrap_or(u32::MAX), Ordering::SeqCst);
    r.eax = file.unwrap_or(u32::MAX);
    r.set_carry(file.is_none());
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4AB155)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| match callee {
                ALLOCATE_RVA => Some(stand_in(0)),
                OPEN_RVA => Some(stand_in(1)),
                _ => None,
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut loaded = 0u32;
    let mut tables: Vec<Vec<u8>> = SIZES.iter().map(|&size| vec![0u8; size]).collect();
    // What the copy reads each table's address and size from.
    let mut pairs: Vec<u32> = tables
        .iter_mut()
        .flat_map(|table| [table.as_mut_ptr() as u32, table.len() as u32])
        .collect();
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named =
            |operand: usize| u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
        let vars = game_addr(VARS_RVA) as *mut u32;
        let expected = TABLE_OPERANDS.iter().zip(READ).all(|(&(to, size), n)| {
            named(to) == pair(vars, n) as u32 && named(size) == pair(vars, n).add(1) as u32
        });
        if !expected
            || named(PATH_OPERAND) != game_addr(PATH_RVA) as u32
            || named(LOADED_OPERAND) != game_addr(LOADED_RVA) as u32
        {
            return t.skip("the original is not what the test was made from");
        }
        copy.patch_u32(PATH_OPERAND, TEST_PATH.as_ptr() as u32);
        copy.patch_u32(LOADED_OPERAND, &mut loaded as *mut u32 as u32);
        for (n, (to, size)) in TABLE_OPERANDS.into_iter().enumerate() {
            copy.patch_u32(to, pairs.as_mut_ptr().add(2 * n) as u32);
            copy.patch_u32(size, pairs.as_mut_ptr().add(2 * n + 1) as u32);
        }
    }

    // A whole file, and files that are not taken or end early.
    let body: usize = SIZES.iter().sum();
    let mut whole = Vec::new();
    whole.extend(MAGIC.to_le_bytes());
    whole.extend(0u32.to_le_bytes());
    whole.extend(VERSION.to_le_bytes());
    whole.extend(0x1234_5678u32.to_le_bytes());
    whole.extend((0..body).map(|n| (n as u32).wrapping_mul(2_654_435_761).to_le_bytes()[3]));
    let with = |at: usize, byte: u8| {
        let mut file = whole.clone();
        file[at] = byte;
        file
    };
    let longer = [&whole[..], &[7u8; 9][..]].concat();
    // (what, the file, whether the head is refused, which leaves the file open)
    let files: Vec<(&str, Option<Vec<u8>>, bool)> = vec![
        ("whole", Some(whole.clone()), false),
        ("with more after it", Some(longer), false),
        ("another first letter", Some(with(0, b'X')), true),
        ("another version", Some(with(8, 5)), true),
        (
            "one byte short",
            Some(whole[..whole.len() - 1].to_vec()),
            false,
        ),
        (
            "ends in the fourth table",
            Some(whole[..0x900].to_vec()),
            false,
        ),
        ("only a head", Some(whole[..0x10].to_vec()), false),
        ("half a head", Some(whole[..8].to_vec()), false),
        ("empty", Some(Vec::new()), false),
        ("not there", None, false),
    ];
    let head = [(game_addr(TEXT_RVA) as u32, HEAD_LEN as usize)];
    let saved = capture(&head);
    let last = crate::files::stream::LastPath::save();
    let mut made = true;
    for (what, file, stays_open) in files {
        let _ = std::fs::remove_file(TEST_FILE);
        if let Some(file) = &file
            && std::fs::write(TEST_FILE, file).is_err()
        {
            made = false;
            break;
        }
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            restore(&head, &vec![0xAA; saved.len()]);
            for table in tables.iter_mut() {
                table.fill(0x55);
            }
            loaded = 0xAAAA_AAAA;
            ALLOCATIONS.store(0, Ordering::SeqCst);
            OPENED.store(u32::MAX, Ordering::SeqCst);
            set_handler(handler);
            let ended = if rust {
                let pairs = &pairs;
                let result = load_best_times(&mut Places {
                    path: TEST_PATH.as_ptr(),
                    allocate: &mut || {
                        act(0, &mut Regs::default());
                        Ok(())
                    },
                    open: &mut |path| {
                        let mut r = Regs {
                            eax: path as u32,
                            ..Regs::default()
                        };
                        act(1, &mut r);
                        Ok((!r.carry()).then_some(r.eax))
                    },
                    table: &|n| (pairs[2 * n], pairs[2 * n + 1]),
                    loaded: &mut loaded,
                });
                match result {
                    Ok(loaded) => loaded as u32,
                    Err(_) => 3,
                }
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                if !ro.preserved_except(&before, 0) || (!ro.carry() && ro.eflags & 0x40 != 0) {
                    4
                } else {
                    !ro.carry() as u32
                }
            };
            // A refused head leaves the file open: closed here, so that it can be replaced.
            let opened = OPENED.load(Ordering::SeqCst);
            if stays_open && opened != u32::MAX {
                close(opened);
            }
            (
                ended,
                loaded,
                ALLOCATIONS.load(Ordering::SeqCst),
                opened != u32::MAX,
                capture(&head),
                tables.clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 < 3, || {
            format!(
                "{what}: original ended {} loaded={:#x} allocations={} opened={}; rust ended {} loaded={:#x} allocations={} opened={}; same head={}; first table that differs {:?} (ended: 0 not loaded, 1 loaded)",
                theirs.0,
                theirs.1,
                theirs.2,
                theirs.3,
                mine.0,
                mine.1,
                mine.2,
                mine.3,
                theirs.4 == mine.4,
                theirs.5.iter().zip(&mine.5).position(|(a, b)| a != b),
            )
        });
    }
    restore(&head, &saved);
    last.restore();
    let _ = std::fs::remove_file(TEST_FILE);
    if !made {
        return t.skip("cannot create a test file in the game's var folder");
    }
    t.finish()
}
