//! `install::check_clio_disc` (0x416653): the Clio update.
//!
//! In:  nothing
//! Out: every register preserved; or it does not return (fatal error 0x90002105)
//! `g_hasClio` (0x529DB4) = 1 if `files\b\obj0570.ro3` is there (`sub_47C63F` reads it:
//! the extra car), and nothing at all if not. The rest is for a start from the update's
//! disc: only if the installer is not running, there is no `var\ral.zog` here, the
//! registry records an installation (status 1, a source) which is not where the game was
//! started (`install::runs_from_source`), and `files\d\gfx\install.bun` is not here. Then
//! base directory B (`unk_5295B4`) goes through `normalise_path` to `byte_95E8F8`, the
//! installation's source is made the current directory, B goes again to `byte_95E9F8`,
//! and the installation's record is read. An installation below "normal" (`g_mediaMode`
//! under 2) is "CLIO: Requires normal or full installation" in the log and the fatal
//! error. Otherwise the installer's script is set up (as in `check_at_start`, with
//! string 0x1038), the car's name `aRenaultClioSpo` is put in capitals up to its `(`,
//! where a line feed ends it, and `off_51DD48` becomes `sub_41675B`.
//! One caller, `start`.
//! Self-test: a redirected copy in worlds of the test's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::install::check_at_start::{
    INSTALLING_RVA, RECORD_RVA, SCRIPT_ENTRY_RVA, set_script_entry,
};
use crate::install::check_test::{self, MEDIA_MODE_RVA, World};
use crate::install::normalise_path::current_directory;
use crate::install::outside::{
    FILE_EXISTS, Game, LOG, Outside, READ_RECORD, SET_DIRECTORY, Tested,
};
use crate::install::read_registry::{SOURCE_RVA, STATUS_RVA};
use crate::install::runs_from_source::{SCRATCH_PATH_RVA, copy_asking_the_test, runs_from_source};
use crate::install::stand_ins::{WENT_FATAL, fatal_landing, stand_in};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::check_clio_disc", rva(0x416653), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CLIO_FILE_RVA: usize = rva(0x529DB8); // "files\b\obj0570.ro3"
const HAS_CLIO_RVA: usize = rva(0x529DB4); // g_hasClio
const INSTALLER_DATA_RVA: usize = rva(0x51BEA1); // "files\d\gfx\install.bun"
const BASE_B_RVA: usize = rva(0x5295B4); // unk_5295B4
const B_BEFORE_RVA: usize = rva(0x95E8F8); // byte_95E8F8
const B_AFTER_RVA: usize = rva(0x95E9F8); // byte_95E9F8
const PATH_ROOM: usize = 0x100;
const CAR_NAME_RVA: usize = rva(0x529F03); // "renault clio sport V6 (A7)"
const CAR_NAME_LEN: usize = 0x1B;
const STEP_RVA: usize = rva(0x51DD48); // off_51DD48
const CLIO_STEP_RVA: usize = rva(0x41675B); // sub_41675B

// ---- Rust version ----

pub unsafe fn check_clio_disc(outside: &mut dyn Outside) -> Result<(), Refused> {
    unsafe {
        if !outside.file_exists(game_addr(CLIO_FILE_RVA) as *const u8)? {
            return Ok(());
        }
        game_write(HAS_CLIO_RVA, 1u32);
        if game_read::<u32>(INSTALLING_RVA) == 1
            || outside.file_exists(game_addr(RECORD_RVA) as *const u8)?
            || runs_from_source(outside)?
            || game_read::<u8>(STATUS_RVA) != 1
            || game_read::<u8>(SOURCE_RVA) == 0
            || outside.file_exists(game_addr(INSTALLER_DATA_RVA) as *const u8)?
        {
            return Ok(());
        }
        let base_b = game_addr(BASE_B_RVA) as *const u8;
        if !outside.normalise_path(base_b, game_addr(B_BEFORE_RVA) as *mut u8)? {
            return Ok(());
        }
        outside.set_directory(game_addr(SOURCE_RVA) as *const u8);
        if !outside.normalise_path(base_b, game_addr(B_AFTER_RVA) as *mut u8)? {
            return Ok(());
        }
        outside.read_record()?;
        if game_read::<u32>(MEDIA_MODE_RVA) < 2 {
            outside.log(
                c"CLIO: Requires normal or full installation"
                    .as_ptr()
                    .cast(),
            );
            return Err(FatalError {
                code: 0x9000_2105,
                message: "This update disc requires normal or full main installation\nPlease update the main installation first and try again",
            }
            .into());
        }
        let name = std::slice::from_raw_parts_mut(game_addr(CAR_NAME_RVA) as *mut u8, CAR_NAME_LEN);
        let Some(end) = name
            .iter()
            .position(|&c| c == b'(')
            .filter(|end| end + 1 < CAR_NAME_LEN)
        else {
            return Err(BadInput("the Clio's name has no ( any more").into());
        };
        set_script_entry(0x1038);
        name[..end].make_ascii_uppercase();
        name[end] = b'\n';
        name[end + 1] = 0;
        game_write(STEP_RVA, game_addr(CLIO_STEP_RVA) as u32);
        Ok(())
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    vec![
        at(HAS_CLIO_RVA),
        (game_addr(B_BEFORE_RVA) as u32, PATH_ROOM),
        (game_addr(B_AFTER_RVA) as u32, PATH_ROOM),
        (game_addr(SCRIPT_ENTRY_RVA + 0x18) as u32, 0x14),
        (game_addr(CAR_NAME_RVA) as u32, CAR_NAME_LEN),
        at(STEP_RVA),
        at(MEDIA_MODE_RVA),
        (game_addr(STATUS_RVA) as u32, 0x100),
        (game_addr(SCRATCH_PATH_RVA) as u32, PATH_ROOM),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { check_clio_disc(&mut Game) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const LEN: usize = 0x108;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x05, 0xB8, 0x9D, 0x52, 0x00, 0xE8];
const CALLS: [usize; 9] = [0x07, 0x2F, 0x3A, 0x65, 0x7C, 0x89, 0x9A, 0xA1, 0xFD];
const FATAL_JUMP_OPERAND: usize = 0x104;
const FILE_EXISTS_RVA: usize = rva(0x402EBE);
const SET_DIRECTORY_RVA: usize = rva(0x403105);
const READ_RECORD_RVA: usize = rva(0x4110C9);
const RUNS_FROM_SOURCE_RVA: usize = rva(0x412DA3);
/// (installing, the registry's status and source, base directory B, the world)
type Case<'a> = (u32, u8, &'a [u8], &'a [u8], World);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(Some(mut here)) = (unsafe { current_directory() }) else {
        return t.skip("the current directory cannot be had");
    };
    here.push(b'\\');
    let log_rva = unsafe {
        let at = game_addr(rva(0x416653)) + 0xFD;
        (at + 5)
            .wrapping_add(((at + 1) as *const i32).read_unaligned() as usize)
            .wrapping_sub(game_addr(0))
    };
    // The original's `runs_from_source` reads the registry itself: a copy of it that asks
    // the test stands in for it.
    let asking = match unsafe { copy_asking_the_test() } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let runs_from_source_copy = asking.addr();
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x416653)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| match callee {
                FILE_EXISTS_RVA => Some(stand_in(FILE_EXISTS)),
                SET_DIRECTORY_RVA => Some(stand_in(SET_DIRECTORY)),
                READ_RECORD_RVA => Some(stand_in(READ_RECORD)),
                RUNS_FROM_SOURCE_RVA => Some(runs_from_source_copy),
                callee if callee == log_rva => Some(stand_in(LOG)),
                _ => None,
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.retarget(FATAL_JUMP_OPERAND, fatal_landing as *const () as usize) };

    let mut regions = regions();
    regions.push(at(INSTALLING_RVA));
    regions.push((game_addr(BASE_B_RVA) as u32, 0x20));
    let saved = capture(&regions);
    let car_name = unsafe {
        std::slice::from_raw_parts(game_addr(CAR_NAME_RVA) as *const u8, CAR_NAME_LEN).to_vec()
    };
    const CLIO: &[u8] = b"files\\b\\obj0570.ro3";
    const RECORD: &[u8] = b"var\\ral.zog";
    const INSTALLER: &[u8] = b"files\\d\\gfx\\install.bun";
    let world = |files: &[&'static [u8]], record_mode: Option<u32>| World {
        files: files.to_vec(),
        cd_drives: b"",
        directories: vec![b"d:\\install\\"],
        record_mode,
        registry: None,
    };
    let elsewhere: &[u8] = b"D:\\Install\\";
    let cases: Vec<Case> = vec![
        (0, 1, elsewhere, b"E:\\", world(&[], Some(2))),
        (1, 1, elsewhere, b"E:\\", world(&[CLIO], Some(2))),
        (0, 1, elsewhere, b"E:\\", world(&[CLIO, RECORD], Some(2))),
        (0, 0, elsewhere, b"E:\\", world(&[CLIO], Some(2))),
        (0, 1, b"", b"E:\\", world(&[CLIO], Some(2))),
        (0, 1, &here, b"E:\\", world(&[CLIO], Some(2))),
        (0, 1, elsewhere, b"E:\\", world(&[CLIO, INSTALLER], Some(2))),
        (0, 1, elsewhere, b"not a path", world(&[CLIO], Some(2))),
        (0, 1, elsewhere, b"E:\\", world(&[CLIO], Some(2))),
        (0, 1, elsewhere, b"e:/Disc", world(&[CLIO], Some(3))),
        (0, 1, elsewhere, b"E:\\", world(&[CLIO], Some(1))),
        (0, 1, elsewhere, b"E:\\", world(&[CLIO], Some(0))),
        (0, 1, elsewhere, b"E:\\", world(&[CLIO], None)),
        (0, 1, elsewhere, b"", world(&[CLIO], Some(2))),
    ];
    for (n, (installing, status, source, base_b, mut world)) in cases.into_iter().enumerate() {
        world.registry = Some((status, source.to_vec()));
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            game_write(SCRATCH_PATH_RVA, 0u8);
            game_write(INSTALLING_RVA, installing);
            game_write(STATUS_RVA, 0u8);
            game_write(SOURCE_RVA, 0u8);
            std::ptr::copy(
                base_b.as_ptr(),
                game_addr(BASE_B_RVA) as *mut u8,
                base_b.len(),
            );
            game_write(BASE_B_RVA + base_b.len(), 0u8);
            std::ptr::copy(
                car_name.as_ptr(),
                game_addr(CAR_NAME_RVA) as *mut u8,
                CAR_NAME_LEN,
            );
            // What the record's reading finds if the world's does not say.
            game_write(MEDIA_MODE_RVA, 1u32);
            check_test::begin(&world);
            let ended = if rust {
                match check_clio_disc(&mut Tested) {
                    Ok(()) => 0,
                    Err(Refused::Fatal(_)) => 1,
                    Err(Refused::BadInput(_)) => 2,
                }
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                match (
                    WENT_FATAL.load(Ordering::SeqCst),
                    ro.preserved_except(&before, 0),
                ) {
                    (1, _) => 1,
                    (_, true) => 0,
                    _ => 3,
                }
            };
            (ended, check_test::asked(), capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 < 2, || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "case {n}: original ended {} asked {:?}; rust ended {} asked {:?}; first different byte {first:?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
