//! `physics::count_ground_probe` (0x438E22): one ground probe, counted: the face mask
//! `dword_620470` 0x40, `dword_608C3C` up by one, `physics::find_ground`; when found
//! `dword_608C40` up by one, and when the face is the last one again (`dword_5CCC78` =
//! `dword_5CCC74`) `dword_608C44` too. A reading: the camera's (or a test drive's) ground
//! statistics.
//!
//! In:  the point (`dword_639824`, `dword_63982C`; the callers set it)
//! Out: as `find_ground` leaves them, eax `dword_5CCC78` when found; no flags read (the
//!      callers compare `g_groundFound`)
//!
//! Callers: `sub_438BF4`, `sub_438CEC` (three). Refuses what `find_ground` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::ground_store::GROUND_WORK;
use crate::race::course_store::COURSE;
use rc2k_core::physics::walls::ProbeCounts;

/// The Rust version. The eax it leaves (`dword_5CCC78` when found) is read by none of its
/// callers.
// Converted: `rc2k_core::physics::walls::ProbeCounts`.
pub unsafe fn count_ground_probe() -> Result<(), Refused> {
    let mut counts = ProbeCounts {
        tries: CAMERA_WORK.ground_probes.get(),
        found: CAMERA_WORK.ground_found.get(),
        same: CAMERA_WORK.ground_same.get(),
    };
    crate::physics::ground_probe_store::GROUND_PROBE
        .settle_mark
        .set(0x40);
    counts.tried();
    CAMERA_WORK.ground_probes.set(counts.tries);
    unsafe { crate::physics::find_ground::find_ground() }?;
    if GROUND_WORK.ground_found.get() != 0 {
        counts.searched(
            true,
            COURSE.ground_face_last.get(),
            COURSE.ground_slot.get(),
        );
        CAMERA_WORK.ground_found.set(counts.found);
        CAMERA_WORK.ground_same.set(counts.same);
    }
    Ok(())
}
