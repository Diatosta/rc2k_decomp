//! The car list the network game's car screen shows: the list with the secret cars when any of
//! the cars won is in the secret list, else the one without. Each won car is compared with the
//! secret list's first entry before its end is looked for, so an empty secret list's -1 end can
//! match a -1 entry (kept). Lists with no end are refused.

use crate::errors::Refused;
use crate::menu::car_lists_store::CarLists;
use crate::menu::lists_store::CarList;

/// Whether one of the cars in `won` is in the list `secret`, each read to its end mark.
/// Both lists end inside their fields: the won cars (`secret_cars`, 12 entries) are the
/// image's, with the -1 at entry 11, and nothing writes them; the second car list (59
/// entries) is written only by `menu::remove_secret_cars`, `menu::apply_cheats`,
/// `menu::arcade_steps` and `saves::rebuild_car_lists`, each within its first 0x28 entries
/// with a -1 inside them, and the entries from 0x28 on keep their -1. An entry past the
/// field is refused.
pub fn any_won(
    secret: &[impl crate::memory::session::Cell32],
    won: &[impl crate::memory::session::Cell32],
) -> Result<bool, Refused> {
    // The logic is in `rc2k_core::net::cars::any_won`, the lists' dwords by index.
    // The core asks infallibly: the first refusal is kept and raised after.
    let refused = std::cell::Cell::new(None);
    fn reader<'a>(
        list: &'a [impl crate::memory::session::Cell32],
        refused: &'a std::cell::Cell<Option<crate::errors::BadInput>>,
    ) -> impl Fn(u32) -> u32 + 'a {
        move |n: u32| match list.get(n as usize) {
            Some(entry) => entry.bits(),
            None => {
                refused.set(Some(crate::errors::BadInput(
                    "a car list's entry past its field",
                )));
                0
            }
        }
    }
    let answer = rc2k_core::net::cars::any_won(reader(secret, &refused), reader(won, &refused));
    if let Some(e) = refused.take() {
        return Err(e.into());
    }
    Ok(answer?)
}
pub fn car_list(lists: &CarLists) -> Result<CarList, Refused> {
    Ok(if any_won(&lists.cars_second, &lists.secret_cars)? {
        CarList::WithClio
    } else {
        CarList::First
    })
}
