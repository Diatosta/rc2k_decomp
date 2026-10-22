//! `race::find_present_courses` (0x4394F8): which of the 100 courses are there: for each,
//! its number in the stage's paths (`race::set_course_number`, `race::set_cdat_number`
//! with 0), its flag (`dword_621518[n]`) 0, and 4 bytes of `srclvls\levelNN\cdat00c`
//! (0x60B02B) read into `dword_623FB8` (`files::read_whole_file`, inside two `pusha`);
//! four read (`g_readBytes`), the flag 1; else the same with
//! `files\levels\levelNN\cdat00c.sq` (0x60B07A).
//!
//! In:  nothing
//! Out: eax 0x30, edx 0x30, ebx 10 (`set_cdat_number`'s), ecx 100, esi `dword_623FB8`,
//!      edi `dword_621518`; the rest kept; no flags read (a store follows)
//!
//! Callers: `scene::set_up_lit_engine`, and code at 0x442BF3 (not a function in IDA).
//! Refuses what `files::read_whole_file` refuses (nothing written then).
//! Shadow mode: by state (the files read twice). Self-test: the original and the Rust
//! version on the game's paths and flags (put back), with the files as they are.

use crate::errors::Refused;
use crate::files::read_whole_file::{BYTES_READ_RVA, read_whole_file};
use crate::framework::{
    EAX, EBX, ECX, EDI, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write,
    hook_asm, rva,
};
use crate::race::course_data::{CDAT_DIGITS, set_cdat_number};
use crate::race::level_init::{NUMBER_PLACES, set_course_number};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::find_present_courses",
    rva(0x4394F8),
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

const FLAGS_RVA: usize = rva(0x621518); // dword_621518, 100 flags
const BUFFER_RVA: usize = rva(0x623FB8); // dword_623FB8
/// `srclvls\level01\cdat00c`, `files\levels\level01\cdat00c.sq`.
const PATHS_RVA: [usize; 2] = [rva(0x60B02B), rva(0x60B07A)];
use rc2k_core::race::stage_objects::COURSES;

fn regions() -> Vec<Region> {
    let a = |x: usize, len: usize| (game_addr(x) as u32, len);
    let mut all: Vec<Region> = NUMBER_PLACES.iter().map(|&p| a(rva(p), 2)).collect();
    all.extend(CDAT_DIGITS.iter().map(|&p| a(p, 2)));
    all.extend([
        a(FLAGS_RVA, 4 * COURSES as usize),
        a(BUFFER_RVA, 4),
        a(BYTES_READ_RVA, 4),
    ]);
    all
}

unsafe fn steps(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::stage_objects::{COURSES, course_present, PROBE_BYTES}`.
    use rc2k_core::race::stage_objects::{PROBE_BYTES, course_present};
    let mut g = *entry;
    g.edi = game_addr(FLAGS_RVA) as u32;
    for n in 0..COURSES {
        g.ecx = n;
        g.eax = n;
        unsafe {
            set_course_number(g.eax);
            g.eax = 0;
            g = set_cdat_number(&g);
            game_write(FLAGS_RVA + 4 * n as usize, 0u32);
            let present = course_present(PATHS_RVA.len() as u32, |file| {
                game_write(BYTES_READ_RVA, 0u32);
                g.esi = game_addr(BUFFER_RVA) as u32;
                let path = game_addr(PATHS_RVA[file as usize]) as *const u8;
                read_whole_file(path, g.esi as *mut u8, PROBE_BYTES, g.esi)?;
                Ok::<u32, Refused>(game_read(BYTES_READ_RVA))
            })?;
            if present {
                game_write(FLAGS_RVA + 4 * n as usize, 1u32);
            }
        }
    }
    g.ecx = COURSES;
    Ok(g)
}

/// The Rust version (nothing written when it refuses).
pub fn find_present_courses(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    match unsafe { steps(entry) } {
        Ok(g) => Ok(Left::nothing()
            .with(EAX, g.eax)
            .with(EBX, g.ebx)
            .with(ECX, g.ecx)
            .with(EDX, g.edx)
            .with(ESI, g.esi)
            .with(EDI, g.edi)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &find_present_courses);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &find_present_courses)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for pattern in [0u8, 0xAA] {
        check_state(
            &mut t,
            &format!("flags filled with {pattern:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                std::ptr::write_bytes(
                    game_addr(FLAGS_RVA) as *mut u8,
                    pattern,
                    4 * COURSES as usize,
                );
            },
            &regions,
            &find_present_courses,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
