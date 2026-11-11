//! The service area's boxes (`screens::draw_service_boxes` and its four frames): flat see-through
//! quads (`scene::draw_tinted_quad`), and the outlines and frames built from them.

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::race::owner::race;
use crate::session::Dword;

/// A variable of the game.
pub fn rd(a: &Dword) -> u32 {
    a.get()
}

pub fn set(a: &Dword, v: u32) {
    a.set(v);
}

/// One quad: x .. x + w, y .. y + h, the colour and see-through, then `draw_tinted_quad`.
pub fn quad(x: u32, y: u32, w: u32, h: u32, colour: u32, see: u32) -> Result<(), Refused> {
    let scene = crate::scene::owner::scene();
    let q = &race().shared.vars;
    let scratch = crate::race::view_scratch::front_end();
    scratch.quad.corners[0].set(x);
    scratch.quad.corners[4].set(y);
    scratch.quad.corners[1].set(x.wrapping_add(w));
    scratch.quad.corners[5].set(y.wrapping_add(h));
    q.quad_colour[0].set(colour);
    q.quad_colour[1].set(see);
    crate::scene::draw_tinted_quad::draw_tinted_quad(&crate::race::view_scratch::DrawCtx::new(
        crate::race::owner::race(),
        scene,
        scratch,
    ))
}

/// Colours of the service area's boxes (0xRRGGBB).
pub const BLACK: u32 = 0x0;
/// The highlight of the item under the cursor.
pub const YELLOW: u32 = 0xFFFF00;
/// The frame of the installed part.
pub const ORANGE: u32 = 0xFF8000;
/// The see-through value of a solid quad.
pub const SOLID: u32 = 0xFF;

/// A one-pixel outline: the top and bottom edges are `w` wide, the left edge `left` tall and
/// the right edge `right` tall (the original's right edge is one pixel taller than the left).
#[derive(Clone, Copy)]
pub struct Outline {
    x: u32,
    y: u32,
    w: u32,
    left: u32,
    right: u32,
}

impl Outline {
    /// The outline round the box `x .. x + w`, `y .. y + h`.
    pub fn around(x: u32, y: u32, w: u32, h: u32) -> Outline {
        Outline {
            x,
            y,
            w: w - 1,
            left: h - 1,
            right: h,
        }
    }

    /// The same outline one pixel further out on every side.
    pub fn grown(self) -> Outline {
        Outline {
            x: self.x - 1,
            y: self.y - 1,
            w: self.w + 2,
            left: self.left + 2,
            right: self.right + 2,
        }
    }

    /// Top, left, right and bottom edges, solid.
    pub fn draw(self, colour: u32) -> Result<(), Refused> {
        let Outline {
            x,
            y,
            w,
            left,
            right,
        } = self;
        quad(x, y, w, 1, colour, SOLID)?;
        quad(x, y, 1, left, colour, SOLID)?;
        quad(x + w, y, 1, right, colour, SOLID)?;
        quad(x, y + left, w, 1, colour, SOLID)
    }

    /// The cursor's highlight: the outline and the one round it, yellow.
    pub fn draw_highlighted(self) -> Result<(), Refused> {
        self.draw(YELLOW)?;
        self.grown().draw(YELLOW)
    }
}

/// A frame of two nested outlines round cell `index` of a row of cells `step` apart, the first
/// one at `first_x`.
pub fn cell_frame(first_x: u32, step: u32, index: u32, colour: u32) -> Result<(), Refused> {
    let x = first_x + step * index;
    Outline::around(x, 0x12B, step, 0x28).draw(colour)?;
    Outline::around(x + 1, 0x12C, step - 2, 0x26).draw(colour)
}

/// The yellow frame round the chosen part of a row of `cells` cells, drawn while the row's
/// service box is turned over and the cursor is on it (when it blinks on), or else when the
/// chosen part differs from the installed one or the row's inspection flag is set.
pub fn chosen_frame(
    screens: &Screens,
    item: u32,
    setup: &crate::menu::state_store::SetupItem,
    flag: &Dword,
    first_x: u32,
    step: u32,
    cells: u32,
) -> Result<(), Refused> {
    let blinking = screens.draw_work.blink.is_set();
    let wanted =
        if screens.state.service_ui.box_shown.is_set() && screens.loop_flags.item.get() == item {
            blinking
        } else {
            setup.installed.get() != setup.chosen.get() || flag.get() == 1
        };
    let chosen = setup.chosen.get();
    if wanted && chosen < cells {
        cell_frame(first_x, step, chosen, YELLOW)?;
    }
    Ok(())
}
