//! The game's 3 x 3 matrix helpers at 0x46574E .. 0x46587C (IDA, 2026-10-16), used by the
//! marks: floats, rows of 12 bytes. The three that make a turn (0x465783, 0x4657A6,
//! 0x4657C9) take their angle in st(0) and pop it: their hooks use `angle_hook!`, whose
//! thunk keeps the angle (80 bits) for the Rust version and loads it again for the original.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::math::x87;
use crate::math::x87::x87;
use crate::scene::vector3::{read_floats, write_floats};
use std::cell::UnsafeCell;

/// `scene::identity_matrix3` (0x46574E): the matrix at esi the identity (1.0 from `fld1`).
/// Every register kept.
pub unsafe fn identity_matrix3(at: u32) {
    unsafe { write_floats(at, &rc2k_core::vector::IDENTITY) };
}

/// `scene::multiply_matrices3` (0x4657EC): ebp = esi x edi, column by column, each written
/// as made (so an ebp that is one of the two behaves as there). Leaves ecx and edx 0, edi
/// and ebp 12 on.
pub unsafe fn multiply_matrices3(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        for column in 0..3 {
            for row in 0..3 {
                x87::row_times_column_3(
                    (g.esi + 12 * row) as *const u32,
                    (g.edi + 4 * column) as *const u32,
                    (g.ebp + 12 * row + 4 * column) as *mut u32,
                );
            }
        }
    }
    g.ecx = 0;
    g.edx = 0;
    g.edi = g.edi.wrapping_add(12);
    g.ebp = g.ebp.wrapping_add(12);
    g
}

/// The converted `multiply_matrices3` (`rc2k_core::vector`, plain floats): the hook's.
pub unsafe fn multiply_matrices3_plain(entry: &Regs) -> Regs {
    use rc2k_core::vector::row_times_column;
    let mut g = *entry;
    unsafe {
        for column in 0..3 {
            for row in 0..3 {
                let a: [f32; 3] = read_floats(g.esi + 12 * row);
                let b = g.edi + 4 * column;
                let b = [b, b + 0xC, b + 0x18].map(|at| read_floats::<1>(at)[0]);
                write_floats(g.ebp + 12 * row + 4 * column, &[row_times_column(a, b)]);
            }
        }
    }
    g.ecx = 0;
    g.edx = 0;
    g.edi = g.edi.wrapping_add(12);
    g.ebp = g.ebp.wrapping_add(12);
    g
}

/// `scene::transform_int_points` (0x465829): ecx points of three whole numbers from esi
/// turned by the matrix at ebp into edi (`math::x87::turn_int_point`). Leaves ecx -1 (one
/// less than it was when negative), esi and edi past them. Refuses ecx 0x80000000 (the
/// count would run away).
pub unsafe fn transform_int_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.ecx == 0x8000_0000 {
        return Err(BadInput("a count of 0x80000000: the original runs away").into());
    }
    unsafe {
        for _ in 0..(g.ecx as i32).max(0) {
            x87::turn_int_point(g.esi as *const i32, g.ebp as *const u32, g.edi as *mut i32);
            g.esi = g.esi.wrapping_add(12);
            g.edi = g.edi.wrapping_add(12);
        }
    }
    g.ecx = if (g.ecx as i32) < 0 {
        g.ecx - 1
    } else {
        u32::MAX
    };
    Ok(g)
}

/// The converted `transform_int_points`: the hook's.
pub unsafe fn transform_int_points_plain(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.ecx == 0x8000_0000 {
        return Err(BadInput("a count of 0x80000000: the original runs away").into());
    }
    unsafe {
        for _ in 0..(g.ecx as i32).max(0) {
            let m: [f32; 9] = read_floats(g.ebp);
            let p: [i32; 3] =
                std::array::from_fn(|n| ((g.esi + 4 * n as u32) as *const i32).read_unaligned());
            let out = rc2k_core::vector::turn_int_point(p, &m);
            for (n, v) in out.into_iter().enumerate() {
                ((g.edi + 4 * n as u32) as *mut i32).write_unaligned(v);
            }
            g.esi = g.esi.wrapping_add(12);
            g.edi = g.edi.wrapping_add(12);
        }
    }
    g.ecx = if (g.ecx as i32) < 0 {
        g.ecx - 1
    } else {
        u32::MAX
    };
    Ok(g)
}

/// Floats by lot for the tests: mostly within ±4, a few large, some negative zero.
pub fn test_floats(seed: u32, out: &mut [u32]) {
    let mut s = seed;
    for v in out.iter_mut() {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        let f = ((s >> 8) as i32 % 0x8000) as f32 / 4096.0;
        *v = match s % 13 {
            0 => 0x8000_0000,
            1 => (f * 70000.0).to_bits(),
            _ => f.to_bits(),
        };
    }
}

/// An angle as the game left it in st(0): the ten bytes of an 80-bit value.
pub struct Angle(pub UnsafeCell<[u8; 10]>);

unsafe impl Sync for Angle {}

impl Angle {
    pub const fn new() -> Self {
        Self(UnsafeCell::new([0; 10]))
    }

    pub fn ptr(&self) -> *mut u8 {
        self.0.get() as *mut u8
    }

    pub fn set(&self, bytes: [u8; 10]) {
        unsafe { *self.0.get() = bytes };
    }

    pub fn get(&self) -> [u8; 10] {
        unsafe { *self.0.get() }
    }
}

impl Default for Angle {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function takes an angle in st(0) and pops it: `$thunk` takes the
/// angle off into `$angle` and calls `$dispatch(regs)` (gated as `hook_asm!`'s);
/// `$with_angle(regs)` loads `$angle` into st(0) and runs `$call_original`, made here too.
macro_rules! angle_hook {
    ($thunk:ident, $call_original:ident, $with_angle:ident, dispatch = $dispatch:ident,
     original = $orig:ident, angle = $angle:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 12]",
                "fstp tbyte ptr [esp]", // the angle, popped as the original would
                "pushfd",
                "pushad",
                "cld",
                "call {enter}",
                "push esp",             // arg: *mut Regs
                "lea eax, [esp + 40]",
                "push eax",
                "call {keep}",          // the angle into $angle
                "add esp, 4",
                "call {dispatch}",
                "add esp, 4",
                "call {leave}",
                "popad",
                "popfd",
                "lea esp, [esp + 12]",  // no flags touched
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_angle,
                dispatch = sym $dispatch,
            )
        }

        extern "C" fn keep_angle(at: *const [u8; 10]) {
            $angle.set(unsafe { at.read_unaligned() });
        }

        /// The original with `$angle` in st(0).
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_angle(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "fld tbyte ptr [{a}]",
                "jmp {co}",
                a = sym $angle,
                co = sym $call_original,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use angle_hook;

/// The axis a turn is about.
#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Refuses an angle `fsincos` leaves alone (finite, 2^63 or more): the original's stack
/// would then come out one short. Infinities and NaNs give NaNs, pushed as usual.
pub fn turnable(angle: &[u8; 10]) -> Result<(), Refused> {
    let exponent = u16::from_le_bytes([angle[8], angle[9]]) & 0x7FFF;
    if (0x3FFF + 63..0x7FFF).contains(&exponent) {
        return Err(BadInput("an angle fsincos cannot take").into());
    }
    Ok(())
}

/// `scene::turn_matrix3_x` (0x465783), `_y` (0x4657A6), `_z` (0x4657C9): the matrix at esi
/// a turn by the angle about that axis (the original's `fsincos` and stores, in its order).
/// Every register kept. Refuses an angle `fsincos` cannot take.
pub unsafe fn turn_matrix3(axis: Axis, at: u32, angle: [u8; 10]) -> Result<(), Refused> {
    turnable(&angle)?;
    let a = angle.as_ptr();
    match axis {
        Axis::X => x87!(
            "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m}]", "fldz",
            "fst dword ptr [{m} + 4]", "fst dword ptr [{m} + 8]",
            "fst dword ptr [{m} + 0xC]", "fstp dword ptr [{m} + 0x18]",
            "fst dword ptr [{m} + 0x10]", "fstp dword ptr [{m} + 0x20]",
            "fst dword ptr [{m} + 0x14]", "fchs", "fstp dword ptr [{m} + 0x1C]";
            a = a, m = at
        ),
        Axis::Y => x87!(
            "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m} + 0x10]",
            "fldz", "fst dword ptr [{m} + 4]", "fst dword ptr [{m} + 0xC]",
            "fst dword ptr [{m} + 0x14]", "fstp dword ptr [{m} + 0x1C]",
            "fst dword ptr [{m}]", "fstp dword ptr [{m} + 0x20]",
            "fst dword ptr [{m} + 8]", "fchs", "fstp dword ptr [{m} + 0x18]";
            a = a, m = at
        ),
        Axis::Z => x87!(
            "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m} + 0x20]",
            "fldz", "fst dword ptr [{m} + 8]", "fst dword ptr [{m} + 0x14]",
            "fst dword ptr [{m} + 0x18]", "fstp dword ptr [{m} + 0x1C]",
            "fst dword ptr [{m}]", "fstp dword ptr [{m} + 0x10]",
            "fst dword ptr [{m} + 4]", "fchs", "fstp dword ptr [{m} + 0xC]";
            a = a, m = at
        ),
    }
    Ok(())
}

/// The converted `turn_matrix3` (`rc2k_core::vector`, plain floats): the hooks'.
pub unsafe fn turn_matrix3_plain(axis: Axis, at: u32, angle: [u8; 10]) -> Result<(), Refused> {
    use rc2k_core::vector::{Axis as A, turn_matrix};
    turnable(&angle)?;
    let axis = match axis {
        Axis::X => A::X,
        Axis::Y => A::Y,
        Axis::Z => A::Z,
    };
    let m = turn_matrix(axis, rc2k_core::extended::to_f64(angle));
    unsafe { write_floats(at, &m) };
    Ok(())
}

/// A float made an 80-bit value (`fld dword; fstp tbyte`).
pub fn extended(f: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let o = out.as_mut_ptr();
    x87!("fld dword ptr [{f}]", "fstp tbyte ptr [{o}]"; f = &f as *const u32, o = o);
    out
}

/// pi times `n` over 8 as an 80-bit value, not a float (`fldpi`, `fimul`, `fidiv`).
pub fn extended_pi(n: i32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let o = out.as_mut_ptr();
    let eight = 8i32;
    x87!(
        "fldpi", "fimul dword ptr [{n}]", "fidiv dword ptr [{e}]", "fstp tbyte ptr [{o}]";
        n = &n as *const i32, e = &eight as *const i32, o = o
    );
    out
}

/// An 80-bit value rounded to a float (`fld tbyte; fstp dword`).
pub fn rounded(v: [u8; 10]) -> u32 {
    let mut out = 0u32;
    x87!("fld tbyte ptr [{v}]", "fstp dword ptr [{o}]"; v = v.as_ptr(), o = &mut out as *mut u32);
    out
}
