//! `screens::draw_service_area` (0x4A5FA2): draws everything on the service area screen that
//! is text, bars and small sprites, over the picture: a title with the car's number, the
//! service time readouts, fourteen bars from the car's values at +0xD4 .. +0x108 with their
//! labels and sprites, the setup bars' labels and markers, the repair times and the car
//! condition bar.
//!
//! In:  nothing
//! Out: every register preserved; carry as `draw_inspection_status` leaves it
//! The order is the original's: the title ("<string 0x190> <car index + 1> <string 0x486>",
//! built in the shared text buffer), `draw_service_time`, the `STEPS` list, the string
//! 0x4B0 + `dword_75AC9C`, `draw_repair_times`, the six setup markers, `draw_inspection_status`.
//! The sprites in the list are blitted with whatever ebx holds as the palette, which is the
//! end of the text drawn just before them; reproduced. It also sets `dword_6218AC` to 0.
//! Its address is in the service area's list at 0x75924A; it is also called from 0x46D001.
//! What is on it, from the owner's screenshot of 2026-10-10 (Portuguese game): the title
//! "JOGADOR 1 ÁREA DE SERVIÇO"; a repair section with fourteen parts in two columns (engine,
//! exhaust, turbo, cooling, clutch, differential, gearbox; body, electronics, lights,
//! steering, suspension, brakes, tyres), each with an icon, a repair time and a damage bar
//! (car +0xD4 .. +0x108, in that row order left then right); "n/d" on the turbo's bar when
//! the car's dword at +0x90 is not 0; five gear ratio pictures (acceleration .. top speed)
//! and six tyre pictures with the name of the chosen tyre (`dword_75AC9C`); a setup section
//! with four sliders; and the inspection status bar. The four big buttons at the bottom
//! (load setup, save setup, view stage, continue) are labels drawn here too.
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on screens::draw_service_area`: the
//! service area of a championship. All text and icons disappeared; the boxes that hold them stayed,
//! so those are drawn by something else (not ported). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::gfx::blit_bar::{Bar, blit_bar};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::screens::draw_inspection_status::draw_inspection_status;
use crate::screens::draw_repair_times::draw_repair_times;
use crate::screens::draw_service_time::draw_service_time;
use crate::screens::setup_markers::draw_marker;
use crate::screens::{
    draw_brake_balance_marker, draw_ride_height_installed_marker, draw_ride_height_marker,
    draw_steering_sensitivity_marker, draw_suspension_stiffness_installed_marker,
    draw_suspension_stiffness_marker, font_at,
};
use crate::service::state::{PLAYER_INDEX_RVA, current_player};
use crate::strings::append_string_by_id::push_string;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use rc2k_core::screens::service::{
    AREA_STEPS, AREA_TITLE, AREA_TITLE_AT, AreaFont, AreaStep, DAMAGE_BAR, TYRE_AT, area_title,
};

// ---- game data ----

const CLEARED_RVA: usize = rva(0x6218AC); // dword_6218AC: set to 0 here, read nowhere known
const WORK_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable
const SETTING_RVA: usize = rva(0x75AC9C); // dword_75AC9C: the chosen tyre, 0 .. 5; its name is string 0x4B0 + this
const SETTING_FIRST_STRING: u32 = rc2k_core::screens::service::TYRE_FIRST_STRING;

/// The fonts' places.
fn font_of(font: AreaFont) -> usize {
    match font {
        AreaFont::Title => 0x51E8A4,
        AreaFont::Heading => 0x51E6A4,
        AreaFont::Label => 0x51E3E4,
        AreaFont::Small => 0x51E3C4,
        AreaFont::Wide => 0x51E404,
        AreaFont::Total => 0x51E6E4,
    }
}

// ---- Rust version ----

/// Returns the carry flag of the last text drawn. Refuses a missing player and a title
/// that does not fit the text buffer.
pub unsafe fn draw_service_area(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        game_write(CLEARED_RVA, 0u32);

        // Converted: `rc2k_core::screens::service` (`area_title`, `AREA_STEPS`).
        let string = |id: u32| {
            let mut bytes = Vec::new();
            push_string(&mut bytes, id);
            bytes
        };
        let title = area_title(
            &string(AREA_TITLE.0),
            game_read::<u32>(PLAYER_INDEX_RVA),
            &string(AREA_TITLE.1),
        );
        if title.len() > TEXT_LEN {
            return Err(BadInput("the title is longer than the text buffer").into());
        }
        std::ptr::copy_nonoverlapping(title.as_ptr(), text(), title.len());
        let (x, y) = AREA_TITLE_AT;
        draw_text_at(font_at(font_of(AreaFont::Title)), text(), x, y, canvas)?;

        draw_service_time(canvas)?;

        let Some(player) = current_player() else {
            return Err(
                BadInput("no current player object: the original reads through null").into(),
            );
        };
        let field = |at: u32| (player.wrapping_add(at) as *const i32).read_unaligned();
        // ebx as the original has it: the sprites get it as their palette.
        let mut ebx = 0u32;
        let (width, fill_glyph, end_glyph) = DAMAGE_BAR;
        for step in AREA_STEPS {
            match step {
                AreaStep::Label(id, font, x, y) => {
                    ebx =
                        draw_string_by_id_at(id, font_at(font_of(font)), x, y, canvas)?.text as u32;
                }
                AreaStep::LabelIf(car_offset, id, font, x, y) => {
                    if field(car_offset) != 0 {
                        ebx = draw_string_by_id_at(id, font_at(font_of(font)), x, y, canvas)?.text
                            as u32;
                    }
                }
                AreaStep::DamageBar(x, y, car_offset) => {
                    Bar {
                        x,
                        y,
                        width,
                        fill_glyph,
                        end_glyph,
                        percent: field(car_offset),
                    }
                    .write();
                    blit_bar(&mut |call| canvas.blit_glyph(call))?;
                }
                AreaStep::Sprite(glyph, x, y) => {
                    canvas.blit_glyph(BlitCall::new(glyph, x, y, ebx, sprite_table()))?;
                }
            }
        }

        let setting = game_read::<u32>(SETTING_RVA).wrapping_add(SETTING_FIRST_STRING);
        game_write(WORK_RVA, setting);
        let (x, y) = TYRE_AT;
        draw_string_by_id_at(setting, font_at(font_of(AreaFont::Small)), x, y, canvas)?;

        // The markers get the car in edi and, as their palette, the ebx the repair times leave.
        let times = draw_repair_times(canvas)?;
        let palette = times.text as u32;
        for marker in [
            draw_brake_balance_marker::MARKER,
            draw_steering_sensitivity_marker::MARKER,
            draw_ride_height_installed_marker::MARKER,
            draw_suspension_stiffness_installed_marker::MARKER,
            draw_ride_height_marker::MARKER,
            draw_suspension_stiffness_marker::MARKER,
        ] {
            draw_marker(marker, palette, player, canvas)?;
        }
        draw_inspection_status(palette, canvas)
    }
}
