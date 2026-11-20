//! The start menus' items that begin an event for one to four players (`ItemFn::StartEvent`).
//!
//! The families differ only in the seats (`starters::Seats`): one player, two on a split
//! screen, or two, three, four in turn; the single rally also has three and four players
//! racing together on one split screen. The single rally and the time trial run their own
//! event flows in `rc2k_core::menu::rounds`, one per seats (event modes: single rally 0x18
//! for one player, 5 split, 8 in turn; time trial 0x14 one, 4 split, 0x15 .. 0x17 in turn;
//! the larger split screen is event mode 5 too, `rounds::single_rally_split_many`);
//! the championships (`Championship::First` and `A8`) share `menu::start_event`'s shape
//! (`starters::Starter`). Each refuses a missing player before anything, and what its callees
//! refuse when they are reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use crate::menu::start_event::{Starter, start_event};
use crate::owners::Owners;
use rc2k_core::menu::rounds;
use rc2k_core::menu::starters::{self, Championship, Seats};
use rc2k_core::menu::tables::EventStart;

/// The split screens of three and four players are the single rally's own items.
fn split_screen_only_in_single_rally() -> Refused {
    crate::errors::BadInput("a split screen of three or four players outside the single rally")
        .into()
}

/// Begins `event` for the players `seats` stand for.
pub(super) fn start_event_item(
    o: Owners<'static>,
    event: EventStart,
    seats: Seats,
) -> Result<(), Refused> {
    match event {
        EventStart::SingleRally => match seats {
            Seats::One => Game::new(o).run(rounds::single_rally),
            Seats::Split => Game::new(o).run(rounds::single_rally_split),
            Seats::Two => Game::new(o).run(rounds::single_rally_two),
            Seats::SplitThree => Game::new(o).run(|game| rounds::single_rally_split_many(game, 3)),
            Seats::SplitFour => Game::new(o).run(|game| rounds::single_rally_split_many(game, 4)),
            Seats::Three => Game::new(o).run(rounds::single_rally_three),
            Seats::Four => Game::new(o).run(rounds::single_rally_four),
        },
        EventStart::TimeTrial => match seats {
            Seats::One => Game::new(o).run(rounds::time_trial_one),
            Seats::Split => Game::new(o).run(rounds::time_trial_split),
            Seats::Two => Game::new(o).run(rounds::time_trial_two),
            Seats::Three => Game::new(o).run(rounds::time_trial_three),
            Seats::Four => Game::new(o).run(rounds::time_trial_four),
            Seats::SplitThree | Seats::SplitFour => Err(split_screen_only_in_single_rally()),
        },
        EventStart::Championship => championship(o, Championship::First, seats),
        EventStart::A8 => championship(o, Championship::A8, seats),
    }
}

fn championship(
    o: Owners<'static>,
    championship: Championship,
    seats: Seats,
) -> Result<(), Refused> {
    start_event(
        o,
        &Starter {
            kind: starters::Starter {
                championship,
                seats,
            },
        },
    )
}
