//! Everything on the service area screen that is text, bars and small sprites, over the
//! picture: a title with the car's number, the service time readouts, fourteen damage bars with
//! their labels and sprites, the setup bars' labels and markers, the repair times and the car
//! condition bar. The order is fixed (the title, `draw_service_time`, the `STEPS` list,
//! `draw_repair_times`, the six setup markers, `draw_inspection_status`); the sprites in the list
//! take the end of the text drawn just before them as their palette (unused). It also sets a
//! service area work cell to 0.
//!
//! Seen in a screenshot of a championship's service area: the title "JOGADOR 1 ÁREA DE SERVIÇO";
//! fourteen parts in two columns (engine, exhaust, turbo, cooling, clutch, differential, gearbox;
//! body, electronics, lights, steering, suspension, brakes, tyres), each with an icon, a repair
//! time and a damage bar; "n/d" on the turbo's bar when the car has no turbo; five gear ratio
//! pictures and six tyre pictures with the name of the chosen tyre; four setup sliders; the
//! inspection status bar; and the four big buttons at the bottom (load setup, save setup, view
//! stage, continue), whose labels are drawn here. Switching it off removed all text and icons; the
//! boxes that hold them are drawn by `draw_service_boxes`.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_bar::{Bar, blit_bar_in};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::owner::Gfx;
use crate::screens::draw_inspection_status::draw_inspection_status;
use crate::screens::draw_repair_times::draw_repair_times;
use crate::screens::draw_service_time::draw_service_time;
use crate::screens::setup_markers::draw_marker;
use crate::screens::{
    draw_brake_balance_marker, draw_ride_height_installed_marker, draw_ride_height_marker,
    draw_steering_sensitivity_marker, draw_suspension_stiffness_installed_marker,
    draw_suspension_stiffness_marker,
};
use crate::service::state::current_player;
use crate::strings::append_string_by_id::push_string;
use crate::strings::text_from_ids::{TEXT_LEN, text_at};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use rc2k_core::screens::service::{
    AREA_STEPS, AREA_TITLE, AREA_TITLE_AT, AreaFont, AreaStep, DAMAGE_BAR, TYRE_AT, area_title,
};

// ---- game data ----
const SETTING_FIRST_STRING: u32 = rc2k_core::screens::service::TYRE_FIRST_STRING;

/// The fonts' places.
fn font_of(font: AreaFont) -> crate::gfx::typefaces::FontId {
    use crate::gfx::typefaces::fonts;
    match font {
        AreaFont::Title => fonts::TITLE,
        AreaFont::Heading => fonts::HEADING_23,
        AreaFont::Label => fonts::LABEL,
        AreaFont::Small => fonts::SMALL,
        AreaFont::Wide => fonts::NOTICE,
        AreaFont::Total => fonts::LABELS,
    }
}

/// Returns whether the last text drawn was too wide for its box. Refuses a missing player and a
/// title that does not fit the text buffer.
pub fn draw_service_area(
    o: crate::owners::Owners<'static>,
    gfx: &Gfx,
    race: &crate::race::owner::Race,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    race.shared.switches.get().service_cleared.set(0);

    // The logic is in `rc2k_core::screens::service` (`area_title`, `AREA_STEPS`).
    let fault = crate::menu::event_flow::Fault::default();
    let string = |id: u32| {
        let mut bytes = Vec::new();
        fault.ok(push_string(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &mut bytes,
            id,
        ));
        bytes
    };
    let title = area_title(
        &string(AREA_TITLE.0),
        screens.event.current_player.get(),
        &string(AREA_TITLE.1),
    );
    fault.check()?;
    if title.len() > TEXT_LEN {
        return Err(BadInput("the title is longer than the text buffer").into());
    }
    text_at(o.strings).write_at(0, &title, "the title")?;
    let (x, y) = AREA_TITLE_AT;
    draw_text_at(font_of(AreaFont::Title), &title, x, y, canvas)?;

    draw_service_time(o, canvas)?;

    let Some(player) = current_player(menus) else {
        return Err(BadInput("no current player object: the original reads through null").into());
    };
    let car = player.get();
    let (width, fill_glyph, end_glyph) = DAMAGE_BAR;
    for step in AREA_STEPS {
        match step {
            AreaStep::Label(id, font, x, y) => {
                draw_string_by_id_at(
                    crate::strings::append_string_by_id::StringOwners::of(o),
                    id,
                    font_of(font),
                    x,
                    y,
                    canvas,
                )?;
            }
            AreaStep::LabelIfClass(id, font, x, y) => {
                if car.class != 0 {
                    draw_string_by_id_at(
                        crate::strings::append_string_by_id::StringOwners::of(o),
                        id,
                        font_of(font),
                        x,
                        y,
                        canvas,
                    )?;
                }
            }
            AreaStep::DamageBar(x, y, part) => {
                Bar {
                    x,
                    y,
                    width,
                    fill_glyph,
                    end_glyph,
                    percent: car.damage[part] as i32,
                }
                .write_in(gfx);
                blit_bar_in(gfx, menus, &mut |call| canvas.blit_glyph(call))?;
            }
            AreaStep::Sprite(glyph, x, y) => {
                canvas.blit_glyph(BlitCall::sprite(
                    menus,
                    glyph,
                    x,
                    y,
                    crate::gfx::surface::clip(gfx).right,
                ))?;
            }
        }
    }

    let setting = screens.state.driver_stats.setups[1]
        .chosen
        .get()
        .wrapping_add(SETTING_FIRST_STRING);
    screens.draw_work.figure.set(setting);
    let (x, y) = TYRE_AT;
    draw_string_by_id_at(
        crate::strings::append_string_by_id::StringOwners::of(o),
        setting,
        font_of(AreaFont::Small),
        x,
        y,
        canvas,
    )?;

    // The markers get the car.
    draw_repair_times(o, canvas)?;
    for marker in [
        draw_brake_balance_marker::MARKER,
        draw_steering_sensitivity_marker::MARKER,
        draw_ride_height_installed_marker::MARKER,
        draw_suspension_stiffness_installed_marker::MARKER,
        draw_ride_height_marker::MARKER,
        draw_suspension_stiffness_marker::MARKER,
    ] {
        draw_marker(menus, gfx, marker, player, canvas)?;
    }
    draw_inspection_status(o, screens, canvas)
}
