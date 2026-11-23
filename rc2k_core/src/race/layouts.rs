//! What the layout byte of a replay block shows: the one rule the viewer's car item, the drawing
//! of the cars shown, the camera choice and the HUD ask. The byte names players, by their number
//! in the order of the replay's players (the first player is 0):
//!
//! - 0 and 1: the first and the second player alone; 4 .. 9: the third to the eighth alone;
//! - 2 and 3: the first two players together, side by side or one above the other, when they are
//!   the only two (a replay of two players);
//! - 10: the first group of players, split as a local split-screen race is: up to four, each in
//!   a part of the screen (for a three- or four-player replay, every player); from three players
//!   on it is the only way the first players are shown together;
//! - 11 and 12: the players after the first four, split the same way: two of them (a replay of
//!   six) are a pair, as the first two are, 11 side by side and 12 one above the other; three or
//!   four of them (seven or eight players) are a group, 11 only.

/// The layout of the first group of players.
pub const FIRST_GROUP: u8 = 10;
/// The layout of the players after the first four: a pair side by side when they are two, else
/// a group.
pub const SECOND_GROUP: u8 = 11;
/// The layout of those players as a pair one above the other (when they are two).
pub const SECOND_PAIR_STACKED: u8 = 12;
/// The last layout.
pub const LAST_LAYOUT: u8 = SECOND_PAIR_STACKED;
/// How many players a group splits the screen between at most.
pub const GROUP_MOST: usize = 4;
/// The fewest players a group splits the screen between (two players are a pair).
pub const GROUP_LEAST: usize = 3;
/// How many views a layout has at most, so how many camera buttons the panel offers.
pub const VIEWS_MOST: usize = GROUP_MOST;
/// How many players a pair shows.
pub const PAIR: usize = 2;
/// The camera byte of a block not edited in the viewer, for every car: a race's recording writes
/// the head of each block as zeros (the layout byte aside), so every car starts on the camera the
/// byte 0 picks. A car whose camera no block holds (`CameraSource::Live`) starts on it too.
pub const UNEDITED_CAMERA: u8 = 0;

/// The players a layout shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// One player alone.
    Player(usize),
    /// Two players from player `first`, in two views.
    Pair { first: usize, stacked: bool },
    /// `count` players from player `first`, each in a part of the screen.
    Group { first: usize, count: usize },
}

/// What `layout` shows of a replay of `players` players; none when the layout is none of the
/// ones this many players have.
pub fn shown(layout: u8, players: usize) -> Option<Shown> {
    let player = |k: usize| (k < players).then_some(Shown::Player(k));
    // The players after the first four.
    let rest = players.saturating_sub(GROUP_MOST);
    match layout {
        0 | 1 => player(usize::from(layout)),
        2 | 3 if players == PAIR => Some(Shown::Pair {
            first: 0,
            stacked: layout == 3,
        }),
        4..=9 => player(usize::from(layout) - 2),
        FIRST_GROUP if players >= GROUP_LEAST => Some(Shown::Group {
            first: 0,
            count: players.min(GROUP_MOST),
        }),
        SECOND_GROUP | SECOND_PAIR_STACKED if rest == PAIR => Some(Shown::Pair {
            first: GROUP_MOST,
            stacked: layout == SECOND_PAIR_STACKED,
        }),
        SECOND_GROUP if rest >= GROUP_LEAST => Some(Shown::Group {
            first: GROUP_MOST,
            count: rest,
        }),
        _ => None,
    }
}

/// Whether a stage may show more than two parts of the screen at once, so that it needs each
/// part's own map picture and car icon: a race of more than two players (`players`), or a viewed
/// replay that offers a group (`viewed_group`, the players of its first group: 0 for none).
pub fn splits_past_two(players: u32, viewed_group: u32) -> bool {
    players as usize > PAIR || viewed_group as usize >= GROUP_LEAST
}

/// What `layout` shows of a replay of the first format: no groups, the first two players
/// together in 2 and 3 (whatever the number of cars), and with the extra tape the third and the
/// fourth car alone in 4 and 5; none for any other byte.
pub fn first_format_shown(layout: u8, extra_tape: bool) -> Option<Shown> {
    match layout {
        0 | 1 => Some(Shown::Player(usize::from(layout))),
        2 | 3 => Some(Shown::Pair {
            first: 0,
            stacked: layout == 3,
        }),
        4 | 5 if extra_tape => Some(Shown::Player(usize::from(layout) - 2)),
        _ => None,
    }
}

impl Shown {
    /// The player each camera button edits (button 0 is the first camera's, on the panel's left,
    /// button 1 the second's, on its right, buttons 2 and 3 the further parts'); none for a
    /// button the layout has no part for. A player alone has the first two buttons, as the
    /// first format's layouts always had: the first player's and the second's own cameras for
    /// the first two players, whatever one is shown; for a further player both buttons edit
    /// that player's camera (there is the one).
    pub fn camera_players(self) -> [Option<usize>; VIEWS_MOST] {
        let mut buttons = [None; VIEWS_MOST];
        match self {
            Shown::Player(player) if player < 2 => {
                buttons[0] = Some(0);
                buttons[1] = Some(1);
            }
            Shown::Player(player) => {
                buttons[0] = Some(player);
                buttons[1] = Some(player);
            }
            Shown::Pair { first, .. } => {
                buttons[0] = Some(first);
                buttons[1] = Some(first + 1);
            }
            Shown::Group { first, count } => {
                for (k, button) in buttons.iter_mut().enumerate().take(count) {
                    *button = Some(first + k);
                }
            }
        }
        buttons
    }

    /// How many camera buttons the layout has.
    pub fn camera_buttons(self) -> usize {
        self.camera_players().iter().flatten().count()
    }
}

/// The layout that shows player `k` alone.
pub fn layout_of_player(k: usize) -> u8 {
    if k < 2 { k as u8 } else { k as u8 + 2 }
}

/// Where the camera of a car is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraSource {
    /// Byte `n` of the block's head: 0 and 1 the first two cars', 6 and 7 the third's and the
    /// fourth's.
    Head(usize),
    /// The choice made in the viewer for car `4 + n`: the blocks hold no byte for it.
    Live(usize),
}

/// Where car `car`'s camera is kept; none for a car past the eighth.
pub fn camera_source(car: usize) -> Option<CameraSource> {
    match car {
        0 | 1 => Some(CameraSource::Head(car)),
        2 | 3 => Some(CameraSource::Head(car + 4)),
        4..=7 => Some(CameraSource::Live(car - 4)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(first: usize, stacked: bool) -> Option<Shown> {
        Some(Shown::Pair { first, stacked })
    }

    fn group(first: usize, count: usize) -> Option<Shown> {
        Some(Shown::Group { first, count })
    }

    #[test]
    fn single_players_pairs_and_groups() {
        // One or two players: the players alone, and for two the pair.
        assert_eq!(shown(1, 1), None);
        assert_eq!(shown(0, 1), Some(Shown::Player(0)));
        assert_eq!(shown(2, 2), pair(0, false));
        assert_eq!(shown(3, 2), pair(0, true));
        assert_eq!(shown(FIRST_GROUP, 2), None);
        // Three and four: every player alone and one group of all of them; no pair of the first
        // two.
        assert_eq!(shown(2, 3), None);
        assert_eq!(shown(3, 4), None);
        assert_eq!(shown(FIRST_GROUP, 3), group(0, 3));
        assert_eq!(shown(FIRST_GROUP, 4), group(0, 4));
        assert_eq!(shown(SECOND_GROUP, 4), None);
        assert_eq!(shown(5, 4), Some(Shown::Player(3)));
        assert_eq!(shown(6, 4), None);
        assert_eq!(shown(LAST_LAYOUT + 1, 8), None);
    }

    #[test]
    fn the_players_after_the_first_four_are_split_as_many_as_they_are() {
        let second =
            |players| [SECOND_GROUP, SECOND_PAIR_STACKED].map(|layout| shown(layout, players));
        // Five players: the fifth has a layout of their own only, nothing to split with.
        assert_eq!(second(5), [None, None]);
        assert_eq!(shown(FIRST_GROUP, 5), group(0, 4));
        assert_eq!(shown(2, 5), None);
        assert_eq!(shown(6, 5), Some(Shown::Player(4)));
        assert_eq!(shown(7, 5), None);
        // Six: the fifth and the sixth as a pair, side by side or one above the other; the first
        // four only as a group.
        assert_eq!(second(6), [pair(4, false), pair(4, true)]);
        assert_eq!([2, 3].map(|layout| shown(layout, 6)), [None, None]);
        // Seven and eight: a group of three or four, with no pair.
        assert_eq!(second(7), [group(4, 3), None]);
        assert_eq!(second(8), [group(4, 4), None]);
        assert_eq!(shown(9, 8), Some(Shown::Player(7)));
        assert_eq!(shown(FIRST_GROUP, 8), group(0, 4));
    }

    #[test]
    fn a_split_past_two_parts_by_race_or_replay() {
        assert!(!splits_past_two(2, 0));
        assert!(splits_past_two(3, 0));
        // A network race's replay (one player at its machine) with a group of four.
        assert!(splits_past_two(1, 4));
        assert!(!splits_past_two(1, 0));
    }

    #[test]
    fn the_first_format_has_its_own_layouts() {
        assert_eq!(first_format_shown(1, false), Some(Shown::Player(1)));
        assert_eq!(first_format_shown(3, true), pair(0, true));
        assert_eq!(first_format_shown(4, false), None);
        assert_eq!(first_format_shown(5, true), Some(Shown::Player(3)));
        assert_eq!(first_format_shown(FIRST_GROUP, true), None);
    }

    #[test]
    fn a_layout_names_its_player() {
        for k in 0..8 {
            assert_eq!(shown(layout_of_player(k), 8), Some(Shown::Player(k)));
        }
    }

    #[test]
    fn the_camera_buttons_follow_the_views() {
        let buttons = |shown: Shown| shown.camera_players();
        // The first format's layouts: the first two players' cameras, whichever one is shown.
        assert_eq!(buttons(Shown::Player(0)), [Some(0), Some(1), None, None]);
        assert_eq!(buttons(Shown::Player(1)), [Some(0), Some(1), None, None]);
        assert_eq!(
            buttons(Shown::Pair {
                first: 0,
                stacked: true
            }),
            [Some(0), Some(1), None, None]
        );
        // A further player alone: both buttons edit that player's camera.
        assert_eq!(buttons(Shown::Player(5)), [Some(5), Some(5), None, None]);
        // A pair of the fifth and sixth: their cameras.
        assert_eq!(
            buttons(Shown::Pair {
                first: 4,
                stacked: false
            }),
            [Some(4), Some(5), None, None]
        );
        // A group has a button for each part: three or four.
        assert_eq!(
            buttons(Shown::Group { first: 0, count: 3 }),
            [Some(0), Some(1), Some(2), None]
        );
        let four = Shown::Group { first: 4, count: 4 };
        assert_eq!(buttons(four), [Some(4), Some(5), Some(6), Some(7)]);
        assert_eq!(four.camera_buttons(), 4);
        assert_eq!(Shown::Player(3).camera_buttons(), 2);
    }

    #[test]
    fn cameras_by_car() {
        assert_eq!(camera_source(0), Some(CameraSource::Head(0)));
        assert_eq!(camera_source(1), Some(CameraSource::Head(1)));
        assert_eq!(camera_source(2), Some(CameraSource::Head(6)));
        assert_eq!(camera_source(3), Some(CameraSource::Head(7)));
        assert_eq!(camera_source(4), Some(CameraSource::Live(0)));
        assert_eq!(camera_source(7), Some(CameraSource::Live(3)));
        assert_eq!(camera_source(8), None);
    }
}
