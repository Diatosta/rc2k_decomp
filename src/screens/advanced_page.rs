//! The advanced page's drawing: the rows' names are the menu's items; the values (on / off, a
//! number, the renderer) are drawn here in the names' column, with "next start" beside a
//! renderer that will only run at the next start.

use crate::errors::Refused;
use crate::menu::advanced_items;
use crate::strings::append_string_by_id::own_text;
use crate::text::draw_text::Canvas;
use rc2k_core::options::advanced::{Kind, ROWS, Renderer};
use rc2k_core::strings::format::{Pad, decimal};
use rc2k_core::strings::own::id;

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
    for (n, row) in ROWS.iter().enumerate() {
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
            Kind::SplitMany => own(if values.split_many { id::ON } else { id::OFF }),
            Kind::Flag { .. } => own(if values.flags[n] { id::ON } else { id::OFF }),
        });
    }
    let mut screen = SettingsScreen::new(canvas, o);
    rc2k_core::screens::settings_pages::advanced(&mut screen, &texts)?;
    Ok(())
}
