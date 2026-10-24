//! `physics::update_camera` (0x437A53, two chunks): the camera's step.
//! `physics::camera_low_switch`; the car's place into `dword_6416E8` ..; the light record
//! of `unk_719FFC` (`g_lightRecords` + 0x134 n: places x 1/65536, two less
//! `flt_613FC0`, into `flt_6065C8` .., `flt_606624` ..; +0xC then +0x1C into
//! `dword_606630`; +0x24 into `flt_606570`; +0x20 x 1/65536, kept for the cars marked
//! in `dword_5C5AC0` (n below 0x22), else +0x28, into `flt_606574`). With no switch under way and a
//! fixed mode (0x2A .. 0x2D, 9, 10: `g_nearestCamera2` / `g_replayCameras2` of the
//! checkpoint `unk_71A070`; 0x2E .. 0x31, 7, 8: the first two) the second chunk: the
//! camera at that face's first scene point, `physics::camera_height_bias`, aimed at the
//! car (`fpatan` heading and pitch, x 65536 x `flt_5C5E28` x `flt_5C5DD4`, << 4, the
//! pitch negated), the distance (`physics::vector_length`) into `flt_60631C`, the zoom
//! `unk_71A6A0` (`dword_608B1C`, or 0x180 x the distance / `flt_5C5D48` held at least
//! `flt_5C5D68` with the record's +0x50), the roll and turns 0. Else: the camera at the
//! car (also into `dword_5CCDA0` .., the heading into `dword_5CCD60`); a switch under way
//! ends after `unk_71A670` (the mode back, `unk_71AF88` = 8, modes 7 on to 0) or its share
//! (held 0 ..= 1) into `unk_71A674`; `physics::take_view_a` of the mode and `take_view_b`
//! of the mode before; their figures blended by the share into the turn targets
//! (`dword_608B50` ..), the offsets (`dword_608B30` .., `608B74` ..) and the turns
//! (`unk_71A694` .., << 16); the shares (`dword_608BE0` ..); the zoom held at least
//! `flt_6062D0`; `physics::ease_view_angles`, `ease_view_blend`,
//! `physics::camera_avoid_ground`; the offset's length into `dword_5CCD80`,
//! `dword_6062BC` into `dword_5CCD40`.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read (the caller
//!      returns)
//!
//! Caller: `physics::camera_frame`. It begins with a call: hooked by a private copy of the
//! first chunk (0x437A53 .. 0x437F53, 0x500 bytes) with its calls and its twelve near
//! jumps into the second chunk (0x4380E9 .. 0x438325, left in the game, which returns from
//! there) relocated, and a hand-made patch.
//! Refuses a mode past 0x33 on the blending path (the original reads past
//! `off_606320`), and what the ground checks refuse; everything written put back. Shadow
//! mode runs the original alone (as the checks).
//! Self-test (`camera_test`, in a race; skipped otherwise): the original and the Rust
//! version on the stage (everything written saved and put back), the camera where it is,
//! moved below the car, every mode and a switch under way.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_view::{rd, record};
use crate::physics::flow::{a, base, g, hold_high, hold_low, p, x87_run};
use crate::physics::touch_face::FACES;
use crate::scene::matrix3::rounded;
use rc2k_core::physics::camera_view;

const MODE: usize = 0x71A65C;
const LIGHTS: usize = 0x5C906C; // g_lightRecords

/// The fixed camera (the second chunk, 0x4380E9 ..): `second` the second tables.
fn fixed_camera(second: bool) {
    let (nearest, replay) = if second {
        (0xA0BA78, 0xA11AE4)
    } else {
        (0xA06A10, 0xA10AE4)
    };
    let face = g(replay + 4 * g(nearest + 4 * g(0x71A070) as usize) as usize);
    let face = (face << 6).wrapping_add(g(FACES));
    let point = rd(face + 4)
        .wrapping_mul(12)
        .wrapping_add(g(crate::physics::scene_point_hits_car::SCENE_POINTS));
    for k in 0..3 {
        p(0x71A67C + 4 * k as usize, rd(point + 4 * k));
    }
    unsafe { crate::physics::camera_height_bias::camera_height_bias() };
    let d = |k: usize| g(0x71A10C + 4 * k).wrapping_sub(g(0x71A67C + 4 * k));
    let (dx, dy, dz) = (d(0), d(1), d(2));
    let v = [dx, dz, dy];
    x87_run!(
        "fild dword ptr [{v}]",
        "fild dword ptr [{v} + 4]",
        ".byte 0xD9, 0xF3", // fpatan
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fmul dword ptr [{b} + 0x1C5E28]",
        "fmul dword ptr [{b} + 0x1C5DD4]",
        "fistp dword ptr [{b} + 0x31A688]",
        "fild dword ptr [{v} + 8]",
        "fild dword ptr [{v}]",
        ".byte 0xD8, 0xC8", // fmul st, st
        "fild dword ptr [{v} + 4]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        ".byte 0xD9, 0xFA", // fsqrt
        ".byte 0xD9, 0xF3", // fpatan
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fmul dword ptr [{b} + 0x1C5E28]",
        "fmul dword ptr [{b} + 0x1C5DD4]",
        "fistp dword ptr [{b} + 0x31A690]";
        b = base(), v = v.as_ptr()
    );
    p(0x71A688, g(0x71A688) << 4);
    p(0x71A690, (g(0x71A690) << 4).wrapping_neg());
    let w = [dx, dy, dz];
    x87_run!(
        "fild dword ptr [{v}]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2062D8]",
        "fild dword ptr [{v} + 4]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2062DC]",
        "fild dword ptr [{v} + 8]",
        "fdiv dword ptr [{b} + 0x1C5D98]",
        "fstp dword ptr [{b} + 0x2062E0]";
        b = base(), v = w.as_ptr()
    );
    p(0x62400C, dz);
    p(
        0x60631C,
        rounded(unsafe { crate::physics::vector_length::vector_length(a(0x6062D8)) }),
    );
    p(0x71A6A0, g(0x608B1C));
    if rd(record() + 0x50) != 0 {
        p(0x71A6A0, 0x180);
        x87_run!(
            "fild dword ptr [{b} + 0x31A6A0]",
            "fmul dword ptr [{b} + 0x20631C]",
            "fdiv dword ptr [{b} + 0x1C5D48]",
            "fstp dword ptr [{b} + 0x31A6A0]";
            b = base()
        );
        hold_low(0x71A6A0, 0x5C5D68);
    }
    for x in [0x71A68C, 0x71A694, 0x71A698, 0x71A69C] {
        p(x, 0);
    }
}

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_view::{fixed_camera, keeps_light_figure,
// switch_over, mode_after_switch, check_modes}` (the x87 sums stay).
pub fn steps() -> Result<(), Refused> {
    crate::physics::camera_low_switch::camera_low_switch()?;
    for k in 0..3 {
        p(0x6416E8 + 4 * k, g(0x71A10C + 4 * k));
    }
    let car = g(0x719FFC);
    let light = car.wrapping_mul(0x134).wrapping_add(a(LIGHTS));
    x87_run!(
        "fild dword ptr [{d}]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fstp dword ptr [{b} + 0x2065C8]",
        "fild dword ptr [{d} + 0x4]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fsub dword ptr [{b} + 0x213FC0]",
        "fstp dword ptr [{b} + 0x2065CC]",
        "fild dword ptr [{d} + 0x8]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fstp dword ptr [{b} + 0x2065D0]",
        "fild dword ptr [{d} + 0x10]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fstp dword ptr [{b} + 0x206624]",
        "fild dword ptr [{d} + 0x14]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fsub dword ptr [{b} + 0x213FC0]",
        "fstp dword ptr [{b} + 0x206628]",
        "fild dword ptr [{d} + 0x18]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fstp dword ptr [{b} + 0x20662C]",
        "fld dword ptr [{d} + 0x24]",
        "fstp dword ptr [{b} + 0x206570]",
        "fild dword ptr [{d} + 0x20]",
        "fmul dword ptr [{b} + 0x1C5DA0]",
        "fstp dword ptr [{b} + 0x206574]";
        b = base(), d = light
    );
    p(0x606630, rd(light + 0xC));
    p(0x606630, rd(light + 0x1C));
    if !camera_view::keeps_light_figure(car, || g(0x5C5AC0 + 4 * car as usize)) {
        p(0x606574, crate::math::x87::loaded(rd(light + 0x28)));
    }
    if g(0x71A66C) == 0
        && let Some(second) = camera_view::fixed_camera(g(MODE))
    {
        fixed_camera(second);
        return Ok(());
    }
    // 0x437BE4: the camera at the car, the views blended.
    for (k, copy) in [0x5CCDA0usize, 0x5CCDC0, 0x5CCDE0].iter().enumerate() {
        let v = g(0x71A10C + 4 * k);
        p(*copy, v);
        p(0x71A67C + 4 * k, v);
    }
    p(0x5CCD60, g(0x71A998));
    if g(0x71A66C) != 0 {
        if camera_view::switch_over(g(0x719FE0), g(0x71A668), g(0x71A670)) {
            p(0x71A66C, 0);
            p(0x71A674, 0);
            p(MODE, g(0x71A660));
            p(0x71AF88, camera_view::SWITCH_HOLD);
            let after = camera_view::mode_after_switch(g(MODE));
            if after != g(MODE) {
                p(MODE, after);
            }
        } else {
            x87_run!(
                "fild dword ptr [{b} + 0x319FE0]",
                "fisub dword ptr [{b} + 0x31A668]",
                "fidiv dword ptr [{b} + 0x31A670]",
                "fstp dword ptr [{b} + 0x31A674]";
                b = base()
            );
            hold_high(0x71A674, 0x5C5D38);
            hold_low(0x71A674, 0x5C5D90);
        }
    }
    let (mode, before) = (g(MODE), g(0x71A660));
    camera_view::check_modes(mode, before)?;
    unsafe { crate::physics::take_view_a::take_view_a(g(0x606320 + 4 * mode as usize)) };
    unsafe { crate::physics::take_view_b::take_view_b(g(0x606320 + 4 * before as usize)) };
    x87_run!(
        "fild dword ptr [{b} + 0x208B70]",
        "fisub dword ptr [{b} + 0x208B64]",
        "fmul dword ptr [{b} + 0x31A674]",
        "fiadd dword ptr [{b} + 0x208B64]",
        "fistp dword ptr [{b} + 0x208B58]",
        "fild dword ptr [{b} + 0x208B68]",
        "fisub dword ptr [{b} + 0x208B5C]",
        "fmul dword ptr [{b} + 0x31A674]",
        "fiadd dword ptr [{b} + 0x208B5C]",
        "fistp dword ptr [{b} + 0x208B50]",
        "fild dword ptr [{b} + 0x208B6C]",
        "fisub dword ptr [{b} + 0x208B60]",
        "fmul dword ptr [{b} + 0x31A674]",
        "fiadd dword ptr [{b} + 0x208B60]",
        "fistp dword ptr [{b} + 0x208B54]";
        b = base()
    );
    // The offsets and turns: (to, from b, from a) each a + (b - a) x share, x 65536.
    for (to, x, y) in [
        (0x608B30usize, 0x608BB0usize, 0x608B98usize),
        (0x608B34, 0x608BB4, 0x608B9C),
        (0x608B38, 0x608BB8, 0x608BA0),
        (0x608B74, 0x608B8C, 0x608B80),
        (0x608B78, 0x608B90, 0x608B84),
        (0x608B7C, 0x608B94, 0x608B88),
        (0x71A694, 0x608BD4, 0x608BC8),
        (0x71A698, 0x608BD8, 0x608BCC),
        (0x71A69C, 0x608BDC, 0x608BD0),
    ] {
        x87_run!(
            "fld dword ptr [{x}]",
            "fsub dword ptr [{y}]",
            "fmul dword ptr [{b} + 0x31A674]",
            "fadd dword ptr [{y}]",
            "fmul dword ptr [{b} + 0x1C5D98]",
            "fistp dword ptr [{t}]";
            b = base(), x = a(x), y = a(y), t = a(to)
        );
    }
    for x in [0x71A694, 0x71A698, 0x71A69C] {
        p(x, g(x) << 16);
    }
    for (to, from) in [
        (0x608BE4, 0x608BF8),
        (0x608BE8, 0x608BFC),
        (0x608BEC, 0x608C00),
        (0x608BE0, 0x608BF4),
        (0x608BF0, 0x608C04),
    ] {
        p(to, g(from));
    }
    x87_run!(
        "fild dword ptr [{b} + 0x208BF0]",
        "fstp dword ptr [{b} + 0x31A6A0]";
        b = base()
    );
    hold_low(0x71A6A0, 0x6062D0);
    unsafe { crate::physics::ease_view_angles::ease_view_angles() };
    unsafe { crate::physics::ease_view_blend::ease_view_blend() };
    crate::physics::camera_avoid_ground::steps()?;
    x87_run!(
        "fild dword ptr [{b} + 0x208B30]",
        ".byte 0xD8, 0xC8", // fmul st, st
        "fild dword ptr [{b} + 0x208B34]",
        ".byte 0xD8, 0xC8",
        "fild dword ptr [{b} + 0x208B38]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA", // fsqrt
        "fistp dword ptr [{b} + 0x1CCD80]";
        b = base()
    );
    p(0x5CCD40, g(0x6062BC));
    Ok(())
}
