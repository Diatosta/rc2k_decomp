//! The ring's work and the surface pictures' names (IDA 0x935620 .. 0x935766).

use crate::effects::blocks::RingPool;
use crate::effects::ring_entries::Entry;
use crate::effects::run_ribbons::Colours257;
use crate::effects::run_ring::RingKinds;
use crate::memory::owned::Owned;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// The ring's work (IDA 0x935620 .. 0x93573C): its list, cursor and count, the emitters' and the draws' integers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RingWork {
    /// 0x935620: `stru_935620`: the ring's memory list (`memory::List`, 16 bytes: `effects::set_up_surface` takes the 0x25000-byte ring on it).
    pub list: PlainBytes<16>,
    /// 0x935660: `g_marksRingCursor`: the ring's next entry to take (an entry number + 1, 0 none; see `ring_entries`).
    pub cursor: PlainDword,
    /// 0x935668: `g_marksRingBlock`: the ring's first entry (an entry number + 1, as the entries' links; the original kept the 0x25000-byte block's address).
    pub block: PlainDword,
    /// 0x935670: `flt_935670`: an entry's +0x14 against it ends the box kinds' draw (`effects::run_ring`).
    pub top: PlainFloat,
    /// 0x935678: `flt_935678`: `effects::emit_rolling_particles`'s fade (the strength times a factor).
    pub fade: PlainFloat,
    /// 0x93567C: `dword_93567C`: `effects::emit_rolling_particles`'s life.
    pub life: PlainDword,
    /// 0x935680: `dword_935680`, `dword_935684`: the box's |x| and |z| (`effects::run_ring`, `fistp`ed).
    pub extent: [PlainDword; 2],
    /// 0x935688: `flt_935688` .. `flt_935690`: `effects::emit_bursts`' direction.
    pub direction: [PlainFloat; 3],
    /// 0x935694: `flt_935694`, `flt_935698`: `effects::emit_bursts`' two halves' strengths.
    pub strength: [PlainFloat; 2],
    /// 0x9356B0: `dword_9356B0`: the emitters' rate (`effects::emitters::gate`).
    pub rate: PlainDword,
    /// 0x9356B4: `flt_9356B4`: the entry's way dotted with the object's (`effects::run_ring`).
    pub facing: PlainFloat,
    /// 0x9356BC: `flt_9356BC`: the place's length less `flt_93A526` (`effects::run_ring`).
    pub near: PlainFloat,
    /// 0x9356C0: `dword_9356C0`: the work integer of the emitters and the ring's draws (a strength, a count, a chance, a size, a depth, an alpha).
    pub whole: PlainDword,
    /// 0x9356C8: `g_marksRingCount`: the entries taken.
    pub count: PlainDword,
    /// 0x935718: `flt_935718` .. `flt_935720`: the place less the object's (`effects::run_ring`).
    pub away: [PlainFloat; 3],
    /// 0x935724: `flt_935724` .. `flt_93572C`: the same turned by the object's matrix.
    pub turned: [PlainFloat; 3],
}

impl RingWork {
    pub const fn new() -> Self {
        RingWork {
            list: PlainBytes::new([0; 16]),
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

/// The surface pictures' names (IDA 0x93573C .. 0x935766): `effects::set_up_surface` reads them.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PictureNames {
    /// 0x93573C: "files\m\surfdata.pcx" (`aFilesMSurfdata`).
    pub surfdata: PlainBytes<21>,
    /// 0x935751: "files\m\surfmask.pcx" (`aFilesMSurfmask`).
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
