//! The map's scroll: where the views' cars are on the course map and the map picture redrawn
//! around them (`race::scroll_maps` -> `race::scroll_map` -> `race::map_tiles`). A car's block
//! holds its last checkpoint, its place, the map picture's corner (16-pixel steps), a flag set
//! when the picture was redrawn, the car on the map (16.16) and its place in the picture.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::owner::Race;

/// Draws the map picture's slot `KindTextureTables.kind_first_textures[198]` (+ `current_car`
/// unless `session`) from tile (corner x >> 4, corner y >> 4).
pub fn map_tiles(
    race: &Race,
    car: &Car,
    car_number: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let frame = &race.frame;
    // Logic in `rc2k_core::race::map::{first_tile, picture_slot}`.
    use rc2k_core::race::map::{first_tile, picture_slot};
    let one_view = race.shared.net_race.is_on();
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
    if race.shared.vars.get().checkpoint_count.get() >= 2
        && (course.course.get().course_faces.get() == 0
            || course.course.get().scene_points.get() == 0)
    {
        return Err(BadInput("no course faces or points").into());
    }
    Ok(())
}

/// With two checkpoints or more, puts the car on the map (`map_x` / `map_y` at its last
/// checkpoint, plus its distance from that checkpoint's face's point x 0x60 / 1000, z negated);
/// unless it is still 0x34 .. 0x4B pixels inside the picture, moves the picture's corner (centred,
/// 16-pixel steps), redraws it (`map_tiles`) and sets the redrawn flag; then sets its place in the
/// picture ((place - corner << 16 - 0x300000) x 2). Refuses (before anything) no course faces or
/// points.
pub fn scroll_map(
    race: &Race,
    car: &Car,
    car_number: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::map::{on_map, pixel, well_inside, centred_corner,
    // picture_place}`.
    use rc2k_core::race::map::{centred_corner, on_map, picture_place, pixel, well_inside};
    scroll_refuses(race)?;
    car.map_redrawn.set(0);
    if race.shared.vars.get().checkpoint_count.get() < 2 {
        return Ok(());
    }
    let [corner_x, corner_y] = &car.map_corner;
    let [map_x, map_y] = &car.map_place;
    let n = car.checkpoint.get();
    map_x.set(crate::race::checkpoints_store::map_x_run_on(race.checkpoints(), n as usize).get());
    map_y.set(crate::race::checkpoints_store::map_y_run_on(race.checkpoints(), n as usize).get());
    let face = crate::race::checkpoints_store::face_run_on(race.checkpoints(), n as usize).get();
    let point = crate::race::course_places::face_number(race.course.reads(), face)?.point(0);
    let px = crate::race::scene_points::point_word(
        race.course.reads(),
        &crate::race::scene_points::no_own(),
        point,
        0,
    )?;
    let pz = crate::race::scene_points::point_word(
        race.course.reads(),
        &crate::race::scene_points::no_own(),
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
        map_tiles(race, car, car_number, scene, gfx)?;
        car.map_redrawn.set(1);
    }
    car.map_picture_place[0].set(picture_place(corner_x.get(), map_x.get()));
    car.map_picture_place[1].set(picture_place(corner_y.get(), map_y.get()));
    Ok(())
}

/// Runs `scroll_map` on each view's car, on its block: with `session` the third view car only,
/// else the first and, unless `views` is 1, the second. Refuses (before anything) a view car
/// without its block, or what `scroll_map` refuses.
pub fn scroll_maps(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::map::scrolled_views`.
    use crate::race::frame::view_car;
    let third_only = race.shared.net_race.is_on();
    // The player count is read only without the third view.
    let views = rc2k_core::race::map::scrolled_views(
        third_only,
        if third_only {
            0
        } else {
            race.shared.view.get().views.get()
        },
    );
    for &v in views {
        let car = view_car(race.shared.view.get(), v).get() & 7;
        if !race.shared.work.get().has_car_block(car as usize) {
            return Err(BadInput("a view car without its block").into());
        }
    }
    scroll_refuses(race)?;
    for &v in views {
        let n = view_car(race.shared.view.get(), v).get();
        let car = race
            .shared
            .race_car_or((n & 7) as usize, "a view car's block")?;
        scroll_map(race, car, n, scene, gfx)?;
    }
    Ok(())
}
