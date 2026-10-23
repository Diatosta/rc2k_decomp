//! The cars drawn (`race::draw_list_entry` place 5: `sub_455105` -> `sub_461669` ->
//! `sub_4616AA`, part R2, IDA 2026-10-18). A car is block esi (`g_carBlocks`, `CarBlock`):
//! +0x24 its model, +0x134 .. +0x13C its place, +0x298 its parts' switches, +0x6FC its
//! state bits, +0x740 .. +0x8BF six matrices (four parts', the body's, the turn), +0xF04 the
//! shown parts' mask.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::scene::matrix::multiply_matrices;
use crate::screens::harness::Left;

fn ga(a: u32) -> usize {
    game_addr(rva(a as usize))
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

const VIEW_MATRIX: u32 = 0x5EA2CC; // g_viewMatrix
const TURNED: u32 = 0x5EA10C; // g_turnedMatrix
const LIT: u32 = 0x5EA14C; // g_litMatrix
const MADE: u32 = 0x5EA30C; // g_madeObjectMatrix
const BODY_TURN: u32 = 0x5EA08C; // flt_5EA08C: the car's +0x840
const CAR_TURN: u32 = 0x641760; // flt_641760: the car's +0x880
/// The four parts' matrices (`flt_6417E0` .., the car's +0x740 ..).
pub const PART_MATRICES: [u32; 4] = [0x6417E0, 0x641820, 0x641860, 0x6418A0];

/// `race::car_part_matrix_1` .. `_4` (0x461F12, 0x461F58, 0x461F9E, 0x461FE4): matrix `a`
/// times the body's turn (`flt_5EA08C`) into `g_turnedMatrix`, times the car's turn
/// (`flt_641760`) into `g_litMatrix`, times `g_viewMatrix` into `g_madeObjectMatrix`.
/// Leaves the registers as the original (esi, edi, ebp the last product's three, then as
/// `scene::multiply_matrices` leaves them).
pub unsafe fn car_part_matrix(r: &mut Regs, a: u32) {
    unsafe {
        multiply_matrices(ga(a), ga(BODY_TURN), ga(TURNED));
        multiply_matrices(ga(TURNED), ga(CAR_TURN), ga(LIT));
        (r.esi, r.edi, r.ebp) = (ga(LIT) as u32, ga(VIEW_MATRIX) as u32, ga(MADE) as u32);
        multiply_matrices(ga(LIT), ga(VIEW_MATRIX), ga(MADE)).apply(r);
    }
}

/// `race::car_body_matrix` (0x461EA1): the turns about x and z by 0 (`scene::set_fine_turn_x`,
/// `scene::set_turn_z`) multiplied into `g_litMatrix`, then as `car_part_matrix` on it.
pub unsafe fn car_body_matrix(r: &mut Regs) {
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    unsafe {
        r.edx = 0;
        set_fine_turn(Axis::X, 0).apply(r);
        r.edx = 0;
        set_fine_turn(Axis::Z, 0).apply(r);
        multiply_matrices(ga(0x5E9F8C), ga(0x5EA00C), ga(LIT));
        car_part_matrix(r, LIT);
    }
}

/// `race::clear_car_marks` (0x46202A): the 0x40 dwords at `dword_5CD688` (the car's
/// marks: number, digits, plates) 0. Leaves eax 0, ecx 0, edi past them.
pub unsafe fn clear_car_marks() -> Left {
    use crate::framework::{EAX, ECX, EDI};
    unsafe {
        for k in 0..0x40 {
            game_write(rva(0x5CD688) + 4 * k, 0u32);
        }
        Left::nothing()
            .with(EAX, 0)
            .with(ECX, 0)
            .with(EDI, ga(0x5CD688) as u32 + 0x100)
    }
}

/// `race::car_number_textures` (0x46203D): car esi's marks' textures (`dword_5CD688` ..):
/// +0x1B8 / +0x1BC (|x| not below `flt_641740`: 1) into `dword_5CD68C`, `dword_5CD698` /
/// `dword_5CD690`, `dword_5CD694`; the letter of its model's name (`unk_71B408` + 0x30 x
/// model) at +0x718 (rounded, held to 0 .. 0x2EDF) / 250, as a digit or letter number, into
/// `dword_5CD69C`, `dword_5CDA88`; its number +0xAA8 as tens and units (`dword_5CD6A0`,
/// `dword_5CD6A4`, 999 for none; `dword_5CD6AC` the units alone below ten); the stage's
/// (held to 11 .. 0x50) tens' plate `dword_71BA10[]` into `dword_5CD6A8`; and while +0x310 is
/// below `flt_5C5D14` its 17 bytes +0xAC0 .. into the places `dword_6416F8[]` names. Gives
/// the registers it leaves.
pub unsafe fn car_number_textures(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::car_marks` (the rounding of +0x718 the x87's, under the
    // game's control word).
    use rc2k_core::race::car_marks::{
        decals_shown, lamp_side, letter_number, letter_place, number_digits, stage_plate,
    };
    let float = |a: u32| f32::from_bits(unsafe { get(a) });
    unsafe {
        let mut r = *entry;
        let car = r.esi;
        let limit = f32::from_bits(game_read(rva(0x641740)));
        r.ebx = lamp_side(float(car + 0x1B8), limit);
        game_write(rva(0x5CD68C), r.ebx);
        game_write(rva(0x5CD698), r.ebx);
        r.ebx = lamp_side(float(car + 0x1BC), limit);
        game_write(rva(0x5CD690), r.ebx);
        game_write(rva(0x5CD694), r.ebx);
        r.ebp = (ga(0x71B408) as u32).wrapping_add(get(car + 0x24).wrapping_mul(0x30));
        let n = crate::math::x87::to_int(get(car + 0x718));
        game_write(rva(0x62400C), n);
        let (place, rest) = letter_place(n);
        r.edx = rest;
        r.ebx = 250;
        r.eax = letter_number(((r.ebp + place) as *const u8).read()) as u32;
        game_write(rva(0x5CD69C), r.eax);
        game_write(rva(0x5CDA88), r.eax);
        let d = number_digits(get(car + 0xAA8));
        game_write(rva(0x5CD6A0), d.tens);
        game_write(rva(0x5CD6A4), d.units);
        game_write(rva(0x5CD6AC), d.alone);
        let (tens, units) = stage_plate(game_read(rva(0x623850)));
        r.ebx = 10;
        r.edx = units;
        r.eax = game_read(rva(0x71BA10) + 4 * tens as usize);
        game_write(rva(0x5CD6A8), r.eax);
        if decals_shown(float(car + 0x310), f32::from_bits(game_read(rva(0x5C5D14)))) {
            for k in 0..0x11u32 {
                r.eax = ((car + 0xAC0 + k) as *const u8).read() as u32;
                r.ebx = game_read(rva(0x6416F8) + 4 * k as usize);
                game_write(rva(0x5CD688) + 4 * r.ebx as usize, r.eax);
            }
            r.ecx = 0x11;
        }
        r
    }
}

/// The model's bit at +`offset` of its 0x134-byte record (`dword_5C90C4` ..): 1 << it, 0 for
/// -1.
unsafe fn model_bit(record: u32, offset: u32) -> u32 {
    unsafe {
        let c = get(ga(0x5C90C4) as u32 + record + offset);
        if c == u32::MAX { 0 } else { 1u32 << (c & 31) }
    }
}

fn below(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

/// `bts value, bit` (bit mod 32).
fn with_bit(value: u32, bit: u32) -> u32 {
    value | 1u32 << (bit & 31)
}

/// `race::draw_car` (0x4616AA): car esi drawn: its marks' textures
/// (`race::car_number_textures`); its six matrices (+0x740 ..) to `flt_6417E0` .. /
/// `flt_5EA08C` / `flt_641760`; its model's parts' bits (`dword_5CCE1C` ..; the shown mask
/// `dword_62150C` by +0x298, `dword_606280` by +0x6FC, the view's depth `dword_6415A4` and
/// the stage's fog, `dword_71BA08` the nearness (0 .. 1) dimming `g_shine`), `dword_606284`,
/// +0xF04; all parts for the drawn view's own car when `dword_60626C`; the light and place;
/// `scene::set_light_direction`; the body (`g_objectNumber` 0x50 + 0x14 x car,
/// `race::car_body_matrix`, part 1 `scene::draw_scene_object` between the kept
/// reflections), and nearer than 0x3C0000 its four parts by +0x298's bits 5 .. 8 (each
/// `car_part_matrix`, part 1); then `race::clear_car_marks`. Refuses (before anything) what
/// `scene::draw_scene_object` would for the car's objects: in a race (`g_inRace`) car
/// `dword_60627C` (below eight) without its block.
pub unsafe fn draw_car(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let n: u32 = game_read(rva(0x60627C));
        if game_read::<u32>(rva(0x5C5CEC)) != 0
            && n < 8
            && game_read::<u32>(rva(0x71B3B8) + 4 * n as usize) == 0
        {
            return Err(BadInput("the drawn car without its block").into());
        }
        let car = r.esi;
        *r = car_number_textures(r);
        for (k, to) in [
            0x6417E0u32,
            0x641820,
            0x641860,
            0x6418A0,
            BODY_TURN,
            CAR_TURN,
        ]
        .into_iter()
        .enumerate()
        {
            std::ptr::copy(
                (car + 0x740 + 0x40 * k as u32) as *const u8,
                ga(to) as *mut u8,
                0x40,
            );
        }
        let w = |a: u32, v: u32| game_write(rva(a as usize), v);
        let k = |a: u32| game_read::<u32>(rva(a as usize));
        w(0x6D18D4, get(car + 0xAB8));
        let model = get(car + 0x24);
        w(0x5E9EC8, model);
        let rec = model.wrapping_mul(0x134);
        r.edi = rec;
        for (to, offset) in [
            (0x5CCE1Cu32, 0u32),
            (0x5CCE20, 4),
            (0x5CCE24, 8),
            (0x5CCE28, 0xC),
            (0x5CCE34, 0x14),
        ] {
            w(to, model_bit(rec, offset));
        }
        let mut lamps = model_bit(rec, 0x34);
        if lamps != 0 {
            if model == 4 || model == 7 {
                lamps |= 0x600_0000;
            }
            if model == 0xD {
                lamps |= 0x300_0000;
            }
        }
        w(0x5CCE40, lamps);
        for (to, offset) in [
            (0x5CCE44u32, 0x38u32),
            (0x5CCE38, 0x2C),
            (0x5CCE3C, 0x30),
            (0x5CCE48, 0x28),
            (0x5CCE4C, 0x3C),
        ] {
            w(to, model_bit(rec, offset));
        }
        let switches = get(car + 0x298);
        let mut shown = u32::MAX;
        for (bit, var) in [
            (9u32, 0x5CCE34u32),
            (12, 0x5CCE48),
            (13, 0x5CCE40),
            (14, 0x5CCE44),
            (15, 0x5CCE4C),
            (16, 0x5CCE38),
            (17, 0x5CCE3C),
        ] {
            if switches & (1 << bit) == 0 {
                shown &= !k(var);
            }
        }
        w(0x62150C, shown);
        let field = |offset: u32| get(ga(0x5C90C4) as u32 + rec + offset);
        w(0x608CAC, field(0x10));
        w(
            0x5CCE2C,
            u32::MAX
                .wrapping_sub(k(0x5CCE1C))
                .wrapping_sub(k(0x5CCE20))
                .wrapping_sub(k(0x5CCE24))
                .wrapping_sub(k(0x5CCE28)),
        );
        w(0x608CA8, 0x210_0000);
        for (to, offset) in [
            (0x5CCE50u32, 0x18u32),
            (0x5CCE54, 0x1C),
            (0x5CCE58, 0x20),
            (0x5CCE5C, 0x24),
            (0x5CCE60, 0x14),
        ] {
            w(to, field(offset));
        }
        let state = get(car + 0x6FC);
        let bit = k(0x5CCE60);
        w(0x606280, u32::MAX);
        if state & 2 != 0 {
            w(0x606280, with_bit(field(0x48), bit));
        }
        if state & 4 != 0 {
            w(0x606280, with_bit(field(0x44), bit));
        }
        w(0x71BA08, k(0x5C5D90));
        let mut stage = k(0x606270);
        if (stage as i32) < 0 {
            stage = 0;
        }
        if !below(stage, 0x33) {
            stage = 0x33;
        }
        let record = k(0x606320 + 4 * stage);
        if get(record + 0x50) != 0 {
            w(0x6415A4, 0x5_0000);
        }
        let and = |mask: u32| w(0x606280, k(0x606280) & mask);
        if state & 0x10 == 0 {
            nearness();
            let depth = k(0x6415A4);
            if !below(depth, 0x14_0000) {
                and(with_bit(field(0x40), bit));
                for var in [0x5CCE50u32, 0x5CCE54, 0x5CCE58, 0x5CCE5C] {
                    and(!(1u32 << (k(var) & 31)));
                }
                and(!k(0x5CCE38));
                and(!k(0x5CCE3C));
            }
            if !below(depth, 0x28_0000) {
                and(!k(0x5CCE40));
                and(!k(0x5CCE44));
                and(!k(0x5CCE34));
            }
            if !below(depth, 0x3C_0000) {
                and(!k(0x5CCE4C));
                and(!k(0x5CCE48));
            }
        }
        let mut ecx = 0x60u32.wrapping_add(near_size());
        w(0x71BA0C, ecx);
        if state & 0x100 != 0 {
            if state & 6 == 0 {
                and(with_bit(field(0x40), bit));
            }
            ecx = 0xF0;
        }
        w(0x606284, ecx);
        ((car + 0xF04) as *mut u32).write_unaligned(k(0x606280));
        if k(0x63DBA8 + 4 * k(0x63DB9C)) == k(0x60627C) && k(0x60626C) != 0 {
            w(0x606280, u32::MAX);
        }
        w(0x635084, 0);
        w(0x635088, 0xFF);
        w(0x63508C, k(0x6092E4));
        w(0x635090, k(0x60935C));
        w(0x635094, k(0x609338).wrapping_neg());
        w(0x635098, k(0x60933C));
        w(0x6365C8, 1);
        w(0x62DAF4, 3);
        let place = |w: &dyn Fn(u32, u32)| {
            w(0x63501C, get(car + 0x134));
            w(0x635020, get(car + 0x138));
            w(0x635024, get(car + 0x13C));
        };
        place(&w);
        w(0x635038, 0x10);
        crate::scene::matrix::set_light_direction();
        place(&w);
        w(0x635028, switches);
        w(0x608CB0, 1);
        w(0x635054, 0);
        w(0x62DAF4, 3);
        w(0x62DAF8, 0);
        w(0x6214FC, k(0x60627C).wrapping_mul(0x14).wrapping_add(0x50));
        car_body_matrix(r);
        crate::race::frame_draw::start_kept_reflections().apply(r);
        crate::race::car_draw::car_part(1, r)?;
        crate::race::frame_draw::end_kept_reflections();
        w(0x62DAF8, 1);
        if below(k(0x6415A4), 0x3C_0000) {
            for (bit, matrix) in [0x20u32, 0x40, 0x80, 0x100].into_iter().zip(PART_MATRICES) {
                w(0x6214FC, k(0x6214FC).wrapping_add(1));
                if k(0x635028) & bit != 0 {
                    car_part_matrix(r, matrix);
                    crate::race::car_draw::car_part(1, r)?;
                }
            }
        }
        w(0x608CB0, 0);
        w(0x608CAC, u32::MAX);
        clear_car_marks().apply(r);
        Ok(())
    }
}

/// `sub_4616AA`'s nearness: `dword_71BA08` = (`dword_6415A4` - 0xA0000) / 0x140000 held to 0 ..
/// 1 (`flt_5C5D90`, `flt_5C5D38`), `g_shine` (and `dword_5CCE78`) times 1 less it; the
/// original's x87 run (`dword_62400C` left 0x140000).
unsafe fn nearness() {
    use crate::math::x87_load::below_or_unordered;
    unsafe {
        let b = game_addr(0) as u32;
        game_write(rva(0x62400C), 0xA_0000u32);
        core::arch::asm!(
            "fild dword ptr [{b} + 0x2415A4]",
            "fisub dword ptr [{b} + 0x22400C]",
            "mov dword ptr [{b} + 0x22400C], 0x140000",
            "fidiv dword ptr [{b} + 0x22400C]",
            "fstp dword ptr [{b} + 0x31BA08]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        let near: u32 = game_read(rva(0x71BA08));
        // `fld zero; fld near; fcompp`: near below 0 (or no number) -> 0.
        if below_or_unordered(near, game_read(rva(0x5C5D90))) {
            game_write(rva(0x71BA08), game_read::<u32>(rva(0x5C5D90)));
        }
        let near: u32 = game_read(rva(0x71BA08));
        if !below_or_unordered(near, game_read(rva(0x5C5D38))) {
            game_write(rva(0x71BA08), game_read::<u32>(rva(0x5C5D38)));
        }
        core::arch::asm!(
            ".byte 0xD9, 0xE8",
            "fsub dword ptr [{b} + 0x31BA08]",
            "fimul dword ptr [{b} + 0x1CCE74]",
            "fist dword ptr [{b} + 0x1CCE78]",
            "fistp dword ptr [{b} + 0x1CCE74]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// `sub_4616AA`'s size: 0x90 (into `dword_62400C`) times `dword_71BA08`, rounded (left in
/// `dword_62400C`).
unsafe fn near_size() -> u32 {
    unsafe {
        let b = game_addr(0) as u32;
        game_write(rva(0x62400C), 0x90u32);
        core::arch::asm!(
            "fild dword ptr [{b} + 0x22400C]",
            "fmul dword ptr [{b} + 0x31BA08]",
            "fistp dword ptr [{b} + 0x22400C]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        game_read(rva(0x62400C))
    }
}

/// `race::draw_car_shined` (0x461669): `g_shine` for the car (0x3FCE0 less `dword_623D80`,
/// held to 0 .. 0x40000, times `dword_60AD0C`, >> 18), the car drawn (part 1:
/// `race::draw_car`), `g_shine` put back.
pub unsafe fn draw_car_shined(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let shine: u32 = game_read(rva(0x5CCE74));
        let mut eax = 0x3_FCE0u32.wrapping_sub(game_read(rva(0x623D80)));
        if (eax as i32) < 0 {
            eax = 0;
        }
        if !below(eax, 0x4_0000) {
            eax = 0x4_0000;
        }
        let product = eax as u64 * game_read::<u32>(rva(0x60AD0C)) as u64;
        r.edx = (product >> 32) as u32;
        r.eax = (product as u32) >> 18;
        game_write(rva(0x5CCE74), r.eax);
        let done = crate::race::car_draw::shined_part(1, r);
        game_write(rva(0x5CCE74), shine);
        done
    }
}

/// `race::draw_car_entry` (0x455105): car ebx & 7 (into `dword_60627C`; `dword_62DAD4` its
/// +0x240): its place turned by `g_viewMatrix` (`scene::rotate_point`) into `dword_64159C` ..
/// `dword_6415A4`; drawn (part 1: `race::draw_car_shined`, counted in `dword_641594`) unless
/// behind (depth + 0x40000 below 0) or aside (|x| - 0x80000 not below the depth), counted in
/// `dword_641598`. Refuses (before anything) a car without its block.
pub unsafe fn draw_car_entry(r: &mut Regs) -> Result<(), Refused> {
    use crate::math::x87_load::tbyte_to_int;
    use crate::race::skidmarks::int_80;
    unsafe {
        let n = r.ebx & 7;
        let car: u32 = game_read(rva(0x71B3B8) + 4 * n as usize);
        if car == 0 {
            return Err(BadInput("a car without its block").into());
        }
        r.ebx = n;
        game_write(rva(0x60627C), n);
        r.esi = car;
        r.eax = get(car + 0x240);
        game_write(rva(0x62DAD4), r.eax);
        let t = crate::scene::rotate_point::rotate_point(
            ga(VIEW_MATRIX) as u32,
            [
                int_80(get(car + 0x134)),
                int_80(get(car + 0x138)),
                int_80(get(car + 0x13C)),
            ],
        );
        let place = t.map(tbyte_to_int);
        for (k, p) in place.iter().enumerate() {
            game_write(rva(0x64159C) + 4 * k, *p);
        }
        let depth = place[2];
        let mut x = place[0];
        r.eax = x;
        if below(depth, 0xFFFC_0000) {
            game_write(
                rva(0x641598),
                game_read::<u32>(rva(0x641598)).wrapping_add(1),
            );
            return Ok(());
        }
        if (x as i32) < 0 {
            x = x.wrapping_neg();
        }
        x = x.wrapping_sub(0x8_0000);
        r.eax = x;
        if !below(x, depth) {
            game_write(
                rva(0x641598),
                game_read::<u32>(rva(0x641598)).wrapping_add(1),
            );
            return Ok(());
        }
        game_write(
            rva(0x641594),
            game_read::<u32>(rva(0x641594)).wrapping_add(1),
        );
        crate::race::car_draw::entry_car_part(1, r)
    }
}

// ---- the parts in the game ----

/// `draw_car`'s part: `scene::draw_scene_object`.
pub fn car_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_scene_object::draw_scene_object_in_game(r) }
}

/// `draw_car_shined`'s part: `race::draw_car`.
pub fn shined_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { draw_car(r) }
}

/// `draw_car_entry`'s part: `race::draw_car_shined`.
pub fn entry_car_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { draw_car_shined(r) }
}
