//! The overview map of a three-player split screen in the quarters layout: the part no player has
//! shows the whole course's map and every car on it. The picture is the course's map tiles fitted
//! into 128 x 128 colours (`rc2k_core::race::overview`), worked out once a stage and drawn by
//! the HUD as runs of one colour (`hud::layout_many`); the cars' places on the map and the fit go
//! to the panel.

use crate::errors::Refused;
use crate::race::game_map_store::GameMap;
use crate::race::game_map_store::Tile;
use crate::race::owner::Race;
use rc2k_core::race::map::{TILE, TILES, pixel_width};
use rc2k_core::race::overview::{Fit, SIDE};

/// The players the overview map serves.
const PLAYERS: u32 = 3;

/// Whether the race is three players on a screen whose layout leaves a part free for the map.
pub fn wanted(race: &Race) -> bool {
    let players = race.shared.view.get().players.get();
    players == PLAYERS
        && rc2k_core::race::view::Split::many(players, crate::race::split_layout::index())
            .is_some_and(|split| split.free_part(players, (SIDE, SIDE)).is_some())
}

/// The overview picture's colours, if it is made: `f` gets them row by row.
pub fn with_picture<R>(
    frame: &crate::race::owner_frame::Frame,
    f: impl FnOnce(&[Option<u32>]) -> R,
) -> Option<R> {
    let kept = frame
        .overview_picture
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    kept.as_deref().map(f)
}

/// The pixel of the map at (`x`, `y`) in the tiles' bytes (`wide` bytes a pixel), `None` for a
/// tile without pixels or a black pixel.
fn map_pixel(map: &GameMap, wide: usize, black: u32, (x, y): (u32, u32)) -> Option<u32> {
    let (tx, ty) = (x / TILE, y / TILE);
    if tx >= TILES || ty >= TILES {
        return None;
    }
    let Some(Tile::At(at)) = map.tiles.get((ty * TILES + tx) as usize) else {
        return None;
    };
    let from = at + (((y % TILE) * TILE + x % TILE) as usize) * wide;
    let bytes = map.pixels.get(from..from + wide)?;
    let value = if wide == 2 {
        u32::from(u16::from_le_bytes([bytes[0], bytes[1]]))
    } else {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    };
    (value != black).then_some(value)
}

/// Puts every car's place on the map into the panel and, the first time in a stage, makes the
/// overview picture; with two checkpoints at least.
pub fn update(race: &Race, gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let panel = &race.frame.panel_and_overlay.get().panel;
    crate::race::map_scroll::scroll_refuses(race)?;
    if race.shared.vars.get().checkpoint_count.get() < 2 {
        panel.overview_on.set(0);
        return Ok(());
    }
    for (n, cells) in panel
        .overview_cars
        .iter()
        .enumerate()
        .take(PLAYERS as usize)
    {
        let car = race
            .shared
            .race_car_or(n, "an overview map's car without its block")?;
        crate::race::map_scroll::place_on_map(race, car)?;
        for (cell, place) in cells.iter().zip(&car.map_place) {
            cell.set(place.get() as i32 as f32 / 65536.0);
        }
    }
    if panel.overview_drawn.get() == 0 {
        let wide = pixel_width(race.frame.map().pixel_bytes.get()) as usize;
        let black = race.frame.map().black.get();
        let (_, _, format) = crate::race::game_map::take_format(gfx)?;
        // The box's pixels averaged, each colour on its own.
        let blend = |pixels: &[u32]| -> u32 {
            let mut sum = [0u32; 4];
            for &pixel in pixels {
                let components =
                    crate::gfx::pixel_to_components::pixel_to_components(&format, pixel);
                for (total, byte) in sum.iter_mut().zip(components.to_le_bytes()) {
                    *total += u32::from(byte);
                }
            }
            let count = pixels.len() as u32;
            let [b, g, r, _] = sum.map(|total| total / count);
            r << 16 | g << 8 | b
        };
        let made = race.course.stage.game_map.with(|map| {
            let used = |x: u32, y: u32| {
                matches!(map.tiles.get((y * TILES + x) as usize), Some(Tile::At(_)))
            };
            Fit::of(used).map(|fit| {
                (
                    fit,
                    fit.paint(|x, y| map_pixel(map, wide, black, (x, y)), blend),
                )
            })
        })?;
        let mut kept = race
            .frame
            .overview_picture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match made {
            Some((fit, picture)) => {
                panel.overview_fit[0].set(fit.origin.0);
                panel.overview_fit[1].set(fit.origin.1);
                panel.overview_fit[2].set(fit.ratio);
                *kept = Some(picture);
            }
            None => {
                panel.overview_fit[2].set(0.0);
                *kept = None;
            }
        }
        panel.overview_drawn.set(1);
    }
    panel
        .overview_on
        .set(u32::from(panel.overview_fit[2].get() > 0.0));
    Ok(())
}
