//! `scene::count_texture_needs` (0x423EB1): how many textures are needed and how much
//! memory they take, for the log.
//!
//! In:  nothing
//! Out: eax = the last number looked at, ecx = 0x400
//!
//! Over the 0x400 textures: those needed in system memory (`dword_5CEAE0` not -1) are
//! counted in `dword_5DC974` and their levels' sizes (`dword_5CDAAC`) added in
//! `dword_5DC96C`; those needed in video memory (`dword_5CFAE0`) in `dword_5DC970`
//! and `dword_5DC968`. Then one line in the game's log:
//! "VID <k> k in <n> texs - SYS <k> k in <n> texs".
//! `dword_5DC970` is also what `scene::load_slot_textures` takes for the slots' number.
//! Two call sites in `sub_42274B`.
//! Refuses a level past the sizes' table.
//! Shadow mode: the original, then the four numbers against the Rust arithmetic.
//! Self-test: a redirected copy whose log calls go to the test, 12 tables by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, ECX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::redirect::RedirectedCopy;
use crate::scene::texture_needs::{
    LEVEL_SIZES_RVA, LEVELS, SYSTEM_LEVELS_RVA, TEXTURES, VIDEO_LEVELS_RVA, levels,
};
use crate::scene::textures::{test_table, write_table};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::count_texture_needs",
    rva(0x423EB1),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `dword_5DC968` video k, `dword_5DC96C` system k, `dword_5DC970` video textures,
/// `dword_5DC974` system textures.
const TOTALS_RVA: usize = rva(0x5DC968);

// ---- Rust version ----

/// The four totals and the last number looked at. Reads only.
unsafe fn totals() -> Result<([u32; 4], u32), BadInput> {
    unsafe {
        let mut totals = [0u32; 4];
        let mut eax = 0;
        for n in 0..TEXTURES {
            for (table, k, count) in [(SYSTEM_LEVELS_RVA, 1, 3), (VIDEO_LEVELS_RVA, 0, 2)] {
                eax = game_read(table + n * 4);
                if eax == u32::MAX {
                    continue;
                }
                if eax >= LEVELS {
                    return Err(BadInput("a texture's level past the sizes' table"));
                }
                totals[count] += 1;
                eax = game_read(LEVEL_SIZES_RVA + eax as usize * 4);
                totals[k] = totals[k].wrapping_add(eax);
            }
        }
        Ok((totals, eax))
    }
}

/// `log`: what writes a piece of the line: a text, or a number.
pub unsafe fn count_texture_needs(
    log: &mut dyn FnMut(Result<&'static std::ffi::CStr, u32>, bool),
) -> Result<Left, BadInput> {
    unsafe {
        let (totals, eax) = totals()?;
        game_write(TOTALS_RVA, totals);
        log(Ok(c"VID"), false);
        log(Err(totals[0]), false);
        log(Ok(c"k in"), false);
        log(Err(totals[2]), false);
        log(Ok(c"texs - SYS"), false);
        log(Err(totals[1]), false);
        log(Ok(c"k in"), false);
        log(Err(totals[3]), false);
        log(Ok(c"texs"), true);
        Ok(Left::nothing().with(EAX, eax).with(ECX, TEXTURES as u32))
    }
}

/// Into the game's log.
pub unsafe fn count_texture_needs_in_game() -> Result<Left, BadInput> {
    unsafe {
        count_texture_needs(&mut |piece, newline| match piece {
            Ok(text) => log_write(text.as_ptr().cast(), newline, 1),
            Err(number) => log_int(number as i32, newline, 1),
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let said = unsafe { totals() }.ok();
        unsafe { call_original(r) };
        let theirs: [u32; 4] = unsafe { game_read(TOTALS_RVA) };
        if said.is_some_and(|(mine, _)| mine != theirs) {
            HOOK.mismatch(|| format!("original {theirs:x?}; rust {said:x?}"));
        }
        return;
    }
    match unsafe { count_texture_needs_in_game() } {
        Ok(left) => left.apply(r),
        Err(e) => {
            let e: Refused = e.into();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

// ---- self-test ----

const START: usize = 0x423EB1;
const LEN: usize = 0xE6;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0x68, 0xC9, 0x5D, 0x00, 0, 0, 0, 0];
const CALL_SITES: [usize; 9] = [
    0x423F2C, 0x423F38, 0x423F45, 0x423F51, 0x423F5E, 0x423F6A, 0x423F77, 0x423F83, 0x423F90,
];

/// Each piece logged: the place (0 a text, 1 a number, 2 a text and a line's end), and
/// the number or the text's first bytes.
static LOGGED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    let value = if place == 1 {
        r.eax
    } else {
        unsafe { (r.eax as *const u32).read_unaligned() }
    };
    LOGGED.lock().unwrap().push((place, value));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = CALL_SITES.iter().map(|site| site - START).collect();
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(START)),
            LEN,
            &ENTRY_BYTES,
            &calls,
            &|callee| {
                // logfile::log_write_l1, log_int_l1, log_write_l1_nl
                [rva(0x401427), rva(0x401472), rva(0x40142B)]
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 1] = [(game_addr(TOTALS_RVA) as u32, 16)];
    let inputs = [levels(SYSTEM_LEVELS_RVA), levels(VIDEO_LEVELS_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0008u32;
    for case in 0..12u32 {
        let system = test_table(&mut seed, TEXTURES as u32, 1 + case % 3, LEVELS);
        let video = test_table(&mut seed, TEXTURES as u32, 1 + case % 4, LEVELS);
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            write_table(SYSTEM_LEVELS_RVA, &system);
            write_table(VIDEO_LEVELS_RVA, &video);
            LOGGED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs::SENTINEL;
            let mut r = before;
            if rust {
                let left = count_texture_needs(&mut |piece, newline| {
                    let logged = match piece {
                        Ok(text) => (
                            if newline { 2 } else { 0 },
                            (text.as_ptr() as *const u32).read_unaligned(),
                        ),
                        Err(number) => (1, number),
                    };
                    LOGGED.lock().unwrap().push(logged);
                });
                match left {
                    Ok(left) => left.apply(&mut r),
                    Err(_) => r = Regs::default(),
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                capture(&regions),
                LOGGED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.2.len() == 9, || {
            format!("case {case}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
