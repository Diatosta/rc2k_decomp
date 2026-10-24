//! The map's scroll: where the views' cars are on the course map and the map picture
//! redrawn around them (`sub_45B4E6` -> `sub_45B527` -> `sub_45B663` -> `sub_45B68F`, part
//! R3, IDA 2026-10-18). The work copy `g_car` holds +0x98 the car's last checkpoint, +0x134 /
//! +0x13C its place, +0xAD4 / +0xAD8 the map picture's corner (16-pixel steps), +0xADC 1 when
//! it was redrawn, +0xAE0 / +0xAE4 the car on the map (16.16), +0xAE8 / +0xAEC its place in
//! the picture.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

const CAR: u32 = 0x719FD8; // g_car
const CORNER_X: u32 = CAR + 0xAD4;
const CORNER_Y: u32 = CAR + 0xAD8;
const REDRAWN: u32 = CAR + 0xADC;
const MAP_X: u32 = CAR + 0xAE0;
const MAP_Y: u32 = CAR + 0xAE4;

/// `race::map_tiles` (0x45B663): the map picture's slot `dword_5D0E00` (+ `g_currentCar`
/// unless `dword_52A910`) drawn from tile (corner x >> 4, corner y >> 4)
/// (`sub_45B68F` -> `race::blit_map_tiles`, ecx / edx the tile, eax the slot).
pub unsafe fn map_tiles() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::{first_tile, picture_slot}`.
    use rc2k_core::race::map::{first_tile, picture_slot};
    let one_view = k(0x52A910) != 0;
    // The car is read only for both views.
    let slot = picture_slot(
        k(0x5D0E00),
        one_view,
        if one_view { 0 } else { k(0x71B3F8) },
    );
    unsafe {
        crate::race::blit_map_tiles::blit_map_tiles(
            slot,
            first_tile(k(CORNER_X)),
            first_tile(k(CORNER_Y)),
        )
    }
}

/// What `scroll_map` reads through the course: refused before anything.
unsafe fn scroll_refuses() -> Result<(), Refused> {
    if k(0x623640) >= 2 && (k(0x5CCCBC) == 0 || k(0x5CCCC0) == 0) {
        return Err(BadInput("no course faces or points").into());
    }
    Ok(())
}

/// `race::scroll_map` (0x45B527): with two checkpoints or more, the car on the map
/// (`g_mapX` / `g_mapY` at its last checkpoint, plus its distance from that checkpoint's
/// face's point x 0x60 / 1000, z negated); unless it is still 0x34 .. 0x4B pixels inside the
/// picture, the picture's corner moved (centred, 16-pixel steps) and redrawn
/// (`map_tiles`), +0xADC 1; its place in the picture ((place - corner << 16 - 0x300000) x 2).
/// Refuses (before anything) no course faces or points.
pub unsafe fn scroll_map() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::{on_map, pixel, well_inside, centred_corner,
    // picture_place}`.
    use rc2k_core::race::map::{centred_corner, on_map, picture_place, pixel, well_inside};
    unsafe {
        scroll_refuses()?;
        w(REDRAWN, 0);
        if k(0x623640) < 2 {
            return Ok(());
        }
        let n = k(CAR + 0x98);
        w(MAP_X, k(0x9FC940 + 4 * n));
        w(MAP_Y, k(0xA019A8 + 4 * n));
        let face = k(0x5CCCBC).wrapping_add(k(0x9E3738 + 4 * n) << 6);
        let point = ((face + 4) as *const u32).read_unaligned();
        let p = point.wrapping_mul(12).wrapping_add(k(0x5CCCC0));
        let px = (p as *const u32).read_unaligned();
        let pz = ((p + 8) as *const u32).read_unaligned();
        w(
            MAP_X,
            k(MAP_X).wrapping_add(on_map(k(CAR + 0x134).wrapping_sub(px)).0),
        );
        let dz = k(CAR + 0x13C).wrapping_sub(pz).wrapping_neg();
        let (y_off, _) = on_map(dz);
        w(MAP_Y, k(MAP_Y).wrapping_add(y_off));
        let x = pixel(k(MAP_X));
        let y = pixel(k(MAP_Y));
        let (dx, dy) = (x.wrapping_sub(k(CORNER_X)), y.wrapping_sub(k(CORNER_Y)));
        if !(well_inside(dx) && well_inside(dy)) {
            w(CORNER_X, centred_corner(x));
            w(CORNER_Y, centred_corner(y));
            map_tiles()?;
            w(REDRAWN, 1);
        }
        w(CAR + 0xAE8, picture_place(k(CORNER_X), k(MAP_X)));
        w(CAR + 0xAEC, picture_place(k(CORNER_Y), k(MAP_Y)));
        Ok(())
    }
}

/// `race::scroll_maps` (0x45B4E6): `scroll_map` on each view's car in the work copy: with `dword_52A910` the third view car only (`race::load_car_c` /
/// `store_car_c`), else the first and, unless `dword_63DB98` is 1, the second. Refuses
/// (before anything) a view car without its block, or what `scroll_map` refuses. Its one
/// caller, `race::draw_race_scene`, reads no register it leaves.
pub unsafe fn scroll_maps() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::scrolled_views`.
    use crate::race::frame::{VIEW_CARS, load_view_car, store_view_car};
    unsafe {
        let third_only = k(0x52A910) != 0;
        // The player count is read only without the third view.
        let views = rc2k_core::race::map::scrolled_views(
            third_only,
            if third_only { 0 } else { k(0x63DB98) },
        );
        for &v in views {
            let car = game_read::<u32>(VIEW_CARS[v]) & 7;
            if k(0x71B3B8 + 4 * car) == 0 {
                return Err(BadInput("a view car without its block").into());
            }
        }
        scroll_refuses()?;
        for &v in views {
            load_view_car(v)?;
            scroll_map()?;
            store_view_car(v)?;
        }
        Ok(())
    }
}
