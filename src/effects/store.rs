//! The effects' variables (IDA 0x764F40 .. 0x93CCC8), the whole of it the effects' (the
//! weather and surface effects: their settings, floats, work areas, the spline, shape,
//! square, windscreen drop, ring and particle tables). Nothing outside `effects` reads or
//! writes it but the one setting `race::frame_views` asks (`SETTINGS`, typed).
//!
//! Each piece that has its typed struct and static (`CARVED`: pong, clock, rain, waves,
//! clip, glow, mark slots, view vectors, snow, the settings pointer) has left the block,
//! with its own `moved::MOVED` entry; what is left is `SEGS`. The block's bytes under a
//! carved range are dead (a `Slot` there does not compile).
//!
//! One block with the image's layout (gaps and unreferenced bytes kept, so a slip that
//! reads past a table reads what the original read, and 16-byte alignment holds for the
//! 16-byte constants): an address `X` of IDA is at `STORE + (X - START)`. A `Slot` is such
//! an address (`Slot::at(0x814724)`), checked against the block when it is written. The x87
//! blocks (`effects::plain`) reach the block through `base()`: `mk + offset` with `offset`
//! IDA's address less `START`.
//!
//! Threads: the marks run on the game's thread alone, as the image's bytes did.

use crate::effects::store_data::RUNS;
use crate::errors::BadInput;
use crate::session::{GameThread, PlainBytes};

/// IDA's address of the block's first byte.
pub const START: usize = 0x764F40;
/// IDA's address past the block's last byte (the next is the zlib texts, 0x93CCC8).
pub const END: usize = 0x93CCC8;
pub const LEN: usize = END - START;

/// The block as one record of cells (`Bytes`, one run), with the image's first values, 16-byte
/// aligned as IDA's `START` is; the tail fills it to the alignment (no byte outside a cell).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct Store(PlainBytes<LEN>, PlainBytes<TAIL>);

const TAIL: usize = LEN.next_multiple_of(16) - LEN;
const _: () = assert!(START.is_multiple_of(16));

const fn initial() -> Store {
    let mut bytes = [0u8; LEN];
    let mut r = 0;
    while r < RUNS.len() {
        let (offset, run) = RUNS[r];
        let mut k = 0;
        while k < run.len() {
            bytes[offset + k] = run[k];
            k += 1;
        }
        r += 1;
    }
    Store(PlainBytes::new(bytes), PlainBytes::new([0; TAIL]))
}

/// The block (its first values are the image's bytes, `RUNS`).
pub static STORE: GameThread<Store> = GameThread::new(initial());

/// One segment of `SEGS` as a store of its own: its bytes in the block, found by offset.
pub struct Segment {
    /// The offset in the block of the segment's first byte.
    from: usize,
    len: usize,
}

// SAFETY: `first` and `size` name a part of the block's one run of shared bytes (`Bytes`).
#[allow(
    unsafe_code,
    reason = "Store's promise: a segment is bytes of the block's one run"
)]
unsafe impl crate::cells::Store for Segment {
    fn size(&self) -> usize {
        self.len
    }

    fn first(&self) -> *const u8 {
        self.first_byte()
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
        self.check(offset, to.len())?;
        crate::cells::load_cells(STORE.get(), self.from + offset, to)
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput> {
        self.check(offset, from.len())?;
        crate::cells::store_cells(STORE.get(), self.from + offset, from)
    }
}

impl Segment {
    fn check(&self, offset: usize, len: usize) -> Result<(), BadInput> {
        if offset.checked_add(len).is_some_and(|end| end <= self.len) {
            Ok(())
        } else {
            Err(BadInput("past the end of a segment of the marks' block"))
        }
    }

    fn first_byte(&self) -> *const u8 {
        STORE
            .unchecked()
            .0
            .as_ptr()
            .cast_const()
            .wrapping_add(self.from)
    }
}

const fn segment(n: usize) -> Segment {
    Segment {
        from: SEGS[n].0 - START,
        len: SEGS[n].1 - SEGS[n].0,
    }
}

/// The segments of `SEGS`, each the store `moved::MOVED` names for it.
pub static SEGMENTS: [Segment; SEGS.len()] = {
    let mut all = [const { Segment { from: 0, len: 0 } }; SEGS.len()];
    let mut n = 0;
    while n < SEGS.len() {
        all[n] = segment(n);
        n += 1;
    }
    all
};

/// The ranges that left the block, each its own typed static in the module that owns it
/// (phase 7; each with its own `moved::MOVED` entry). The block's bytes there are dead: a
/// `Slot` in one does not compile or stops the program, and `SEGS` (the rest of the block,
/// each with its own `MOVED` entry) leaves them out.
#[cfg(test)]
pub const CARVED: &[(usize, usize)] = &[
    (0x764F40, 0x764F7C), // `effects::shape_points::ShapeInputs`
    (0x764F90, 0x764FC8), // `effects::view_vectors::ViewSources`
    (0x764FC8, 0x764FFC), // `effects::setup::MarksClock`
    (0x765110, 0x765140), // `effects::tuning::MarksFrame`
    (0x765140, 0x765238), // `effects::tuning::MarksFloats`
    (0x765238, 0x7652A4), // `effects::tuning::MarksKindFloats`
    (0x767A28, 0x767A2C), // `effects::setup::SettingsPointer`
    (0x768658, 0x7687DC), // `effects::pong::PongState`
    (0x7687E4, 0x7688B4), // `effects::clip::ClipScreen`
    (0x769182, 0x769232), // `effects::pong::PongRules`
    (0x769232, 0x769236), // `effects::kind_values::MarksKind`
    (0x769236, 0x769312), // `effects::view_vectors::KeptView`
    (0x769312, 0x76A1D2), // `effects::splines::SplineHead`
    (0x76A1D2, 0x76B9B2), // `effects::splines::SplineWork`
    (0x76B9B2, 0x788BB2), // `effects::splines::SplineTables`
    (0x788BB2, 0x788EFA), // `effects::splines::SplineKinds`
    (0x788EFA, 0x788F0D), // `effects::splines::SplinePath`
    (0x788F0D, 0x788F69), // `effects::splines::SplineConstants`
    (0x809BC3, 0x811BC3), // `effects::edges::EdgeMask`
    (0x811BC3, 0x811BEB), // `effects::tilt_shape::ShapeTilt`
    (0x811BEB, 0x8122D3), // `effects::edges::KindBoxes`
    (0x814710, 0x814770), // `effects::windscreen_drop_state::WindscreenDropState`
    (0x814772, 0x81518A), // `effects::windscreen_drop_state::WindscreenDropWork`
    (0x816192, 0x82E342), // `effects::windscreen_drop_vars::WindscreenDropVars`
    (0x82E342, 0x844602), // `effects::mask_vars::MaskVars`
    (0x844604, 0x884E5C), // `effects::mask_picks::MaskPicks`
    (0x884E5C, 0x89BCA4), // `effects::shape_work::ShapeWork`
    (0x89C724, 0x89C994), // `effects::shape_points::ShapeSetTuning`
    (0x89C994, 0x89CA30), // `effects::mask_constants::MaskConstants`
    (0x89CA30, 0x89CA3C), // `effects::windscreen_flakes_store::WindscreenFlakeTiming`
    (0x89CACA, 0x89D16A), // `effects::windscreen_flakes_store::WindscreenFlakeTables`
    (0x89E18A, 0x8BC18A), // `effects::windscreen_flakes_store::WindscreenFlakeItems`
    (0x8BC18A, 0x8BC266), // `effects::windscreen_flakes_store::WindscreenFlakeState`
    (0x8BC39E, 0x8BC3FD), // `effects::edges::EdgeSettings`
    (0x8BC405, 0x8BDE55), // `effects::rain::RainState`
    (0x8C5F91, 0x8C5FFD), // `effects::rain::RainRules`
    (0x8C6000, 0x8D2000), // `effects::snow::SnowTables`
    (0x8D2000, 0x8D2310), // `effects::snow::SnowState`
    (0x8D2310, 0x8DA310), // `effects::snow::SnowCurve`
    (0x8DA410, 0x8DB430), // `effects::snow::SnowCurveVars`
    (0x8DB430, 0x8DB558), // `effects::snow::SnowLight`
    (0x8DB570, 0x8DB5C0), // `effects::blocks::PlacesBlock`
    (0x8DD061, 0x8DD865), // `effects::profiles::ProfileRings`
    (0x8E506D, 0x8EB86D), // `effects::profiles::ProfileTables`
    (0x8EB871, 0x8EB971), // `effects::frame_state::DrawWork`
    (0x8EB971, 0x8F3971), // `effects::blocks::MarkCorners`
    (0x8F3975, 0x8F39CD), // `effects::frame_state::ViewVars`
    (0x9152F6, 0x91532A), // `effects::profiles::ProfileLists`
    (0x91FF3E, 0x91FFE2), // `effects::frame_state::DrawFrame`
    (0x92001E, 0x92051E), // `effects::waves::MarksSines`
    (0x92051E, 0x9205F6), // `effects::frame_state::FlareParts`
    (0x9205F6, 0x92063A), // `effects::waves::WaveLevels`
    (0x9206C2, 0x9206D2), // `effects::waves::WaveState`
    (0x9206D2, 0x9206E6), // `effects::frame_state::FlareScales`
    (0x9206F0, 0x920734), // `effects::blocks::MarkLinks`
    (0x920734, 0x922B34), // `effects::blocks::CornerSlopes`
    (0x922B38, 0x922FE0), // `effects::run_particles::TrailColours`
    (0x923004, 0x923008), // `effects::spread_point::SpreadRandom`
    (0x9331B4, 0x9332AC), // `effects::kind_store::Quad1`
    (0x9332AC, 0x933E70), // `effects::kind_store::Tables1`
    (0x933E70, 0x933E7C), // `effects::kind_store::KindExtras1`
    (0x933E7C, 0x933ED0), // `effects::kind_store::Consts1`
    (0x933ED0, 0x933FC8), // `effects::kind_store::Quad2`
    (0x933FC8, 0x934B78), // `effects::kind_store::Tables2`
    (0x934B78, 0x934B7C), // `effects::kind_store::KindExtra2`
    (0x934B7C, 0x934BD0), // `effects::kind_store::Consts2`
    (0x934BD0, 0x934CC8), // `effects::kind_store::Quad3`
    (0x934CC8, 0x934D6E), // `effects::mask_store::GroundNames`
    (0x934D6E, 0x934E29), // `effects::mask_store::MaskBlocks`
    (0x934E29, 0x934EA8), // `effects::mask_store::PictureCopy`
    (0x934EA8, 0x934F30), // `effects::mask_store::KindPictures`
    (0x934F30, 0x934F7C), // `effects::draw_plane::PlaneTuning`
    (0x934F7C, 0x935300), // `effects::mark_slots::MarkSlots`
    (0x9353A6, 0x935412), // `effects::mark_slots::SlotTuning`
    (0x935416, 0x935562), // `effects::emitters::EmitterVars`
    (0x935620, 0x93573C), // `effects::ring_store::RingWork`
    (0x93573C, 0x935766), // `effects::ring_store::PictureNames`
    (0x935766, 0x939DB2), // `effects::surface_store::SurfaceTables`
    (0x939DB2, 0x93A1B6), // `effects::run_ribbons::Colours257`
    (0x93A1B6, 0x93A1D6), // `effects::emit_twin_particle_first::TwinState`
    (0x93A1D6, 0x93A1DA), // `effects::emit_rolling_particles::RollingChoice`
    (0x93A1DA, 0x93A21A), // `effects::set_up_surface::MaskList`
    (0x93A21A, 0x93A23A), // `effects::draw_turned_sprite::SpriteUv`
    (0x93A23A, 0x93A262), // `effects::emit_slip_spray::SlipSpraySet`
    (0x93A262, 0x93A312), // `effects::emit_slip_spray_long::SlipSprayLongSet`
    (0x93A312, 0x93A39A), // `effects::emit_rolling_particles_rated::RollingRatedSet`
    (0x93A39A, 0x93A3D2), // `effects::emit_rolling_particles::RollingSet`
    (0x93A3D2, 0x93A3F6), // `effects::emit_bursts::FirstTuning`
    (0x93A3F6, 0x93A41A), // `effects::emit_bursts_kind_3::BurstKind3Tuning`
    (0x93A41A, 0x93A462), // `effects::emit_wheel_fade_particle::WheelFadeSet`
    (0x93A462, 0x93A48A), // `effects::emit_exhaust_smoke::ExhaustSet`
    (0x93A48A, 0x93A4B2), // `effects::emit_twin_particle_first::TwinFirstTuning`
    (0x93A4B2, 0x93A4DA), // `effects::emit_twin_particle_second::TwinSecondTuning`
    (0x93A4DA, 0x93A576), // `effects::run_ring::RingKinds`
    (0x93A580, 0x93A5F0), // `effects::blocks::RingPool`
    (0x93A5F0, 0x93A618), // `effects::run_particles::PuffWork`
    (0x93A618, 0x93C618), // `effects::run_particles::BurstColours`
    (0x93C618, 0x93C6C8), // `effects::run_particles::ParticleTuning`
    (0x93C6C8, 0x93C6CC), // `effects::blend_corner::BlendWork`
    (0x93C6CC, 0x93C6F4), // `effects::morph_object_points::MorphWork`
    (0x93C738, 0x93C8C0), // `effects::lines::LinesWork`
    (0x93C8C0, 0x93C8D0), // `effects::run_frame::FrameK`
    (0x93CC10, 0x93CC18), // `effects::setup::ClockK`
    (0x93CC70, 0x93CCB0), // `effects::clip::GlowState`
];

/// What is left of the block after `CARVED`: (first address, address past the last).
/// `MOVED` has an entry for each, by index (`home_of`): a carve rewrites the segment it
/// cuts in place and appends the piece to the right of it, or puts it in the index of a
/// segment that was carved whole, so the order is not the addresses'.
pub const SEGS: &[(usize, usize)] = &[
    (0x764F7C, 0x764F90),
    (0x764FFC, 0x765110),
    (0x7652A4, 0x767A28),
    (0x767A2C, 0x768658),
    (0x7687DC, 0x7687E4),
    (0x7688B4, 0x769182),
    (0x788F69, 0x809BC3),
    (0x8122D3, 0x814710),
    (0x814770, 0x814772),
    (0x81518A, 0x816192),
    (0x844602, 0x844604),
    (0x89BCA4, 0x89C724),
    (0x89CA3C, 0x89CACA),
    (0x89D16A, 0x89E18A),
    (0x8BC266, 0x8BC39E),
    (0x8BC3FD, 0x8BC405),
    (0x8BDE55, 0x8C5F91),
    (0x8C5FFD, 0x8C6000),
    (0x8DA310, 0x8DA410),
    (0x8DB558, 0x8DB570),
    (0x8DB5C0, 0x8DD061),
    (0x8DD865, 0x8E506D),
    (0x8EB86D, 0x8EB871),
    (0x8F3971, 0x8F3975),
    (0x8F39CD, 0x9152F6),
    (0x91532A, 0x91FF3E),
    (0x91FFE2, 0x92001E),
    (0x92063A, 0x9206C2),
    (0x9206E6, 0x9206F0),
    (0x922B34, 0x922B38),
    (0x922FE0, 0x923004),
    (0x923008, 0x9331B4),
    (0x935300, 0x9353A6),
    (0x935412, 0x935416),
    (0x935562, 0x935620),
    (0x93A576, 0x93A580),
    (0x93C6F4, 0x93C738),
    (0x93C8D0, 0x93CC10),
    (0x93CC18, 0x93CC70),
    (0x93CCB0, END),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_and_carved_ranges_partition_the_block() {
        let mut all: Vec<(usize, usize)> = SEGS.iter().chain(CARVED).copied().collect();
        all.sort();
        assert_eq!(all[0].0, START);
        assert_eq!(all.last().unwrap().1, END);
        for w in all.windows(2) {
            assert_eq!(
                w[0].1, w[1].0,
                "{:#x} .. {:#x} and the next",
                w[0].0, w[0].1
            );
        }
    }
}
