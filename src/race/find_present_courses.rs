//! `race::find_present_courses` (0x4394F8): which of the 100 courses are there: for each,
//! its number in the stage's paths (`race::set_course_number`, `race::set_cdat_number`
//! with 0), its flag (`dword_621518[n]`) 0, and 4 bytes of `srclvls\levelNN\cdat00c`
//! (0x60B02B) read into `dword_623FB8` (`files::read_whole_file`, inside two `pusha`);
//! four read (`g_readBytes`), the flag 1; else the same with
//! `files\levels\levelNN\cdat00c.sq` (0x60B07A).
//!
//! In:  nothing
//! Out: nothing its callers read (conversion phase 3, 2026-10-24: `scene::set_up_lit_engine`
//!      works on a copy of its registers; the code at 0x442BF3 is not reached in Rust)
//!
//! Callers: `scene::set_up_lit_engine`, and code at 0x442BF3 (not a function in IDA). Refuses
//! what `files::read_whole_file` refuses (nothing written then).

use crate::errors::Refused;
use crate::files::read_whole_file::read_whole_file;
use crate::files::state::BYTES_READ;
use crate::framework::{game_addr, game_write, rva};
use crate::race::course_data::set_cdat_number;
use crate::race::level_init::set_course_number;
use crate::scene::draw_store::DRAW;

const FLAGS_RVA: usize = rva(0x621518); // dword_621518, 100 flags
/// `srclvls\level01\cdat00c`, `files\levels\level01\cdat00c.sq`.
const PATHS_RVA: [usize; 2] = [rva(0x60B02B), rva(0x60B07A)];
use rc2k_core::race::stage_objects::COURSES;

/// The Rust version (nothing written when it refuses).
pub fn find_present_courses() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage_objects::{COURSES, course_present, PROBE_BYTES}`.
    use rc2k_core::race::stage_objects::{PROBE_BYTES, course_present};
    for n in 0..COURSES {
        unsafe {
            set_course_number(n);
            set_cdat_number(0);
            game_write(FLAGS_RVA + 4 * n as usize, 0u32);
            let present = course_present(PATHS_RVA.len() as u32, |file| {
                BYTES_READ.store(0, std::sync::atomic::Ordering::Relaxed);
                let buffer = DRAW.work_dword.addr() as u32;
                let path = game_addr(PATHS_RVA[file as usize]) as *const u8;
                read_whole_file(path, buffer as *mut u8, PROBE_BYTES)?;
                Ok::<u32, Refused>(BYTES_READ.load(std::sync::atomic::Ordering::Relaxed))
            })?;
            if present {
                game_write(FLAGS_RVA + 4 * n as usize, 1u32);
            }
        }
    }
    Ok(())
}
