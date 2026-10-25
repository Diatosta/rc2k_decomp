//! `scene::sort_listed_faces` (0x45098C): puts an object's listed faces in the order
//! they are drawn in, the largest key first.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx, edx, esi, edi and ebp changed when it
//!      sorts; its caller goes on without them, and the Rust version keeps them
//!
//! Nothing with fewer than two faces listed. Else the list's entries (0x10 bytes; the key is
//! the dword +4, compared unsigned) are sorted by `sub_4509BF`, a quicksort written out here
//! step for step, since which of two equal keys comes first is its doing: the key in the middle
//! is the pivot; the left place moves on one entry if its key is above the pivot, the right
//! place moves back while its key is below; if the two places are not the same the entries
//! change places, and the left one moves on once more if their keys are now equal; when they
//! meet, the part before and the part after are sorted the same way. One caller, `sub_45045D`.
//! Refuses a list longer than 0x1000 entries, and a sort that would go past the list (never met
//! in the core's tests), before changing anything.

use crate::errors::BadInput;
use crate::framework::game_addr;
use crate::scene::draw_object_faces::LIST_RVA;
use crate::scene::light_work_store::LIGHT_WORK;

const MOST: u32 = 0x1000;

// ---- Rust version ----

/// Converted: `rc2k_core::scene::face_order::sort_listed_faces`.
pub unsafe fn sort_listed_faces() -> Result<(), BadInput> {
    unsafe {
        let listed: u32 = LIGHT_WORK.listed_face_count.bits();
        if listed > MOST {
            return Err(BadInput("more faces listed than the list holds"));
        }
        let list = std::slice::from_raw_parts_mut(
            game_addr(LIST_RVA) as *mut rc2k_core::scene::face_order::ListedFace,
            listed as usize,
        );
        Ok(rc2k_core::scene::face_order::sort_listed_faces(list)?)
    }
}
