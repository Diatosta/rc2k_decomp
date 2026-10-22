//! `physics::wall_edge_contact` (0x45DA14, two chunks): the wall met between two faces,
//! `dword_6D3170` (the wall, ebp) and `dword_6D3174` (esi). The wall's four edges
//! ((p1, p2), (p3, p1), (p4, p3), (p2, p4) of its corner words +4 .. +0x10) are looked
//! for in the other face (`physics::face_has_edge`); none shared: `physics::wall_push_back`.
//! The first shared, its ends swapped, into `dword_6D3178`, `dword_6D317C`; their scene
//! points' x, z into `dword_6141AC` .. `6141B8`; the edge's heading (`fpatan` of the
//! differences, x `flt_5C5DD4` x `flt_5C5D98`, & 0xFFFF) << 16 + 0x40000000 into
//! `dword_6141C8`; the normal (0, 0, 1) turned by it (`math::turn_float_point_xz`) into
//! `qword_6D3078` ..; the speeds taken, the point of probe `dword_6D3214` (its y x
//! `flt_5C5DD0`), no slide limit; `physics::contact_force` into contact record 4,
//! `physics::place_sound_e1`.
//!
//! In:  nothing
//! Out: ebp the wall, and the registers as the edge tests and `contact_force` leave them;
//!      no flags read (the caller goes on)
//!
//! Caller: `physics::wall_contacts`. Its second chunk (0x45DA9E .. 0x45DC12) is entered
//! only by the first's jumps: nothing to copy, the hook patches the first chunk's start.
//! Refuses a probe point past 0x3F on the edge path (the tables' room), and what its
//! calls refuse (what was written put back).
//! Self-test: `physics/wall_contacts.rs` (`test_walls`): the original and the Rust version
//! on the game's variables (saved and put back) with test faces and scene points
//! (`g_courseFaces`, `g_scenePoints` pointed at them), every edge shared and none, under
//! three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::math::x87::mul;
use crate::physics::contact_force::{RECORD, contact_force};
use crate::physics::flow::{a, base, copy3, g, guard, p, x87_run};
use crate::physics::scene_point_hits_car::SCENE_POINTS;
use crate::physics::touch_face::FACES;
use crate::physics::wall_contacts::{WALL_RECORD, regions, test_walls};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::walls;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::wall_edge_contact",
    rva(0x45DA14),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The normal across the edge from scene point `first` to `second` (0x45DAAA ..
/// 0x45DB70, and `crossed_edge_contact`'s): their x, z into `dword_6141AC`, `6141B0`
/// (first) and `6141B4`, `6141B8`; the heading (`fpatan` of the differences, x
/// `flt_5C5DD4` x `flt_5C5D98`, & 0xFFFF, through `dword_62400C`) << 16 + 0x40000000
/// into `dword_6141C8`; (0, 0, 1) turned by it (`math::turn_float_point_xz`) into
/// `qword_6D3078` ... Returns what that leaves in eax, ebx, ecx, edx.
pub fn edge_normal(first: u32, second: u32) -> [u32; 4] {
    let point = |k: u32| k.wrapping_mul(12).wrapping_add(g(SCENE_POINTS));
    p(0x6141AC, rd(point(first)));
    p(0x6141B0, rd(point(first) + 8));
    p(0x6141B4, rd(point(second)));
    p(0x6141B8, rd(point(second) + 8));
    let dx = g(0x6141B4).wrapping_sub(g(0x6141AC));
    let dz = g(0x6141B8).wrapping_sub(g(0x6141B0));
    let d = [dx, dz];
    x87_run!(
        "fild dword ptr [{d}]",
        "fild dword ptr [{d} + 4]",
        ".byte 0xD9, 0xF3", // fpatan
        "fmul dword ptr [{b} + 0x1C5DD4]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fistp dword ptr [{b} + 0x22400C]";
        b = base(), d = d.as_ptr()
    );
    p(0x62400C, g(0x62400C) & 0xFFFF);
    let heading = (g(0x62400C) << 16).wrapping_add(0x4000_0000);
    p(0x6141C8, heading);
    let out = unsafe { turn_float_point_xz(g(0x5C5D90), g(0x5C5D90), g(0x5C5D38), heading) };
    p(0x6D3078, out[0]);
    p(0x6D307C, out[1]);
    p(0x6D3080, out[2]);
    out
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::walls::shared_edge` (the edge's heading stays x87).
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    r.ebp = (g(0x6D3170) << 6).wrapping_add(g(FACES));
    r.esi = (g(0x6D3174) << 6).wrapping_add(g(FACES));
    let corners = [
        rd(r.esi + 4),
        rd(r.esi + 8),
        rd(r.esi + 0xC),
        rd(r.esi + 0x10),
    ];
    let wall = r.ebp;
    let shared;
    (shared, r.eax, r.ebx) = walls::shared_edge(|j| rd(wall + 4 + 4 * j as u32), corners);
    if !shared {
        return crate::physics::wall_push_back::steps(r);
    }
    // 0x45DA9E: along the edge.
    let n = g(0x6D3214);
    walls::check_probe(n)?;
    let (from, to) = (r.ebx, r.eax);
    p(0x6D3178, from);
    p(0x6D317C, to);
    let [x, y, z, edx] = edge_normal(to, from);
    r.eax = x;
    r.ebx = y;
    r.ecx = z;
    r.edx = edx;
    copy3(0x6D30A8, 0x71A170);
    copy3(0x6D30B4, 0x71A15C);
    r.ebx = n;
    let n = n as usize;
    p(0x6D3090, g(0x6D2214 + 4 * n));
    p(0x6D3094, g(0x6D2314 + 4 * n));
    p(0x6D3098, g(0x6D2414 + 4 * n));
    p(0x6D3094, mul(g(0x6D3094), g(0x5C5DD0)));
    p(0x6D3100, 0);
    p(RECORD, a(WALL_RECORD));
    unsafe { contact_force() }.apply(r);
    unsafe { crate::physics::place_sound_e1::place_sound_e1() }?;
    Ok(())
}

/// The Rust version.
pub unsafe fn wall_edge_contact(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { wall_edge_contact(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_walls(&HOOK, call_original, &rust)
}
