//! The object faces and the two sort lists, typed (phase 7): IDA 0x9A3730 .. 0x9E3738, in
//! the image's zeroed tail (`race::buffers_store`). Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Faces {
    /// +0x0 (0x9A3730): `g_faces`: the objects' faces, 0x30000 bytes (`scene::objects`, `scene::draw_object_faces`, `scene::list_object_faces`, `race::frame_draw`).
    pub faces: [Dword; 0xC000],
    /// +0x30000 (0x9D3730): the two sort lists (`SORT_LISTS` for the running thread's).
    pub lists: SortLists,
}

/// The two sort lists a view's drawing fills, sorts and draws (`race::draw_view_scene`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SortLists {
    /// +0x0 (0x9D3730): `g_depthEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub depth_entries: [Dword; 0x2000],
    /// +0x8000 (0x9DB730): `g_sortEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub sort_entries: [Dword; 0x2000],
    /// +0x10000 (0x9E3730): not named (`dword_9E3734` is the second).
    pub unknown_40000: [Dword; 2],
}

/// A drawing thread's sort lists (`workers`: each run fills its own), with room for what is
/// read past them as in the buffers block: a flare's entry read as a description of 0x3C
/// bytes from its second word (`scene::draw_described_object`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WorkerLists {
    pub lists: SortLists,
    pub overrun: [Dword; 0x10],
}

/// The piece, a field of `crate::race::owner::race().shared.buffers`.
pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = Faces;
    fn deref(&self) -> &Faces {
        &race().shared.buffers.faces
    }
}

pub static FACES: InBuffers = InBuffers;

/// The running thread's sort lists: the game's thread's in `FACES`, each other drawing
/// thread's in `WORKER_LISTS`.
pub struct Lists;

static WORKER_LISTS: [WorkerLists; crate::workers::COUNT - 1] =
    [const { crate::pointers::cells_of_bytes([0u8; core::mem::size_of::<WorkerLists>()]) };
        crate::workers::COUNT - 1];

impl core::ops::Deref for Lists {
    type Target = SortLists;
    #[inline]
    fn deref(&self) -> &SortLists {
        match crate::workers::index() {
            0 => &FACES.lists,
            n => &WORKER_LISTS[n - 1].lists,
        }
    }
}

impl crate::workers::Copies for Lists {
    fn copy(&self, n: usize) -> &dyn crate::pointers::Store {
        match n {
            0 => &*FACES,
            n => &WORKER_LISTS[n - 1],
        }
    }
}

pub static SORT_LISTS: Lists = Lists;

/// The running thread's list at byte `off` of `SortLists`, as an address: bounded by the
/// whole buffers block for the game's thread (the sort list's last entries are read past
/// the store), by its `WorkerLists` for another.
pub fn list_at(off: usize) -> crate::pointers::GamePtr {
    use crate::race::buffers_store::Buffers;
    use core::mem::offset_of;
    match crate::workers::index() {
        0 => crate::pointers::GamePtr::to(&race().shared.buffers)
            .offset((offset_of!(Buffers, faces) + offset_of!(Faces, lists) + off) as u32),
        n => crate::pointers::GamePtr::to(&WORKER_LISTS[n - 1]).offset(off as u32),
    }
}

/// IDA 0x9A3730, the first byte of the piece.
pub const START: usize = 0x9A3730;
/// IDA 0x9E3738, the end of the piece.
pub const END: usize = 0x9E3738;

const _: () = assert!(END - START == core::mem::size_of::<Faces>());

/// Where IDA 0x9A3730 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::moved::place(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        let lists = offset_of!(Faces, lists);
        assert_eq!(
            lists + offset_of!(SortLists, depth_entries),
            0x9D3730 - START
        );
        assert_eq!(
            lists + offset_of!(SortLists, sort_entries),
            0x9DB730 - START
        );
        assert_eq!(
            lists + offset_of!(SortLists, unknown_40000),
            0x9E3730 - START
        );
    }
}
