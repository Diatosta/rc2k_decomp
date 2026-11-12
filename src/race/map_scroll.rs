//! The map's scroll: where the views' cars are on the course map and the map picture
//! redrawn around them (`race::scroll_maps` -> `race::scroll_map` -> `race::map_tiles` -> 0x45B68F,
//! part R3). A car's block (the original's work copy `g_car`) holds +0x98 the car's last checkpoint, +0x134 /
//! +0x13C its place, +0xAD4 / +0xAD8 the map picture's corner (16-pixel steps), +0xADC 1 when
//! it was redrawn, +0xAE0 / +0xAE4 the car on the map (16.16), +0xAE8 / +0xAEC its place in
//! the picture.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::owner::Race;

/// `race::map_tiles` (0x45B663): the map picture's slot `KindTextureTables.kind_first_textures[198]` (+ `g_currentCar` unless
/// `session`) drawn from tile (corner x >> 4, corner y >> 4).
pub fn map_tiles(
    race: &Race,
    car: &Car,
    car_number: u32,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let frame = &race.frame;
    // Converted: `rc2k_core::race::map::{first_tile, picture_slot}`.
    use rc2k_core::race::map::{first_tile, picture_slot};
    let one_view = net.vars.session.get() != 0;
    // The car is read only for both views.
    let slot = picture_slot(
        scene.textures.kinds.get().kind_first_textures[198].get(),
        one_view,
        if one_view { 0 } else { car_number },
    );
    crate::race::blit_map_tiles::blit_map_tiles(
        frame,
        slot,
        first_tile(car.map_corner[0].get()),
        first_tile(car.map_corner[1].get()),
        gfx,
    )
}

/// What `scroll_map` reads through the course: refused before anything.
fn scroll_refuses(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    if race.shared.vars.checkpoint_count.get() >= 2
        && (course.course.course_faces.get() == 0 || course.course.scene_points.get() == 0)
    {
        return Err(BadInput("no course faces or points").into());
    }
    Ok(())
}

/// `race::scroll_map` (0x45B527): with two checkpoints or more, the car on the map (`g_mapX` /
/// `g_mapY` at its last checkpoint, plus its distance from that checkpoint's face's point x 0x60 /
/// 1000, z negated); unless it is still 0x34 .. 0x4B pixels inside the picture, the picture's
/// corner moved (centred, 16-pixel steps) and redrawn (`map_tiles`), +0xADC 1; its place in the
/// picture ((place - corner << 16 - 0x300000) x 2). Refuses (before anything) no course faces or
/// points.
pub fn scroll_map(
    race: &Race,
    car: &Car,
    car_number: u32,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::{on_map, pixel, well_inside, centred_corner,
    // picture_place}`.
    use rc2k_core::race::map::{centred_corner, on_map, picture_place, pixel, well_inside};
    scroll_refuses(race)?;
    car.map_redrawn.set(0);
    if race.shared.vars.checkpoint_count.get() < 2 {
        return Ok(());
    }
    let [corner_x, corner_y] = &car.map_corner;
    let [map_x, map_y] = &car.map_place;
    let n = car.checkpoint.get();
    map_x.set(crate::race::checkpoints_store::map_x(race.checkpoints(), n as usize).get());
    map_y.set(crate::race::checkpoints_store::map_y(race.checkpoints(), n as usize).get());
    let face = crate::race::checkpoints_store::face(race.checkpoints(), n as usize).get();
    let point = crate::race::course_places::face_number(&race.course, face)?.point(0);
    let px = crate::race::scene_points::point_word(
        &race.course,
        &crate::race::scene_points::NO_OWN,
        point,
        0,
    )?;
    let pz = crate::race::scene_points::point_word(
        &race.course,
        &crate::race::scene_points::NO_OWN,
        point,
        2,
    )?;
    map_x.set(
        map_x
            .get()
            .wrapping_add(on_map(car.position[0].get().wrapping_sub(px)).0),
    );
    let dz = car.position[2].get().wrapping_sub(pz).wrapping_neg();
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
        map_tiles(race, car, car_number, scene, net, gfx)?;
        car.map_redrawn.set(1);
    }
    car.map_picture_place[0].set(picture_place(corner_x.get(), map_x.get()));
    car.map_picture_place[1].set(picture_place(corner_y.get(), map_y.get()));
    Ok(())
}

/// `race::scroll_maps` (0x45B4E6): `scroll_map` on each view's car, on its block (the original
/// loaded it into the work copy and stored it back whole): with `session` the third view car
/// only (`race::load_car_c` / `store_car_c`), else the first and, unless `views` is 1, the
/// second. Refuses (before anything) a view car without its block, or what `scroll_map` refuses.
pub fn scroll_maps(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::map::scrolled_views`.
    use crate::race::frame::view_car;
    let third_only = net.vars.session.get() != 0;
    // The player count is read only without the third view.
    let views = rc2k_core::race::map::scrolled_views(
        third_only,
        if third_only {
            0
        } else {
            race.shared.view.views.get()
        },
    );
    for &v in views {
        let car = view_car(&race.shared, v).get() & 7;
        if !race.shared.work.has_car_block(car as usize) {
            return Err(BadInput("a view car without its block").into());
        }
    }
    scroll_refuses(race)?;
    for &v in views {
        let n = view_car(&race.shared, v).get();
        let car = race
            .shared
            .race_car_or((n & 7) as usize, "a view car's block")?;
        scroll_map(race, car, n, scene, net, gfx)?;
    }
    Ok(())
}
