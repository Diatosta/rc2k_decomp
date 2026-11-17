//! The advanced page's drawing: the rows' names are the menu's items; the values (on / off, a
//! number, the renderer) are drawn here in the names' column, with "next start" beside a
//! renderer that is not the one running.

use crate::errors::Refused;
use crate::menu::advanced_items;
use crate::strings::append_string_by_id::own_text;
use crate::text::draw_text::Canvas;
use rc2k_core::options::advanced::{Kind, ROWS, Renderer};
use rc2k_core::strings::format::{Pad, decimal};
use rc2k_core::strings::own::id;

/// The renderer the program is running with, as the page names it.
fn running() -> Renderer {
    if crate::render::gl_running() {
        Renderer::Gl
    } else if crate::render::gpu_running() {
        Renderer::Gpu
    } else {
        Renderer::Software
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
                if values.renderer != running() {
                    text.extend(b" - ");
                    text.extend(own(id::NEXT_START));
                }
                text
            }
            Kind::Flag { .. } => own(if values.flags[n] { id::ON } else { id::OFF }),
        });
    }
    let mut screen = SettingsScreen::new(canvas, o);
    rc2k_core::screens::settings_pages::advanced(&mut screen, &texts)?;
    Ok(())
}
