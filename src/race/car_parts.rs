//! The cars' parts made ready to draw (`sub_460D5C` -> `sub_460E7C`, part R3, IDA
//! 2026-10-18): each car's marks' switches (+0xAB4 .. +0xAD0) from its state, and its four
//! parts' matrices (the wheels: turned about their axles' places) into `flt_6417E0` .., then
//! kept in the block (+0x740 .. +0x8FF) for `race::draw_car`.

use crate::errors::{BadInput, Refused};
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::frame_store::FRAME_WORK;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::matrix::{multiply_matrices, set_place, set_turn_x, set_turn_y};
use crate::scene::object_store::OBJECT_WORK;

/// The five axle values (`rc2k_core::race::car_parts::axles`) from the model's record
/// (`dword_5C9098` .. of its 0x134-byte record) and the lift.
fn axles(model: u32) -> Option<[u32; 5]> {
    let at = LIGHT_RECORDS.records[0].axles[0].addr() as u32 + model.wrapping_mul(0x134);
    let record =
        [0u32, 4, 8, 0xC, 0x10].map(|o| unsafe { ((at + o) as *const u32).read_unaligned() });
    rc2k_core::race::car_parts::axles(record, PHYSICS.ground_drop.get())
}

/// `race::car_part_matrices` (0x460E7C): car esi's marks' switches and its parts' matrices
/// (see the header): +0x718 the size from +0x1D8 (not below `flt_6140F8`); the switch bytes
/// from +0x84, `dword_62DAD0` / `dword_62DACC`, +0x63C, +0x684, +0x1D4, each cut by its
/// broken part (+0x410 .. +0x424; the original cuts them twice); +0xABC the marks' count x
/// 0x20 (0 when +0x310 is not below `flt_5C5D14` or +0x84 is 0); +0xAAC the steering; the
/// axles from the model's record; the light and place; the car's turn (+0xB4) and place
/// into `flt_5EA08C` / `flt_641760`; each wheel placed at minus its axle, turned about x by
/// its spin (+0x1F8 front, +0x1FC rear) and about y by the steering (front only), placed at
/// its axle with its height (+0x910 ..), into `flt_6417E0` .. `flt_6418A0`. Refuses (before
/// anything) axle values whose scaling would fault.
pub unsafe fn car_part_matrices(car: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::car_parts::{axles, mark_switches, marks_bytes,
    // steering}` (the size's and the speed's floats the x87's).
    use crate::math::x87::loaded;
    use crate::math::x87_load::below_or_unordered;
    use rc2k_core::race::car_parts::{
        FIRST_SWITCH, PartState, mark_switches, marks_bytes, steering,
    };
    unsafe {
        let f = |o: u32| ((car + o) as *const u32).read_unaligned();
        let set = |o: u32, v: u32| ((car + o) as *mut u32).write_unaligned(v);
        let Some(ax) = axles(f(0x24)) else {
            return Err(BadInput("a car model whose axle scaling would fault").into());
        };
        set(0x718, loaded(f(0x1D8)));
        if below_or_unordered(f(0x718), PHYSICS.idle_engine_speed.bits()) {
            set(0x718, loaded(PHYSICS.idle_engine_speed.bits()));
        }
        let on = f(0x84);
        let s = mark_switches(&PartState {
            on,
            marks_mask: LIGHT_WORK.marks_mask.get(),
            lamps_mask: LIGHT_WORK.lamps_mask.get(),
            inside: f(0x63C),
            mode: f(0x684),
            reversing: f(0x1D4),
            broken: [0x410, 0x414, 0x418, 0x41C, 0x420, 0x424].map(f),
            flags: f(0x298),
        });
        set(0xAB4, s.marks_on);
        set(0xAB8, s.mode_on);
        std::ptr::copy_nonoverlapping(
            s.bytes.as_ptr(),
            (car + FIRST_SWITCH) as *mut u8,
            s.bytes.len(),
        );
        let slow = below_or_unordered(f(0x310), NUMBERS.hundred.bits());
        set(0xABC, marks_bytes(s.count, slow, on));
        set(0xAAC, steering(f(0x650)));
        let [front, rear_lift, height, front_z, _] = ax;
        CAMERA_WORK.axle_front.set(front);
        CAMERA_WORK.axle_front_neg.set(front.wrapping_neg());
        CAMERA_WORK.axle_rear_lift.set(rear_lift);
        CAMERA_WORK.axle_height.set(height);
        CAMERA_WORK.axle_front_z.set(front_z);
        CAMERA_WORK.car_axles.set(ax[4]);
        OBJECT_WORK.size.set(0x10);
        OBJECT_WORK.part_values[2].set(f(0xAAC));
        OBJECT_WORK.part_values[0].set(f(0x1F8));
        OBJECT_WORK.part_values[1].set(f(0x1FC));
        for (i, o) in [0x134u32, 0x138, 0x13C].into_iter().enumerate() {
            OBJECT_WORK.place[i].set(f(o));
        }
        std::ptr::copy(
            (car + 0xB4) as *const u8,
            MATRICES.body_turn_matrix[0].addr() as *mut u8,
            0x40,
        );
        for i in 0..4 {
            OBJECT_WORK.part_weights[i as usize].set(f(0x910 + 4 * i));
        }
        set_place([
            OBJECT_WORK.place[0].get(),
            OBJECT_WORK.place[1].get(),
            OBJECT_WORK.place[2].get(),
        ]);
        std::ptr::copy(
            MATRICES.placed_matrix[0].addr() as *const u8,
            FRAME_WORK.placed_matrices[0].addr() as *mut u8,
            0x40,
        );
        // (x, z, the x turn, the height, the matrix): the four wheels.
        let (c, o) = (&CAMERA_WORK, &OBJECT_WORK);
        let parts = |n: usize| FRAME_WORK.part_matrices[n].addr();
        for (n, (x, z, spin, lift, out)) in [
            (
                &c.axle_front_neg,
                &c.axle_height,
                &o.part_values[0],
                &o.part_weights[0],
                parts(0),
            ),
            (
                &c.axle_front,
                &c.axle_height,
                &o.part_values[0],
                &o.part_weights[1],
                parts(16),
            ),
            (
                &c.axle_front_neg,
                &c.axle_front_z,
                &o.part_values[1],
                &o.part_weights[2],
                parts(32),
            ),
            (
                &c.axle_front,
                &c.axle_front_z,
                &o.part_values[1],
                &o.part_weights[3],
                parts(48),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if n == 2 {
                OBJECT_WORK.part_values[2].set(0);
            }
            set_place([
                x.get().wrapping_neg(),
                CAMERA_WORK.axle_rear_lift.get().wrapping_neg(),
                z.get().wrapping_neg(),
            ]);
            set_turn_x(spin.get());
            let placed = MATRICES.placed_matrix[0].addr();
            let turned = MATRICES.turned_matrix[0].addr();
            let lit = MATRICES.lit_matrix[0].addr();
            multiply_matrices(placed, MATRICES.turn_x_matrix[0].addr(), turned);
            set_turn_y(OBJECT_WORK.part_values[2].get());
            multiply_matrices(turned, MATRICES.turn_y_matrix[0].addr(), lit);
            set_place([x.get(), lift.get(), z.get()]);
            multiply_matrices(lit, placed, out);
        }
        Ok(())
    }
}

/// `race::ready_car_parts` (0x460D5C): for each race car with its parts shown (+0x25C):
/// `car_part_matrices` (esi the block), then the six matrices kept in the block
/// (+0x740 .. +0x8BF) and the body's turn times the car's (`flt_5EA3CC`) at +0x8C0. Refuses
/// (before anything) no race cars (the original's loop would run away), more than eight, or
/// one without its block.
pub unsafe fn ready_car_parts() -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::car_parts::{check_part_cars, parts_shown,
        // KEPT_MATRICES}`.
        use rc2k_core::race::car_parts::{
            BODY_AT, KEPT_AT, KEPT_MATRICES, check_part_cars, parts_shown,
        };
        let count = RACE_VIEW.race_cars.get();
        check_part_cars(count, |n| RACE_WORK.car_block(n as usize))?;
        // The loop's test is at its end: no cars still looks at car 0.
        let mut c = 0u32;
        loop {
            let car = RACE_WORK.car_block(c as usize);
            if parts_shown(((car + 0x25C) as *const u32).read_unaligned()) {
                car_part_matrices(car)?;
                for (i, from) in [
                    FRAME_WORK.part_matrices[0].addr(),
                    FRAME_WORK.part_matrices[16].addr(),
                    FRAME_WORK.part_matrices[32].addr(),
                    FRAME_WORK.part_matrices[48].addr(),
                    MATRICES.body_turn_matrix[0].addr(),
                    FRAME_WORK.placed_matrices[0].addr(),
                ]
                .into_iter()
                .take(KEPT_MATRICES as usize)
                .enumerate()
                {
                    std::ptr::copy(
                        from as *const u8,
                        (car + KEPT_AT + 0x40 * i as u32) as *mut u8,
                        0x40,
                    );
                }
                multiply_matrices(
                    MATRICES.body_turn_matrix[0].addr(),
                    FRAME_WORK.placed_matrices[0].addr(),
                    MATRICES.view_place[0].addr(),
                );
                std::ptr::copy(
                    MATRICES.view_place[0].addr() as *const u8,
                    (car + BODY_AT) as *mut u8,
                    0x40,
                );
            }
            c += 1;
            if c == count {
                break;
            }
        }
        Ok(())
    }
}
