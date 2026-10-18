//! `physics::place_camera` (0x438E5A): the camera placed. Seven figures from
//! `unk_6077CC` (with `dword_6238A4` at 2) or `unk_6077B0` into `dword_606678`,
//! `60667C`, `6066D4`, `6066D8`, `606730`, `606734` and the last into `dword_60668C`,
//! `6066E8`, `606744`; the offset (`dword_608B30` ..) turned by `unk_71A69C`,
//! `71A694`, `71A698` then `71A68C`, `71A690`, `71A688` (`math::turn_point_xz_ebx`,
//! `_ecx`, `turn_point_xz`, in the original's registers) into the place `unk_71A67C`
//! ..; then the second offset (`dword_608B74` .., / 65536) turned by `g_crashMatrix`
//! (`scene::rotate_point`), x 65536 (`fistp`), plus `dword_6416E8` .., added to it.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the last offset added, edx the last turn's, esi `g_crashMatrix`;
//!      the rest kept; no flags read
//!
//! Callers: `sub_43907F` (seven).
//! Self-test: `physics/camera_view.rs` (`view_test`), figures by lot, three precisions.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, ESI, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{rd, view_regions, view_test};
use crate::physics::flow::{a, base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::place_camera", rva(0x438E5A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn place_camera(_: &Regs) -> Left {
    let esi = if g(0x6238A4) == 2 {
        a(0x6077CC)
    } else {
        a(0x6077B0)
    };
    for (k, to) in [
        0x606678usize,
        0x60667C,
        0x6066D4,
        0x6066D8,
        0x606730,
        0x606734,
    ]
    .iter()
    .enumerate()
    {
        p(*to, rd(esi + 4 * k as u32));
    }
    let last = rd(esi + 0x18);
    for to in [0x60668C, 0x6066E8, 0x606744] {
        p(to, last);
    }
    let (mut eax, mut ebx, mut ecx) = (g(0x608B30), g(0x608B34), g(0x608B38));
    let mut edx = 0;
    for (ebx_turn, ecx_turn, eax_turn) in [
        (0x71A69C, 0x71A694, 0x71A698),
        (0x71A68C, 0x71A690, 0x71A688),
    ] {
        let i = |v: u32| v as i32;
        let [x, z, e] = unsafe {
            crate::math::turn_point_xz_ebx::turn_point_xz_ebx(i(ebx), i(ecx), i(eax), g(ebx_turn))
        };
        ebx = x;
        eax = z;
        let [x, z, _] = unsafe {
            crate::math::turn_point_xz_ecx::turn_point_xz_ecx(i(ecx), i(eax), i(ebx), g(ecx_turn))
        };
        ecx = x;
        ebx = z;
        let [x, _, z, e2] = unsafe {
            crate::math::turn_point_xz::turn_point_xz(i(eax), i(ebx), i(ecx), g(eax_turn))
        };
        eax = x;
        ecx = z;
        edx = e2;
        let _ = e;
    }
    p(0x71A67C, eax);
    p(0x71A680, ebx);
    p(0x71A684, ecx);
    // 0x438F14: the second offset, turned with the car.
    let mut parts = [[0u8; 10]; 3];
    for (k, x) in [0x608B74usize, 0x608B78, 0x608B7C].iter().enumerate() {
        x87_run!(
            "fild dword ptr [{v}]",
            "fdiv dword ptr [{b} + 0x1C5D98]",
            "fstp tbyte ptr [{o}]";
            b = base(), v = a(*x), o = parts[k].as_mut_ptr()
        );
    }
    let turned = unsafe { crate::scene::rotate_point::rotate_point(a(0x71A08C), parts) };
    let k = g(0x5C5D98);
    let whole: Vec<u32> = turned
        .iter()
        .map(|e| crate::math::x87_run::ext_to_int(crate::math::x87_run::ext_times(*e, k)) as u32)
        .collect();
    p(0x62400C, whole[2]);
    eax = whole[0].wrapping_add(g(0x6416E8));
    ebx = whole[1].wrapping_add(g(0x6416EC));
    ecx = whole[2].wrapping_add(g(0x6416F0));
    p(0x71A67C, g(0x71A67C).wrapping_add(eax));
    p(0x71A680, g(0x71A680).wrapping_add(ebx));
    p(0x71A684, g(0x71A684).wrapping_add(ecx));
    Left::nothing()
        .with(EAX, eax)
        .with(EBX, ebx)
        .with(ECX, ecx)
        .with(EDX, edx)
        // 0x438F5A: esi is left at the crash matrix (`lea esi, g_crashMatrix`).
        .with(ESI, a(0x71A08C))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { place_camera(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &view_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    view_test(&HOOK, call_original, &rust)
}
