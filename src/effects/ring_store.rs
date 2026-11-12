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
    /// 0x935630: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_935630: PlainBytes<48>,
    /// 0x935660: `g_marksRingCursor`: the ring's next entry to take (an entry number + 1, 0 none; see `ring_entries`).
    pub cursor: PlainDword,
    /// 0x935664: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_935664: PlainBytes<4>,
    /// 0x935668: `g_marksRingBlock`: the ring's first entry (an entry number + 1, as the entries' links; the original kept the 0x25000-byte block's address).
    pub block: PlainDword,
    /// 0x93566C: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_93566c: PlainBytes<4>,
    /// 0x935670: `flt_935670`: an entry's +0x14 against it ends the box kinds' draw (`effects::run_ring`).
    pub top: PlainFloat,
    /// 0x935674: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_935674: PlainBytes<4>,
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
    /// 0x93569C: Not named: 20 bytes (a gap, or unreferenced data).
    pub _gap_93569c: PlainBytes<20>,
    /// 0x9356B0: `dword_9356B0`: the emitters' rate (`effects::emitters::gate`).
    pub rate: PlainDword,
    /// 0x9356B4: `flt_9356B4`: the entry's way dotted with the object's (`effects::run_ring`).
    pub facing: PlainFloat,
    /// 0x9356B8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_9356b8: PlainBytes<4>,
    /// 0x9356BC: `flt_9356BC`: the place's length less `flt_93A526` (`effects::run_ring`).
    pub near: PlainFloat,
    /// 0x9356C0: `dword_9356C0`: the work integer of the emitters and the ring's draws (a strength, a count, a chance, a size, a depth, an alpha).
    pub whole: PlainDword,
    /// 0x9356C4: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_9356c4: PlainBytes<4>,
    /// 0x9356C8: `g_marksRingCount`: the entries taken.
    pub count: PlainDword,
    /// 0x9356CC: Not named: 76 bytes (a gap, or unreferenced data).
    pub _gap_9356cc: PlainBytes<76>,
    /// 0x935718: `flt_935718` .. `flt_935720`: the place less the object's (`effects::run_ring`).
    pub away: [PlainFloat; 3],
    /// 0x935724: `flt_935724` .. `flt_93572C`: the same turned by the object's matrix.
    pub turned: [PlainFloat; 3],
    /// 0x935730: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_935730: PlainBytes<12>,
}

impl RingWork {
    pub const fn new() -> Self {
        RingWork {
            list: PlainBytes::new([0; 16]),
            _gap_935630: PlainBytes::new([0; 48]),
            cursor: PlainDword::new(0x0),
            _gap_935664: PlainBytes::new([0; 4]),
            block: PlainDword::new(0x0),
            _gap_93566c: PlainBytes::new([0x00, 0x80, 0xFF, 0xFF]),
            top: PlainFloat::new(-32768.0),
            _gap_935674: PlainBytes::new([0; 4]),
            fade: PlainFloat::new(0.0),
            life: PlainDword::new(0x0),
            extent: [const { PlainDword::new(0x0) }; 2],
            direction: [const { PlainFloat::new(0.0) }; 3],
            strength: [const { PlainFloat::new(0.0) }; 2],
            _gap_93569c: PlainBytes::new([0; 20]),
            rate: PlainDword::new(0x0),
            facing: PlainFloat::new(0.0),
            _gap_9356b8: PlainBytes::new([0; 4]),
            near: PlainFloat::new(0.0),
            whole: PlainDword::new(0x0),
            _gap_9356c4: PlainBytes::new([0; 4]),
            count: PlainDword::new(0x0),
            _gap_9356cc: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xC5, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
            away: [const { PlainFloat::new(0.0) }; 3],
            turned: [const { PlainFloat::new(0.0) }; 3],
            _gap_935730: PlainBytes::new([0; 12]),
        }
    }
}

impl RingWork {
    pub const START: usize = 0x935620;
    pub const LEN: usize = 0x11C;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_ring_work_starts_as_the_games() {
        assert_eq!(size_of::<RingWork>(), RingWork::LEN);
        let p = (&raw const crate::effects::owner::effects().ring.work).cast::<u8>();
        let bytes: Vec<u8> = (0..size_of::<RingWork>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            bytes
                == crate::test_support::first_values::bytes("effects.owner.EFFECTS.ring.work/284"),
            "RING_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(RingWork, list), 0x0);
        assert_eq!(offset_of!(RingWork, cursor), 0x40);
        assert_eq!(offset_of!(RingWork, block), 0x48);
        assert_eq!(offset_of!(RingWork, top), 0x50);
        assert_eq!(offset_of!(RingWork, fade), 0x58);
        assert_eq!(offset_of!(RingWork, life), 0x5c);
        assert_eq!(offset_of!(RingWork, extent), 0x60);
        assert_eq!(offset_of!(RingWork, direction), 0x68);
        assert_eq!(offset_of!(RingWork, strength), 0x74);
        assert_eq!(offset_of!(RingWork, rate), 0x90);
        assert_eq!(offset_of!(RingWork, facing), 0x94);
        assert_eq!(offset_of!(RingWork, near), 0x9c);
        assert_eq!(offset_of!(RingWork, whole), 0xa0);
        assert_eq!(offset_of!(RingWork, count), 0xa8);
        assert_eq!(offset_of!(RingWork, away), 0xf8);
        assert_eq!(offset_of!(RingWork, turned), 0x104);
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

impl PictureNames {
    pub const START: usize = 0x93573C;
    pub const LEN: usize = 0x2A;
}

#[cfg(test)]
mod name_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_picture_names_starts_as_the_games() {
        assert_eq!(size_of::<PictureNames>(), PictureNames::LEN);
        let p = (&raw const crate::effects::owner::effects().ring.names).cast::<u8>();
        let bytes: Vec<u8> = (0..size_of::<PictureNames>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            bytes
                == crate::test_support::first_values::bytes("effects.owner.EFFECTS.ring.names/42"),
            "PICTURE_NAMES starts differently from the image"
        );
        assert_eq!(offset_of!(PictureNames, surfdata), 0x0);
        assert_eq!(offset_of!(PictureNames, surfmask), 0x15);
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
