//! The matrices an object's place in the view is made with, and the functions that fill them:
//! shared by their seven functions.
//!
//! A matrix is sixteen floats, rows 0x10 apart; a point is turned by the first three
//! rows and columns and moved by the fourth row's first three floats.
//! The two turning matrices keep their other floats (an identity's) from the image.

use crate::framework::{game_addr, game_ptr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::project_point::rotate;

pub const TURN_X_RVA: usize = rva(0x5E9F8C); // flt_5E9F8C
pub const TURN_Y_RVA: usize = rva(0x5E9FCC); // flt_5E9FCC
pub const PLACED_RVA: usize = rva(0x5EA04C); // flt_5EA04C, its fourth row the place
pub const TURNED_RVA: usize = rva(0x5EA10C); // flt_5EA10C
pub const LIT_RVA: usize = rva(0x5EA14C); // flt_5EA14C
pub const VIEW_TURN_RVA: usize = rva(0x5EA24C); // dword_5EA24C
pub const OBJECT_RVA: usize = rva(0x5EA30C); // flt_5EA30C
pub const SINES_RVA: usize = rva(0xA23AE4); // flt_A23AE4, 0x1000 floats
pub const COSINES_RVA: usize = rva(0xA24AE4); // dword_A24AE4
pub const TURN_X_WORK_RVA: usize = rva(0x624148); // dword_624148, dword_62414C
pub const TURN_Y_WORK_RVA: usize = rva(0x624150); // dword_624150, dword_624154
pub const LIGHT_DIRECTION_RVA: usize = rva(0x6240A8); // g_lightDirection, three floats
pub const LIGHT_TURN_Y_RVA: usize = rva(0x635094); // dword_635094
pub const LIGHT_TURN_X_RVA: usize = rva(0x635098); // dword_635098
pub const PLACE_RVA: usize = rva(0x63501C); // dword_63501C, dword_635020, dword_635024
pub const SIZE_RVA: usize = rva(0x635038); // dword_635038
pub const ANGLE_X_RVA: usize = rva(0x63503C); // dword_63503C
pub const ANGLE_Y_RVA: usize = rva(0x635040); // dword_635040

/// An angle's sine and cosine from the game's tables: sixteen bits a turn, the low
/// four dropped.
unsafe fn sine_cosine(angle: u32) -> (u32, u32) {
    let place = ((angle & 0xFFFF) >> 4) as usize;
    unsafe {
        (
            game_read(SINES_RVA + place * 4),
            game_read(COSINES_RVA + place * 4),
        )
    }
}

/// `scene::set_turn_x` (0x426FE1).
pub unsafe fn set_turn_x(angle: u32) {
    unsafe {
        let (sine, cosine) = sine_cosine(angle);
        game_write(TURN_X_WORK_RVA, [sine, cosine]);
        game_write(TURN_X_RVA + 0x14, [cosine, sine]);
        game_write(TURN_X_RVA + 0x24, [sine ^ 0x8000_0000, cosine]);
    }
}

/// `scene::set_turn_y` (0x42702A).
pub unsafe fn set_turn_y(angle: u32) {
    unsafe {
        let (sine, cosine) = sine_cosine(angle);
        game_write(TURN_Y_WORK_RVA, [sine, cosine]);
        game_write(TURN_Y_RVA, cosine);
        game_write(TURN_Y_RVA + 8, sine ^ 0x8000_0000);
        game_write(TURN_Y_RVA + 0x20, sine);
        game_write(TURN_Y_RVA + 0x28, cosine);
    }
}

/// `scene::set_turn_z_16` (0x427072): as `set_turn_x`, about the z axis (the matrix and
/// the work of `scene::set_turn_z`, whose angle is 32 bits a turn).
pub unsafe fn set_turn_z_16(angle: u32) {
    use crate::scene::view_matrix::{TURN_Z_RVA, TURN_Z_WORK_RVA};
    unsafe {
        let (sine, cosine) = sine_cosine(angle);
        game_write(TURN_Z_WORK_RVA, [sine, cosine]);
        game_write(TURN_Z_RVA, [cosine, sine]);
        game_write(TURN_Z_RVA + 0x10, [sine ^ 0x8000_0000, cosine]);
    }
}

/// `scene::set_place` (0x4271AE).
pub unsafe fn set_place(place: [u32; 3]) {
    unsafe {
        game_write(
            PLACED_RVA + 0x30,
            place.map(|part| x87::from_int(part as i32)),
        );
        game_write(CHECKED_RVA, place[2]);
    }
}

/// `scene::multiply_matrices` (0x42720D): the first three columns of `out`, column by
/// column, each row of `a` times that column of `b`; `b`'s fourth row added to the
/// fourth. Written straight to `out`, in the original's order, so that an `out` that
/// is one of the two behaves as there.
pub unsafe fn multiply_matrices(a: usize, b: usize, out: usize) {
    unsafe {
        for column in 0..3 {
            for row in 0..4 {
                x87::row_times_column(
                    (a + row * 0x10) as *const u32,
                    (b + column * 4) as *const u32,
                    (out + row * 0x10 + column * 4) as *mut u32,
                );
            }
            x87::add_into(
                (b + 0x30 + column * 4) as *const u32,
                (out + 0x30 + column * 4) as *mut u32,
            );
        }
    }
}

/// `scene::copy_matrix_with_place` (0x4271E4): `out` = `from`, dword by dword upwards,
/// then the first three floats of `other`'s fourth row added to `out`'s.
pub unsafe fn copy_matrix_with_place(from: usize, other: usize, out: usize) {
    unsafe {
        for n in 0..0x10 {
            let dword = ((from + n * 4) as *const u32).read_unaligned();
            ((out + n * 4) as *mut u32).write_unaligned(dword);
        }
        for n in 0..3 {
            x87::add_into(
                (other + 0x30 + n * 4) as *const u32,
                (out + 0x30 + n * 4) as *mut u32,
            );
        }
    }
}

/// `scene::set_light_direction` (0x427326).
pub unsafe fn set_light_direction() {
    unsafe {
        set_turn_x((game_read::<u32>(LIGHT_TURN_X_RVA) << 8).wrapping_neg());
        set_turn_y((game_read::<u32>(LIGHT_TURN_Y_RVA) << 8).wrapping_neg());
        multiply_matrices(
            game_addr(TURN_X_RVA),
            game_addr(TURN_Y_RVA),
            game_addr(TURNED_RVA),
        );
        multiply_matrices(
            game_addr(TURNED_RVA),
            game_addr(VIEW_TURN_RVA),
            game_addr(LIT_RVA),
        );
        // `fld1; fldz; fldz; call scene::rotate_point`, the results popped in turn.
        let ahead = [0i32, 0, 1];
        rotate(
            ahead.as_ptr(),
            game_ptr(LIT_RVA),
            (game_addr(LIGHT_DIRECTION_RVA) - 0x10) as *mut u32,
        );
    }
}

/// `scene::make_object_matrix` (0x427812).
pub unsafe fn make_object_matrix() {
    unsafe {
        set_light_direction();
        set_turn_y(game_read::<u32>(ANGLE_Y_RVA).wrapping_neg());
        set_turn_x(game_read::<u32>(ANGLE_X_RVA).wrapping_neg());
        let place: [u32; 3] = game_read(PLACE_RVA);
        set_place(place);
        multiply_matrices(
            game_addr(TURN_Y_RVA),
            game_addr(TURN_X_RVA),
            game_addr(TURNED_RVA),
        );
        let size: u32 = game_read(SIZE_RVA);
        let factor = x87::int_over_16(size as i32);
        game_write(CHECKED_RVA, factor);
        for n in 0..0x10 {
            let at = TURNED_RVA + n * 4;
            game_write(at, x87::mul(game_read(at), factor));
        }
        copy_matrix_with_place(
            game_addr(TURNED_RVA),
            game_addr(PLACED_RVA),
            game_addr(OBJECT_RVA),
        );
    }
}
