//! What the layout byte of a replay block shows: the one rule the viewer's car item, the drawing
//! of the cars shown, the camera choice and the HUD ask. The byte names players, by their number
//! in the order of the replay's players (the first player is 0):
//!
//! - 0 and 1: the first and the second player alone; 4 .. 9: the third to the eighth alone;
//! - 2 and 3: the first two players together, side by side or one above the other, when they are
//!   the only two (a replay of two players);
//! - 10, 13, 14 and 15: the first group of players, up to four, each in a part of the screen (for
//!   a three- or four-player replay, every player), in one of the arrangements a local
//!   split-screen race of as many players has (`Split::many_layouts`, in its order: 10 the first,
//!   the race's default); from three players on it is the only way the first players are shown
//!   together;
//! - 11, 12, 16, 17 and 18: the players after the first four: two of them (a replay of six) are a
//!   pair, as the first two are, 11 side by side and 12 one above the other; three or four of them
//!   (seven or eight players) are a group, arranged as the first group is by 11, 16, 17 and 18.
//!
//! The groups' first layouts (10 and 11) are the numbers written before a group kept its
//! arrangement: such a block shows its group in the default arrangement.

use crate::race::view::Split;

/// The layout of the first group of players, in its default arrangement.
pub const FIRST_GROUP: u8 = 10;
/// The layout of the players after the first four: a pair side by side when they are two, else
/// a group in its default arrangement.
pub const SECOND_GROUP: u8 = 11;
/// The layout of those players as a pair one above the other (when they are two).
pub const SECOND_PAIR_STACKED: u8 = 12;
/// The most arrangements a group has (three players' four, `Split::many_layouts`).
pub const ARRANGEMENTS_MOST: usize = 4;
/// The layouts of the first group, by arrangement (`Split::many_layouts`' order).
pub const FIRST_GROUP_LAYOUTS: [u8; ARRANGEMENTS_MOST] = [FIRST_GROUP, 13, 14, 15];
/// The layouts of the group of the players after the first four, by arrangement.
pub const SECOND_GROUP_LAYOUTS: [u8; ARRANGEMENTS_MOST] = [SECOND_GROUP, 16, 17, 18];
/// The last layout.
pub const LAST_LAYOUT: u8 = SECOND_GROUP_LAYOUTS[ARRANGEMENTS_MOST - 1];
/// Every layout in the order the viewer's car item goes through them: the players alone, the
/// first two together, the first group in each arrangement (the default first), then the players
/// after the first four as a pair and as a group in each arrangement.
pub const CYCLE: [u8; LAST_LAYOUT as usize + 1] = [
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    FIRST_GROUP_LAYOUTS[0],
    FIRST_GROUP_LAYOUTS[1],
    FIRST_GROUP_LAYOUTS[2],
    FIRST_GROUP_LAYOUTS[3],
    SECOND_GROUP_LAYOUTS[0],
    SECOND_PAIR_STACKED,
    SECOND_GROUP_LAYOUTS[1],
    SECOND_GROUP_LAYOUTS[2],
    SECOND_GROUP_LAYOUTS[3],
];
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
    /// `count` players from player `first`, each in a part of the screen as `split` divides it
    /// (one of `Split::many_layouts(count)`).
    Group {
        first: usize,
        count: usize,
        split: Split,
    },
}

/// The groups' layouts by the group's first player: the first group's and the one after the
/// first four players.
const GROUP_LAYOUTS: [(usize, [u8; ARRANGEMENTS_MOST]); 2] =
    [(0, FIRST_GROUP_LAYOUTS), (GROUP_MOST, SECOND_GROUP_LAYOUTS)];

/// The group a layout shows and its arrangement: the group's first player and the arrangement's
/// number among `Split::many_layouts`; none for a layout of no group.
fn group_arrangement(layout: u8) -> Option<(usize, usize)> {
    GROUP_LAYOUTS.iter().find_map(|&(first, layouts)| {
        let arrangement = layouts.iter().position(|&l| l == layout)?;
        Some((first, arrangement))
    })
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
        SECOND_GROUP | SECOND_PAIR_STACKED if rest == PAIR => Some(Shown::Pair {
            first: GROUP_MOST,
            stacked: layout == SECOND_PAIR_STACKED,
        }),
        _ => {
            let (first, arrangement) = group_arrangement(layout)?;
            let count = players.saturating_sub(first).min(GROUP_MOST);
            if count < GROUP_LEAST {
                return None;
            }
            let split = *Split::many_layouts(count as u32).get(arrangement)?;
            Some(Shown::Group {
                first,
                count,
                split,
            })
        }
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
            Shown::Group { first, count, .. } => {
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

    /// The layout byte that shows this: `shown`'s other way round, for a replay that has the
    /// players named. None for what no layout shows: a player past the eighth, a pair of others
    /// than the first two or the fifth and sixth, a group from another player than the first or
    /// the fifth, or of a count or split no group has.
    pub fn layout(self) -> Option<u8> {
        match self {
            Shown::Player(player) => {
                (player < crate::race::replay_v2::MAX_CARS).then(|| layout_of_player(player))
            }
            Shown::Pair { first: 0, stacked } => Some(if stacked { 3 } else { 2 }),
            Shown::Pair {
                first: GROUP_MOST,
                stacked,
            } => Some(if stacked {
                SECOND_PAIR_STACKED
            } else {
                SECOND_GROUP
            }),
            Shown::Pair { .. } => None,
            Shown::Group {
                first,
                count,
                split,
            } => {
                let &(_, layouts) = GROUP_LAYOUTS.iter().find(|&&(f, _)| f == first)?;
                if !(GROUP_LEAST..=GROUP_MOST).contains(&count) {
                    return None;
                }
                let arrangement = Split::many_layouts(count as u32)
                    .iter()
                    .position(|&s| s == split)?;
                Some(layouts[arrangement])
            }
        }
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

    /// A group in its default arrangement.
    fn group(first: usize, count: usize) -> Option<Shown> {
        arranged(first, count, Split::many_layouts(count as u32)[0])
    }

    fn arranged(first: usize, count: usize, split: Split) -> Option<Shown> {
        Some(Shown::Group {
            first,
            count,
            split,
        })
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
    fn each_group_has_a_layout_for_each_arrangement() {
        let first = |players| FIRST_GROUP_LAYOUTS.map(|layout| shown(layout, players));
        let second = |players| SECOND_GROUP_LAYOUTS.map(|layout| shown(layout, players));
        // Three players: one across the top and two below (the default), rows, columns,
        // quarters; four: quarters (the default), rows, columns, and no fourth.
        let three = [Split::WideTop, Split::Rows, Split::Columns, Split::Quarters];
        assert_eq!(first(3), three.map(|split| arranged(0, 3, split)));
        let four = [Split::Quarters, Split::Rows, Split::Columns];
        assert_eq!(first(4)[..3], four.map(|split| arranged(0, 4, split)));
        assert_eq!(first(4)[3], None);
        // Every replay of more than four players has the first four's three.
        for players in 5..=8 {
            assert_eq!(first(players)[..3], first(4)[..3]);
            assert_eq!(first(players)[3], None);
        }
        // The players after the first four: none of the group's arrangements for five or six
        // (six have the pair in 11), the three-player ones for seven, the four-player ones for
        // eight.
        assert_eq!(second(5), [None; ARRANGEMENTS_MOST]);
        assert_eq!(second(6)[1..], [None; ARRANGEMENTS_MOST - 1]);
        assert_eq!(second(7), three.map(|split| arranged(4, 3, split)));
        assert_eq!(second(8)[..3], four.map(|split| arranged(4, 4, split)));
        assert_eq!(second(8)[3], None);
        // A two-player replay has no group at all.
        assert_eq!(first(2), [None; ARRANGEMENTS_MOST]);
    }

    #[test]
    fn the_groups_arrangements_follow_the_race_split_layouts() {
        for count in GROUP_LEAST..=GROUP_MOST {
            assert!(Split::many_layouts(count as u32).len() <= ARRANGEMENTS_MOST);
        }
        assert_eq!(Split::many_layouts(3).len(), ARRANGEMENTS_MOST);
    }

    #[test]
    fn the_cycle_holds_every_layout_once_the_default_arrangements_first() {
        let mut sorted = CYCLE;
        sorted.sort_unstable();
        assert_eq!(sorted, std::array::from_fn(|k| k as u8));
        let at = |layout| CYCLE.iter().position(|&l| l == layout).unwrap();
        for layouts in [FIRST_GROUP_LAYOUTS, SECOND_GROUP_LAYOUTS] {
            for pair in layouts.windows(2) {
                assert!(at(pair[0]) < at(pair[1]));
            }
        }
        // The first group's arrangements together, before the players after the first four.
        assert_eq!(at(FIRST_GROUP_LAYOUTS[3]) + 1, at(SECOND_GROUP));
        assert_eq!(at(SECOND_GROUP) + 1, at(SECOND_PAIR_STACKED));
    }

    #[test]
    fn a_layout_reads_back_from_what_it_shows() {
        for players in 1..=8 {
            for layout in 0..=LAST_LAYOUT {
                if let Some(what) = shown(layout, players) {
                    assert_eq!(what.layout(), Some(layout), "{players} players, {what:?}");
                }
            }
        }
        // What no layout shows.
        assert_eq!(Shown::Player(8).layout(), None);
        let pair = |first| Shown::Pair {
            first,
            stacked: false,
        };
        assert_eq!(pair(2).layout(), None);
        let group = |first, count, split| Shown::Group {
            first,
            count,
            split,
        };
        assert_eq!(group(2, 3, Split::Rows).layout(), None);
        assert_eq!(group(0, 2, Split::Beside).layout(), None);
        assert_eq!(group(0, 4, Split::WideTop).layout(), None);
        assert_eq!(group(4, 3, Split::Quarters).layout(), Some(18));
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
            buttons(group(0, 3).unwrap()),
            [Some(0), Some(1), Some(2), None]
        );
        let four = group(4, 4).unwrap();
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
