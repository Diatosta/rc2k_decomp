//! What the DirectInput functions' hooks share (`acquire_device`, `list_joysticks`,
//! `joystick_found`, `axis_found`, `begin_force`, `end_force`, `effect_found`,
//! `make_effect`, `effect_objects`): the game's variables they use, their calls into the
//! game, and their self-tests' sides, which run on DirectInput's objects made in Rust
//! (`input::di_objects`) with a scripted backend: the original and the Rust version each
//! from the same start (the same objects at the same addresses), and what they leave
//! compared: the game's memory, the objects, the backend's notes and a result.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, SelfTest, call_game, game_addr, rva};
use crate::input::devices::com_call;
use crate::input::di_objects::{TestWorld, describe, take_test_notes};
use crate::service::harness::{Region, capture, diff_summary, restore};

pub const DIRECT_INPUT_RVA: usize = rva(0x4F6904); // dword_4F6904: the IDirectInputA
pub const NEW_DEVICE_RVA: usize = rva(0x4F6910); // dword_4F6910: a device being made
pub const NEW_DEVICE2_RVA: usize = rva(0x4F6914); // dword_4F6914: its IDirectInputDevice2A
pub const CAPS_RVA: usize = rva(0x4F6A78); // dword_4F6A78: a DIDEVCAPS (0x2C bytes)
pub const CAPS_LEN: usize = 0x2C;
pub const DWORD_PROP_RVA: usize = rva(0x4F6AA4); // dword_4F6AA4: a DIPROPDWORD (0x14)
pub const RANGE_PROP_RVA: usize = rva(0x4F6AB8); // dword_4F6AB8: a DIPROPRANGE (0x18)
pub const CONTROLLER_COUNT_RVA: usize = rva(0x4F78E0); // g_controllerCount
pub const CONTROLLERS_RVA: usize = rva(0x4F78E4); // g_controllers: 16 of 0x50 bytes
pub const CONTROLLER_LEN: u32 = 0x50;
pub const CONTROLLERS: u32 = 16;
pub const HWND_RVA: usize = rva(0x5184A4); // hWnd
pub const IID_DEVICE2_RVA: usize = rva(0x4F7E18); // IID_IDirectInputDevice2A
pub const JOYSTICK_FORMAT_RVA: usize = rva(0x4D7220); // c_dfDIJoystick
/// A record's six effects: from +0x3C, 0xA4 bytes each.
pub const EFFECTS_AT: u32 = 0x3C;
pub const EFFECT_LEN: u32 = 0xA4;

/// DirectInput's methods, by their byte offsets in the tables.
pub mod slot {
    pub const QUERY: u32 = 0x00;
    pub const RELEASE: u32 = 0x08;
    // IDirectInputA
    pub const CREATE_DEVICE: u32 = 0x0C;
    pub const ENUM_DEVICES: u32 = 0x10;
    // IDirectInputDevice2A
    pub const GET_CAPABILITIES: u32 = 0x0C;
    pub const ENUM_OBJECTS: u32 = 0x10;
    pub const SET_PROPERTY: u32 = 0x18;
    pub const ACQUIRE: u32 = 0x1C;
    pub const SET_DATA_FORMAT: u32 = 0x2C;
    pub const SET_COOPERATIVE_LEVEL: u32 = 0x34;
    pub const CREATE_EFFECT: u32 = 0x48;
    pub const ENUM_EFFECTS: u32 = 0x4C;
    // IDirectInputEffect
    pub const SET_PARAMETERS: u32 = 0x18;
    pub const START: u32 = 0x1C;
    pub const STOP: u32 = 0x20;
    pub const DOWNLOAD: u32 = 0x28;
}

/// A dword of the game's.
pub fn g(rva: usize) -> u32 {
    unsafe { crate::framework::game_read(rva) }
}

/// Writes a dword of the game's.
pub fn p(rva: usize, v: u32) {
    unsafe { crate::framework::game_write(rva, v) }
}

/// A game variable's address.
pub fn a(rva: usize) -> u32 {
    game_addr(rva) as u32
}

/// A method of a DirectInput object (`stdcall`, the object first): its result.
pub fn method(object: u32, slot: u32, args: &[u32]) -> u32 {
    unsafe { com_call(object, slot, args) }
}

/// The controller's entry for a record (`g_controllers` + 0x50 its number), unchecked as
/// the original (`imul edi, [esi], 50h`).
pub fn entry_of(record: u32) -> u32 {
    a(CONTROLLERS_RVA)
        .wrapping_add(crate::input::devices::at(record, 0).wrapping_mul(CONTROLLER_LEN))
}

/// The game's fatal error `code` with no message.
pub const fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

/// A `pusha` .. `popa` function of the game's called: every register given, the result in
/// what comes back (the function keeps them all).
pub fn call_pusha(at: usize, r: &Regs) -> Result<(), Refused> {
    let mut r = *r;
    unsafe { call_game(&mut r, game_addr(rva(at))) };
    Ok(())
}

// ---- the self-tests' sides ----

/// What a side left: the regions' bytes, the objects, the backend's notes, its result.
type Left = (Vec<u8>, String, Vec<String>, String);

/// One side: the regions from `start`, the world's side begun, `prepare`, `run`.
fn side(
    world: &TestWorld,
    regions: &[Region],
    start: &[u8],
    prepare: &dyn Fn(),
    run: &mut dyn FnMut() -> String,
) -> Left {
    restore(regions, start);
    world.side();
    prepare();
    take_test_notes();
    let result = run();
    (capture(regions), describe(), take_test_notes(), result)
}

/// One case: the original's side and the Rust version's from the same start, compared;
/// the regions put back.
#[allow(clippy::too_many_arguments)] // the case: what, world, memory, the three runs
pub fn compare(
    t: &mut SelfTest,
    what: &str,
    world: &TestWorld,
    regions: &[Region],
    prepare: &dyn Fn(),
    original: &mut dyn FnMut() -> String,
    rust: &mut dyn FnMut() -> String,
) {
    let start = capture(regions);
    let theirs = side(world, regions, &start, prepare, original);
    let mine = side(world, regions, &start, prepare, rust);
    restore(regions, &start);
    world.side();
    t.check(theirs == mine, || {
        format!(
            "{what}: original {} / rust {}; memory: {}; objects: original {} / rust {}; \
             notes: original {:?} / rust {:?}",
            theirs.3,
            mine.3,
            diff_summary(&theirs.0, &mine.0),
            theirs.1,
            mine.1,
            theirs.2,
            mine.2
        )
    });
}

/// The registers a `pusha` .. `popa` function's side gives back, for `compare`.
pub fn regs_text(r: &Regs) -> String {
    format!(
        "eax {:#x} ebx {:#x} ecx {:#x} edx {:#x} esi {:#x} edi {:#x} ebp {:#x}",
        r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp
    )
}

/// The game's DirectInput variables, and the controllers' table and count, as regions.
pub fn shared_regions() -> Vec<Region> {
    vec![
        (a(DIRECT_INPUT_RVA), 4),
        (a(NEW_DEVICE_RVA), 8),
        (a(CAPS_RVA), CAPS_LEN),
        (a(DWORD_PROP_RVA), 0x14),
        (a(RANGE_PROP_RVA), 0x18),
        (a(CONTROLLER_COUNT_RVA), 4),
        (a(CONTROLLERS_RVA), (CONTROLLERS * CONTROLLER_LEN) as usize),
    ]
}

/// A `stdcall` function of two arguments (DirectInput's enumeration callbacks): its eax.
pub unsafe fn call_stdcall2(code: usize, a: u32, b: u32) -> u32 {
    type F = unsafe extern "system" fn(u32, u32) -> u32;
    unsafe { std::mem::transmute::<usize, F>(code)(a, b) }
}

/// An enumeration callback hook's body: counted, gated; shadow mode runs the original
/// alone (it is DirectInput's caller's), else the Rust version (which refuses nothing).
pub fn dispatch_callback(
    hook: &crate::framework::RawHook,
    original: usize,
    a: u32,
    b: u32,
    rust: fn(u32, u32) -> u32,
) -> u32 {
    crate::framework::hook_enter();
    hook.count_call();
    let eax = if crate::framework::shadow() {
        unsafe { call_stdcall2(original, a, b) }
    } else {
        rust(a, b)
    };
    crate::framework::hook_leave();
    eax
}
