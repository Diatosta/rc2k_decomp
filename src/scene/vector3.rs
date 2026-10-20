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

/// `n` floats at `at` (the game's memory, unaligned).
pub unsafe fn read_floats<const N: usize>(at: u32) -> [f32; N] {
    std::array::from_fn(|n| unsafe { ((at + 4 * n as u32) as *const f32).read_unaligned() })
}

/// The floats written at `at`.
pub unsafe fn write_floats(at: u32, v: &[f32]) {
    for (n, x) in v.iter().enumerate() {
        unsafe { ((at + 4 * n as u32) as *mut f32).write_unaligned(*x) };
    }
}

/// The converted `normalise_vector3` (`rc2k_core::math::vector`, plain floats): the hook's.
pub unsafe fn normalise_vector3_plain(v: u32) {
    unsafe { write_floats(v, &rc2k_core::math::vector::normalise(read_floats(v))) };
}

/// The converted `dot_product3`: the hook's.
pub unsafe fn dot_product3_plain(a: u32, b: u32) -> [u8; 10] {
    let dot = unsafe { rc2k_core::math::vector::dot(read_floats(a), read_floats(b)) };
    rc2k_core::math::extended::from_f64(dot)
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

/// The converted `angle_between_vectors`: the hook's. The vectors normalised through
/// memory in the original's order (so that overlapping ones behave as there).
pub unsafe fn angle_between_vectors_plain(a: u32, b: u32) -> [u8; 10] {
    use rc2k_core::math::vector::{Search, dot, search_angle};
    unsafe {
        normalise_vector3_plain(b);
        normalise_vector3_plain(a);
        let cosine = dot(read_floats(a), read_floats(b)) as f32;
        let float = |r: usize| f32::from_bits(game_read(r));
        let search = Search {
            start: float(START_RVA),
            step: float(STEP_RVA),
            half: float(HALF_RVA),
        };
        let (angle, status) = search_angle(cosine, search);
        game_write(STATUS_RVA, status);
        rc2k_core::math::extended::from_f64(angle as f64)
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

/// Three 80-bit values in (st(0) .. st(2)) and three out, for `triple_hook!`.
#[repr(C)]
pub struct Triple80 {
    pub inputs: UnsafeCell<[[u8; 10]; 3]>,
    pub outputs: UnsafeCell<[[u8; 10]; 3]>,
}

unsafe impl Sync for Triple80 {}

impl Triple80 {
    pub const fn new() -> Self {
        Self {
            inputs: UnsafeCell::new([[0; 10]; 3]),
            outputs: UnsafeCell::new([[0; 10]; 3]),
        }
    }

    pub fn inputs(&self) -> [[u8; 10]; 3] {
        unsafe { *self.inputs.get() }
    }

    pub fn set_inputs(&self, v: [[u8; 10]; 3]) {
        unsafe { *self.inputs.get() = v };
    }

    pub fn outputs(&self) -> [[u8; 10]; 3] {
        unsafe { *self.outputs.get() }
    }

    pub fn set_outputs(&self, v: [[u8; 10]; 3]) {
        unsafe { *self.outputs.get() = v };
    }

    pub fn output_region(&self) -> crate::service::harness::Region {
        (self.outputs.get() as u32, 30)
    }
}

impl Default for Triple80 {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function takes st(0) .. st(2) and leaves three values there: as
/// `pair_hook!` with three (inputs at +0, +10, +20, outputs at +30, +40, +50).
macro_rules! triple_hook {
    ($thunk:ident, $call_original:ident, $with_triple:ident, dispatch = $dispatch:ident,
     original = $orig:ident, triple = $triple:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 30]",
                "fstp tbyte ptr [esp]",
                "fstp tbyte ptr [esp + 10]",
                "fstp tbyte ptr [esp + 20]",
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
                "lea esp, [esp + 30]",  // no flags touched from here
                "fld tbyte ptr [{p} + 50]",
                "fld tbyte ptr [{p} + 40]",
                "fld tbyte ptr [{p} + 30]",
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_triple,
                dispatch = sym $dispatch,
                p = sym $triple,
            )
        }

        extern "C" fn keep_triple(at: *const [[u8; 10]; 3]) {
            $triple.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) .. st(2); its three taken off into the
        /// outputs.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_triple(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 20]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "fstp tbyte ptr [{p} + 30]",
                "fstp tbyte ptr [{p} + 40]",
                "fstp tbyte ptr [{p} + 50]",
                "ret",
                co = sym $call_original,
                p = sym $triple,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use triple_hook;

/// Four 80-bit values in (st(0) .. st(3)) and three out, for `quad_triple_hook!`.
#[repr(C)]
pub struct Quad80 {
    pub inputs: UnsafeCell<[[u8; 10]; 4]>,
    pub outputs: UnsafeCell<[[u8; 10]; 3]>,
}

unsafe impl Sync for Quad80 {}

impl Quad80 {
    pub const fn new() -> Self {
        Self {
            inputs: UnsafeCell::new([[0; 10]; 4]),
            outputs: UnsafeCell::new([[0; 10]; 3]),
        }
    }

    pub fn inputs(&self) -> [[u8; 10]; 4] {
        unsafe { *self.inputs.get() }
    }

    pub fn set_inputs(&self, v: [[u8; 10]; 4]) {
        unsafe { *self.inputs.get() = v };
    }

    pub fn set_outputs(&self, v: [[u8; 10]; 3]) {
        unsafe { *self.outputs.get() = v };
    }

    pub fn output_region(&self) -> crate::service::harness::Region {
        (self.outputs.get() as u32, 30)
    }
}

impl Default for Quad80 {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function takes st(0) .. st(3) and leaves three values: as
/// `triple_hook!` with four in (inputs at +0 .. +30, outputs at +40, +50, +60).
macro_rules! quad_triple_hook {
    ($thunk:ident, $call_original:ident, $with_quad:ident, dispatch = $dispatch:ident,
     original = $orig:ident, quad = $quad:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 40]",
                "fstp tbyte ptr [esp]",
                "fstp tbyte ptr [esp + 10]",
                "fstp tbyte ptr [esp + 20]",
                "fstp tbyte ptr [esp + 30]",
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
                "lea esp, [esp + 40]",  // no flags touched from here
                "fld tbyte ptr [{p} + 60]",
                "fld tbyte ptr [{p} + 50]",
                "fld tbyte ptr [{p} + 40]",
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_quad,
                dispatch = sym $dispatch,
                p = sym $quad,
            )
        }

        extern "C" fn keep_quad(at: *const [[u8; 10]; 4]) {
            $quad.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) .. st(3); its three taken off into the
        /// outputs.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_quad(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 30]",
                "fld tbyte ptr [{p} + 20]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "fstp tbyte ptr [{p} + 40]",
                "fstp tbyte ptr [{p} + 50]",
                "fstp tbyte ptr [{p} + 60]",
                "ret",
                co = sym $call_original,
                p = sym $quad,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use quad_triple_hook;

/// For a hook whose function takes nothing on the x87 stack and leaves three values there
/// (st(0) .. st(2)): `$thunk` calls `$dispatch(regs)` (gated as `hook_asm!`'s) and loads
/// `$triple.outputs` (the third first); `$with_out3(regs)` runs `$call_original` and pops
/// its three into the outputs. `Triple80`'s inputs are not used.
macro_rules! out3_hook {
    ($thunk:ident, $call_original:ident, $with_out3:ident, dispatch = $dispatch:ident,
     original = $orig:ident, triple = $triple:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "pushfd",
                "pushad",
                "cld",
                "call {enter}",
                "push esp", // arg: *mut Regs
                "call {dispatch}",
                "add esp, 4",
                "call {leave}",
                "popad",
                "popfd", // no flags touched from here
                "fld tbyte ptr [{p} + 50]",
                "fld tbyte ptr [{p} + 40]",
                "fld tbyte ptr [{p} + 30]",
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                dispatch = sym $dispatch,
                p = sym $triple,
            )
        }

        /// The original; its three taken off into the outputs.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_out3(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "call {co}",
                "add esp, 4",
                "fstp tbyte ptr [{p} + 30]",
                "fstp tbyte ptr [{p} + 40]",
                "fstp tbyte ptr [{p} + 50]",
                "ret",
                co = sym $call_original,
                p = sym $triple,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use out3_hook;

/// Six 80-bit values in (st(0) .. st(5)), none out, for `six_in_hook!`.
#[repr(C)]
pub struct Six80 {
    pub inputs: UnsafeCell<[[u8; 10]; 6]>,
}

unsafe impl Sync for Six80 {}

impl Six80 {
    pub const fn new() -> Self {
        Self {
            inputs: UnsafeCell::new([[0; 10]; 6]),
        }
    }

    pub fn inputs(&self) -> [[u8; 10]; 6] {
        unsafe { *self.inputs.get() }
    }

    pub fn set_inputs(&self, v: [[u8; 10]; 6]) {
        unsafe { *self.inputs.get() = v };
    }
}

impl Default for Six80 {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function takes st(0) .. st(5) and pops them all: `$thunk` takes the
/// six off into `$six.inputs` and calls `$dispatch(regs)` (gated as `hook_asm!`'s);
/// `$with_six(regs)` loads the inputs again and runs `$call_original` (which pops them).
macro_rules! six_in_hook {
    ($thunk:ident, $call_original:ident, $with_six:ident, dispatch = $dispatch:ident,
     original = $orig:ident, six = $six:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 60]",
                "fstp tbyte ptr [esp]",
                "fstp tbyte ptr [esp + 10]",
                "fstp tbyte ptr [esp + 20]",
                "fstp tbyte ptr [esp + 30]",
                "fstp tbyte ptr [esp + 40]",
                "fstp tbyte ptr [esp + 50]",
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
                "lea esp, [esp + 60]",  // no flags touched from here
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_six,
                dispatch = sym $dispatch,
            )
        }

        extern "C" fn keep_six(at: *const [[u8; 10]; 6]) {
            $six.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) .. st(5) (it pops them).
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_six(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 50]",
                "fld tbyte ptr [{p} + 40]",
                "fld tbyte ptr [{p} + 30]",
                "fld tbyte ptr [{p} + 20]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "ret",
                co = sym $call_original,
                p = sym $six,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use six_in_hook;

/// Calls code at `addr` (a test's copy) with `regs` loaded and `inputs` in st(0) ..
/// st(2), as `call_game` does with the registers; the registers and flags it returns
/// with go back into `regs`. For copies of functions that take three values on the x87
/// stack and pop them.
#[unsafe(naked)]
pub unsafe extern "C" fn call_with_three(
    _regs: *mut crate::framework::Regs,
    _addr: usize,
    _inputs: *const [[u8; 10]; 3],
) {
    core::arch::naked_asm!(
        "push ebx", "push esi", "push edi", "push ebp",
        "mov eax, [esp + 28]",           // inputs
        "fld tbyte ptr [eax + 20]",
        "fld tbyte ptr [eax + 10]",
        "fld tbyte ptr [eax]",
        "mov eax, [esp + 20]",           // eax = regs
        "push eax",                      // remember regs across the call
        "mov ecx, [eax + {o_ecx}]",
        "mov edx, [eax + {o_edx}]",
        "mov ebx, [eax + {o_ebx}]",
        "mov esi, [eax + {o_esi}]",
        "mov edi, [eax + {o_edi}]",
        "mov ebp, [eax + {o_ebp}]",
        "mov eax, [eax + {o_eax}]",
        "call dword ptr [esp + 28]",     // addr
        "pushfd",
        "cld",
        "push eax",
        "mov eax, [esp + 8]",            // eax = regs again
        "mov [eax + {o_ecx}], ecx",
        "mov [eax + {o_edx}], edx",
        "mov [eax + {o_ebx}], ebx",
        "mov [eax + {o_esi}], esi",
        "mov [eax + {o_edi}], edi",
        "mov [eax + {o_ebp}], ebp",
        "pop dword ptr [eax + {o_eax}]",
        "pop dword ptr [eax + {o_eflags}]",
        "add esp, 4",
        "pop ebp", "pop edi", "pop esi", "pop ebx",
        "ret",
        o_eax = const core::mem::offset_of!(crate::framework::Regs, eax),
        o_eflags = const core::mem::offset_of!(crate::framework::Regs, eflags),
        o_ecx = const core::mem::offset_of!(crate::framework::Regs, ecx),
        o_edx = const core::mem::offset_of!(crate::framework::Regs, edx),
        o_ebx = const core::mem::offset_of!(crate::framework::Regs, ebx),
        o_esi = const core::mem::offset_of!(crate::framework::Regs, esi),
        o_edi = const core::mem::offset_of!(crate::framework::Regs, edi),
        o_ebp = const core::mem::offset_of!(crate::framework::Regs, ebp),
    )
}

/// For a hook whose function takes st(0) .. st(2) and pops them (leaving nothing): `$thunk`
/// takes the three off into `$triple.inputs` and calls `$dispatch(regs)` (gated as
/// `hook_asm!`'s); `$with_three(regs)` loads them again and runs `$call_original`.
macro_rules! three_in_hook {
    ($thunk:ident, $call_original:ident, $with_three:ident, dispatch = $dispatch:ident,
     original = $orig:ident, triple = $triple:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 30]",
                "fstp tbyte ptr [esp]",
                "fstp tbyte ptr [esp + 10]",
                "fstp tbyte ptr [esp + 20]",
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
                "lea esp, [esp + 30]",  // no flags touched from here
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_three,
                dispatch = sym $dispatch,
            )
        }

        extern "C" fn keep_three(at: *const [[u8; 10]; 3]) {
            $triple.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) .. st(2) (it pops them).
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_three(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 20]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "ret",
                co = sym $call_original,
                p = sym $triple,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use three_in_hook;

/// Calls code at `addr` (a test's copy) with `regs` loaded and `inputs` in st(0) and
/// st(1), as `call_game` does with the registers; the registers and flags it returns
/// with go back into `regs`. For copies of functions that take two values on the x87
/// stack and pop them.
#[unsafe(naked)]
pub unsafe extern "C" fn call_with_two(
    _regs: *mut crate::framework::Regs,
    _addr: usize,
    _inputs: *const [[u8; 10]; 2],
) {
    core::arch::naked_asm!(
        "push ebx", "push esi", "push edi", "push ebp",
        "mov eax, [esp + 28]",           // inputs
        "fld tbyte ptr [eax + 10]",
        "fld tbyte ptr [eax]",
        "mov eax, [esp + 20]",           // eax = regs
        "push eax",                      // remember regs across the call
        "mov ecx, [eax + {o_ecx}]",
        "mov edx, [eax + {o_edx}]",
        "mov ebx, [eax + {o_ebx}]",
        "mov esi, [eax + {o_esi}]",
        "mov edi, [eax + {o_edi}]",
        "mov ebp, [eax + {o_ebp}]",
        "mov eax, [eax + {o_eax}]",
        "call dword ptr [esp + 28]",     // addr
        "pushfd",
        "cld",
        "push eax",
        "mov eax, [esp + 8]",            // eax = regs again
        "mov [eax + {o_ecx}], ecx",
        "mov [eax + {o_edx}], edx",
        "mov [eax + {o_ebx}], ebx",
        "mov [eax + {o_esi}], esi",
        "mov [eax + {o_edi}], edi",
        "mov [eax + {o_ebp}], ebp",
        "pop dword ptr [eax + {o_eax}]",
        "pop dword ptr [eax + {o_eflags}]",
        "add esp, 4",
        "pop ebp", "pop edi", "pop esi", "pop ebx",
        "ret",
        o_eax = const core::mem::offset_of!(crate::framework::Regs, eax),
        o_eflags = const core::mem::offset_of!(crate::framework::Regs, eflags),
        o_ecx = const core::mem::offset_of!(crate::framework::Regs, ecx),
        o_edx = const core::mem::offset_of!(crate::framework::Regs, edx),
        o_ebx = const core::mem::offset_of!(crate::framework::Regs, ebx),
        o_esi = const core::mem::offset_of!(crate::framework::Regs, esi),
        o_edi = const core::mem::offset_of!(crate::framework::Regs, edi),
        o_ebp = const core::mem::offset_of!(crate::framework::Regs, ebp),
    )
}

/// For a hook whose function takes st(0) and st(1) and pops them (leaving nothing): `$thunk`
/// takes the two off into `$pair.inputs` and calls `$dispatch(regs)` (gated as
/// `hook_asm!`'s); `$with_two(regs)` loads them again and runs `$call_original`.
macro_rules! two_in_hook {
    ($thunk:ident, $call_original:ident, $with_two:ident, dispatch = $dispatch:ident,
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
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_two,
                dispatch = sym $dispatch,
            )
        }

        extern "C" fn keep_two(at: *const [[u8; 10]; 2]) {
            $pair.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) and st(1) (it pops them).
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_two(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "ret",
                co = sym $call_original,
                p = sym $pair,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use two_in_hook;

/// Four 80-bit values in (st(0) .. st(3)), none out, for `four_in_hook!`.
#[repr(C)]
pub struct Four80 {
    pub inputs: UnsafeCell<[[u8; 10]; 4]>,
}

unsafe impl Sync for Four80 {}

impl Four80 {
    pub const fn new() -> Self {
        Self {
            inputs: UnsafeCell::new([[0; 10]; 4]),
        }
    }

    pub fn inputs(&self) -> [[u8; 10]; 4] {
        unsafe { *self.inputs.get() }
    }

    pub fn set_inputs(&self, v: [[u8; 10]; 4]) {
        unsafe { *self.inputs.get() = v };
    }
}

impl Default for Four80 {
    fn default() -> Self {
        Self::new()
    }
}

/// For a hook whose function takes st(0) .. st(3) and pops them all: `$thunk` takes the
/// four off into `$four.inputs` and calls `$dispatch(regs)` (gated as `hook_asm!`'s);
/// `$with_four(regs)` loads the inputs again and runs `$call_original` (which pops them).
macro_rules! four_in_hook {
    ($thunk:ident, $call_original:ident, $with_four:ident, dispatch = $dispatch:ident,
     original = $orig:ident, four = $four:ident) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "lea esp, [esp - 40]",
                "fstp tbyte ptr [esp]",
                "fstp tbyte ptr [esp + 10]",
                "fstp tbyte ptr [esp + 20]",
                "fstp tbyte ptr [esp + 30]",
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
                "lea esp, [esp + 40]",  // no flags touched from here
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_four,
                dispatch = sym $dispatch,
            )
        }

        extern "C" fn keep_four(at: *const [[u8; 10]; 4]) {
            $four.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) .. st(3) (it pops them).
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_four(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 30]",
                "fld tbyte ptr [{p} + 20]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "add esp, 4",
                "ret",
                co = sym $call_original,
                p = sym $four,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use four_in_hook;

/// Calls code at `addr` (a test's copy) with `regs` loaded and `inputs` in st(0) ..
/// st(3), as `call_game` does with the registers; the registers and flags it returns
/// with go back into `regs`. For copies of functions that take four values on the x87
/// stack and pop them.
#[unsafe(naked)]
pub unsafe extern "C" fn call_with_four(
    _regs: *mut crate::framework::Regs,
    _addr: usize,
    _inputs: *const [[u8; 10]; 4],
) {
    core::arch::naked_asm!(
        "push ebx", "push esi", "push edi", "push ebp",
        "mov eax, [esp + 28]",           // inputs
        "fld tbyte ptr [eax + 30]",
        "fld tbyte ptr [eax + 20]",
        "fld tbyte ptr [eax + 10]",
        "fld tbyte ptr [eax]",
        "mov eax, [esp + 20]",           // eax = regs
        "push eax",                      // remember regs across the call
        "mov ecx, [eax + {o_ecx}]",
        "mov edx, [eax + {o_edx}]",
        "mov ebx, [eax + {o_ebx}]",
        "mov esi, [eax + {o_esi}]",
        "mov edi, [eax + {o_edi}]",
        "mov ebp, [eax + {o_ebp}]",
        "mov eax, [eax + {o_eax}]",
        "call dword ptr [esp + 28]",     // addr
        "pushfd",
        "cld",
        "push eax",
        "mov eax, [esp + 8]",            // eax = regs again
        "mov [eax + {o_ecx}], ecx",
        "mov [eax + {o_edx}], edx",
        "mov [eax + {o_ebx}], ebx",
        "mov [eax + {o_esi}], esi",
        "mov [eax + {o_edi}], edi",
        "mov [eax + {o_ebp}], ebp",
        "pop dword ptr [eax + {o_eax}]",
        "pop dword ptr [eax + {o_eflags}]",
        "add esp, 4",
        "pop ebp", "pop edi", "pop esi", "pop ebx",
        "ret",
        o_eax = const core::mem::offset_of!(crate::framework::Regs, eax),
        o_eflags = const core::mem::offset_of!(crate::framework::Regs, eflags),
        o_ecx = const core::mem::offset_of!(crate::framework::Regs, ecx),
        o_edx = const core::mem::offset_of!(crate::framework::Regs, edx),
        o_ebx = const core::mem::offset_of!(crate::framework::Regs, ebx),
        o_esi = const core::mem::offset_of!(crate::framework::Regs, esi),
        o_edi = const core::mem::offset_of!(crate::framework::Regs, edi),
        o_ebp = const core::mem::offset_of!(crate::framework::Regs, ebp),
    )
}

/// Calls code at `addr` (a test's copy) as `call_with_two` does, for functions that read
/// st(0) and st(1) and leave them: they are taken off after the call.
#[unsafe(naked)]
pub unsafe extern "C" fn call_with_two_kept(
    _regs: *mut crate::framework::Regs,
    _addr: usize,
    _inputs: *const [[u8; 10]; 2],
) {
    core::arch::naked_asm!(
        "push ebx", "push esi", "push edi", "push ebp",
        "mov eax, [esp + 28]",           // inputs
        "fld tbyte ptr [eax + 10]",
        "fld tbyte ptr [eax]",
        "mov eax, [esp + 20]",           // eax = regs
        "push eax",                      // remember regs across the call
        "mov ecx, [eax + {o_ecx}]",
        "mov edx, [eax + {o_edx}]",
        "mov ebx, [eax + {o_ebx}]",
        "mov esi, [eax + {o_esi}]",
        "mov edi, [eax + {o_edi}]",
        "mov ebp, [eax + {o_ebp}]",
        "mov eax, [eax + {o_eax}]",
        "call dword ptr [esp + 28]",     // addr
        "fstp st(0)",
        "fstp st(0)",
        "pushfd",
        "cld",
        "push eax",
        "mov eax, [esp + 8]",            // eax = regs again
        "mov [eax + {o_ecx}], ecx",
        "mov [eax + {o_edx}], edx",
        "mov [eax + {o_ebx}], ebx",
        "mov [eax + {o_esi}], esi",
        "mov [eax + {o_edi}], edi",
        "mov [eax + {o_ebp}], ebp",
        "pop dword ptr [eax + {o_eax}]",
        "pop dword ptr [eax + {o_eflags}]",
        "add esp, 4",
        "pop ebp", "pop edi", "pop esi", "pop ebx",
        "ret",
        o_eax = const core::mem::offset_of!(crate::framework::Regs, eax),
        o_eflags = const core::mem::offset_of!(crate::framework::Regs, eflags),
        o_ecx = const core::mem::offset_of!(crate::framework::Regs, ecx),
        o_edx = const core::mem::offset_of!(crate::framework::Regs, edx),
        o_ebx = const core::mem::offset_of!(crate::framework::Regs, ebx),
        o_esi = const core::mem::offset_of!(crate::framework::Regs, esi),
        o_edi = const core::mem::offset_of!(crate::framework::Regs, edi),
        o_ebp = const core::mem::offset_of!(crate::framework::Regs, ebp),
    )
}

/// For a hook whose function reads st(0) and st(1) and leaves them: `$thunk` takes the two
/// off into `$pair.inputs`, calls `$dispatch(regs)` (gated as `hook_asm!`'s) and puts
/// them back; `$with_two(regs)` loads them, runs `$call_original` and takes them off.
macro_rules! two_kept_hook {
    ($thunk:ident, $call_original:ident, $with_two:ident, dispatch = $dispatch:ident,
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
                "fld tbyte ptr [esp + 10]", // no flags touched from here
                "fld tbyte ptr [esp]",
                "lea esp, [esp + 20]",
                "ret",
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
                keep = sym keep_two,
                dispatch = sym $dispatch,
            )
        }

        extern "C" fn keep_two(at: *const [[u8; 10]; 2]) {
            $pair.set_inputs(unsafe { at.read_unaligned() });
        }

        /// The original with the inputs in st(0) and st(1), taken off after.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $with_two(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push dword ptr [esp + 4]",
                "fld tbyte ptr [{p} + 10]",
                "fld tbyte ptr [{p}]",
                "call {co}",
                "fstp st(0)",
                "fstp st(0)",
                "add esp, 4",
                "ret",
                co = sym $call_original,
                p = sym $pair,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };
}
pub(crate) use two_kept_hook;
