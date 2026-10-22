//! `net::destroy_connection` (0x4187EE): with a DirectPlay object (`dword_5AB288`), the
//! network race ended (`dword_52A910` 0), "NET: Destroy connection" logged, and through
//! DirectPlay the own player destroyed (`sub_41916B`), the session closed (`sub_418DD3`),
//! the object released (`sub_41864E`) and the lobby's (`sub_41900F`); either way the
//! network's memory given back (`net::free_messages`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `test eax, eax` (not read)
//!
//! See `net/mod.rs`. Two callers: `net::stop` and `sub_41976F` (a message from DirectPlay,
//! 0x31). Its calls through `part`: DirectPlay's four through the game (section 15), the
//! log's and `free_messages` the Rust versions. Shadow mode runs the original alone.
//! Self-test: a copy whose two variables are the test's and whose calls go to noting
//! stand-ins, against the Rust version, with an object and without.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{
    CLOSE_SESSION, DESTROY_PLAYER, FREE, LOG_TEXT_2_NL, ON, Part, RELEASE_DPLAY, RELEASE_LOBBY, a,
    dispatch_net, net_entry,
};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::destroy_connection", rva(0x4187EE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// dword_5AB288: DirectPlay's object (IDirectPlay), 0 without one.
pub const DPLAY: usize = 0x5AB288;
const TEXT: usize = 0x5C2EA5; // "NET: Destroy connection"

/// The Rust version on the object's pointer at `dplay` and the switch at `on`.
pub fn destroy_connection_with(
    r: &mut Regs,
    dplay: u32,
    on: u32,
    part: Part,
) -> Result<(), Refused> {
    let mut g = *r;
    if unsafe { (dplay as *const u32).read_volatile() } != 0 {
        unsafe { (on as *mut u32).write_volatile(0) };
        let mut l = Regs { eax: a(TEXT), ..g };
        part(LOG_TEXT_2_NL, &mut l)?;
        for place in [DESTROY_PLAYER, CLOSE_SESSION, RELEASE_DPLAY, RELEASE_LOBBY] {
            g = *r;
            part(place, &mut g)?;
        }
    }
    g = *r;
    part(FREE, &mut g)?;
    r.set_carry(false);
    r.set_zero(r.eax == 0);
    Ok(())
}

/// The Rust version.
pub fn destroy_connection(r: &mut Regs, part: Part) -> Result<(), Refused> {
    destroy_connection_with(r, a(DPLAY), a(ON), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, destroy_connection);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, destroy_connection)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 2];
    let (dplay, on) = (mem.as_mut_ptr() as u32, mem.as_mut_ptr() as u32 + 4);
    // pusha; cmp dword_5AB288, 0
    let entry = [0x60, 0x83, 0x3D, 0x88, 0xB2, 0x5A, 0x00, 0x00];
    let copy = match copy_with(
        0x4187EE,
        0x3E,
        &entry,
        &[0x1B, 0x21, 0x26, 0x2B, 0x30, 0x35],
        &[
            LOG_TEXT_2_NL,
            DESTROY_PLAYER,
            CLOSE_SESSION,
            RELEASE_DPLAY,
            RELEASE_LOBBY,
            FREE,
        ],
        &[],
        &[(DPLAY, 4, dplay), (ON, 4, on)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for object in [0u32, 0x0BAD_0B1E] {
        compare(
            &mut t,
            &format!("object {object:#x}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (dplay as *mut [u32; 2]).write_volatile([object, 1]) },
            &[(dplay, 8)],
            &|r, part| destroy_connection_with(r, dplay, on, part),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
