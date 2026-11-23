//! The race's work variables after the work car: the cars' blocks and the current car, the stage's
//! direction, the cars' number textures, the loose parts' slots, the computer driver's state, the
//! credits' race, the network's message, the `patch*` settings, the stages' checkpoint counts.
//! Named from the code's uses; most names are readings, and `reserved_*` runs are places no code
//! names. `#[repr(C)]`, the unused runs at either end and in the middle left out.

use crate::errors::BadInput;

/// A loose part flying off a car (0x78 bytes, `physics::fill_race_slot`; field names are readings).
#[cfg_attr(test, repr(C))]
#[derive(Clone, rc2k_derive::PlainRecord)]
pub struct RaceSlot {
    /// The part (0 the slot empty).
    pub part: u32,
    /// Its place, whole numbers.
    pub place: [u32; 3],
    /// Its matrix.
    pub matrix: [f32; 16],
    /// Its motion.
    pub motion: [f32; 3],
    /// Added to the part's height when it is drawn (`race::frame_draw::scene_objects`); no
    /// ported code writes it (0 from the start).
    pub draw_height_offset: u32,
    /// The car's `car_word` when it broke off.
    pub car_word: u32,
    /// The car's step clock when it broke off (the lowest is reused).
    pub age: u32,
    /// Its spin.
    pub spin: [f32; 3],
    /// The spin's length.
    pub spin_length: f32,
}

impl RaceSlot {
    /// An empty slot.
    pub const ZERO: Self = Self {
        part: 0,
        place: [0; 3],
        matrix: [0.0; 16],
        motion: [0.0; 3],
        draw_height_offset: 0,
        car_word: 0,
        age: 0,
        spin: [0.0; 3],
        spin_length: 0.0,
    };
}

const _: () = assert!(core::mem::size_of::<RaceSlot>() == 0x78);

#[cfg_attr(test, repr(C))]
#[derive(Clone, rc2k_derive::PlainRecord)]
pub struct RaceWork {
    /// The eight cars' blocks (`Car`, by handle:
    /// `blocks_store::block_handle`).
    pub car_blocks: [u32; 8],
    /// A handle for each car's view marks record (`race::frame_views`; see
    /// `car_record_number`).
    pub car_records: [u32; 8],
    /// The car being stepped.
    pub current_car: u32,
    /// The stage run forwards or backwards ("dir").
    pub stage_direction: u32,
    /// The current car's block by handle (`drive::race_cars_step`, the
    /// replay; `blocks_store::block_handle`).
    pub work_block: u32,
    /// 32 names of 0x30 bytes by car kind: the cars' number textures (`race::car_draw`).
    pub number_texture_names: [u32; 384],
    /// The drawn car's nearness (`race::car_draw`); now each view's
    /// (`ctx.view.cars.car_nearness`); unused.
    pub car_nearness: f32,
    /// A word of the drawn car (`race::car_draw`); now each view's
    /// (`ctx.view.cars.car_draw_word`); unused.
    pub car_draw_word: u32,
    /// The number's tens' textures (`race::car_draw`).
    pub number_tens_textures: [u32; 7],
    /// The parts' break limits (`physics::drop_broken_parts`).
    pub part_break_limits: [u32; 7],
    /// The matrix a part at rest takes (`physics::move_slot`).
    pub part_matrix_still: [u32; 16],
    /// The matrix a part lying flat takes (`physics::move_slot`).
    pub part_matrix_flat: [u32; 16],
    /// The loose parts flying off the cars (`physics::fill_race_slot`).
    pub race_slots: [RaceSlot; 8],
    /// The slot count (`physics::move_slots`).
    pub slot_count: u32,
    /// The parts' throw (`physics::fill_race_slot`).
    pub part_fling: f32,
    /// The place kept (`physics::move_slot`).
    pub slot_place: [u32; 3],
    /// The computer driver's speed by bend (`drive::drive_computer_car`).
    pub bend_speeds: [f32; 32],
    /// Three tables of seven: a wanted speed by the bend's turn.
    pub wanted_speeds: [u32; 21],
    /// The driver's tilt limit (`drive::drive_computer_car`, `drive::read_car_controls`).
    pub driver_tilt_limit: f32,
    /// The driver's vector (`drive::drive_computer_car`).
    pub driver_vector: [f32; 4],
    /// The replay's extra start piece (`drive::save_replay_cars`).
    pub replay_extra: u32,
    /// The computer driver's steering.
    pub driver_steer: u32,
    /// Its throttle (0x100 full).
    pub driver_throttle: u32,
    /// Its brake.
    pub driver_brake: u32,
    /// Its other controls (1 on).
    pub driver_flags: [u32; 4],
    /// The driver's start blend (`drive::read_car_controls`).
    pub driver_start_blend: f32,
    /// Steps by car (`drive::drive_computer_car`).
    pub driver_steps: [u32; 10],
    /// Reversing steps left by car.
    pub driver_reverse_left: [u32; 10],
    /// The distance moved.
    pub driver_moved: u32,
    /// The bend's turn.
    pub driver_turn: u32,
    /// The bend's index.
    pub driver_bend_index: u32,
    /// The speed wanted.
    pub driver_wanted_speed: u32,
    /// The speed.
    pub driver_speed: u32,
    /// A bend noted by car: where it ends.
    pub driver_bend_to: [u32; 10],
    /// A bend noted by car.
    pub driver_bend_noted: [u32; 10],
    /// Reversing tries by car.
    pub driver_reverse_tries: [u32; 10],
    /// The noted bend's turn by car.
    pub driver_bend_turn: [u32; 10],
    /// A driver flag (`drive::drive_computer_car`).
    pub driver_flag_bc: u32,
    /// The step along x.
    pub driver_step_x: u32,
    /// The step along z.
    pub driver_step_z: u32,
    /// From `bend_speeds`.
    pub driver_speed_scale: f32,
    /// The car driven (`current_car` copied).
    pub driver_slot: u32,
    /// 1 when looking ahead.
    pub driver_lookahead: u32,
    /// A bend value (`drive::drive_computer_car`).
    pub driver_bend_a: u32,
    /// Another bend value (`drive::drive_computer_car`).
    pub driver_bend_b: u32,
    /// Its point in `driver_points_now` (a byte offset).
    pub driver_point_now: u32,
    /// Its point in `driver_points_then` (a byte offset).
    pub driver_point_then: u32,
    /// 1: aim ahead.
    pub driver_aim: u32,
    /// Ten cars' points of 16 bytes.
    pub driver_points_now: [u32; 40],
    /// Ten cars' points of 16 bytes, the step before.
    pub driver_points_then: [u32; 40],
    /// The driver's start point (x, 0, z).
    pub driver_from: [u32; 4],
    /// The driver's target point (x, 0, z).
    pub driver_to: [u32; 4],
    /// The way from here to `driver_to` on the flat.
    pub driver_way: [f32; 4],
    /// It normalised.
    pub driver_way_unit: [f32; 4],
    /// The way ahead normalised.
    pub driver_ahead: [f32; 4],
    /// `driver_from` to `driver_to` on the flat.
    pub driver_flat: [f32; 4],
    /// It normalised.
    pub driver_flat_unit: [f32; 4],
    /// The car's side.
    pub driver_side: [f32; 4],
    /// The driver's work values (`drive::drive_computer_car`).
    pub driver_work: [u32; 47],
    /// Kept by `menu::scale_stage_times`.
    pub stage_time_scales: [f32; 2],
    /// The credits' race on.
    pub in_credits: u32,
    /// The credits' stages.
    pub credit_stages: [u32; 36],
    /// The credits' car kinds.
    pub credit_car_kinds: [u32; 23],
    /// The network's message built (`net::send_inputs`).
    pub net_message: [u32; 128],
    /// The physics patches as the settings file gives them: `patch_switch`
    /// (`fizpatch`), then `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`,
    /// `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`,
    /// `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel`.
    pub patches: [u32; 13],
    /// The last ten settings / 100 (`physics::apply_patches`).
    pub patch_factors: [f32; 10],
    /// Whether the patches are applied.
    pub patches_applied: u32,
    /// The checkpoint counts by stage (`saves::default_times`).
    pub checkpoint_counts: [u32; 36],
    /// The guessed checkpoint counts.
    pub guessed_checkpoints: [u32; 16],
    /// The `noprecipitation` setting.
    pub no_precipitation: u32,
}

impl RaceWork {
    /// The first of the patches.
    pub fn patch_switch(&self) -> u32 {
        self.patches[0]
    }

    /// The twelve patch settings after the switch.
    pub fn patch_settings(&self) -> &[u32] {
        &self.patches[1..]
    }

    /// Car `n`'s block by handle (`car_blocks[n]`: `blocks_store::block_handle`, 0 for none). A number
    /// past the eight is read on through the variables that follow the table (`past_the_eight`).
    /// A handle among the eight that names no block ends the program.
    pub fn car_block(&self, n: usize) -> u32 {
        if let Some(&handle) = self.car_blocks.get(n) {
            crate::race::blocks_store::block_number(handle);
            return handle;
        }
        self.past_the_eight(n - self.car_blocks.len())
            .unwrap_or_else(|e| crate::memory::host_hooks::end("a car block's pointer", e))
    }

    /// The dword `k` places after the table of car blocks, which is what `car_blocks[8 + k]`
    /// reads: the eight car records' handles, `current_car`, a dword nothing uses (0), the stage's
    /// direction, the current car's block (a handle too), then the number texture names. A
    /// number past those would be a car number of nearly 400.
    fn past_the_eight(&self, k: usize) -> Result<u32, BadInput> {
        let records = self.car_records.len();
        let value = match k {
            _ if k < records => self.car_records[k],
            _ if k == records => self.current_car,
            _ if k == records + 1 => 0,
            _ if k == records + 2 => self.stage_direction,
            _ if k == records + 3 => self.work_block,
            _ => self
                .number_texture_names
                .get(k - (records + 4))
                .ok_or(BadInput("a car block's number past the work's variables"))?
                .to_owned(),
        };
        Ok(value)
    }

    /// Car `n`'s block as `car_blocks[n]` keeps it: its handle (0 for none), the value other
    /// places keep for "that car" (`block_ref`).
    pub fn car_handle(&self, n: usize) -> u32 {
        self.car_blocks
            .get(n)
            .map_or_else(|| self.car_block(n), |&handle| handle)
    }

    /// The number of the block car `n` has, refused for none (as `frame_view::car_at`).
    pub fn block_number(&self, n: usize) -> Result<usize, BadInput> {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        if let Some(&handle) = self.car_blocks.get(n) {
            let k = (handle ^ BLOCK_TAG) as usize;
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
        match self.car_blocks.get(n).copied() {
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
            .to_owned() as usize;
        let number = name.wrapping_sub(CAR_RECORD_TAG as usize);
        if number >= CAR_RECORDS {
            return Err(BadInput("a car record off the view marks' records"));
        }
        Ok(number)
    }
}

/// The race's work variables as one reader finds them (`memory::drawn_table`).
pub type RaceWorkRef<'a> = crate::memory::drawn_table::DrawnTable<'a, RaceWork>;

/// The work variables borrowed from a `RaceWorkRef`: read through `Deref`.
pub enum WorkGuard<'a> {
    Store(std::cell::Ref<'a, RaceWork>),
    Copy(&'a RaceWork),
}

impl std::ops::Deref for WorkGuard<'_> {
    type Target = RaceWork;

    fn deref(&self) -> &RaceWork {
        match self {
            Self::Store(work) => work,
            Self::Copy(work) => work,
        }
    }
}

impl<'a> RaceWorkRef<'a> {
    /// The work variables for a short read (the store's own, on the game's thread, must not be
    /// written while the guard lives).
    pub fn borrow(self) -> WorkGuard<'a> {
        match self {
            Self::Store(store) => WorkGuard::Store(store.borrow()),
            Self::Copy(work) => WorkGuard::Copy(work),
        }
    }
}

impl RaceWorkRef<'_> {
    /// `RaceWork::has_car_block`.
    pub fn has_car_block(self, n: usize) -> bool {
        self.read(|work| work.has_car_block(n))
    }

    /// `RaceWork::block_number`.
    pub fn block_number(self, n: usize) -> Result<usize, BadInput> {
        self.read(|work| work.block_number(n))
    }

    /// Whether the credits' race is on.
    pub fn in_credits(self) -> bool {
        self.read(|work| work.in_credits != 0)
    }
}

/// A car record by handle: `CAR_RECORD_TAG | n` for record `n` of
/// `effects_settings_store::EffectsSettings::cars`. No car block's handle has this tag.
const CAR_RECORD_TAG: u32 = 0xCE00_0000;
const CAR_RECORDS: usize = 8;

impl RaceWork {
    /// Every field zero.
    const ZERO: Self = Self {
        car_blocks: [0; 8],
        car_records: [0; 8],
        current_car: 0,
        stage_direction: 0,
        work_block: 0,
        number_texture_names: [0; 384],
        car_nearness: 0.0,
        car_draw_word: 0,
        number_tens_textures: [0; 7],
        part_break_limits: [0; 7],
        part_matrix_still: [0; 16],
        part_matrix_flat: [0; 16],
        race_slots: [RaceSlot::ZERO; 8],
        slot_count: 0,
        part_fling: 0.0,
        slot_place: [0; 3],
        bend_speeds: [0.0; 32],
        wanted_speeds: [0; 21],
        driver_tilt_limit: 0.0,
        driver_vector: [0.0; 4],
        replay_extra: 0,
        driver_steer: 0,
        driver_throttle: 0,
        driver_brake: 0,
        driver_flags: [0; 4],
        driver_start_blend: 0.0,
        driver_steps: [0; 10],
        driver_reverse_left: [0; 10],
        driver_moved: 0,
        driver_turn: 0,
        driver_bend_index: 0,
        driver_wanted_speed: 0,
        driver_speed: 0,
        driver_bend_to: [0; 10],
        driver_bend_noted: [0; 10],
        driver_reverse_tries: [0; 10],
        driver_bend_turn: [0; 10],
        driver_flag_bc: 0,
        driver_step_x: 0,
        driver_step_z: 0,
        driver_speed_scale: 0.0,
        driver_slot: 0,
        driver_lookahead: 0,
        driver_bend_a: 0,
        driver_bend_b: 0,
        driver_point_now: 0,
        driver_point_then: 0,
        driver_aim: 0,
        driver_points_now: [0; 40],
        driver_points_then: [0; 40],
        driver_from: [0; 4],
        driver_to: [0; 4],
        driver_way: [0.0; 4],
        driver_way_unit: [0.0; 4],
        driver_ahead: [0.0; 4],
        driver_flat: [0.0; 4],
        driver_flat_unit: [0.0; 4],
        driver_side: [0.0; 4],
        driver_work: [0; 47],
        stage_time_scales: [0.0; 2],
        in_credits: 0,
        credit_stages: [0; 36],
        credit_car_kinds: [0; 23],
        net_message: [0; 128],
        patches: [0; 13],
        patch_factors: [0.0; 10],
        patches_applied: 0,
        checkpoint_counts: [0; 36],
        guessed_checkpoints: [0; 16],
        no_precipitation: 0,
    };

    /// The starting state: the car blocks named, the computer driver's tables, the credits' race,
    /// the `patch*` settings at their defaults and the stages' checkpoint counts; the rest is
    /// zero.
    pub const fn new() -> Self {
        RaceWork {
            car_blocks: block_handles(),
            // The eight car records, one after the other.
            car_records: car_record_names(),
            number_texture_names: number_texture_names(),
            number_tens_textures: [0, 6, 3, 1, 5, 2, 4],
            part_break_limits: [0x28, 0x3c, 0x3c, 0x14, 0x14, 0x4b, 0x3c],
            // The matrices a part takes lying still and lying flat.
            part_matrix_still: [
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
            ],
            part_matrix_flat: [
                F32_ONE, 0, 0, 0, 0, F32_ONE, 0, 0, 0, 0, F32_ONE, 0, 0, 0, 0, 0,
            ],
            part_fling: 6e-05,
            // The computer driver's speed by bend: a triangle from -1 up to 1 and back.
            bend_speeds: [
                -1.0, -0.9, -0.8, -0.7, -0.5, -0.3, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 0.8,
                0.9, 1.0, 0.9, 0.8, 0.7, 0.5, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2, -0.3, -0.5, -0.7,
                -0.8, -0.9,
            ],
            // Three tables of seven speeds (four to a word).
            wanted_speeds: [
                0xfafafafa, 0xfafafafa, 0xc8fafafa, 0x5a646496, 0x46464646, 0x323c3c3c, 0x1e283232,
                0xfafafafa, 0xfafafafa, 0xfafafafa, 0x323c4664, 0x32323232, 0x28282832, 0x1414141e,
                0xfafafafa, 0xfafafafa, 0x506efafa, 0x323c4646, 0x32323232, 0x1e1e2828, 0x1414141e,
            ],
            driver_tilt_limit: 0.2,
            driver_vector: [0.0, 1.0, 0.0, 0.0],
            replay_extra: 1,
            credit_stages: [
                0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x29, 0x2a,
                0x2b, 0x2c, 0x2d, 0x2e, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x3d, 0x3e, 0x3f, 0x40,
                0x41, 0x42, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c,
            ],
            credit_car_kinds: credit_car_kinds(),
            patches: [
                0, 0xc, 0, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64,
            ],
            patch_factors: [1.0; 10],
            checkpoint_counts: [
                9, 0xb, 0xc, 8, 0xc, 0xd, 0xb, 8, 9, 0xa, 0xe, 0xe, 0xb, 0xb, 0xd, 0xe, 0xd, 0xc,
                0xc, 0xc, 0xc, 0xb, 0xa, 0xb, 0x10, 0xe, 0xe, 0xf, 0xa, 0xe, 8, 9, 9, 9, 4, 0xa,
            ],
            ..Self::ZERO
        }
    }
}

/// `car_blocks` at the start: the eight cars' blocks, by handle.
const fn block_handles() -> [u32; 8] {
    let mut out = [0; 8];
    let mut k = 0;
    while k < 8 {
        out[k] = crate::race::blocks_store::block_handle(k);
        k += 1;
    }
    out
}

/// `car_records` at the start: the handles of the eight car records in order.
const fn car_record_names() -> [u32; 8] {
    let mut out = [0; 8];
    let mut k = 0;
    while k < 8 {
        out[k] = CAR_RECORD_TAG | k as u32;
        k += 1;
    }
    out
}

/// The credits' race drives every car kind once, in order.
const fn credit_car_kinds() -> [u32; 23] {
    let mut out = [0; 23];
    let mut k = 0;
    while k < 23 {
        out[k] = k as u32;
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

const fn number_texture_names() -> [u32; 384] {
    let mut out = [0; 384];
    let mut name = 0;
    while name < 32 {
        let text = NUMBER_NAMES[name];
        let mut word = 0;
        while word < 12 {
            out[name * 12 + word] = u32::from_le_bytes([
                text[4 * word],
                text[4 * word + 1],
                text[4 * word + 2],
                text[4 * word + 3],
            ]);
            word += 1;
        }
        name += 1;
    }
    out
}

const F32_ONE: u32 = 1.0f32.to_bits();
const F32_MINUS_ONE: u32 = (-1.0f32).to_bits();

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut work = RaceWork::new();
        work.car_records[3] = 0x11;
        work.current_car = 0x22;
        work.stage_direction = 0x33;
        work.work_block = 0x44;
        work.number_texture_names[5] = 0x55;
        assert_eq!(work.past_the_eight(3).unwrap(), 0x11);
        assert_eq!(work.past_the_eight(8).unwrap(), 0x22);
        assert_eq!(work.past_the_eight(9).unwrap(), 0);
        assert_eq!(work.past_the_eight(10).unwrap(), 0x33);
        assert_eq!(work.past_the_eight(11).unwrap(), 0x44);
        assert_eq!(work.past_the_eight(12 + 5).unwrap(), 0x55);
        assert!(work.past_the_eight(12 + 384).is_err());
    }

    /// The walk reports every byte of the record once, as the cells' walk did.
    #[test]
    fn the_walk_covers_the_whole_record() {
        use crate::memory::cells::CellVisitor;
        use crate::memory::plain_record::PlainRecord;
        struct Count(usize);
        impl CellVisitor for Count {
            fn enter(&mut self, _: &'static str) {}
            fn index(&mut self, _: usize) {}
            fn leave(&mut self) {}
            fn leaf(&mut self, bytes: &[u8]) {
                self.0 += bytes.len();
            }
        }
        let mut count = Count(0);
        RaceWork::new().visit(&mut count);
        assert_eq!(count.0, size_of::<RaceWork>());
    }
}
