//! The ring's work and the surface pictures' names.

use crate::effects::blocks::RingPool;
use crate::effects::ring_entries::Entry;
use crate::effects::run_ribbons::Colours257;
use crate::effects::run_ring::RingKinds;
use crate::memory::owned::Owned;
use crate::memory::session::{PlainBytes, PlainDword, PlainFloat};

/// The ring's work: its list, cursor and count, the emitters' and the draws' integers.
#[derive(rc2k_derive::Cells)]
pub struct RingWork {
    /// The ring's memory list (`memory::List`, 16 bytes: `effects::set_up_surface` takes the
    /// 0x25000-byte ring on it).
    pub list: crate::memory::ListCells,
    /// The ring's next entry to take (an entry number + 1, 0 none; see `ring_entries`).
    pub cursor: PlainDword,
    /// The ring's first entry (an entry number + 1, as the entries' links).
    pub block: PlainDword,
    /// An entry's +0x14 against it ends the box kinds' draw (`effects::run_ring`).
    pub top: PlainFloat,
    /// `effects::emit_rolling_dust`'s fade (the strength times a factor).
    pub fade: PlainFloat,
    /// `effects::emit_rolling_dust`'s life.
    pub life: PlainDword,
    /// The box's |x| and |z| (`effects::run_ring`, rounded).
    pub extent: [PlainDword; 2],
    /// `effects::emit_wheel_grit_bursts`' direction.
    pub direction: [PlainFloat; 3],
    /// `effects::emit_wheel_grit_bursts`' two halves' strengths.
    pub strength: [PlainFloat; 2],
    /// The emitters' rate (`effects::emitters::gate`).
    pub rate: PlainDword,
    /// The entry's way dotted with the object's (`effects::run_ring`).
    pub facing: PlainFloat,
    /// The place's length less a constant (`effects::run_ring`).
    pub near: PlainFloat,
    /// The work integer of the emitters and the ring's draws (a strength, a count, a chance, a
    /// size, a depth, an alpha).
    pub whole: PlainDword,
    /// The entries taken.
    pub count: PlainDword,
    /// The place less the object's (`effects::run_ring`).
    pub away: [PlainFloat; 3],
    /// The same turned by the object's matrix.
    pub turned: [PlainFloat; 3],
}

impl RingWork {
    pub const fn new() -> Self {
        RingWork {
            list: crate::memory::ListCells::empty(),
            cursor: PlainDword::new(0x0),
            block: PlainDword::new(0x0),
            top: PlainFloat::new(-32768.0),
            fade: PlainFloat::new(0.0),
            life: PlainDword::new(0x0),
            extent: [const { PlainDword::new(0x0) }; 2],
            direction: [const { PlainFloat::new(0.0) }; 3],
            strength: [const { PlainFloat::new(0.0) }; 2],
            rate: PlainDword::new(0x0),
            facing: PlainFloat::new(0.0),
            near: PlainFloat::new(0.0),
            whole: PlainDword::new(0x0),
            count: PlainDword::new(0x0),
            away: [const { PlainFloat::new(0.0) }; 3],
            turned: [const { PlainFloat::new(0.0) }; 3],
        }
    }
}

/// The surface pictures' names: `effects::set_up_surface` reads them.
#[derive(rc2k_derive::Cells)]
pub struct PictureNames {
    /// "files\m\surfdata.pcx".
    pub surfdata: PlainBytes<21>,
    /// "files\m\surfmask.pcx".
    pub surfmask: PlainBytes<21>,
}

impl PictureNames {
    pub const fn new() -> Self {
        PictureNames {
            surfdata: PlainBytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x75, 0x72, 0x66, 0x64, 0x61,
                0x74, 0x61, 0x2E, 0x70, 0x63, 0x78, 0x00,
            ]),
            surfmask: PlainBytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x75, 0x72, 0x66, 0x6D, 0x61,
                0x73, 0x6B, 0x2E, 0x70, 0x63, 0x78, 0x00,
            ]),
        }
    }
}

/// The marks ring's state, one part of the effects' owner (`effects::owner::Effects`): the ring's
/// entries, the particle pool, the ring work, the picture names, the kinds' tuning and the
/// 257 colours.
pub struct Ring {
    pub entries: Owned<Vec<Entry>>,
    pub pool: RingPool,
    pub work: RingWork,
    pub names: PictureNames,
    pub kinds: RingKinds,
    pub colours: Colours257,
}

impl Ring {
    pub const fn new() -> Self {
        Ring {
            entries: Owned::new("no marks ring"),
            pool: RingPool::new(),
            work: RingWork::new(),
            names: PictureNames::new(),
            kinds: RingKinds::new(),
            colours: Colours257::new(),
        }
    }
}
