//! The matrices an object's place in the view is made with, and the functions that
//! fill them: shared by their seven hooks.
//!
//! A matrix is sixteen floats, rows 0x10 apart; a point is turned by the first three
//! rows and columns and moved by the fourth row's first three floats.
//! The two turning matrices keep their other floats (an identity's) from the image.

use crate::framework::{
    EAX, EBP, EBX, ECX, EDI, EDX, ESI, game_addr, game_ptr, game_read, game_write, rva,
};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::project_point::rotate;
use crate::screens::harness::Left;
use crate::service::harness::Region;

pub const TURN_X_RVA: usize = rva(0x5E9F8C); // flt_5E9F8C
pub const TURN_Y_RVA: usize = rva(0x5E9FCC); // flt_5E9FCC
pub const PLACED_RVA: usize = rva(0x5EA04C); // flt_5EA04C, its fourth row the place
pub const TURNED_RVA: usize = rva(0x5EA10C); // flt_5EA10C
pub const LIT_RVA: usize = rva(0x5EA14C); // flt_5EA14C
pub const VIEW_TURN_RVA: usize = rva(0x5EA24C); // dword_5EA24C
pub const OBJECT_RVA: usize = rva(0x5EA30C); // flt_5EA30C
pub const MATRIX_LEN: usize = 0x40;

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

pub fn matrix(rva: usize) -> Region {
    (game_addr(rva) as u32, MATRIX_LEN)
}

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

fn turned(sine: u32, cosine: u32, matrix: usize) -> Left {
    Left::nothing()
        .with(EAX, sine)
        .with(EBX, cosine)
        .with(ECX, sine ^ 0x8000_0000)
        .with(EDI, game_addr(matrix) as u32)
}

/// `scene::set_turn_x` (0x426FE1).
pub unsafe fn set_turn_x(angle: u32) -> Left {
    unsafe {
        let (sine, cosine) = sine_cosine(angle);
        game_write(TURN_X_WORK_RVA, [sine, cosine]);
        game_write(TURN_X_RVA + 0x14, [cosine, sine]);
        game_write(TURN_X_RVA + 0x24, [sine ^ 0x8000_0000, cosine]);
        turned(sine, cosine, TURN_X_RVA)
    }
}

/// `scene::set_turn_y` (0x42702A).
pub unsafe fn set_turn_y(angle: u32) -> Left {
    unsafe {
        let (sine, cosine) = sine_cosine(angle);
        game_write(TURN_Y_WORK_RVA, [sine, cosine]);
        game_write(TURN_Y_RVA, cosine);
        game_write(TURN_Y_RVA + 8, sine ^ 0x8000_0000);
        game_write(TURN_Y_RVA + 0x20, sine);
        game_write(TURN_Y_RVA + 0x28, cosine);
        turned(sine, cosine, TURN_Y_RVA)
    }
}

/// `scene::set_turn_z_16` (0x427072): as `set_turn_x`, about the z axis (the matrix and
/// the work of `scene::set_turn_z`, whose angle is 32 bits a turn).
pub unsafe fn set_turn_z_16(angle: u32) -> Left {
    use crate::scene::view_matrix::{TURN_Z_RVA, TURN_Z_WORK_RVA};
    unsafe {
        let (sine, cosine) = sine_cosine(angle);
        game_write(TURN_Z_WORK_RVA, [sine, cosine]);
        game_write(TURN_Z_RVA, [cosine, sine]);
        game_write(TURN_Z_RVA + 0x10, [sine ^ 0x8000_0000, cosine]);
        turned(sine, cosine, TURN_Z_RVA)
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
pub unsafe fn multiply_matrices(a: usize, b: usize, out: usize) -> Left {
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
    Left::nothing().with(ECX, 0).with(EDX, 0)
}

/// `scene::copy_matrix_with_place` (0x4271E4): `out` = `from`, dword by dword upwards,
/// then the first three floats of `other`'s fourth row added to `out`'s.
pub unsafe fn copy_matrix_with_place(from: usize, other: usize, out: usize) -> Left {
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
    Left::nothing().with(ECX, 0)
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
pub unsafe fn make_object_matrix() -> Left {
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
        Left::nothing()
            .with(EAX, place[0])
            .with(EBX, size)
            .with(ECX, 0)
            .with(EDX, 0)
            .with(ESI, game_addr(TURNED_RVA) as u32)
            .with(EDI, game_addr(PLACED_RVA) as u32)
            .with(EBP, game_addr(OBJECT_RVA) as u32)
    }
}

// ---- the converted versions (`rc2k_core::scene::matrices`) ----

/// The game's memory as the core's `Places`: a place is an address (unaligned allowed).
pub struct GamePlaces(());

impl GamePlaces {
    /// # Safety
    /// Every place the core is given must be the game's memory, readable and writable
    /// where the original reads and writes.
    pub unsafe fn new() -> Self {
        Self(())
    }
}

impl rc2k_core::scene::places::Places for GamePlaces {
    fn dword(&self, at: u32) -> u32 {
        // SAFETY: as `new` says.
        unsafe { (at as *const u32).read_unaligned() }
    }

    fn set_dword(&mut self, at: u32, value: u32) {
        // SAFETY: as `new` says.
        unsafe { (at as *mut u32).write_unaligned(value) }
    }
}

/// Where the matrices and their work are in the game.
pub fn layout() -> rc2k_core::scene::matrices::Layout {
    use crate::scene::view_matrix::*;
    let a = |rva: usize| game_addr(rva) as u32;
    rc2k_core::scene::matrices::Layout {
        turn_x: a(TURN_X_RVA),
        turn_y: a(TURN_Y_RVA),
        turn_z: a(TURN_Z_RVA),
        placed: a(PLACED_RVA),
        turned: a(TURNED_RVA),
        lit: a(LIT_RVA),
        view_turn: a(VIEW_TURN_RVA),
        object: a(OBJECT_RVA),
        identity: a(IDENTITY_RVA),
        kept_place: a(KEPT_PLACE_RVA),
        view_back: a(VIEW_BACK_RVA),
        view: a(VIEW_RVA),
        turn_x_work: a(TURN_X_WORK_RVA),
        turn_y_work: a(TURN_Y_WORK_RVA),
        turn_z_work: a(TURN_Z_WORK_RVA),
        light_direction: a(LIGHT_DIRECTION_RVA),
        checked: a(CHECKED_RVA),
        squared: a(SQUARED_RVA),
        inverted: a(INVERTED_RVA),
        sines: a(SINES_RVA),
        cosines: a(COSINES_RVA),
        half_turns: a(HALF_TURNS_RVA),
        turn: a(TURN_RVA),
        nothing: a(NOTHING_RVA),
    }
}

/// A table turning through the core, with the registers it leaves.
unsafe fn set_table_turn_plain(axis: rc2k_core::scene::matrices::Axis, angle: u32) -> Left {
    let l = layout();
    let t = rc2k_core::scene::matrices::set_table_turn(
        &mut unsafe { GamePlaces::new() },
        &l,
        axis,
        angle,
    );
    Left::nothing()
        .with(EAX, t.sine)
        .with(EBX, t.cosine)
        .with(ECX, t.negated())
        .with(EDI, axis.places(&l).0)
}

/// The converted `set_turn_x`: the hook's.
// Converted: `rc2k_core::scene::matrices::set_table_turn`
pub unsafe fn set_turn_x_plain(angle: u32) -> Left {
    unsafe { set_table_turn_plain(rc2k_core::scene::matrices::Axis::X, angle) }
}

/// The converted `set_turn_y`: the hook's.
// Converted: `rc2k_core::scene::matrices::set_table_turn`
pub unsafe fn set_turn_y_plain(angle: u32) -> Left {
    unsafe { set_table_turn_plain(rc2k_core::scene::matrices::Axis::Y, angle) }
}

/// The converted `set_turn_z_16`: the hook's.
// Converted: `rc2k_core::scene::matrices::set_table_turn`
pub unsafe fn set_turn_z_16_plain(angle: u32) -> Left {
    unsafe { set_table_turn_plain(rc2k_core::scene::matrices::Axis::Z, angle) }
}

/// The converted `set_place`: the hook's.
// Converted: `rc2k_core::scene::matrices::set_place`
pub unsafe fn set_place_plain(place: [u32; 3]) {
    rc2k_core::scene::matrices::set_place(&mut unsafe { GamePlaces::new() }, &layout(), place);
}

/// The converted `multiply_matrices` (plain floats): the hook's.
// Converted: `rc2k_core::scene::matrices::multiply`
pub unsafe fn multiply_matrices_plain(a: usize, b: usize, out: usize) -> Left {
    let p = &mut unsafe { GamePlaces::new() };
    rc2k_core::scene::matrices::multiply(p, a as u32, b as u32, out as u32);
    Left::nothing().with(ECX, 0).with(EDX, 0)
}

/// The converted `copy_matrix_with_place` (plain floats): the hook's.
// Converted: `rc2k_core::scene::matrices::copy_with_place`
pub unsafe fn copy_matrix_with_place_plain(from: usize, other: usize, out: usize) -> Left {
    let p = &mut unsafe { GamePlaces::new() };
    rc2k_core::scene::matrices::copy_with_place(p, from as u32, other as u32, out as u32);
    Left::nothing().with(ECX, 0)
}

/// The light's two angles (x, y), as `set_light_direction` reads them.
unsafe fn light_angles() -> [u32; 2] {
    unsafe { [game_read(LIGHT_TURN_X_RVA), game_read(LIGHT_TURN_Y_RVA)] }
}

/// The converted `set_light_direction` (plain floats): the hook's.
// Converted: `rc2k_core::scene::matrices::set_light_direction`
pub unsafe fn set_light_direction_plain() {
    unsafe {
        rc2k_core::scene::matrices::set_light_direction(
            &mut GamePlaces::new(),
            &layout(),
            light_angles(),
        )
    };
}

/// The converted `make_object_matrix` (plain floats): the hook's.
// Converted: `rc2k_core::scene::matrices::make_object_matrix`
pub unsafe fn make_object_matrix_plain() -> Left {
    use rc2k_core::scene::matrices::{Object, make_object_matrix};
    unsafe {
        let object = Object {
            angles: [game_read(ANGLE_Y_RVA), game_read(ANGLE_X_RVA)],
            place: game_read(PLACE_RVA),
            size: game_read(SIZE_RVA),
        };
        make_object_matrix(&mut GamePlaces::new(), &layout(), light_angles(), object);
        Left::nothing()
            .with(EAX, object.place[0])
            .with(EBX, object.size)
            .with(ECX, 0)
            .with(EDX, 0)
            .with(ESI, game_addr(TURNED_RVA) as u32)
            .with(EDI, game_addr(PLACED_RVA) as u32)
            .with(EBP, game_addr(OBJECT_RVA) as u32)
    }
}

/// Each region a float place held to the converted functions' tolerance.
pub fn float_places(regions: &[Region]) -> Vec<(Region, crate::service::floats::Tolerance)> {
    regions
        .iter()
        .map(|&r| (r, crate::service::floats::FLOAT))
        .collect()
}

// ---- for the self-tests ----

/// Floats for a test's matrix, by lot: mostly of a size a matrix has, some not.
pub fn test_floats(seed: &mut u32) -> [u32; 16] {
    std::array::from_fn(|_| {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let lot = *seed >> 8;
        match lot % 23 {
            0 => 0,
            1 => 0x8000_0000,
            2 => 1.0f32.to_bits(),
            3 => 1e20f32.to_bits(),
            4 => f32::NAN.to_bits(),
            5 => 1e-30f32.to_bits(),
            _ => (((lot / 23) % 20001) as f32 / 4097.0 - 2.4).to_bits(),
        }
    })
}

/// Every place the two composed functions write, and the variables they read.
pub fn composed_regions() -> Vec<Region> {
    let mut regions: Vec<Region> = [
        TURN_X_RVA, TURN_Y_RVA, PLACED_RVA, TURNED_RVA, LIT_RVA, OBJECT_RVA,
    ]
    .map(matrix)
    .to_vec();
    regions.extend([
        (game_addr(TURN_X_WORK_RVA) as u32, 8),
        (game_addr(TURN_Y_WORK_RVA) as u32, 8),
        (game_addr(LIGHT_DIRECTION_RVA) as u32, 12),
        (game_addr(CHECKED_RVA) as u32, 4),
    ]);
    regions
}

pub fn composed_inputs() -> Vec<Region> {
    vec![
        matrix(VIEW_TURN_RVA),
        (game_addr(LIGHT_TURN_Y_RVA) as u32, 8),
        (game_addr(PLACE_RVA) as u32, 12),
        (game_addr(SIZE_RVA) as u32, 12),
    ]
}

/// A world of the test's for the composed functions: every matrix and variable set.
pub unsafe fn set_composed(seed: &mut u32, angles: [u32; 4], place: [u32; 3], size: u32) {
    unsafe {
        for at in [
            TURN_X_RVA,
            TURN_Y_RVA,
            PLACED_RVA,
            TURNED_RVA,
            LIT_RVA,
            OBJECT_RVA,
            VIEW_TURN_RVA,
        ] {
            game_write(at, test_floats(seed));
        }
        game_write(TURN_X_WORK_RVA, [0xAAAA_AAAAu32; 4]);
        game_write(LIGHT_DIRECTION_RVA, [0xAAAA_AAAAu32; 3]);
        game_write(CHECKED_RVA, 0xAAAA_AAAAu32);
        game_write(LIGHT_TURN_Y_RVA, [angles[0], angles[1]]);
        game_write(ANGLE_X_RVA, [angles[2], angles[3]]);
        game_write(PLACE_RVA, place);
        game_write(SIZE_RVA, size);
    }
}
