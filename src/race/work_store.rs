//! The race's work variables after the work car: the cars' blocks and the current car, the stage's
//! direction, the cars' number textures, the loose parts' slots, the computer driver's state, the
//! credits' race, the network's message, the `patch*` settings, the stages' checkpoint counts.
//! Named from the code's uses; most names are readings, and `unknown_*` runs are places no code
//! names. `#[repr(C)]`, the unused runs at either end and in the middle left out.

use crate::errors::BadInput;
use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFlag, PlainFloat};

/// A loose part flying off a car (0x78 bytes, `physics::fill_race_slot`; field names are readings).
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceSlot {
    /// The part (0 the slot empty).
    pub part: PlainDword,
    /// Its place, whole numbers.
    pub place: [PlainDword; 3],
    /// Its matrix.
    pub matrix: [PlainFloat; 16],
    /// Its motion.
    pub motion: [PlainFloat; 3],
    /// Added to the part's height when it is drawn (`race::frame_draw::scene_objects`); no
    /// ported code writes it (0 from the start).
    pub draw_height_offset: PlainDword,
    /// The car's `car_word` when it broke off.
    pub car_word: PlainDword,
    /// The car's step clock when it broke off (the lowest is reused).
    pub age: PlainDword,
    /// Its spin.
    pub spin: [PlainFloat; 3],
    /// The spin's length.
    pub spin_length: PlainFloat,
}

const _: () = assert!(core::mem::size_of::<RaceSlot>() == 0x78);

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceWork {
    /// The eight cars' blocks (`Car`, by handle:
    /// `blocks_store::block_handle`).
    pub car_blocks: [PlainDword; 8],
    /// A handle for each car's view marks record (`race::frame_views`; see
    /// `car_record_number`).
    pub car_records: [PlainDword; 8],
    /// The car being stepped.
    pub current_car: PlainDword,
    /// The stage run forwards or backwards ("dir").
    pub stage_direction: PlainDword,
    /// The current car's block by handle (`drive::race_cars_step`, the
    /// replay; `blocks_store::block_handle`).
    pub work_block: PlainDword,
    /// 32 names of 0x30 bytes by car kind: the cars' number textures (`race::car_draw`).
    pub number_texture_names: [PlainDword; 384],
    /// The drawn car's nearness (`race::car_draw`); now each view's
    /// (`ctx.view.cars.car_nearness`); unused.
    pub car_nearness: PlainFloat,
    /// A word of the drawn car (`race::car_draw`); now each view's
    /// (`ctx.view.cars.car_draw_word`); unused.
    pub car_draw_word: PlainDword,
    /// The number's tens' textures (`race::car_draw`).
    pub number_tens_textures: [PlainDword; 7],
    /// The parts' break limits (`physics::drop_broken_parts`).
    pub part_break_limits: [PlainDword; 7],
    /// The matrix a part at rest takes (`physics::move_slot`).
    pub part_matrix_still: [PlainDword; 16],
    /// The matrix a part lying flat takes (`physics::move_slot`).
    pub part_matrix_flat: [PlainDword; 16],
    /// The loose parts flying off the cars (`physics::fill_race_slot`).
    pub race_slots: [RaceSlot; 8],
    /// The slot count (`physics::move_slots`).
    pub slot_count: PlainDword,
    /// The parts' throw (`physics::fill_race_slot`).
    pub part_fling: PlainFloat,
    /// The place kept (`physics::move_slot`).
    pub slot_place: [PlainDword; 3],
    /// The computer driver's speed by bend (`drive::drive_computer_car`).
    pub bend_speeds: [PlainFloat; 32],
    /// Three tables of seven: a wanted speed by the bend's turn.
    pub wanted_speeds: [PlainDword; 21],
    /// The driver's tilt limit (`drive::drive_computer_car`, `drive::read_car_controls`).
    pub driver_tilt_limit: PlainFloat,
    /// The driver's vector (`drive::drive_computer_car`).
    pub driver_vector: [PlainFloat; 4],
    /// The replay's extra start piece (`drive::save_replay_cars`).
    pub replay_extra: PlainDword,
    /// The computer driver's steering.
    pub driver_steer: PlainDword,
    /// Its throttle (0x100 full).
    pub driver_throttle: PlainDword,
    /// Its brake.
    pub driver_brake: PlainDword,
    /// Its other controls (1 on).
    pub driver_flags: [PlainDword; 4],
    /// The driver's start blend (`drive::read_car_controls`).
    pub driver_start_blend: PlainFloat,
    /// Steps by car (`drive::drive_computer_car`).
    pub driver_steps: [PlainDword; 10],
    /// Reversing steps left by car.
    pub driver_reverse_left: [PlainDword; 10],
    /// The distance moved.
    pub driver_moved: PlainDword,
    /// The bend's turn.
    pub driver_turn: PlainDword,
    /// The bend's index.
    pub driver_bend_index: PlainDword,
    /// The speed wanted.
    pub driver_wanted_speed: PlainDword,
    /// The speed.
    pub driver_speed: PlainDword,
    /// A bend noted by car: where it ends.
    pub driver_bend_to: [PlainDword; 10],
    /// A bend noted by car.
    pub driver_bend_noted: [PlainDword; 10],
    /// Reversing tries by car.
    pub driver_reverse_tries: [PlainDword; 10],
    /// The noted bend's turn by car.
    pub driver_bend_turn: [PlainDword; 10],
    /// A driver flag (`drive::drive_computer_car`).
    pub driver_flag_bc: PlainDword,
    /// The step along x.
    pub driver_step_x: PlainDword,
    /// The step along z.
    pub driver_step_z: PlainDword,
    /// From `bend_speeds`.
    pub driver_speed_scale: PlainFloat,
    /// The car driven (`current_car` copied).
    pub driver_slot: PlainDword,
    /// 1 when looking ahead.
    pub driver_lookahead: PlainDword,
    /// A bend value (`drive::drive_computer_car`).
    pub driver_bend_a: PlainDword,
    /// Another bend value (`drive::drive_computer_car`).
    pub driver_bend_b: PlainDword,
    /// Its point in `driver_points_now` (a byte offset).
    pub driver_point_now: PlainDword,
    /// Its point in `driver_points_then` (a byte offset).
    pub driver_point_then: PlainDword,
    /// 1: aim ahead.
    pub driver_aim: PlainDword,
    /// Ten cars' points of 16 bytes.
    pub driver_points_now: [PlainDword; 40],
    /// Ten cars' points of 16 bytes, the step before.
    pub driver_points_then: [PlainDword; 40],
    /// The driver's start point (x, 0, z).
    pub driver_from: [PlainDword; 4],
    /// The driver's target point (x, 0, z).
    pub driver_to: [PlainDword; 4],
    /// The way from here to `driver_to` on the flat.
    pub driver_way: [PlainFloat; 4],
    /// It normalised.
    pub driver_way_unit: [PlainFloat; 4],
    /// The way ahead normalised.
    pub driver_ahead: [PlainFloat; 4],
    /// `driver_from` to `driver_to` on the flat.
    pub driver_flat: [PlainFloat; 4],
    /// It normalised.
    pub driver_flat_unit: [PlainFloat; 4],
    /// The car's side.
    pub driver_side: [PlainFloat; 4],
    /// The driver's work values (`drive::drive_computer_car`).
    pub driver_work: [PlainDword; 47],
    /// Kept by `menu::scale_stage_times`.
    pub stage_time_scales: [PlainFloat; 2],
    /// The credits' race on.
    pub in_credits: PlainFlag,
    /// The credits' stages.
    pub credit_stages: [PlainDword; 36],
    /// The credits' car kinds.
    pub credit_car_kinds: [PlainDword; 23],
    /// The network's message built (`net::send_inputs`).
    pub net_message: [PlainDword; 128],
    /// The physics patches as the settings file gives them: `patch_switch`
    /// (`fizpatch`), then `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`,
    /// `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`,
    /// `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel`.
    pub patches: [PlainDword; 13],
    /// The last ten settings / 100 (`physics::apply_patches`).
    pub patch_factors: [PlainFloat; 10],
    /// Whether the patches are applied.
    pub patches_applied: PlainDword,
    /// The checkpoint counts by stage (`saves::default_times`).
    pub checkpoint_counts: [PlainDword; 36],
    /// The guessed checkpoint counts.
    pub guessed_checkpoints: [PlainDword; 16],
    /// The `noprecipitation` setting.
    pub no_precipitation: PlainDword,
}

impl RaceWork {
    /// The first of the patches.
    pub fn patch_switch(&self) -> &PlainDword {
        &self.patches[0]
    }

    /// The twelve patch settings after the switch.
    pub fn patch_settings(&self) -> &[PlainDword] {
        &self.patches[1..]
    }

    /// Car `n`'s block by handle (`car_blocks[n]`: `blocks_store::block_handle`, 0 for none). A number
    /// past the eight is read on through the variables that follow the table (`past_the_eight`).
    /// A handle among the eight that names no block ends the program.
    pub fn car_block(&self, n: usize) -> u32 {
        if let Some(cell) = self.car_blocks.get(n) {
            let handle = cell.get();
            crate::race::blocks_store::block_number(handle);
            return handle;
        }
        self.past_the_eight(n - self.car_blocks.len())
            .unwrap_or_else(|e| crate::fatal::raise::end("a car block's pointer", e))
    }

    /// The dword `k` places after the table of car blocks, which is what `car_blocks[8 + k]`
    /// reads: the eight car records' handles, `current_car`, a dword nothing uses (0), the stage's
    /// direction, the current car's block (a handle too), then the number texture names. A
    /// number past those would be a car number of nearly 400.
    fn past_the_eight(&self, k: usize) -> Result<u32, BadInput> {
        let records = self.car_records.len();
        let value = match k {
            _ if k < records => self.car_records[k].get(),
            _ if k == records => self.current_car.get(),
            _ if k == records + 1 => 0,
            _ if k == records + 2 => self.stage_direction.get(),
            _ if k == records + 3 => self.work_block.get(),
            _ => self
                .number_texture_names
                .get(k - (records + 4))
                .ok_or(BadInput("a car block's number past the work's variables"))?
                .get(),
        };
        Ok(value)
    }

    /// Car `n`'s block as `car_blocks[n]` keeps it: its handle (0 for none), the value other
    /// places keep for "that car" (`block_ref`).
    pub fn car_handle(&self, n: usize) -> u32 {
        self.car_blocks
            .get(n)
            .map_or_else(|| self.car_block(n), PlainDword::get)
    }

    /// The number of the block car `n` has, refused for none (as `frame_view::car_at`).
    pub fn block_number(&self, n: usize) -> Result<usize, BadInput> {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        if let Some(cell) = self.car_blocks.get(n) {
            let k = (cell.get() ^ BLOCK_TAG) as usize;
            if k < BLOCK_COUNT {
                return Ok(k);
            }
        }
        crate::race::blocks_store::known_block(self.car_block(n))
            .ok_or(BadInput("no car block's place for a snapshot"))
    }

    /// Whether car `n` has a block (`car_block(n) != 0`; a handle that names none ends the
    /// program).
    pub fn has_car_block(&self, n: usize) -> bool {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        match self.car_blocks.get(n).map(PlainDword::get) {
            Some(0) => false,
            Some(h) if ((h ^ BLOCK_TAG) as usize) < BLOCK_COUNT => true,
            _ => self.car_block(n) != 0,
        }
    }

    /// The number of car record `n` (`car_records[n]`, the handle of one of the effects' car
    /// records, `race::effects_settings_store`; `RaceShared::car_record`). Refused past the eight
    /// and for a value that is no record's handle.
    pub(super) fn car_record_number(&self, n: usize) -> Result<usize, BadInput> {
        let name = self
            .car_records
            .get(n)
            .ok_or(BadInput("a car record past the eight"))?
            .get() as usize;
        let number = name.wrapping_sub(CAR_RECORD_TAG as usize);
        if number >= CAR_RECORDS {
            return Err(BadInput("a car record off the view marks' records"));
        }
        Ok(number)
    }
}

/// A car record by handle: `CAR_RECORD_TAG | n` for record `n` of
/// `effects_settings_store::EffectsSettings::cars`. No car block's handle has this tag.
const CAR_RECORD_TAG: u32 = 0xCE00_0000;
const CAR_RECORDS: usize = 8;

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceWork {
    /// The starting state: the car blocks named, the computer driver's tables, the credits' race,
    /// the `patch*` settings at their defaults and the stages' checkpoint counts; the rest is
    /// zero.
    pub const fn new() -> Self {
        RaceWork {
            car_blocks: block_handles(),
            // The eight car records, one after the other.
            car_records: car_record_names(),
            number_texture_names: number_texture_names(),
            number_tens_textures: dwords([0, 6, 3, 1, 5, 2, 4]),
            part_break_limits: dwords([0x28, 0x3c, 0x3c, 0x14, 0x14, 0x4b, 0x3c]),
            // The matrices a part takes lying still and lying flat.
            part_matrix_still: dwords([
                0,
                F32_MINUS_ONE,
                0,
                0,
                F32_ONE,
                0,
                0,
                0,
                0,
                0,
                F32_ONE,
                0,
                0,
                0,
                0,
                0,
            ]),
            part_matrix_flat: dwords([
                F32_ONE, 0, 0, 0, 0, F32_ONE, 0, 0, 0, 0, F32_ONE, 0, 0, 0, 0, 0,
            ]),
            part_fling: PlainFloat::new(6e-05),
            // The computer driver's speed by bend: a triangle from -1 up to 1 and back.
            bend_speeds: floats([
                -1.0, -0.9, -0.8, -0.7, -0.5, -0.3, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 0.8,
                0.9, 1.0, 0.9, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2, -0.3, -0.5, -0.7,
                -0.8, -0.9,
            ]),
            // Three tables of seven speeds (four to a word).
            wanted_speeds: dwords([
                0xfafafafa, 0xfafafafa, 0xc8fafafa, 0x5a646496, 0x46464646, 0x323c3c3c, 0x1e283232,
                0xfafafafa, 0xfafafafa, 0xfafafafa, 0x323c4664, 0x32323232, 0x28282832, 0x1414141e,
                0xfafafafa, 0xfafafafa, 0x506efafa, 0x323c4646, 0x32323232, 0x1e1e2828, 0x1414141e,
            ]),
            driver_tilt_limit: PlainFloat::new(0.2),
            driver_vector: floats([0.0, 1.0, 0.0, 0.0]),
            replay_extra: PlainDword::new(1),
            credit_stages: dwords([
                0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x29, 0x2a,
                0x2b, 0x2c, 0x2d, 0x2e, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x3d, 0x3e, 0x3f, 0x40,
                0x41, 0x42, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c,
            ]),
            credit_car_kinds: credit_car_kinds(),
            patches: dwords([
                0, 0xc, 0, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64,
            ]),
            patch_factors: floats([1.0; 10]),
            checkpoint_counts: dwords([
                9, 0xb, 0xc, 8, 0xc, 0xd, 0xb, 8, 9, 0xa, 0xe, 0xe, 0xb, 0xb, 0xd, 0xe, 0xd, 0xc,
                0xc, 0xc, 0xc, 0xb, 0xa, 0xb, 0x10, 0xe, 0xe, 0xf, 0xa, 0xe, 8, 9, 9, 9, 4, 0xa,
            ]),
            ..ZeroCells::ZERO
        }
    }
}

/// `car_blocks` at the start: the eight cars' blocks, by handle.
const fn block_handles() -> [PlainDword; 8] {
    let mut out = [const { PlainDword::new(0) }; 8];
    let mut k = 0;
    while k < 8 {
        out[k] = PlainDword::new(crate::race::blocks_store::block_handle(k));
        k += 1;
    }
    out
}

/// `car_records` at the start: the handles of the eight car records in order.
const fn car_record_names() -> [PlainDword; 8] {
    let mut out = [const { PlainDword::new(0) }; 8];
    let mut k = 0;
    while k < 8 {
        out[k] = PlainDword::new(CAR_RECORD_TAG | k as u32);
        k += 1;
    }
    out
}

/// The credits' race drives every car kind once, in order.
const fn credit_car_kinds() -> [PlainDword; 23] {
    let mut out = [const { PlainDword::new(0) }; 23];
    let mut k = 0;
    while k < 23 {
        out[k] = PlainDword::new(k as u32);
        k += 1;
    }
    out
}

/// The names of the number textures, 48 characters for each of the 32 car kinds.
const NUMBER_NAMES: [&[u8; 48]; 32] = [
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00001111222233334444555566778899aabbbbbbbbbbdddd",
    b"0000000000112222333344445555667789abcddddddddddd",
    b"0000000011112222333344445555667789abcdeeeeeeeeee",
    b"0000000000000011223344556789abcdeeeeeeeeeeeeeeee",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"000000000000000000000000000000000000000000000000",
    b"000000000000000000000000000000000000000000000000",
    b"00001111222233334444555566778899aabbcccccccccccc",
    b"000011112222333344445555789a8899aabbbbbbbbbbbbbb",
    b"000000000000000000000000000000000000000000000000",
    b"000000000000000000000000000000000000000000000000",
    b"000000000000000000000000000000000000000000000000",
    b"00001111222233334444555566778899abbbbbbbbbbbbbbb",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"000000000000000000000000000000000000000000000000",
    b"0000000011112222333344445566667789abcdeeeeeeeeee",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"000000000000000000000000000000000000000000000000",
    b"000000000000000000000000000000000000000000000000",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"000000000000000000000000000000000000000000000000",
    b"000000000000000000000000000000000000000000000000",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
    b"00000000000011112222333344445566789abcdddddddddd",
];

const fn number_texture_names() -> [PlainDword; 384] {
    let mut out = [const { PlainDword::new(0) }; 384];
    let mut name = 0;
    while name < 32 {
        let text = NUMBER_NAMES[name];
        let mut word = 0;
        while word < 12 {
            out[name * 12 + word] = PlainDword::new(u32::from_le_bytes([
                text[4 * word],
                text[4 * word + 1],
                text[4 * word + 2],
                text[4 * word + 3],
            ]));
            word += 1;
        }
        name += 1;
    }
    out
}

const fn dwords<const N: usize>(values: [u32; N]) -> [PlainDword; N] {
    let mut out = [const { PlainDword::new(0) }; N];
    let mut k = 0;
    while k < N {
        out[k] = PlainDword::new(values[k]);
        k += 1;
    }
    out
}

const fn floats<const N: usize>(values: [f32; N]) -> [PlainFloat; N] {
    let mut out = [const { PlainFloat::new(0.0) }; N];
    let mut k = 0;
    while k < N {
        out[k] = PlainFloat::new(values[k]);
        k += 1;
    }
    out
}

const F32_ONE: u32 = 1.0f32.to_bits();
const F32_MINUS_ONE: u32 = (-1.0f32).to_bits();

#[cfg(test)]
mod tests {

    #[test]
    fn car_records_resolve_to_their_places() {
        for k in 0..8 {
            assert!(std::ptr::eq(
                crate::test_support::own::race().car_records().unwrap()[k],
                crate::test_support::own::race().car_record(k).unwrap()
            ));
        }
        assert!(crate::test_support::own::race().car_record(8).is_err());
    }

    /// `car_block` past the eight reads the variables that follow the table, in their order.
    #[test]
    fn the_car_block_past_the_eight_reads_on_by_name() {
        let _g = crate::test_support::test_lock();
        let work = crate::test_support::own::race().shared.work.get();
        let saved = (
            work.car_records[3].get(),
            work.current_car.get(),
            work.stage_direction.get(),
            work.work_block.get(),
            work.number_texture_names[5].get(),
        );
        work.car_records[3].set(0x11);
        work.current_car.set(0x22);
        work.stage_direction.set(0x33);
        work.work_block.set(0x44);
        work.number_texture_names[5].set(0x55);
        assert_eq!(work.past_the_eight(3).unwrap(), 0x11);
        assert_eq!(work.past_the_eight(8).unwrap(), 0x22);
        assert_eq!(work.past_the_eight(9).unwrap(), 0);
        assert_eq!(work.past_the_eight(10).unwrap(), 0x33);
        assert_eq!(work.past_the_eight(11).unwrap(), 0x44);
        assert_eq!(work.past_the_eight(12 + 5).unwrap(), 0x55);
        assert!(work.past_the_eight(12 + 384).is_err());
        work.car_records[3].set(saved.0);
        work.current_car.set(saved.1);
        work.stage_direction.set(saved.2);
        work.work_block.set(saved.3);
        work.number_texture_names[5].set(saved.4);
    }
}
