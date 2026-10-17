//! The game's three-float vector helpers at 0x4654EA .. 0x4659A1 (IDA, 2026-10-16), used by
//! the marks. Two leave their result in st(0): their hooks use `result_hook!`, whose thunk
//! loads the result (80 bits) the Rust version left in the hook's `Result80`, and whose
//! `original_keeping_result` takes the original's off into it for shadow mode and tests.

use crate::framework::{game_read, game_write, rva};
use crate::math::x87::x87;
use std::cell::UnsafeCell;

/// An 80-bit result as it goes in st(0).
pub struct Result80(pub UnsafeCell<[u8; 10]>);

unsafe impl Sync for Result80 {}

impl Result80 {
    pub const fn new() -> Self {
        Self(UnsafeCell::new([0; 10]))
    }

    pub fn addr(&self) -> u32 {
        self.0.get() as u32
    }

    pub fn set(&self, bytes: [u8; 10]) {
        unsafe { *self.0.get() = bytes };
    }

    pub fn region(&self) -> crate::service::harness::Region {
        (self.addr(), 10)
    }
}

impl Default for Result80 {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function leaves a result in st(0): `$thunk` calls `$dispatch(regs)`
/// (gated as `hook_asm!`'s), then loads `$result`; `$keeping(regs)` runs `$call_original`
/// and takes its st(0) off into `$result`.
macro_rules! result_hook {
    ($thunk:ident, $call_original:ident, $keeping:ident, dispatch = $dispatch:ident,
     original = $orig:ident, result = $result:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "pushfd",
                "pushad",
                "cld",
                "call {enter}",
                "push esp",             // arg: *mut Regs
                "call {dispatch}",
                "add esp, 4",
                "call {leave}",
                "popad",
                "popfd",
                "fld tbyte ptr [{r}]",  // the result; no flags touched
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                dispatch = sym $dispatch,
                r = sym $result,
            )
        }

        /// The original, its st(0) taken off into `$result`.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $keeping(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "call {co}",
                "add esp, 4",
                "fstp tbyte ptr [{r}]",
                "ret",
                co = sym $call_original,
                r = sym $result,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use result_hook;

/// `scene::normalise_vector3` (0x4654EA): the three floats at esi each times 1 over their
/// length (`fsqrt` of the squares, the last two added first; `fld1; fdivrp`). Every
/// register kept.
pub unsafe fn normalise_vector3(v: u32) {
    x87!(
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fsqrt", "fld1", ".byte 0xDE, 0xF1",
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xCB",
        "fstp dword ptr [{v} + 8]", "fstp dword ptr [{v} + 4]", "fstp dword ptr [{v}]",
        "fstp st(0)";
        v = v
    );
}

/// `scene::dot_product3` (0x46553C): esi . edi, x x' + (y y' + z z'), unrounded. Every
/// register kept.
pub unsafe fn dot_product3(a: u32, b: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!(
        "fld dword ptr [{a}]", "fmul dword ptr [{b}]",
        "fld dword ptr [{a} + 4]", "fmul dword ptr [{b} + 4]",
        "fld dword ptr [{a} + 8]", "fmul dword ptr [{b} + 8]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fstp tbyte ptr [{o}]";
        a = a, b = b, o = out.as_mut_ptr()
    );
    out
}

const START_RVA: usize = rva(0x71CA36); // flt_71CA36, pi / 2
const STEP_RVA: usize = rva(0x71CA3A); // flt_71CA3A, pi / 4
const HALF_RVA: usize = rva(0x71CA3E); // flt_71CA3E, 0.5
pub const STATUS_RVA: usize = rva(0x71C814); // dword_71C814: the last compare's C0 .. C2

/// `fld a; fcos; fcomp b; fstsw`: the status word's C0 .. C2 (0x700).
fn cosine_against(a: u32, b: u32) -> u32 {
    let mut status: u16 = 0;
    x87!(
        "fld dword ptr [{a}]", "fcos", "fcomp dword ptr [{b}]", "fstsw word ptr [{s}]";
        a = &a as *const u32, b = &b as *const u32, s = &mut status as *mut u16
    );
    status as u32 & 0x700
}

/// `scene::angle_between_vectors` (0x4658CF): the vectors at edi and esi normalised in
/// place (that order), their dot product kept as a float, and the angle whose cosine it
/// is found by 16 halvings from pi / 2 by pi / 4 (floats each step; out when the compare
/// is unordered); the last compare's C0 .. C2 in `dword_71C814`. Every register kept.
/// Returns the angle (a float, loaded).
pub unsafe fn angle_between_vectors(a: u32, b: u32) -> [u8; 10] {
    unsafe {
        let mut angle = crate::math::x87::loaded(game_read(START_RVA));
        let mut step = crate::math::x87::loaded(game_read(STEP_RVA));
        normalise_vector3(b);
        normalise_vector3(a);
        let dot = crate::scene::matrix3::rounded(dot_product3(a, b));
        let half: u32 = game_read(HALF_RVA);
        for _ in 0..16 {
            let status = cosine_against(angle, dot);
            game_write(STATUS_RVA, status);
            match status {
                0x400 => break,
                0x100 => angle = crate::math::x87::sub(angle, step),
                _ => angle = crate::math::x87::add(angle, step),
            }
            step = crate::math::x87::mul(step, half);
        }
        crate::scene::matrix3::extended(angle)
    }
}

/// Two 80-bit values in (st(0), st(1)) and two out, for `pair_hook!`: 20 bytes each way.
#[repr(C)]
pub struct Pair80 {
    pub inputs: UnsafeCell<[[u8; 10]; 2]>,
    pub outputs: UnsafeCell<[[u8; 10]; 2]>,
}

unsafe impl Sync for Pair80 {}

impl Pair80 {
    pub const fn new() -> Self {
        Self {
            inputs: UnsafeCell::new([[0; 10]; 2]),
            outputs: UnsafeCell::new([[0; 10]; 2]),
        }
    }

    pub fn inputs(&self) -> [[u8; 10]; 2] {
        unsafe { *self.inputs.get() }
    }

    pub fn set_inputs(&self, v: [[u8; 10]; 2]) {
        unsafe { *self.inputs.get() = v };
    }

    pub fn set_outputs(&self, v: [[u8; 10]; 2]) {
        unsafe { *self.outputs.get() = v };
    }

    pub fn output_region(&self) -> crate::service::harness::Region {
        (self.outputs.get() as u32, 20)
    }
}

impl Default for Pair80 {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function takes st(0), st(1) and leaves two values there: `$thunk` pops
/// the two into `$pair.inputs`, calls `$dispatch(regs)` (gated as `hook_asm!`'s), and loads
/// `$pair.outputs` (the second first); `$with_pair(regs)` loads the inputs, runs
/// `$call_original` and pops its two into the outputs.
macro_rules! pair_hook {
    ($thunk:ident, $call_original:ident, $with_pair:ident, dispatch = $dispatch:ident,
     original = $orig:ident, pair = $pair:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 20]",
                "fstp tbyte ptr [esp]",
                "fstp tbyte ptr [esp + 10]",
                "pushfd",
                "pushad",
                "cld",
                "call {enter}",
                "push esp",             // arg: *mut Regs
                "lea eax, [esp + 40]",
                "push eax",
                "call {keep}",
                "add esp, 4",
                "call {dispatch}",
                "add esp, 4",
                "call {leave}",
                "popad",
                "popfd",
                "lea esp, [esp + 20]",  // no flags touched from here
                "fld tbyte ptr [{p} + 30]",
                "fld tbyte ptr [{p} + 20]",
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_pair,
                dispatch = sym $dispatch,
                p = sym $pair,
            )
        }

        extern "C" fn keep_pair(at: *const [[u8; 10]; 2]) {
            $pair.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0), st(1); its two taken off into the outputs.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_pair(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "fstp tbyte ptr [{p} + 20]",
                "fstp tbyte ptr [{p} + 30]",
                "ret",
                co = sym $call_original,
                p = sym $pair,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use pair_hook;
