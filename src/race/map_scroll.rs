//! The map's scroll: where the views' cars are on the course map and the map picture redrawn
//! around them (`race::scroll_maps` -> `race::scroll_map` -> `race::map_tiles`). A car's block
//! holds its last checkpoint, its place, the map picture's corner (16-pixel steps), a flag set
//! when the picture was redrawn, the car on the map (16.16) and its place in the picture.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::owner::Race;

/// Draws the map picture's slot `Textures::first_textures[198]` (+ `current_car`
/// unless `session`) from tile (corner x >> 4, corner y >> 4).
pub(super) fn map_tiles(
    race: &Race,
    car: &Car,
    car_number: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    let frame = &race.frame;
    // Logic in `rc2k_core::race::map::{first_tile, picture_slot}`.
    use rc2k_core::race::map::{first_tile, picture_slot};
    let one_view = net_race.is_on();
    // The car is read only for both views.
    let slot = picture_slot(
        scene.textures.first_textures()[198],
        one_view,
        if one_view { 0 } else { car_number },
    );
    crate::race::blit_map_tiles::blit_map_tiles(
        frame,
        &race.course.stage.game_map,
        slot,
        first_tile(car.map_corner[0].get()),
        first_tile(car.map_corner[1].get()),
        gfx,
    )
}

/// What `scroll_map` reads through the course: refused before anything.
pub(super) fn scroll_refuses(race: &Race) -> Result<(), Refused> {
    if race.shared.vars.get().checkpoint_count.get() >= 2
        && (!crate::race::course_places::held(&race.course.stage.course)
            || !crate::race::scene_points::held(&race.course.stage.course))
    {
        return Err(BadInput("no course faces or points").into());
    }
    Ok(())
}

/// Puts the car on the map: `map_place` at its last checkpoint's place plus its distance from that
/// checkpoint's face's point (x as it is, z negated, both on the map's scale).
pub(super) fn place_on_map(race: &Race, car: &Car) -> Result<(), Refused> {
    let [x, y] = map_position(race, car)?;
    car.map_place[0].set(x);
    car.map_place[1].set(y);
    Ok(())
}

/// Where the car is on the map (16.16 map pixels, x then y), as `place_on_map` works it out,
/// written nowhere.
pub(super) fn map_position(race: &Race, car: &Car) -> Result<[u32; 2], Refused> {
    use rc2k_core::race::map::on_map;
    let n = car.checkpoint.get();
    let map_x = crate::race::checkpoints_store::map_x_run_on(race.checkpoints(), n as usize);
    let map_y = crate::race::checkpoints_store::map_y_run_on(race.checkpoints(), n as usize);
    let face = crate::race::checkpoints_store::face_run_on(race.checkpoints(), n as usize);
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
    let x = map_x.wrapping_add(on_map(car.position.bits_at(0).wrapping_sub(px)).0);
    let dz = car.position.bits_at(2).wrapping_sub(pz).wrapping_neg();
    let (y_off, _) = on_map(dz);
    Ok([x, map_y.wrapping_add(y_off)])
}

/// With two checkpoints or more, puts the car on the map (`map_x` / `map_y` at its last
/// checkpoint, plus its distance from that checkpoint's face's point x 0x60 / 1000, z negated);
/// unless it is still 0x34 .. 0x4B pixels inside the picture, moves the picture's corner (centred,
/// 16-pixel steps), redraws it (`map_tiles`) and sets the redrawn flag; then sets its place in the
/// picture ((place - corner << 16 - 0x300000) x 2). Refuses (before anything) no course faces or
/// points.
fn scroll_map(
    race: &Race,
    car: &Car,
    car_number: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    scroll_map_with(race, car, || {
        map_tiles(race, car, car_number, scene, gfx, net_race)
    })
}

/// Draws the map picture of the further player `n` (2 or 3) into the player's own texture and
/// makes the texture load again.
fn further_map_tiles(
    race: &Race,
    car: &Car,
    n: usize,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    use rc2k_core::race::map::first_tile;
    let kind = rc2k_core::scene::texture_levels::FURTHER_MAP_TEXTURES[n - 2] as usize;
    let texture = scene.textures.first_textures()[kind];
    if texture == u32::MAX {
        return Err(BadInput("a further player's map texture was not made").into());
    }
    crate::race::blit_map_tiles::blit_map_tiles(
        &race.frame,
        &race.course.stage.game_map,
        texture,
        first_tile(car.map_corner[0].get()),
        first_tile(car.map_corner[1].get()),
        gfx,
    )?;
    crate::scene::forget_slot_texture::forget_slot_texture(
        scene.textures.kinds.borrow().kind_first_slots[kind],
        scene,
    )
}

/// `scroll_map` with the picture's redraw (`redraw`, run once the corner has moved) left to the
/// caller.
fn scroll_map_with(
    race: &Race,
    car: &Car,
    redraw: impl FnOnce() -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::map::{on_map, pixel, well_inside, centred_corner,
    // picture_place}`.
    use rc2k_core::race::map::{centred_corner, picture_place, pixel, well_inside};
    scroll_refuses(race)?;
    car.map_redrawn.set(0);
    if race.shared.vars.get().checkpoint_count.get() < 2 {
        return Ok(());
    }
    let [corner_x, corner_y] = &car.map_corner;
    let [map_x, map_y] = &car.map_place;
    place_on_map(race, car)?;
    let x = pixel(map_x.get());
    let y = pixel(map_y.get());
    let (dx, dy) = (
        x.wrapping_sub(corner_x.get()),
        y.wrapping_sub(corner_y.get()),
    );
    if !(well_inside(dx) && well_inside(dy)) {
        corner_x.set(centred_corner(x));
        corner_y.set(centred_corner(y));
        redraw()?;
        car.map_redrawn.set(1);
    }
    car.map_picture_place[0].set(picture_place(corner_x.get(), map_x.get()));
    car.map_picture_place[1].set(picture_place(corner_y.get(), map_y.get()));
    Ok(())
}

/// Runs `scroll_map` on each view's car, on its block: with `session` the third view car only,
/// else the first and, unless `views` is 1, the second. Refuses (before anything) a view car
/// without its block, or what `scroll_map` refuses.
pub(super) fn scroll_maps(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::map::scrolled_views`.
    use crate::race::frame::view_car;
    let third_only = net_race.is_on();

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
        scroll_map(race, car, n, scene, gfx, net_race)?;
    }
    // A split screen of three or four: the further players' maps, each in a texture of its own.
    if !third_only {
        let players = race.shared.view.get().players.get().min(4) as usize;
        for n in 2..players {
            let car = race.shared.race_car_or(n, "a further player's car block")?;
            scroll_map_with(race, car, || further_map_tiles(race, car, n, scene, gfx))?;
        }
    }
    // Three players with the overview map in the free part: every car's place on it.
    if !third_only && crate::race::overview_map::wanted(race) {
        crate::race::overview_map::update(race, gfx)?;
    } else {
        race.frame.panel.borrow_mut().overview_on = 0;
    }
    Ok(())
}
