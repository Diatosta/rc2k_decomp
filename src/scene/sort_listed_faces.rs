//! `scene::sort_listed_faces`: puts an object's listed faces in the order they are drawn in,
//! the largest key first.
//!
//! Nothing with fewer than two faces listed. Else the list's entries (0x10 bytes; the key is
//! the dword +4, compared unsigned) are sorted by a quicksort written out here step for step,
//! since which of two equal keys comes first is its doing: the key in the middle is the pivot;
//! the left place moves on one entry if its key is above the pivot, the right place moves back
//! while its key is below; if the two places are not the same the entries change places, and
//! the left one moves on once more if their keys are now equal; when they meet, the part before
//! and the part after are sorted the same way. One caller, `scene::draw_object`. Refuses a list
//! longer than 0x1000 entries, and a sort that would go past the list (never met in the core's
//! tests), before changing anything.

use crate::errors::BadInput;
use crate::scene::face_list::FaceList;

/// Logic: `rc2k_core::scene::face_order::sort_listed_faces`.
pub fn sort_listed_faces(list: &mut FaceList) -> Result<(), BadInput> {
    Ok(rc2k_core::scene::face_order::sort_listed_faces(list)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A list longer than 0x400 entries is sorted whole.
    #[test]
    fn sorts_a_long_list() {
        const LISTED: u32 = 0x408;
        let mut list: FaceList = (0..LISTED).map(|n| [n, n, 0, 0]).collect();
        sort_listed_faces(&mut list).unwrap();
        let keys: Vec<u32> = list.iter().map(|face| face[1]).collect();
        assert_eq!(keys.len(), LISTED as usize);
        assert!(keys.windows(2).all(|k| k[0] >= k[1]), "largest key first");
    }
}
