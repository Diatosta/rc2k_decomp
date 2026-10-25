//! The map's scroll: where the views' cars are on the course map and the map picture
//! redrawn around them (`sub_45B4E6` -> `sub_45B527` -> `sub_45B663` -> `sub_45B68F`, part
//! R3, IDA 2026-10-18). The work copy `g_car` holds +0x98 the car's last checkpoint, +0x134 /
//! +0x13C its place, +0xAD4 / +0xAD8 the map picture's corner (16-pixel steps), +0xADC 1 when
//! it was redrawn, +0xAE0 / +0xAE4 the car on the map (16.16), +0xAE8 / +0xAEC its place in
//! the picture.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::kind_texture_store::KIND_TEXTURES;

/// `race::map_tiles` (0x45B663): the map picture's slot `dword_5D0E00` (+ `g_currentCar`
/// unless `dword_52A910`) drawn from tile (corner x >> 4, corner y >> 4)
/// (`sub_45B68F` -> `race::blit_map_tiles`, ecx / edx the tile, eax the slot).
pub unsafe fn map_tiles() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::{first_tile, picture_slot}`.
    use rc2k_core::race::map::{first_tile, picture_slot};
    let one_view = crate::net::store::NET.session.get() != 0;
    // The car is read only for both views.
    let slot = picture_slot(
        KIND_TEXTURES.kind_first_textures[198].get(),
        one_view,
        if one_view {
            0
        } else {
            RACE_WORK.current_car.get()
        },
    );
    unsafe {
        crate::race::blit_map_tiles::blit_map_tiles(
            slot,
            first_tile(CAR.map_corner[0].get()),
            first_tile(CAR.map_corner[1].get()),
        )
    }
}

/// What `scroll_map` reads through the course: refused before anything.
unsafe fn scroll_refuses() -> Result<(), Refused> {
    if RACE_VARS.checkpoint_count.get() >= 2
        && (COURSE.course_faces.get() == 0 || COURSE.scene_points.get() == 0)
    {
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
        CAR.map_redrawn.set(0);
        if RACE_VARS.checkpoint_count.get() < 2 {
            return Ok(());
        }
        let [corner_x, corner_y] = &CAR.map_corner;
        let [map_x, map_y] = &CAR.map_place;
        let n = CAR.checkpoint.get();
        map_x.set(
            crate::race::checkpoints_store::nth(
                &crate::race::checkpoints_store::CHECKPOINTS.map_x[0],
                n as usize,
            )
            .get(),
        );
        map_y.set(
            crate::race::checkpoints_store::nth(
                &crate::race::checkpoints_store::CHECKPOINTS.map_y[0],
                n as usize,
            )
            .get(),
        );
        let face = COURSE
            .course_faces
            .get()
            .wrapping_add(crate::race::checkpoints_store::face(n as usize).get() << 6);
        let point = ((face + 4) as *const u32).read_unaligned();
        let p = point
            .wrapping_mul(12)
            .wrapping_add(COURSE.scene_points.get());
        let px = (p as *const u32).read_unaligned();
        let pz = ((p + 8) as *const u32).read_unaligned();
        map_x.set(
            map_x
                .get()
                .wrapping_add(on_map(CAR.position[0].get().wrapping_sub(px)).0),
        );
        let dz = CAR.position[2].get().wrapping_sub(pz).wrapping_neg();
        let (y_off, _) = on_map(dz);
        map_y.set(map_y.get().wrapping_add(y_off));
        let x = pixel(map_x.get());
        let y = pixel(map_y.get());
        let (dx, dy) = (
            x.wrapping_sub(corner_x.get()),
            y.wrapping_sub(corner_y.get()),
        );
        if !(well_inside(dx) && well_inside(dy)) {
            corner_x.set(centred_corner(x));
            corner_y.set(centred_corner(y));
            map_tiles()?;
            CAR.map_redrawn.set(1);
        }
        CAR.map_picture_place[0].set(picture_place(corner_x.get(), map_x.get()));
        CAR.map_picture_place[1].set(picture_place(corner_y.get(), map_y.get()));
        Ok(())
    }
}

/// `race::scroll_maps` (0x45B4E6): `scroll_map` on each view's car in the work copy: with `dword_52A910` the third view car only (`race::load_car_c` /
/// `store_car_c`), else the first and, unless `dword_63DB98` is 1, the second. Refuses
/// (before anything) a view car without its block, or what `scroll_map` refuses. Its one
/// caller, `race::draw_race_scene`, reads no register it leaves.
pub unsafe fn scroll_maps() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::scrolled_views`.
    use crate::race::frame::{load_view_car, store_view_car, view_car};
    unsafe {
        let third_only = crate::net::store::NET.session.get() != 0;
        // The player count is read only without the third view.
        let views = rc2k_core::race::map::scrolled_views(
            third_only,
            if third_only { 0 } else { RACE_VIEW.views.get() },
        );
        for &v in views {
            let car = view_car(v).get() & 7;
            if RACE_WORK.car_block(car as usize) == 0 {
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
