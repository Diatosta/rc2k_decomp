//! The race's work variables after the work car (IDA 0x71AFB8 .. 0x71C814; phase 7, typed):
//! the cars' blocks and the current car, the stage's direction, the cars' number textures,
//! the loose parts' slots, the computer driver's state, the credits' race, the network's
//! message, the `patch*` settings, the stages' checkpoint counts. Named from the code's uses
//! (2026-10-25); most names are readings, `unknown_*` runs are places no ported code names.
//! `#[repr(C)]` with the image's layout, over the race block's bytes (`RACE_WORK`, as
//! `physics::car_store::CAR`) until the code reaching them by RVA is typed.

use crate::session::{Dword, Float};

/// A loose part flying off a car (0x78 bytes, `physics::fill_race_slot`; readings).
#[repr(C)]
pub struct RaceSlot {
    /// +0x0: the part (0 the slot empty).
    pub part: Dword,
    /// +0x4: its place, whole numbers.
    pub place: [Dword; 3],
    /// +0x10: its matrix.
    pub matrix: [Float; 16],
    /// +0x50: its motion.
    pub motion: [Float; 3],
    /// +0x5c: not named.
    pub unknown_5c: Dword,
    /// +0x60: the car's +0xE00 when it broke off.
    pub car_word: Dword,
    /// +0x64: the car's step clock when it broke off (the lowest is reused).
    pub age: Dword,
    /// +0x68: its spin.
    pub spin: [Float; 3],
    /// +0x74: the spin's length.
    pub spin_length: Float,
}

const _: () = assert!(core::mem::size_of::<RaceSlot>() == 0x78);

#[repr(C)]
pub struct RaceWork {
    /// +0x0 (0x71AFB8): not named.
    pub unknown_0: [Dword; 256],
    /// +0x400 (0x71B3B8): `g_carBlocks`: the eight cars' blocks (`CarBlock`, addresses).
    pub car_blocks: [Dword; 8],
    /// +0x420 (0x71B3D8): `g_carRecords`: a word for each car's view marks (`race::frame_views`).
    pub car_records: [Dword; 8],
    /// +0x440 (0x71B3F8): `g_currentCar`: the car being stepped.
    pub current_car: Dword,
    /// +0x444 (0x71B3FC): not named.
    pub unknown_444: Dword,
    /// +0x448 (0x71B400): `g_stageDirection`: the stage run forwards or backwards ("dir").
    pub stage_direction: Dword,
    /// +0x44c (0x71B404): the current car's block (`drive::race_cars_step`, the replay).
    pub work_block: Dword,
    /// +0x450 (0x71B408): 32 names of 0x30 bytes by car kind: the cars' number textures (`race::car_draw`).
    pub number_texture_names: [Dword; 384],
    /// +0xa50 (0x71BA08): the drawn car's nearness (`race::car_draw`).
    pub car_nearness: Float,
    /// +0xa54 (0x71BA0C): (`race::car_draw`).
    pub car_draw_word: Dword,
    /// +0xa58 (0x71BA10): the number's tens' textures (`race::car_draw`).
    pub number_tens_textures: [Dword; 7],
    /// +0xa74 (0x71BA2C): the parts' break limits (`physics::drop_broken_parts`).
    pub part_break_limits: [Dword; 7],
    /// +0xa90 (0x71BA48): the matrix a part at rest takes (`physics::move_slot`).
    pub part_matrix_still: [Dword; 16],
    /// +0xad0 (0x71BA88): the matrix a part lying flat takes (`physics::move_slot`).
    pub part_matrix_flat: [Dword; 16],
    /// +0xb10 (0x71BAC8): `g_raceSlots`: the loose parts flying off the cars (`physics::fill_race_slot`).
    pub race_slots: [RaceSlot; 8],
    /// +0xed0 (0x71BE88): (`physics::move_slots`).
    pub slot_count: Dword,
    /// +0xed4 (0x71BE8C): the parts' throw (`physics::fill_race_slot`).
    pub part_fling: Float,
    /// +0xed8 (0x71BE90): the place kept (`physics::move_slot`).
    pub slot_place: [Dword; 3],
    /// +0xee4 (0x71BE9C): the computer driver's speed by bend (`drive::drive_computer_car`).
    pub bend_speeds: [Float; 32],
    /// +0xf64 (0x71BF1C): three tables of seven: a wanted speed by the bend's turn.
    pub wanted_speeds: [Dword; 21],
    /// +0xfb8 (0x71BF70): (`drive::drive_computer_car`, `drive::read_car_controls`).
    pub driver_tilt_limit: Float,
    /// +0xfbc (0x71BF74): (`drive::drive_computer_car`).
    pub driver_vector: [Float; 4],
    /// +0xfcc (0x71BF84): the replay's extra start piece (`drive::save_replay_cars`).
    pub replay_extra: Dword,
    /// +0xfd0 (0x71BF88): the computer driver's steering.
    pub driver_steer: Dword,
    /// +0xfd4 (0x71BF8C): its throttle (0x100 full).
    pub driver_throttle: Dword,
    /// +0xfd8 (0x71BF90): its brake.
    pub driver_brake: Dword,
    /// +0xfdc (0x71BF94): its other controls (1 on).
    pub driver_flags: [Dword; 4],
    /// +0xfec (0x71BFA4): (`drive::read_car_controls`).
    pub driver_start_blend: Float,
    /// +0xff0 (0x71BFA8): steps by car (`drive::drive_computer_car`).
    pub driver_steps: [Dword; 10],
    /// +0x1018 (0x71BFD0): reversing steps left by car.
    pub driver_reverse_left: [Dword; 10],
    /// +0x1040 (0x71BFF8): the distance moved.
    pub driver_moved: Dword,
    /// +0x1044 (0x71BFFC): the bend's turn.
    pub driver_turn: Dword,
    /// +0x1048 (0x71C000): the bend's index.
    pub driver_bend_index: Dword,
    /// +0x104c (0x71C004): not named.
    pub unknown_104c: Dword,
    /// +0x1050 (0x71C008): the speed wanted.
    pub driver_wanted_speed: Dword,
    /// +0x1054 (0x71C00C): the speed.
    pub driver_speed: Dword,
    /// +0x1058 (0x71C010): a bend noted by car: where it ends.
    pub driver_bend_to: [Dword; 10],
    /// +0x1080 (0x71C038): a bend noted by car.
    pub driver_bend_noted: [Dword; 10],
    /// +0x10a8 (0x71C060): reversing tries by car.
    pub driver_reverse_tries: [Dword; 10],
    /// +0x10d0 (0x71C088): the noted bend's turn by car.
    pub driver_bend_turn: [Dword; 10],
    /// +0x10f8 (0x71C0B0): not named.
    pub unknown_10f8: [Dword; 3],
    /// +0x1104 (0x71C0BC): (`drive::drive_computer_car`).
    pub driver_flag_bc: Dword,
    /// +0x1108 (0x71C0C0): the step along x.
    pub driver_step_x: Dword,
    /// +0x110c (0x71C0C4): the step along z.
    pub driver_step_z: Dword,
    /// +0x1110 (0x71C0C8): from `bend_speeds`.
    pub driver_speed_scale: Float,
    /// +0x1114 (0x71C0CC): the car driven (`g_currentCar` copied).
    pub driver_slot: Dword,
    /// +0x1118 (0x71C0D0): 1 when looking ahead.
    pub driver_lookahead: Dword,
    /// +0x111c (0x71C0D4): (`drive::drive_computer_car`).
    pub driver_bend_a: Dword,
    /// +0x1120 (0x71C0D8): (`drive::drive_computer_car`).
    pub driver_bend_b: Dword,
    /// +0x1124 (0x71C0DC): its point in `driver_points_now`.
    pub driver_point_now: Dword,
    /// +0x1128 (0x71C0E0): its point in `driver_points_then`.
    pub driver_point_then: Dword,
    /// +0x112c (0x71C0E4): 1: aim ahead.
    pub driver_aim: Dword,
    /// +0x1130 (0x71C0E8): ten cars' points of 16 bytes.
    pub driver_points_now: [Dword; 40],
    /// +0x11d0 (0x71C188): ten cars' points of 16 bytes, the step before.
    pub driver_points_then: [Dword; 40],
    /// +0x1270 (0x71C228): (x, 0, z).
    pub driver_from: [Dword; 4],
    /// +0x1280 (0x71C238): (x, 0, z).
    pub driver_to: [Dword; 4],
    /// +0x1290 (0x71C248): not named.
    pub unknown_1290: [Dword; 4],
    /// +0x12a0 (0x71C258): the way from here to `driver_to` on the flat.
    pub driver_way: [Float; 4],
    /// +0x12b0 (0x71C268): it normalised.
    pub driver_way_unit: [Float; 4],
    /// +0x12c0 (0x71C278): the way ahead normalised.
    pub driver_ahead: [Float; 4],
    /// +0x12d0 (0x71C288): not named.
    pub unknown_12d0: [Dword; 4],
    /// +0x12e0 (0x71C298): `driver_from` to `driver_to` on the flat.
    pub driver_flat: [Float; 4],
    /// +0x12f0 (0x71C2A8): it normalised.
    pub driver_flat_unit: [Float; 4],
    /// +0x1300 (0x71C2B8): the car's side.
    pub driver_side: [Float; 4],
    /// +0x1310 (0x71C2C8): (`drive::drive_computer_car`).
    pub driver_work: [Dword; 47],
    /// +0x13cc (0x71C384): `math::nearest_on_segment`'s work block until 2026-10-24 (its own since): nothing reads it.
    pub segment_work_unused: [Dword; 21],
    /// +0x1420 (0x71C3D8): kept by `menu::scale_stage_times`.
    pub stage_time_scales: [Float; 2],
    /// +0x1428 (0x71C3E0): `g_inCredits`: the credits' race on.
    pub in_credits: Dword,
    /// +0x142c (0x71C3E4): `g_creditStages`: the credits' stages.
    pub credit_stages: [Dword; 36],
    /// +0x14bc (0x71C474): `g_creditCarKinds`: the credits' car kinds.
    pub credit_car_kinds: [Dword; 23],
    /// +0x1518 (0x71C4D0): the network's message built (`net::send_inputs`).
    pub net_message: [Dword; 128],
    /// +0x1718 (0x71C6D0): `g_patchSwitch`: `fizpatch`.
    pub patch_switch: Dword,
    /// +0x171c (0x71C6D4): `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`, `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`, `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel`.
    pub patch_settings: [Dword; 12],
    /// +0x174c (0x71C704): the last ten settings / 100 (`physics::apply_patches`).
    pub patch_factors: [Float; 10],
    /// +0x1774 (0x71C72C): `g_patchesApplied`.
    pub patches_applied: Dword,
    /// +0x1778 (0x71C730): `g_checkpointCounts`: by stage (`saves::default_times`).
    pub checkpoint_counts: [Dword; 36],
    /// +0x1808 (0x71C7C0): `g_guessedCheckpoints`.
    pub guessed_checkpoints: [Dword; 16],
    /// +0x1848 (0x71C800): the `noprecipitation` setting.
    pub no_precipitation: Dword,
    /// +0x184c (0x71C804): not named.
    pub unknown_184c: [Dword; 4],
}

impl RaceWork {
    /// Car `n`'s block (`g_carBlocks[n]`), read as the original reads it: past the eight
    /// without a check (into `car_records`).
    pub fn car_block(&self, n: usize) -> u32 {
        // SAFETY: the original's own unchecked read; inside the race block for the numbers
        // the game passes.
        unsafe {
            (crate::pointers::host_at(
                (crate::pointers::game_address(self.car_blocks.as_ptr()) + (4 * n) as u32) as usize,
            ) as *const u32)
                .read_unaligned()
        }
    }

    /// Car record `n` (`g_carRecords[n]`, a pointer to the car's view word), read without a
    /// check as the original reads it.
    pub fn car_record(&self, n: usize) -> u32 {
        // SAFETY: as `car_block`.
        unsafe {
            (crate::pointers::host_at(
                (crate::pointers::game_address(self.car_records.as_ptr()) + (4 * n) as u32)
                    as usize,
            ) as *const u32)
                .read_unaligned()
        }
    }
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for RaceWork {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static RACE_WORK: RaceWork =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x71AFB8, the first byte of the piece.
pub const START: usize = 0x71AFB8;
/// IDA 0x71C814, the end of the piece.
pub const END: usize = 0x71C814;

const _: () = assert!(END - START == core::mem::size_of::<RaceWork>());

/// Where IDA 0x71AFB8 is now (its own static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&RACE_WORK))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&RACE_WORK, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(RaceWork, car_blocks), 0x71B3B8 - START);
        assert_eq!(offset_of!(RaceWork, car_nearness), 0x71BA08 - START);
        assert_eq!(offset_of!(RaceWork, race_slots), 0x71BAC8 - START);
        assert_eq!(offset_of!(RaceWork, driver_tilt_limit), 0x71BF70 - START);
        assert_eq!(offset_of!(RaceWork, driver_flags), 0x71BF94 - START);
        assert_eq!(offset_of!(RaceWork, driver_bend_index), 0x71C000 - START);
        assert_eq!(offset_of!(RaceWork, driver_bend_turn), 0x71C088 - START);
        assert_eq!(offset_of!(RaceWork, driver_lookahead), 0x71C0D0 - START);
        assert_eq!(offset_of!(RaceWork, driver_points_now), 0x71C0E8 - START);
        assert_eq!(offset_of!(RaceWork, driver_ahead), 0x71C278 - START);
        assert_eq!(offset_of!(RaceWork, stage_time_scales), 0x71C3D8 - START);
        assert_eq!(offset_of!(RaceWork, patch_settings), 0x71C6D4 - START);
    }
}
