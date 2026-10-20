//! What the six marker functions of the service area's setup bars share. There are four bars,
//! 204 pixels long: an upper pair at y = 409 and a lower pair at y = 465, the left ones at
//! x = 129 and the right ones at x = 464. They belong to menu items 0x10 (upper left), 0x11
//! (upper right), 0x12 (lower left) and 0x13 (lower right). Each function sets up the bar
//! globals and calls `gfx::blit_bar_marker`, so the marker sits at a percentage of the bar.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10), setup section: 0x10 is the
//! steering sensitivity (slow .. fast), 0x11 the brake balance (front .. rear), 0x12 the ride
//! height (low .. high), 0x13 the suspension stiffness (soft .. hard). The two lower bars
//! have two markers each. The one always drawn (glyph 0x77, orange) is at the value the car
//! has now (`dword_75ACAC`, `dword_75ACB8`): the `_installed_marker` hooks. The other (0x76,
//! yellow) is at the value being chosen (`dword_75ACA8`, `dword_75ACB4`, which the keys step)
//! and only appears while it differs or its change is selected. The steering and brake
//! values are changed in the player directly, so those bars have one marker.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, ESI, RawHook, Regs, SelfTest, game_read, game_write, rva};
use crate::gfx::blit_bar::Bar;
use crate::gfx::blit_bar_marker::blit_bar_marker;
use crate::screens::harness::{CallOriginal, Left, check_draw, dispatch_draw, push_dwords};
use crate::service::state::{
    BLINK_RVA, EDIT_MODE_RVA, MENU_ITEM_RVA, blink_on, edit_mode, menu_item, repair_flag,
    repair_flag_addr,
};
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::setup_markers::{
    BAR_WIDTH, LEFT_X, LOWER_Y, MARKER_GLYPH, RIGHT_X, SECOND_MARKER_GLYPH, UPPER_Y,
};

pub const LOWER_LEFT_CHOSEN_RVA: usize = rva(0x75ACA8); // dword_75ACA8
pub const LOWER_LEFT_INSTALLED_RVA: usize = rva(0x75ACAC); // dword_75ACAC
pub const LOWER_RIGHT_CHOSEN_RVA: usize = rva(0x75ACB4); // dword_75ACB4
pub const LOWER_RIGHT_INSTALLED_RVA: usize = rva(0x75ACB8); // dword_75ACB8

/// One of the six functions.
#[derive(Clone, Copy)]
pub enum Marker {
    /// Always drawn, at the percentage in a variable (glyph 0x77).
    Always { x: i32, percent_rva: usize },
    /// Glyph 0x76 on a lower bar, at the chosen value. While its item is being edited it
    /// blinks. Otherwise it is drawn if the chosen value differs from the installed one, or
    /// if the item's flag in the repair flag table is 1.
    Lower {
        item: u32,
        x: i32,
        chosen_rva: usize,
        installed_rva: usize,
    },
    /// Glyph 0x76 on an upper bar, at the percentage in a field of the player in edi.
    /// While its item is being edited it blinks.
    Upper { item: u32, x: i32, car_offset: u32 },
}

/// Puts the bar in the bar globals and draws the marker: what is left has eax = percent.
unsafe fn place(
    canvas: &mut dyn Canvas,
    palette_at_start: u32,
    (x, y, glyph): (i32, i32, u32),
    percent: i32,
    left: Left,
) -> Result<Left, Refused> {
    Bar {
        x,
        y,
        width: BAR_WIDTH,
        fill_glyph: glyph,
        percent,
        ..Bar::read()
    }
    .write();
    unsafe { blit_bar_marker(palette_at_start, &mut |call| canvas.blit_glyph(call)) }?;
    Ok(left.with(EAX, percent as u32))
}

/// `r` holds the registers at entry: ebx goes to the blitter as the palette when the
/// percentage is 0 or less, and edi is the player for the upper bars.
pub unsafe fn draw_marker(
    marker: Marker,
    r: &Regs,
    canvas: &mut dyn Canvas,
) -> Result<Left, Refused> {
    // Converted: `rc2k_core::screens::setup_markers::plan`; here the values read and the bar drawn.
    use rc2k_core::screens::setup_markers::{Marker as M, Menu, Plan};
    let menu = Menu {
        edit_mode: edit_mode(),
        item: menu_item(),
        blink_on: blink_on(),
    };
    let read = |rva: usize| -> Result<i32, Refused> { Ok(unsafe { game_read(rva) }) };
    let (core, item): (M, u32) = match marker {
        Marker::Always { x, .. } => (M::Always { x }, 0),
        Marker::Lower { item, x, .. } => (M::Lower { item, x }, item),
        Marker::Upper { item, x, .. } => (M::Upper { item, x }, item),
    };
    let percent = || -> Result<i32, Refused> {
        match marker {
            Marker::Always { percent_rva, .. } => read(percent_rva),
            Marker::Lower { chosen_rva, .. } => read(chosen_rva),
            Marker::Upper { car_offset, .. } => {
                if r.edi == 0 {
                    return Err(
                        BadInput("no player in edi: the original reads through null").into(),
                    );
                }
                Ok(unsafe { (r.edi.wrapping_add(car_offset) as *const i32).read_unaligned() })
            }
        }
    };
    let installed = || match marker {
        Marker::Lower { installed_rva, .. } => unsafe { game_read::<i32>(installed_rva) },
        _ => 0,
    };
    let (plan, looked) =
        rc2k_core::screens::setup_markers::plan(core, menu, &percent, &installed, &|| {
            repair_flag(item)
        })?;
    // The flag's address stays in esi whether the marker is drawn or not.
    let left = if looked {
        Left::nothing().with(ESI, repair_flag_addr(item))
    } else {
        Left::nothing()
    };
    match plan {
        Plan::Hidden { item } => Ok(left.with(EAX, item)),
        Plan::Draw {
            x,
            y,
            glyph,
            percent,
        } => unsafe { place(canvas, r.ebx, (x, y, glyph), percent, left) },
    }
}

// ---- hook ----

/// The bar globals: the marker's position is left in them.
fn state() -> Vec<u8> {
    let mut s = Vec::new();
    push_dwords(
        &mut s,
        &[
            rva(0x755224),
            rva(0x755228),
            rva(0x75522C),
            rva(0x755230),
            rva(0x755234),
            rva(0x755238),
        ],
    );
    s
}

pub fn dispatch_marker(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, marker: Marker) {
    dispatch_draw(
        hook,
        r,
        call_original,
        &|r, canvas| unsafe { draw_marker(marker, r, canvas) },
        &state,
    );
}

// ---- self-test ----

/// Size of the stand-in for the player: the fields read are at +0x110 and +0x11C.
const FAKE_CAR_DWORDS: usize = 0x50;

/// Every combination of edit mode, highlighted item, blink, flag and a few percentages, with
/// the blitter calls swallowed (the caller checks `drawing_unavailable` first). Everything
/// written is put back.
pub fn check_marker(t: &mut SelfTest, call_original: CallOriginal, marker: Marker) {
    let (item, value_rvas): (u32, Vec<usize>) = match marker {
        Marker::Always { percent_rva, .. } => (0x12, vec![percent_rva]),
        Marker::Lower {
            item,
            chosen_rva,
            installed_rva,
            ..
        } => (item, vec![chosen_rva, installed_rva]),
        Marker::Upper { item, .. } => (item, vec![]),
    };
    let saved_bar = Bar::read();
    let saved_values: Vec<u32> = value_rvas
        .iter()
        .map(|&at| unsafe { game_read(at) })
        .collect();
    let saved_mode: [u32; 3] = unsafe {
        [
            game_read(EDIT_MODE_RVA),
            game_read(MENU_ITEM_RVA),
            game_read(BLINK_RVA),
        ]
    };
    let flag = repair_flag_addr(item) as *mut u32;
    let saved_flag = unsafe { flag.read_unaligned() };

    let mut fake_car = [0u32; FAKE_CAR_DWORDS];
    let mut before = Regs::SENTINEL;
    before.edi = fake_car.as_mut_ptr() as u32;

    // The second value only matters to the lower markers: equal to the first, or not.
    for (percent, other) in [
        (0i32, 0i32),
        (-7, -7),
        (1, 2),
        (50, 50),
        (100, 50),
        (130, 130),
    ] {
        for edit in [0u32, 1, 2] {
            for selected in [item, item ^ 1] {
                for blink in [0u32, 1] {
                    for flag_value in [0u32, 1, 2] {
                        if let Marker::Upper { car_offset, .. } = marker {
                            fake_car[car_offset as usize / 4] = percent as u32;
                        }
                        let prepare = || unsafe {
                            game_write(EDIT_MODE_RVA, edit);
                            game_write(MENU_ITEM_RVA, selected);
                            game_write(BLINK_RVA, blink);
                            flag.write_unaligned(flag_value);
                            for (&at, v) in value_rvas.iter().zip([percent, other]) {
                                game_write(at, v);
                            }
                            Bar {
                                x: 0x5EED,
                                y: 0x5EED,
                                width: 0x5EED,
                                fill_glyph: 0x5EED,
                                end_glyph: 0x5EED,
                                percent: 0x5EED,
                            }
                            .write();
                        };
                        check_draw(
                            t,
                            &format!(
                                "percent={percent} other={other} edit={edit} selected={selected:#x} blink={blink} flag={flag_value}"
                            ),
                            call_original,
                            before,
                            &prepare,
                            &|r, canvas| unsafe { draw_marker(marker, r, canvas) },
                            &state,
                        );
                    }
                }
            }
        }
    }

    unsafe {
        game_write(EDIT_MODE_RVA, saved_mode[0]);
        game_write(MENU_ITEM_RVA, saved_mode[1]);
        game_write(BLINK_RVA, saved_mode[2]);
        flag.write_unaligned(saved_flag);
        for (&at, &v) in value_rvas.iter().zip(&saved_values) {
            game_write(at, v);
        }
    }
    saved_bar.write();
}
