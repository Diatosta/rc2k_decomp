//! The advanced page's drawing: the rows' names are the menu's items; the values (on / off, a
//! number, the renderer) are drawn here in the names' column, with "next start" beside a
//! renderer that will only run at the next start.

use crate::errors::Refused;
use crate::menu::advanced_items;
use crate::menu::axes_items::{set_look, shown_look};
use crate::menu::item_sprites_store::ItemSprite;
use crate::strings::append_string_by_id::own_text;
use crate::text::draw_text::Canvas;
use rc2k_core::options::advanced::{Kind, ROW_SLOTS, ROWS, Renderer, rows_on_page};
use rc2k_core::strings::format::{Pad, decimal};
use rc2k_core::strings::own::id;

/// The sprite of a setting's row (the left / right arrows) and the one of an action (the single
/// right arrow of the menus' entries that open something).
const SETTING_SPRITE: u32 = 2;
const ACTION_SPRITE: u32 = 1;

/// The text id of a row's name.
fn row_name(row: usize) -> u32 {
    // The names of the rows follow each other in the own texts (the eighth, rain and snow, is
    // the graphics page's).
    let first = |n: u32| id::ADVANCED_ROWS + n;
    match ROWS[row].kind {
        Kind::Threads => first(0),
        Kind::Renderer => first(1),
        Kind::QuitPictures => first(10),
        Kind::DeleteData => id::DELETE_DATA,
        Kind::SmoothFrames => id::SMOOTH_FRAMES,
        Kind::Flag { key, .. } => match key {
            "framecounter" => first(2),
            "nofmv" => first(3),
            "screensnapshot" => first(4),
            "nonlinearsteering" => first(5),
            "noautoneutral" => first(6),
            "nomusic" => first(8),
            "noforcefeedback" => first(9),
            "usesavegamekeys" => id::SAVE_CONTROLS,
            _ => id::BLANK,
        },
    }
}

/// The renderer the program is running with, as the page names it.
fn running() -> Renderer {
    use crate::render::choice::Wish;
    match crate::render::running_wish() {
        Wish::Gl => Renderer::Gl,
        Wish::GpuFirst => Renderer::Gpu,
        Wish::Software => Renderer::Software,
    }
}

pub fn draw_advanced(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use crate::screens::game_screen::SettingsScreen;
    let own = |n: u32| own_text(o.strings, n);
    let values = advanced_items::shown(o);
    let mut texts = Vec::new();
    let page = advanced_items::page();
    let shown_rows = rows_on_page(page);
    // The rows of the part shown take the row items in order; the rest are hidden.
    for slot in 0..ROW_SLOTS {
        let row = shown_rows.start + slot;
        let name = if row < shown_rows.end {
            row_name(row)
        } else {
            id::BLANK
        };
        set_look(
            o.menus,
            2 + slot as u32,
            shown_look(name),
            row >= shown_rows.end,
        )?;
        // A setting's row has the left / right arrows; the deletion row is an action, which
        // the buttons' single arrow says (the slots are shared by both parts of the page).
        let sprite = if row < shown_rows.end && matches!(ROWS[row].kind, Kind::DeleteData) {
            ItemSprite::Sprite(ACTION_SPRITE)
        } else {
            ItemSprite::Sprite(SETTING_SPRITE)
        };
        crate::menu::item_at::item_at(o.menus, 2 + slot as u32)?
            .sprites
            .set(sprite)?;
    }
    let turn = if page == 0 {
        id::ADVANCED_MORE
    } else {
        id::ADVANCED_FIRST
    };
    // A button: only its text changes.
    crate::menu::item_at::item_at(o.menus, 2 + ROW_SLOTS as u32 + 1)?
        .string
        .set(turn);
    for n in shown_rows {
        let row = &ROWS[n];
        texts.push(match row.kind {
            Kind::Threads => decimal(values.threads, Pad::None),
            Kind::Renderer => {
                let mut text = own(match values.renderer {
                    Renderer::Gpu if cfg!(all(target_os = "linux", target_arch = "aarch64")) => {
                        id::RENDERER_GLES
                    }
                    Renderer::Gpu => id::RENDERER_GPU,
                    Renderer::Gl => id::RENDERER_GL,
                    Renderer::Software => id::RENDERER_SOFTWARE,
                });
                // Beside a renderer that is not the one running: on the handheld a change only
                // counts at the next start; on the desktop the swap is made at Apply, so the
                // note is only for a renderer that was asked for and did not start.
                let waits = values.renderer != running()
                    && (cfg!(all(target_os = "linux", target_arch = "aarch64"))
                        || advanced_items::applied_renderer(o) == values.renderer);
                if waits {
                    text.extend(b" - ");
                    text.extend(own(id::NEXT_START));
                }
                text
            }
            Kind::QuitPictures => own(if values.quit_pictures {
                id::ON
            } else {
                id::OFF
            }),
            Kind::SmoothFrames => own(if values.smooth_frames {
                id::ON
            } else {
                id::OFF
            }),
            Kind::Flag { .. } => own(if values.flags[n] { id::ON } else { id::OFF }),
            // An action: no value beside the name.
            Kind::DeleteData => Vec::new(),
        });
    }
    let mut screen = SettingsScreen::new(canvas, o);
    rc2k_core::screens::settings_pages::advanced(&mut screen, &texts)?;
    Ok(())
}
