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
use crate::scene::owner::scene;

const MOST: u32 = 0x1000;

// ---- Rust version ----

/// Converted: `rc2k_core::scene::face_order::sort_listed_faces`.
pub fn sort_listed_faces() -> Result<(), BadInput> {
    let scene = scene();
    let listed: u32 = scene.lights.work.listed_face_count.bits();
    if listed > MOST {
        return Err(BadInput("more faces listed than the list holds"));
    }
    // The list runs on past `listed_faces` into the store's next fields, as
    // `scene::list_object_face` writes it.
    let (slots, cells) = (&scene.objects.slots, &scene.objects.slots.listed_faces);
    let mut list = Vec::with_capacity(listed as usize);
    for n in 0..listed as usize {
        let mut face: rc2k_core::scene::face_order::ListedFace = [0; 4];
        for (w, word) in face.iter_mut().enumerate() {
            *word = crate::session::read_on_in(slots, cells, 4 * n + w, "the list of faces")?;
        }
        list.push(face);
    }
    rc2k_core::scene::face_order::sort_listed_faces(&mut list)?;
    for (n, face) in list.iter().enumerate() {
        for (w, &word) in face.iter().enumerate() {
            crate::session::write_on_in(slots, cells, 4 * n + w, word, "the list of faces")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{read_on_in, write_on_in};

    /// A list longer than `listed_faces`' 0x400 entries runs on into the store's next fields.
    #[test]
    fn sorts_a_list_past_the_field() {
        let scene = scene();
        let _lock = crate::store::test_lock();
        const LISTED: usize = 0x408;
        let cells = &scene.objects.slots.listed_faces;
        let word = |n: usize| read_on_in(&scene.objects.slots, cells, n, "a test").unwrap();
        let put = |n: usize, v: u32| {
            write_on_in(&scene.objects.slots, cells, n, v, "a test").unwrap();
        };
        let kept: Vec<u32> = (0..4 * LISTED).map(word).collect();
        let count = scene.lights.work.listed_face_count.bits();
        for n in 0..LISTED {
            put(4 * n, n as u32);
            put(4 * n + 1, n as u32);
        }
        scene.lights.work.listed_face_count.set_bits(LISTED as u32);
        let sorted = sort_listed_faces();
        let keys: Vec<u32> = (0..LISTED).map(|n| word(4 * n + 1)).collect();
        for (n, &v) in kept.iter().enumerate() {
            put(n, v);
        }
        scene.lights.work.listed_face_count.set_bits(count);
        sorted.unwrap();
        assert!(keys.windows(2).all(|k| k[0] >= k[1]), "largest key first");
    }
}
