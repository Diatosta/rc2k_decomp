//! The race's work variables after the work car (IDA 0x71AFB8 .. 0x71C814; phase 7, typed):
//! the cars' blocks and the current car, the stage's direction, the cars' number textures,
//! the loose parts' slots, the computer driver's state, the credits' race, the network's
//! message, the `patch*` settings, the stages' checkpoint counts. Named from the code's uses
//! (2026-10-25); most names are readings, `unknown_*` runs are places no ported code names.
//! `#[repr(C)]` with the image's order of fields (the unused runs at either end and in the
//! middle left out).

use crate::errors::BadInput;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainDword, PlainFlag, PlainFloat};

/// A loose part flying off a car (0x78 bytes, `physics::fill_race_slot`; readings).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceSlot {
    /// +0x0: the part (0 the slot empty).
    pub part: PlainDword,
    /// +0x4: its place, whole numbers.
    pub place: [PlainDword; 3],
    /// +0x10: its matrix.
    pub matrix: [PlainFloat; 16],
    /// +0x50: its motion.
    pub motion: [PlainFloat; 3],
    /// +0x5c: added to the part's height when it is drawn (`race::frame_draw::scene_objects`); no ported code writes it (0 from the start).
    pub draw_height_offset: PlainDword,
    /// +0x60: the car's +0xE00 when it broke off.
    pub car_word: PlainDword,
    /// +0x64: the car's step clock when it broke off (the lowest is reused).
    pub age: PlainDword,
    /// +0x68: its spin.
    pub spin: [PlainFloat; 3],
    /// +0x74: the spin's length.
    pub spin_length: PlainFloat,
}

const _: () = assert!(core::mem::size_of::<RaceSlot>() == 0x78);

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceWork {
    /// +0x400 (0x71B3B8): `g_carBlocks`: the eight cars' blocks (`CarBlock`, by handle:
    /// `blocks_store::block_ref`).
    pub car_blocks: [PlainDword; 8],
    /// +0x420 (0x71B3D8): `g_carRecords`: a word for each car's view marks (`race::frame_views`),
    /// kept as IDA's addresses, as names (phase 13; `RaceShared::car_record` makes one real).
    pub car_records: [PlainDword; 8],
    /// +0x440 (0x71B3F8): `g_currentCar`: the car being stepped.
    pub current_car: PlainDword,
    /// +0x448 (0x71B400): `g_stageDirection`: the stage run forwards or backwards ("dir").
    pub stage_direction: PlainDword,
    /// +0x44c (0x71B404): the current car's block by handle (`drive::race_cars_step`, the
    /// replay; `blocks_store::block_ref`).
    pub work_block: PlainDword,
    /// +0x450 (0x71B408): 32 names of 0x30 bytes by car kind: the cars' number textures (`race::car_draw`).
    pub number_texture_names: [PlainDword; 384],
    /// +0xa50 (0x71BA08): the drawn car's nearness (`race::car_draw`); now each view's (`ctx.view.cars.car_nearness`); unused.
    pub car_nearness: PlainFloat,
    /// +0xa54 (0x71BA0C): (`race::car_draw`); now each view's (`ctx.view.cars.car_draw_word`); unused.
    pub car_draw_word: PlainDword,
    /// +0xa58 (0x71BA10): the number's tens' textures (`race::car_draw`).
    pub number_tens_textures: [PlainDword; 7],
    /// +0xa74 (0x71BA2C): the parts' break limits (`physics::drop_broken_parts`).
    pub part_break_limits: [PlainDword; 7],
    /// +0xa90 (0x71BA48): the matrix a part at rest takes (`physics::move_slot`).
    pub part_matrix_still: [PlainDword; 16],
    /// +0xad0 (0x71BA88): the matrix a part lying flat takes (`physics::move_slot`).
    pub part_matrix_flat: [PlainDword; 16],
    /// +0xb10 (0x71BAC8): `g_raceSlots`: the loose parts flying off the cars (`physics::fill_race_slot`).
    pub race_slots: [RaceSlot; 8],
    /// +0xed0 (0x71BE88): (`physics::move_slots`).
    pub slot_count: PlainDword,
    /// +0xed4 (0x71BE8C): the parts' throw (`physics::fill_race_slot`).
    pub part_fling: PlainFloat,
    /// +0xed8 (0x71BE90): the place kept (`physics::move_slot`).
    pub slot_place: [PlainDword; 3],
    /// +0xee4 (0x71BE9C): the computer driver's speed by bend (`drive::drive_computer_car`).
    pub bend_speeds: [PlainFloat; 32],
    /// +0xf64 (0x71BF1C): three tables of seven: a wanted speed by the bend's turn.
    pub wanted_speeds: [PlainDword; 21],
    /// +0xfb8 (0x71BF70): (`drive::drive_computer_car`, `drive::read_car_controls`).
    pub driver_tilt_limit: PlainFloat,
    /// +0xfbc (0x71BF74): (`drive::drive_computer_car`).
    pub driver_vector: [PlainFloat; 4],
    /// +0xfcc (0x71BF84): the replay's extra start piece (`drive::save_replay_cars`).
    pub replay_extra: PlainDword,
    /// +0xfd0 (0x71BF88): the computer driver's steering.
    pub driver_steer: PlainDword,
    /// +0xfd4 (0x71BF8C): its throttle (0x100 full).
    pub driver_throttle: PlainDword,
    /// +0xfd8 (0x71BF90): its brake.
    pub driver_brake: PlainDword,
    /// +0xfdc (0x71BF94): its other controls (1 on).
    pub driver_flags: [PlainDword; 4],
    /// +0xfec (0x71BFA4): (`drive::read_car_controls`).
    pub driver_start_blend: PlainFloat,
    /// +0xff0 (0x71BFA8): steps by car (`drive::drive_computer_car`).
    pub driver_steps: [PlainDword; 10],
    /// +0x1018 (0x71BFD0): reversing steps left by car.
    pub driver_reverse_left: [PlainDword; 10],
    /// +0x1040 (0x71BFF8): the distance moved.
    pub driver_moved: PlainDword,
    /// +0x1044 (0x71BFFC): the bend's turn.
    pub driver_turn: PlainDword,
    /// +0x1048 (0x71C000): the bend's index.
    pub driver_bend_index: PlainDword,
    /// +0x1050 (0x71C008): the speed wanted.
    pub driver_wanted_speed: PlainDword,
    /// +0x1054 (0x71C00C): the speed.
    pub driver_speed: PlainDword,
    /// +0x1058 (0x71C010): a bend noted by car: where it ends.
    pub driver_bend_to: [PlainDword; 10],
    /// +0x1080 (0x71C038): a bend noted by car.
    pub driver_bend_noted: [PlainDword; 10],
    /// +0x10a8 (0x71C060): reversing tries by car.
    pub driver_reverse_tries: [PlainDword; 10],
    /// +0x10d0 (0x71C088): the noted bend's turn by car.
    pub driver_bend_turn: [PlainDword; 10],
    /// +0x1104 (0x71C0BC): (`drive::drive_computer_car`).
    pub driver_flag_bc: PlainDword,
    /// +0x1108 (0x71C0C0): the step along x.
    pub driver_step_x: PlainDword,
    /// +0x110c (0x71C0C4): the step along z.
    pub driver_step_z: PlainDword,
    /// +0x1110 (0x71C0C8): from `bend_speeds`.
    pub driver_speed_scale: PlainFloat,
    /// +0x1114 (0x71C0CC): the car driven (`g_currentCar` copied).
    pub driver_slot: PlainDword,
    /// +0x1118 (0x71C0D0): 1 when looking ahead.
    pub driver_lookahead: PlainDword,
    /// +0x111c (0x71C0D4): (`drive::drive_computer_car`).
    pub driver_bend_a: PlainDword,
    /// +0x1120 (0x71C0D8): (`drive::drive_computer_car`).
    pub driver_bend_b: PlainDword,
    /// +0x1124 (0x71C0DC): its point in `driver_points_now` (a byte offset; the original kept the address).
    pub driver_point_now: PlainDword,
    /// +0x1128 (0x71C0E0): its point in `driver_points_then` (a byte offset; the original kept the address).
    pub driver_point_then: PlainDword,
    /// +0x112c (0x71C0E4): 1: aim ahead.
    pub driver_aim: PlainDword,
    /// +0x1130 (0x71C0E8): ten cars' points of 16 bytes.
    pub driver_points_now: [PlainDword; 40],
    /// +0x11d0 (0x71C188): ten cars' points of 16 bytes, the step before.
    pub driver_points_then: [PlainDword; 40],
    /// +0x1270 (0x71C228): (x, 0, z).
    pub driver_from: [PlainDword; 4],
    /// +0x1280 (0x71C238): (x, 0, z).
    pub driver_to: [PlainDword; 4],
    /// +0x12a0 (0x71C258): the way from here to `driver_to` on the flat.
    pub driver_way: [PlainFloat; 4],
    /// +0x12b0 (0x71C268): it normalised.
    pub driver_way_unit: [PlainFloat; 4],
    /// +0x12c0 (0x71C278): the way ahead normalised.
    pub driver_ahead: [PlainFloat; 4],
    /// +0x12e0 (0x71C298): `driver_from` to `driver_to` on the flat.
    pub driver_flat: [PlainFloat; 4],
    /// +0x12f0 (0x71C2A8): it normalised.
    pub driver_flat_unit: [PlainFloat; 4],
    /// +0x1300 (0x71C2B8): the car's side.
    pub driver_side: [PlainFloat; 4],
    /// +0x1310 (0x71C2C8): (`drive::drive_computer_car`).
    pub driver_work: [PlainDword; 47],
    /// +0x1420 (0x71C3D8): kept by `menu::scale_stage_times`.
    pub stage_time_scales: [PlainFloat; 2],
    /// +0x1428 (0x71C3E0): `g_inCredits`: the credits' race on.
    pub in_credits: PlainFlag,
    /// +0x142c (0x71C3E4): `g_creditStages`: the credits' stages.
    pub credit_stages: [PlainDword; 36],
    /// +0x14bc (0x71C474): `g_creditCarKinds`: the credits' car kinds.
    pub credit_car_kinds: [PlainDword; 23],
    /// +0x1518 (0x71C4D0): the network's message built (`net::send_inputs`).
    pub net_message: [PlainDword; 128],
    /// +0x1718 (0x71C6D0): the physics patches as the settings file gives them: `g_patchSwitch`
    /// (`fizpatch`), then `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`,
    /// `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`,
    /// `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel`.
    pub patches: [PlainDword; 13],
    /// +0x174c (0x71C704): the last ten settings / 100 (`physics::apply_patches`).
    pub patch_factors: [PlainFloat; 10],
    /// +0x1774 (0x71C72C): `g_patchesApplied`.
    pub patches_applied: PlainDword,
    /// +0x1778 (0x71C730): `g_checkpointCounts`: by stage (`saves::default_times`).
    pub checkpoint_counts: [PlainDword; 36],
    /// +0x1808 (0x71C7C0): `g_guessedCheckpoints`.
    pub guessed_checkpoints: [PlainDword; 16],
    /// +0x1848 (0x71C800): the `noprecipitation` setting.
    pub no_precipitation: PlainDword,
}

impl RaceWork {
    /// `g_patchSwitch`: the first of the patches.
    pub fn patch_switch(&self) -> &PlainDword {
        &self.patches[0]
    }

    /// The twelve patch settings after the switch.
    pub fn patch_settings(&self) -> &[PlainDword] {
        &self.patches[1..]
    }

    /// Car `n`'s block by handle (`g_carBlocks[n]`: `blocks_store::block_ref`, 0 for none),
    /// read as the original reads it: past the eight without a check, on through the variables
    /// that follow the table (`past_the_eight`). A handle among the eight that names no block
    /// ends the program.
    pub fn car_block(&self, n: usize) -> u32 {
        if let Some(cell) = self.car_blocks.get(n) {
            let handle = cell.get();
            crate::race::blocks_store::block_number(handle);
            return handle;
        }
        self.past_the_eight(n - self.car_blocks.len())
            .unwrap_or_else(|e| crate::fatal::raise::end("a car block's pointer", e))
    }

    /// The dword `k` places after the table of car blocks, which is what `g_carBlocks[8 + k]`
    /// reads: the eight car records' names, `current_car`, a dword nothing uses (0), the stage's
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

    /// Car `n`'s block as `g_carBlocks[n]` keeps it: its handle (0 for none), the value other
    /// places keep for "that car" (`block_ref`).
    pub fn car_handle(&self, n: usize) -> u32 {
        self.car_blocks
            .get(n)
            .map_or_else(|| self.car_block(n), PlainDword::get)
    }

    /// The number of the block car `n` has, refused for none (as `frame_view::car_at` on its
    /// address).
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

    /// Whether car `n` has a block (`car_block(n) != 0`, ending the program on a handle that
    /// names none as it does).
    pub fn has_car_block(&self, n: usize) -> bool {
        use crate::race::blocks_store::{BLOCK_COUNT, BLOCK_TAG};
        match self.car_blocks.get(n).map(PlainDword::get) {
            Some(0) => false,
            Some(h) if ((h ^ BLOCK_TAG) as usize) < BLOCK_COUNT => true,
            _ => self.car_block(n) != 0,
        }
    }

    /// The number of car record `n` (`g_carRecords[n]`, the name of one of the effects' car
    /// records, `race::effects_settings_store`; `RaceShared::car_record`). Refused past the
    /// eight (the original reads on into the next variables) and for a name that is not a
    /// record's start.
    pub fn car_record_number(&self, n: usize) -> Result<usize, BadInput> {
        let name = self
            .car_records
            .get(n)
            .ok_or(BadInput("a car record past the eight"))?
            .get() as usize;
        let off = name.wrapping_sub(CAR_RECORDS_IDA);
        if off >= CAR_RECORDS_LEN
            || !off.is_multiple_of(crate::race::effects_settings_store::CAR_LEN)
        {
            return Err(BadInput("a car record off the view marks' records"));
        }
        Ok(off / crate::race::effects_settings_store::CAR_LEN)
    }
}

/// IDA 0x5C2FE4, where the eight car records (0x4C8 bytes each, in
/// `effects_settings_store::EffectsSettings::cars`) begin, and their length.
pub const CAR_RECORDS_IDA: usize = 0x5C2FE4;
pub const CAR_RECORDS_LEN: usize = 8 * 0x4C8;

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceWork {
    /// The piece, with the image's first values of each run of fields (the image's unused
    /// words before the car blocks, after `current_car`, between the driver's fields and at the
    /// end are not kept).
    pub const fn new() -> Self {
        use crate::race::store::first_values;
        const SIZE: usize = core::mem::size_of::<RaceWork>();
        const FIRST: usize = crate::race::blocks_store::CAR_BLOCKS_IDA;
        // Up to `current_car`, then from `stage_direction` to the end of the driver's work (in
        // five runs, round the driver's unused words), then from the stage times' scales to the
        // end of the fields.
        const A: usize = 0x71B3FC - FIRST;
        const B1: usize = 0x71C004 - 0x71B400;
        const B2: usize = 0x71C0B0 - 0x71C008;
        const B3: usize = 0x71C248 - 0x71C0BC;
        const B4: usize = 0x71C288 - 0x71C258;
        const B5: usize = 0x71C384 - 0x71C298;
        const C: usize = 0x71C804 - 0x71C3D8;
        const _: () = assert!(A + B1 + B2 + B3 + B4 + B5 + C == SIZE);
        let mut bytes = [0u8; SIZE];
        let parts: [(usize, &[u8]); 7] = [
            (0, &first_values::<A>(FIRST)),
            (A, &first_values::<B1>(0x71B400)),
            (A + B1, &first_values::<B2>(0x71C008)),
            (A + B1 + B2, &first_values::<B3>(0x71C0BC)),
            (A + B1 + B2 + B3, &first_values::<B4>(0x71C258)),
            (A + B1 + B2 + B3 + B4, &first_values::<B5>(0x71C298)),
            (A + B1 + B2 + B3 + B4 + B5, &first_values::<C>(0x71C3D8)),
        ];
        let mut p = 0;
        while p < parts.len() {
            let (at, part) = parts[p];
            let mut k = 0;
            while k < part.len() {
                bytes[at + k] = part[k];
                k += 1;
            }
            p += 1;
        }
        crate::cells::cells_of_bytes(crate::race::blocks_store::with_block_refs(bytes, 0, 8))
    }
}

/// A byte offset in the 1999 layout of the race work (from `g_carBlocks`) as it is in `RaceWork`
/// now: the driver's unused words were taken out. For the tests whose recorded vectors name the
/// 1999 offsets.
#[cfg(test)]
pub fn from_1999(old: usize) -> usize {
    const TAKEN_OUT: [(usize, usize); 4] = [(0xC4C, 4), (0xCF8, 12), (0xE90, 16), (0xED0, 16)];
    let mut shift = 0;
    for (at, len) in TAKEN_OUT {
        assert!(
            !(at..at + len).contains(&old),
            "{old:#x} was an unused place"
        );
        if old >= at + len {
            shift += len;
        }
    }
    old - shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        let _g = crate::test_support::test_lock();
        // The car blocks are handles where the game has addresses.
        crate::test_support::first_values::assert_store_starts_as_the_games(
            "store:race.owner.RACE.shared.work",
        );
    }

    #[test]
    fn car_records_resolve_to_what_the_image_named() {
        let shared = &race().shared;
        for k in 0..8 {
            let name = crate::test_support::first_values::dword(
                "race.owner.RACE.shared.work.car_records[0]/32",
                k,
            );
            assert_eq!(name as usize, CAR_RECORDS_IDA + 0x4C8 * k);
            assert!(std::ptr::eq(
                shared.car_records().unwrap()[k],
                shared.car_record(k).unwrap()
            ));
        }
        assert!(shared.car_record(8).is_err());
    }

    /// `car_block` past the eight reads the variables that follow the table, in their order.
    #[test]
    fn the_car_block_past_the_eight_reads_on_by_name() {
        let _g = crate::test_support::test_lock();
        let work = race().shared.work.get();
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
