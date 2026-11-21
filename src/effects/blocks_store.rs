//! The marks' blocks: the ring's particles, the places, links, slopes and corners.

use crate::errors::BadInput;
use crate::memory::session::PlainWord;

const RING_ENTRY: u32 = rc2k_core::effects::rings::RING_ENTRY;

/// A particle of the ring (0x4C-byte entries), its dwords in order. Every field holds the
/// dword's bits as the ring stores them: the places, motions and tunings are floats, the rest
/// whole numbers.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Particle {
    /// The particle before in the ring (a particle's number + 1).
    pub before: u32,
    /// The particle after in the ring (a particle's number + 1).
    pub after: u32,
    pub kind: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub last_x: u32,
    pub last_y: u32,
    pub last_z: u32,
    pub ground: u32,
    pub motion_x: u32,
    pub motion_y: u32,
    /// The lift (added to the vertical motion by the burst kinds).
    pub lift: u32,
    pub motion_z: u32,
    pub size: u32,
    pub colour: u32,
    pub colour_step: u32,
    pub life: u32,
    /// Where the particle's colours start in the burst colours' table.
    pub colours_place: u32,
}

/// What a particle is, by its `kind` dword. The kinds are the ones the ring holds; its tunings
/// are `particle_tuning.dust`, `.trail`, `.burst8` and `.burst9`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ParticleKind {
    /// Dust that rises, bursts when it is past its least height and leaves a trail.
    Dust = 0,
    /// A trail particle, drawn by a colour table of its own.
    Trail = 4,
    /// A burst particle (the first tuning).
    Burst8 = 8,
    /// A burst particle (the second tuning), sometimes drawn bigger.
    Burst9 = 9,
}

impl TryFrom<u32> for ParticleKind {
    type Error = BadInput;

    fn try_from(kind: u32) -> Result<Self, BadInput> {
        match kind {
            0 => Ok(Self::Dust),
            4 => Ok(Self::Trail),
            8 => Ok(Self::Burst8),
            9 => Ok(Self::Burst9),
            _ => Err(BadInput("a particle of another kind")),
        }
    }
}

const _: () = assert!(size_of::<Particle>() == RING_ENTRY as usize);

impl Particle {
    /// The place's coordinate on `axis` (0 x, 1 y, 2 z).
    pub(super) fn place(&self, axis: usize) -> u32 {
        [self.x, self.y, self.z][axis]
    }

    pub(super) fn set_place(&mut self, axis: usize, v: u32) {
        *[&mut self.x, &mut self.y, &mut self.z][axis] = v;
    }

    /// The last place's coordinate on `axis`.
    pub(super) fn last_place(&self, axis: usize) -> u32 {
        [self.last_x, self.last_y, self.last_z][axis]
    }

    /// The motion's component on `axis` (the lift is not one).
    pub(super) fn motion(&self, axis: usize) -> u32 {
        [self.motion_x, self.motion_y, self.motion_z][axis]
    }

    pub(super) fn set_motion(&mut self, axis: usize, v: u32) {
        *[&mut self.motion_x, &mut self.motion_y, &mut self.motion_z][axis] = v;
    }
}

/// The list the places' block is counted on (`effects::allocate_places`; the block itself is
/// not kept).
pub struct PlacesBlock {
    /// The list the places' block is taken on (16 bytes).
    pub list: crate::memory::PlainList,
}

impl PlacesBlock {
    pub const fn new() -> Self {
        Self {
            list: crate::memory::PlainList::empty(),
        }
    }
}

/// The list the links' block is counted on (`effects::allocate_links`; the block itself is not
/// kept).
pub struct MarkLinks {
    /// The list the links' block is taken on (16 bytes).
    pub list: crate::memory::PlainList,
}

impl MarkLinks {
    pub const fn new() -> Self {
        Self {
            list: crate::memory::PlainList::empty(),
        }
    }
}

/// A corner's slope record (0x24 bytes): the slope of its sides and its eight scaled values
/// (`effects::scale_corners`; `effects::draw_cell_sprite` scales a sprite by the slope, and
/// `effects::set_quad_uvs` reads record 0x58's eight values as four texture places).
pub struct Slope {
    /// |v of the third pair - v of the first| over |u of the second - u of the first| of the
    /// corner record, as a float.
    pub slope: f32,
    /// The corner's four word pairs times `DrawWork::uv_k`.
    pub values: [f32; 8],
}

impl Slope {
    pub const fn new() -> Self {
        Slope {
            slope: 0.0,
            values: [0.0; 8],
        }
    }
}

/// The corners' slopes (0x100 records of 0x24 bytes).
pub struct CornerSlopes {
    /// One record for each corner record.
    pub slopes: [Slope; 0x100],
}

impl CornerSlopes {
    pub const fn new() -> Self {
        Self {
            slopes: [const { Slope::new() }; 0x100],
        }
    }
}

/// A corner record (0x80 bytes of 16-bit words: four (u, v) pairs of a cell's texture
/// places, and what `effects::scale_corners` scales).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct Corner {
    pub words: [PlainWord; 0x40],
}

impl Corner {
    pub const fn new() -> Self {
        Corner {
            words: [const { PlainWord::new(0) }; 0x40],
        }
    }
}

/// The marks' corner table (0x100 records of 0x80 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct MarkCorners {
    /// The corner records (the course's 0x8000 bytes of `markinfo` are copied here).
    pub corners: [Corner; 0x100],
}

impl MarkCorners {
    pub const fn new() -> Self {
        Self {
            corners: [const { Corner::new() }; 0x100],
        }
    }
}
