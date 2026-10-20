//! The controller page's axis learning (`sub_46B131` -> `sub_46B4AE`, `sub_46B27A`; stream
//! A, part F1, 2026-10-18): while `dword_7551F4` is 1 (for at most 3 s from
//! `dword_7551F8`), the player's control `dword_7551FC` (1 .. ) learns the axis or the
//! direction the player moves, into the player's +0x50 table.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

fn w(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}

const HISTORY: [usize; 4] = [0x7551E4, 0x7551E8, 0x7551EC, 0x7551F0];
const FOUND: usize = 0x7559EC; // dword_7559EC

/// `screens::classify_controller` (0x46B27A): from the controller's four values (ecx x, edx
/// y, eax z, ebx the buttons): the button (its number + 1, when exactly one is down), and
/// each of left (x below 0), right (x above 0), y and z shifted into its history
/// (`dword_7551E4` ..: a 1 for "under 0x80"); a direction held over 0x1D of the last 32 polls
/// while the others were not (its history empty) gives 0x400 / 0x800 / 0x100 / 0x200 with the
/// button, else 0; into `dword_7559EC`. Every register kept.
pub unsafe fn classify_controller(eax: u32, mut ebx: u32, mut ecx: u32, edx: u32) {
    let mut found = 0u32;
    if ebx != 0 {
        loop {
            found += 1;
            let out = ebx & 1;
            ebx >>= 1;
            if out != 0 {
                break;
            }
        }
        if ebx != 0 {
            found = 0;
        }
    }
    let mut left = ecx;
    if (left as i32) > 0 {
        left = 0;
    }
    if (left as i32) < 0 {
        left = left.wrapping_neg();
    }
    if (ecx as i32) < 0 {
        ecx = 0;
    }
    for (var, v) in HISTORY.into_iter().zip([left, ecx, edx, eax]) {
        w(var, (k(var) << 1) | u32::from(v < 0x80));
    }
    let [l, r, y, z] = HISTORY.map(|v| k(v).count_ones());
    let min = |a: u32, b: u32, c: u32| a.min(b).min(c);
    let bit = if l == 0 {
        (min(z, r, y) >= 0x1D).then_some(0x400)
    } else if r == 0 {
        (min(z, l, y) >= 0x1D).then_some(0x800)
    } else if y == 0 {
        (min(z, l, r) >= 0x1D).then_some(0x100)
    } else if z == 0 {
        (min(l, r, y) >= 0x1D).then_some(0x200)
    } else {
        Some(0)
    };
    w(
        FOUND,
        match bit {
            Some(b) => found | b,
            None => 0,
        },
    );
}

/// `screens::learn_axis` (0x46B131): see the header. With 3 s gone the learning stops
/// (`dword_7551FC` -1). The chosen item's prompt (string 0x3BE at x 0x1B8, the item's y)
/// while `dword_7559A4` is 1. A player of kind 3 (+0x34) learning control 1 or 2: the axis
/// moved (`input::axis_moved` 1) into +0x3B or +0x3A, `dword_4F6F08` its +0x38, the found
/// 0x200 or 0x100; else the controller read and classified. Space (key 0x39) stops it with
/// nothing; anything found goes to the player's +0x50 + 4 x (control - 1). Every register
/// kept. Parts: 1 the clock (eax), 2 the prompt (eax id, ebx font, ecx x, edx y), 3
/// `axis_moved` (eax in; carry, eax), 4 the controller's read (ecx, edx, eax, ebx), 5 the
/// key (al). Refuses (before anything) a player without its record.
pub unsafe fn learn_axis(entry: &Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    unsafe {
        if k(0x7551F4) != 1 {
            return Ok(());
        }
        let player = crate::menu::player::player_at(k(0x75598C));
        if player == 0 {
            return Err(crate::errors::BadInput("no current player record").into());
        }
        let mut r = *entry;
        w(FOUND, 0);
        part(1, &mut r)?;
        r.eax = r.eax.wrapping_sub(k(0x7551F8));
        if (r.eax.wrapping_sub(0xBB8) as i32) >= 0 {
            w(0x7551FC, u32::MAX);
            w(0x7551F4, 0);
            return Ok(());
        }
        if k(0x7559A4) == 1 {
            let items = k(0x75558D)
                .wrapping_add(4)
                .wrapping_add(k(0x755525).wrapping_sub(1).wrapping_mul(0x5C));
            r.eax = 0x3BE;
            r.ebx = crate::framework::game_addr(rva(0x51E8C4)) as u32;
            r.ecx = 0x1B8;
            r.edx = ((items + 0xC) as *const u32).read_unaligned();
            r.edi = items;
            part(2, &mut r)?;
        }
        // The original's `mov eax, dword_75598C; mov edi, g_players[eax*4]`.
        r.eax = k(0x75598C);
        r.edi = player;
        let field = |o: u32| (player + o) as *mut u8;
        let mut take = true;
        let control = k(0x7551FC);
        let learning =
            ((player + 0x34) as *const u32).read_unaligned() == 3 && (control == 1 || control == 2);
        if learning {
            r.eax = 1;
            part(3, &mut r)?;
            if !r.carry() {
                field(if control == 1 { 0x3B } else { 0x3A }).write(r.eax as u8);
                w(0x4F6F08, ((player + 0x38) as *const u32).read_unaligned());
                w(FOUND, if control == 1 { 0x200 } else { 0x100 });
                w(0x7551F4, 0);
                take = false;
            }
        } else {
            part(4, &mut r)?;
            classify_controller(r.eax, r.ebx, r.ecx, r.edx);
        }
        if take {
            part(5, &mut r)?;
            if r.eax as u8 == 0x39 {
                w(FOUND, 0);
                w(0x7551F4, 0);
            } else if k(FOUND) == 0 {
                return Ok(());
            }
        }
        let slot = (player + 0x50)
            .wrapping_add(k(0x7551FC).wrapping_mul(4))
            .wrapping_sub(4);
        (slot as *mut u32).write_unaligned(k(FOUND));
        Ok(())
    }
}

/// `learn_axis`' parts in the game.
pub fn learn_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    unsafe {
        match place {
            1 => r.eax = crate::system::tick_count::tick_count(),
            2 => {
                let out = crate::text::draw_string_by_id::draw_string_by_id_at(
                    r.eax,
                    r.ebx as *mut crate::text::font::Font,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut GameCanvas,
                )?;
                (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
            }
            3 => {
                let m = crate::input::axes::axis_moved(r.eax);
                r.eax = m.eax;
                r.set_carry(m.carry);
            }
            4 => {
                [r.ecx, r.edx, r.eax, r.ebx] = crate::input::axes::first_device_state();
            }
            _ => r.eax = (r.eax & !0xFF) | crate::input::take_key::take_key() as u32,
        }
    }
    Ok(())
}

/// `screens::draw_no_controller` (0x46AEAA): string 0x56D at (0x1BC, 0xD5) in the font at
/// 0x51E8C4. Gives the registers `draw_string_by_id` leaves.
pub unsafe fn draw_no_controller(
    canvas: &mut dyn crate::text::draw_text::Canvas,
) -> Result<crate::screens::harness::Left, Refused> {
    // Converted: `rc2k_core::screens::notes::NO_CONTROLLER`.
    crate::screens::game_screen::draw_note(canvas, &rc2k_core::screens::notes::NO_CONTROLLER)
}

/// What the controller page reads: refused before anything.
pub unsafe fn controller_page_refuses() -> Result<u32, Refused> {
    unsafe {
        let player = crate::menu::player::player_at(k(0x75598C));
        if player == 0 {
            return Err(crate::errors::BadInput("no current player record").into());
        }
        let device = ((player + 0x2C) as *const u32).read_unaligned();
        if device != 0 && k(0x4F6EE8) == 0 {
            return Err(crate::errors::BadInput("no input device").into());
        }
        Ok(player)
    }
}

/// `screens::draw_controller_page` (0x46A454): the player's controller (+0x2C, 0 none: items
/// 3 .. 8 off and string 0x56D): items 3 .. 8 on; the device's name (record +0x30, the record
/// in `dword_755A0C`); the mode (string 0x56D + player +0x3C); item 4 only for a device that
/// has it (+0x1C 1) with its strength (player +0x48 as a percent, or string 0x56D); the
/// sensitivity (string 0x3A1 + player +0x34); two sliders (player +0x40 and +0x44: their
/// fills clipped in, their ends at `dword_7559DC` .. `dword_7559E8`); and the stick's place
/// now (x of the first device's state, x 100 / 256 from 0x21A; a pressed button sprite 9,
/// else 8; `hhmm` the buttons). The sprites' palettes are ebx as the original leaves it.
/// Every register kept. Refuses (before anything) no current player record, or no input
/// device for a player that has one; an x whose x 100 is 0 is the original's fatal error
/// 0x80701010.
pub unsafe fn draw_controller_page(
    canvas: &mut dyn crate::text::draw_text::Canvas,
) -> Result<(), Refused> {
    use crate::gfx::blit_glyph::BlitCall;
    use crate::gfx::surface::{CLIP_LEFT_RVA, CLIP_RIGHT_RVA};
    use crate::menu::disable_item::disable_item;
    use crate::menu::enable_item::enable_item;
    use crate::text::draw_string_by_id::draw_string_by_id_at;
    use crate::text::draw_text::draw_text_at;
    unsafe {
        let player = controller_page_refuses()?;
        let f = |o: u32| ((player + o) as *const u32).read_unaligned();
        let font = crate::screens::font_at(0x51E8C4);
        let device = f(0x2C);
        if device == 0 {
            for n in 3..=8 {
                disable_item(n)?;
            }
            draw_no_controller(canvas)?;
            return Ok(());
        }
        for n in 3..=8 {
            enable_item(n)?;
        }
        let record = crate::framework::game_addr(rva(0x4F78E4)) as u32
            + device.wrapping_sub(1).wrapping_mul(0x50);
        w(0x755A0C, record);
        draw_text_at(font, (record + 0x30) as *const u8, 0x1BC, 0xD5, canvas)?;
        w(0x7559AC, f(0x3C).wrapping_add(0x56D));
        draw_string_by_id_at(k(0x7559AC), font, 0x1BC, 0xF0, canvas)?;
        let has = ((record + 0x1C) as *const u32).read_unaligned() == 1;
        if has {
            enable_item(4)?;
        } else {
            disable_item(4)?;
        }
        if !has || (f(0x48) as i32) <= 0 {
            draw_string_by_id_at(0x56D, font, 0x1BC, 0x10B, canvas)?;
        } else {
            let mut line = crate::strings::append_percent::append_percent(f(0x48));
            line.push(0);
            std::ptr::copy_nonoverlapping(line.as_ptr(), crate::screens::line_buffer(), line.len());
            draw_text_at(font, crate::screens::line_buffer(), 0x1BC, 0x10B, canvas)?;
        }
        w(0x7559AC, f(0x34).wrapping_add(0x3A1));
        let out = draw_string_by_id_at(k(0x7559AC), font, 0x1BC, 0x126, canvas)?;
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        canvas.blit_glyph(BlitCall::new(5, 0x1B8, 0x14C, out.text as u32, sprites))?;
        let (right, left) = (k(0x4F5040), k(0x4F503C));
        let a = f(0x40);
        game_write(CLIP_RIGHT_RVA, 0x281u32.wrapping_sub(a).wrapping_sub(1));
        w(0x7559E0, 0x281u32.wrapping_sub(a));
        game_write(CLIP_LEFT_RVA, 0x1C0u32.wrapping_add(a));
        w(0x7559DC, 0x1C0u32.wrapping_add(a).wrapping_sub(4));
        let done = (|| -> Result<(), Refused> {
            canvas.blit_glyph(BlitCall::new(3, 0x1BC, 0x14C, k(0x7559DC), sprites))?;
            let t = 100u32.wrapping_sub(f(0x44));
            game_write(CLIP_RIGHT_RVA, 0x285u32.wrapping_sub(t).wrapping_sub(1));
            w(0x7559E8, 0x285u32.wrapping_sub(t));
            game_write(CLIP_LEFT_RVA, 0x1BCu32.wrapping_add(t));
            w(0x7559E4, 0x1BCu32.wrapping_add(t).wrapping_sub(4));
            canvas.blit_glyph(BlitCall::new(4, 0x1BC, 0x14C, k(0x7559E4), sprites))?;
            Ok(())
        })();
        game_write(CLIP_LEFT_RVA, left);
        game_write(CLIP_RIGHT_RVA, right);
        done?;
        let palette = k(0x7559E4);
        for (glyph, x) in [
            (7u32, 0x7559DCusize),
            (7, 0x7559E0),
            (6, 0x7559E4),
            (6, 0x7559E8),
        ] {
            canvas.blit_glyph(BlitCall::new(glyph, k(x) as i32, 0x14C, palette, sprites))?;
        }
        let [x, _, _, buttons] = crate::input::axes::first_device_state();
        w(0x7559F0, buttons);
        let place = if x == 0 {
            0x21A
        } else if (x as i32) > 0 {
            let v = x.wrapping_mul(100);
            if v == 0 {
                return Err(crate::gfx::blit_bar::BAR_HAS_NO_LENGTH.into());
            }
            (v / 0x100).wrapping_add(0x21A)
        } else {
            let v = x.wrapping_neg().wrapping_mul(100);
            if v == 0 {
                return Err(crate::gfx::blit_bar::BAR_HAS_NO_LENGTH.into());
            }
            (v / 0x100)
                .wrapping_neg()
                .wrapping_add(1)
                .wrapping_add(0x21A)
        };
        w(0x7559EC, place);
        let glyph = if buttons != 0 { 9 } else { 8 };
        canvas.blit_glyph(BlitCall::new(glyph, place as i32, 0x156, buttons, sprites))?;
        Ok(())
    }
}
