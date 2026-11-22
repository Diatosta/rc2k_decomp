//! The players' notices (the question to retire, "get ready", the split-time messages) in a split
//! screen of three or four players. One or two players see their car's notice on a banner strip
//! (`hud::draw_view_strips`); the further parts have no banner texture, so each part shows its
//! player's notice as text in the race's notice style (`gfx::notice_text`), centred on the part
//! and enlarged with it, blinking as the banner does.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use rc2k_core::race::view::Split;

/// Where the letters start in a part, in 800 x 600 layout units (the one-player question's place:
/// 22 % down).
const TOP: i32 = 132;

/// The block of the car part `view` shows.
fn car_of(race: &Race, view: usize) -> Option<usize> {
    let number = race.shared.view.get().view_number(view).ok()?;
    (number < race.car_blocks().cars.len()).then_some(number)
}

/// Takes the notice the third and fourth players' cars ask for, as the banner's own update does for
/// the first two (`race::update_name_plates`): the asked notice becomes the shown one, so the next
/// one in the car's queue can follow.
pub(super) fn take_asked(race: &Race) {
    let views = race.shared.view.get().views.get();
    if views <= 2 {
        return;
    }
    for view in 2..views.min(4) as usize {
        let Some(car) = car_of(race, view) else {
            continue;
        };
        let asked = race.car_blocks().cars[car].notice_id;
        if asked == 0 {
            continue;
        }
        let mut blocks = race.car_blocks_mut();
        let car = &mut blocks.cars[car];
        car.notice_last = asked;
        car.notice_id = 0;
        car.notice_counter = 1;
        car.notice_shown = 1;
    }
}

/// Draws each part's notice, while its flash is on, over the frame (the surface locked).
pub fn draw(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    let views = race.shared.view.get().views.get();
    if views <= 2 || race.shared.replay_file().hold_car_revs.get() != 0 {
        return Ok(());
    }
    let Some(split) = Split::many(views, crate::race::split_layout::index()) else {
        return Ok(());
    };
    let screen = o.system.session.screen.get();
    let size = (screen.width.get(), screen.height.get());
    for view in 0..views.min(4) {
        let Some(part) = split.part_of(views, view, size) else {
            continue;
        };
        let Some(car) = car_of(race, view as usize) else {
            continue;
        };
        let blocks = race.car_blocks();
        let car = &blocks.cars[car];
        let shown = rc2k_core::hud::strips::strip(
            car.view_covered,
            car.notice_shown,
            car.notice_time,
            car.notice_time_b,
        )
        .shown();
        let id = car.notice_last;
        if !shown || id == 0 {
            continue;
        }
        let mut text = Vec::new();
        if crate::strings::append_string_by_id::push_string(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &mut text,
            id,
        )
        .is_err()
        {
            continue;
        }
        crate::gfx::notice_text::draw_in(
            o,
            &text,
            [
                part.x as i32,
                part.y as i32,
                part.width as i32,
                part.height as i32,
            ],
            TOP,
            255,
        );
    }
    Ok(())
}
