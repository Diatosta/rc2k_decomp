//! `saves::load_options` (0x4AAB32): reads the game's kept options, `var\ral.dat`.
//!
//! In:  nothing
//! Out: carry = not loaded; every register preserved
//! `g_optionsLoaded` (0x75557D) = 0. Up to 0x10000 bytes of the file into the scratch block
//! at 0x93E8B0; it is taken if it starts "R5CI" and has 0x50004 at +8. From +0x10 on it is
//! copied out piece by piece to the 69 places of `OPTIONS` (0xB656 bytes: the screen
//! modes, the sound's and the network's settings, the four car objects, the record
//! table, the players' names and much else). After the last piece the file must have
//! "####", else fatal error 0x80ACFF01 "ral.dat out of date" (the pieces have been copied
//! by then). Then `saves::rebuild_car_lists`, and `g_optionsLoaded` = 1.
//! A file shorter than that is not noticed as such: what the scratch block held decides.
//! One caller, `sub_411239`. `sub_4AAE57` writes the file, the same pieces the other way.
//! Shadow mode reads twice and compares every place. Self-test: a redirected copy that
//! reads files of the test's and copies the pieces into memory of the test's, so that
//! the game's own options are not touched.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_file::read_file;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::install::stand_ins::{WENT_FATAL, act, fatal_landing, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state};
use crate::settings::{TEXT_ROOM, TEXT_RVA};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::load_options", rva(0x4AAB32), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x5295D8); // "var\ral.dat"
pub const LOADED_RVA: usize = rva(0x75557D); // g_optionsLoaded
const MAGIC: u32 = 0x4943_3552; // "R5CI"
const VERSION: u32 = 0x50004;
const END_MARK: u32 = 0x2323_2323; // "####"
const HEAD_LEN: usize = 0x10;
/// The file's pieces after its head, in order: (where each goes, its length).
#[rustfmt::skip]
pub const OPTIONS: [(u32, usize); 69] = [
    (0x518BB0, 0x4),
    (0x4F4944, 0x4),
    (0x4F4948, 0x4),
    (0x4F50D0, 0x4),
    (0x4F82C4, 0x4),
    (0x4F82C8, 0x4),
    (0x4F82D8, 0x4),
    (0x5180F0, 0x4),
    (0x5180F4, 0x4),
    (0x5180F8, 0x4),
    (0x5180FC, 0x4),
    (0x518100, 0x4),
    (0x52AB90, 0x40),
    (0x52ABD0, 0x4),
    (0x52ABE4, 0x40),
    (0x52AC24, 0x4),
    (0x52AC94, 0x10),
    (0x51EBB0, 0x4),
    (0x527834, 0x4),
    (0x528500, 0x4),
    (0x5BEC08, 0x20),
    (0x5BEC28, 0x4),
    (0x4F66DD, 0x4),
    (0x4F66E1, 0x4),
    (0x4F66E5, 0x4),
    (0x4F66E9, 0x4),
    (0x755A4C, 0x28C),
    (0x755CD8, 0x28C),
    (0x755F64, 0x28C),
    (0x7561F0, 0x28C),
    (0x755684, 0x4),
    (0x755688, 0x4),
    (0x75568C, 0x4),
    (0x755690, 0x4),
    (0x755694, 0x4),
    (0x755698, 0x4),
    (0x75569C, 0x4),
    (0x7556A0, 0x4),
    (0x755640, 0x2C),
    (0x75566C, 0x18),
    (0x755254, 0x4),
    (0x7554C8, 0x4),
    (0x75ACCC, 0x4),
    (0x758800, 0x6),
    (0x755258, 0xA0),
    (0x7552F8, 0x50),
    (0x735576, 0x2A48),
    (0x739642, 0x47C),
    (0x7391C2, 0x47C),
    (0x738D42, 0x47C),
    (0x7388C2, 0x47C),
    (0x738442, 0x47C),
    (0x737FC2, 0x47C),
    (0x739AC2, 0x47C),
    (0x739F42, 0x47C),
    (0x73A3C2, 0x47C),
    (0x73A842, 0x47C),
    (0x73ACC2, 0x47C),
    (0x73B142, 0x47C),
    (0x742CB0, 0xC30),
    (0x7438E0, 0xC30),
    (0x744510, 0xC30),
    (0x745140, 0xC30),
    (0x745D70, 0xC30),
    (0x7469A0, 0xC30),
    (0x755784, 0x20),
    (0x7557A4, 0x20),
    (0x7557C4, 0x20),
    (0x7557E4, 0x20),
];
const PIECES_LEN: usize = 0xB656;
const _: () = {
    let mut total = 0;
    let mut n = 0;
    while n < OPTIONS.len() {
        total += OPTIONS[n].1;
        n += 1;
    }
    assert!(total == PIECES_LEN);
};

// ---- Rust version ----

/// Where the function works: the game's own places, or a test's.
pub struct Places<'a> {
    pub loaded: *mut u32,
    /// Where the piece with this number goes.
    pub piece: &'a dyn Fn(usize) -> *mut u8,
    pub rebuild_car_lists: &'a mut dyn FnMut() -> Result<(), BadInput>,
}

/// True if the options were loaded.
pub unsafe fn load_options(path: *const u8, places: &mut Places) -> Result<bool, Refused> {
    unsafe {
        places.loaded.write_unaligned(0);
        let file = game_addr(TEXT_RVA) as *mut u8;
        let read = read_file(path, file, TEXT_ROOM + 1, 0)?;
        if !read.ok
            || game_read::<u32>(TEXT_RVA) != MAGIC
            || game_read::<u32>(TEXT_RVA + 8) != VERSION
        {
            return Ok(false);
        }
        let mut from = file.add(HEAD_LEN);
        for (n, (_, len)) in OPTIONS.into_iter().enumerate() {
            std::ptr::copy(from, (places.piece)(n), len);
            from = from.add(len);
        }
        if (from as *const u32).read_unaligned() != END_MARK {
            return Err(FatalError {
                code: 0x80AC_FF01,
                message: "ral.dat out of date",
            }
            .into());
        }
        (places.rebuild_car_lists)()?;
        places.loaded.write_unaligned(1);
        Ok(true)
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = OPTIONS
        .iter()
        .map(|&(place, len)| (game_addr(rva(place as usize)) as u32, len))
        .collect();
    regions.extend([
        at(LOADED_RVA),
        (game_addr(rva(0x75A042)) as u32, 0x28 * 4),
        (game_addr(rva(0x75A12E)) as u32, 0x14 * 4),
        (game_addr(TEXT_RVA) as u32, TEXT_ROOM as usize + 1),
    ]);
    regions
}

/// `load_options` on the game's own file and places.
pub unsafe fn load_game_options() -> Result<bool, Refused> {
    unsafe {
        load_options(
            game_addr(PATH_RVA) as *const u8,
            &mut Places {
                loaded: game_addr(LOADED_RVA) as *mut u32,
                piece: &|n| game_addr(rva(OPTIONS[n].0 as usize)) as *mut u8,
                rebuild_car_lists: &mut || rebuild_car_lists(),
            },
        )
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let loaded = unsafe { load_game_options() }?;
    Ok(Left::carry(!loaded))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const LEN: usize = 0x325;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xC7, 0x05, 0x7D, 0x55, 0x75, 0x00, 0x00];
/// `files::read_file`, `files::version_differs`, `saves::rebuild_car_lists`.
const CALLS: [usize; 3] = [0x1E, 0x3A, 0x30F];
const LOADED_OPERANDS: [usize; 2] = [0x03, 0x316];
const PATH_OPERAND: usize = 0x0D;
const FATAL_JUMP_OPERAND: usize = 0x30B;
const REBUILD_RVA: usize = rva(0x47088C);
const TEST_FILE: &str = "var\\rc2k_hook_ral.dat";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_ral.dat\0";

static REBUILDS: AtomicU32 = AtomicU32::new(0);

fn note_rebuild(_: u32, _: &mut Regs) {
    REBUILDS.fetch_add(1, Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(own) = std::fs::read("var\\ral.dat") else {
        return t.skip("the game has no var\\ral.dat to make test files from");
    };
    if own.len() != HEAD_LEN + PIECES_LEN + 4 {
        return t.skip("var\\ral.dat is not the length the test was made for");
    }
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4AAB32)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == REBUILD_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The copy's places become the test's: the switch, the path, and every piece's.
    let mut loaded = 0u32;
    let mut pieces = vec![0u8; PIECES_LEN];
    let piece_offsets: Vec<usize> = OPTIONS
        .iter()
        .scan(0, |offset, &(_, len)| {
            let at = *offset;
            *offset += len;
            Some(at)
        })
        .collect();
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named =
            |operand: usize| u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
        if LOADED_OPERANDS
            .iter()
            .any(|&operand| named(operand) != game_addr(LOADED_RVA) as u32)
            || named(PATH_OPERAND) != game_addr(PATH_RVA) as u32
        {
            return t.skip("the original is not what the test was made from");
        }
        // Each piece's `lea edi, <place>`, in the order of the pieces.
        let mut from = 0;
        let mut operands = Vec::new();
        for &(place, _) in &OPTIONS {
            let wanted = [
                &[0x8D, 0x3D][..],
                &(game_addr(rva(place as usize)) as u32).to_le_bytes(),
            ]
            .concat();
            let Some(found) = code[from..]
                .windows(6)
                .position(|bytes| bytes == &wanted[..])
            else {
                return t.skip("a piece's place is not named in the original as expected");
            };
            operands.push(from + found + 2);
            from += found + 6;
        }
        for operand in LOADED_OPERANDS {
            copy.patch_u32(operand, &mut loaded as *mut u32 as u32);
        }
        copy.patch_u32(PATH_OPERAND, TEST_PATH.as_ptr() as u32);
        for (operand, offset) in operands.into_iter().zip(&piece_offsets) {
            copy.patch_u32(operand, pieces.as_mut_ptr().add(*offset) as u32);
        }
        copy.retarget(FATAL_JUMP_OPERAND, fatal_landing as *const () as usize);
    }

    // Files: the game's own; one with other contents; and ones that are not taken.
    let mut other = own.clone();
    for (n, byte) in other[HEAD_LEN..HEAD_LEN + PIECES_LEN]
        .iter_mut()
        .enumerate()
    {
        *byte = (n as u32).wrapping_mul(2_654_435_761).to_le_bytes()[3];
    }
    let with = |at: usize, byte: u8| {
        let mut file = other.clone();
        file[at] = byte;
        file
    };
    let files: Vec<(&str, Option<Vec<u8>>)> = vec![
        ("the game's own", Some(own.clone())),
        ("other contents", Some(other.clone())),
        ("another first letter", Some(with(0, b'X'))),
        ("another version", Some(with(8, 5))),
        ("no end mark", Some(with(HEAD_LEN + PIECES_LEN + 3, b'!'))),
        ("cut short", Some(other[..0x5000].to_vec())),
        ("only a head", Some(other[..HEAD_LEN].to_vec())),
        ("empty", Some(Vec::new())),
        ("not there", None),
    ];
    let scratch = [(game_addr(TEXT_RVA) as u32, TEXT_ROOM as usize + 1)];
    let saved = capture(&scratch);
    let last = crate::files::stream::LastPath::save();
    let mut made = true;
    for (what, file) in files {
        let _ = std::fs::remove_file(TEST_FILE);
        if let Some(file) = &file
            && std::fs::write(TEST_FILE, file).is_err()
        {
            made = false;
            break;
        }
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            crate::service::harness::restore(&scratch, &vec![0xAA; saved.len()]);
            pieces.fill(0x55);
            loaded = 0xAAAA_AAAA;
            REBUILDS.store(0, Ordering::SeqCst);
            set_handler(note_rebuild);
            let pieces_at = pieces.as_mut_ptr();
            let ended = if rust {
                let result = load_options(
                    TEST_PATH.as_ptr(),
                    &mut Places {
                        loaded: &mut loaded,
                        piece: &|n| pieces_at.add(piece_offsets[n]),
                        rebuild_car_lists: &mut || {
                            act(0, &mut Regs::default());
                            Ok(())
                        },
                    },
                );
                match result {
                    Ok(loaded) => loaded as u32,
                    Err(Refused::Fatal(_)) => 2,
                    Err(Refused::BadInput(_)) => 3,
                }
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                match (
                    WENT_FATAL.load(Ordering::SeqCst),
                    ro.preserved_except(&before, 0),
                ) {
                    (1, _) => 2,
                    (_, true) => !ro.carry() as u32,
                    _ => 4,
                }
            };
            (
                ended,
                loaded,
                REBUILDS.load(Ordering::SeqCst),
                pieces.clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 < 3, || {
            let first = theirs.3.iter().zip(&mine.3).position(|(a, b)| a != b);
            format!(
                "{what}: original ended {} loaded={:#x} rebuilds={}; rust ended {} loaded={:#x} rebuilds={}; first different byte of the pieces {first:?} (ended: 0 not loaded, 1 loaded, 2 fatal)",
                theirs.0, theirs.1, theirs.2, mine.0, mine.1, mine.2
            )
        });
    }
    crate::service::harness::restore(&scratch, &saved);
    last.restore();
    let _ = std::fs::remove_file(TEST_FILE);
    if !made {
        return t.skip("cannot create a test file in the game's var folder");
    }
    t.finish()
}
