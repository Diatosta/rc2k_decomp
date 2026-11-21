//! The four player setup pages of each of the four players, one description per page kind
//! parameterised by the player's index: the details page, the controller page, the button
//! binding page and the keyboard redefinition page. `ScreenId` still names each of the sixteen
//! screens; `PlayerPage::<N>` holds what they share, and what differs between players is written
//! out in the details page, where it does differ.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::fn_list::FnList;
use crate::menu::owner::{Menus, Screens};
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Callee, Entry, Flow, Frames, Keys, List, Screen, ScreenFlag, ScreenVar, Step,
    quitting,
};
use crate::system::owner::System;

/// The field of `$source` that belongs to player `$n` (0 to 3), from the four fields in player
/// order.
macro_rules! of_player {
    ($source:expr, $n:expr, [$first:ident, $second:ident, $third:ident, $fourth:ident]) => {
        match $n {
            0 => &$source.$first,
            1 => &$source.$second,
            2 => &$source.$third,
            3 => &$source.$fourth,
            _ => unreachable!("a player page of a fifth player"),
        }
    };
}

/// Player `N`'s setup pages (`N` is 0 for the first player).
pub(super) struct PlayerPage<const N: usize>;

impl<const N: usize> PlayerPage<N> {
    /// The details page: name, nationality and the way into the controller pages.
    pub const DETAILS: Screen = Screen {
        lists: &[
            List {
                pointer: |m| &m.screens.flow.unread_list,
                table: |_| DRAW_LISTS.unread_draw_list.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.numbered_details_draw[N].id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.numbered_details_overlay[N].id(),
                run: false,
            },
        ],
        // Only the first player's page starts with the event not over.
        before: match N {
            0 => &[
                Step::WriteFlag(ScreenFlag::EventOver, false),
                Step::Write(ScreenVar::CurrentPlayer, 0x0),
            ],
            _ => &[],
        },
        background: match N {
            0 => &[
                Step::Write(ScreenVar::ClearsEscapeFlag, 0x1),
                Step::Call(Callee::LoadBackground(|m| {
                    of_player!(
                        m.texts,
                        N,
                        [
                            player1_details_picture,
                            player2_details_picture,
                            player3_details_picture,
                            player4_details_picture
                        ]
                    )
                })),
            ],
            _ => &[Step::Call(Callee::LoadBackground(|m| {
                of_player!(
                    m.texts,
                    N,
                    [
                        player1_details_picture,
                        player2_details_picture,
                        player3_details_picture,
                        player4_details_picture
                    ]
                )
            }))],
        },
        after_first_frame: &[Step::Write(ScreenVar::CurrentPlayer, N as u32)],
        // The third player's page checks the action's result in the short way.
        keys: match N {
            2 => Keys {
                action_check: ActionCheck::Short,
                ..Keys::DEFAULT
            },
            _ => Keys::DEFAULT,
        },
        frames: Frames {
            first_frame_lists: Some(&[
                List {
                    pointer: |m| &m.screens.flow.scene_list,
                    table: |_| DRAW_LISTS.numbered_details_scene[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.draw_list,
                    table: |_| DRAW_LISTS.numbered_details_draw[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.overlay_list,
                    table: |_| DRAW_LISTS.numbered_details_overlay[N].id(),
                    run: false,
                },
            ]),
            each_frame: &[Step::DoFlow(details_each_frame::<N>)],
            ..Frames::DEFAULT
        },
        ..Screen::new(
            NAMES[0][N],
            |m| {
                of_player!(
                    m.tables.menus,
                    N,
                    [
                        player1_details,
                        player2_details,
                        player3_details,
                        player4_details
                    ]
                )
                .id()
            },
            [
                slot::PLAYER1_DETAILS,
                slot::PLAYER2_DETAILS,
                slot::PLAYER3_DETAILS,
                slot::PLAYER4_DETAILS,
            ][N],
        )
    };

    /// The controller page: the controller, force feedback and axis mode.
    pub(super) const CONTROLLER: Screen = Screen {
        lists: &[
            List {
                pointer: |m| &m.screens.flow.unread_list,
                table: |_| DRAW_LISTS.unread_draw_list.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.player_controller_draw[N].id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.setup_list,
                table: |_| DRAW_LISTS.player_controller_setup[N].id(),
                run: true,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.player_controller_overlay[N].id(),
                run: false,
            },
        ],
        set_up_first: &[Step::Call(Callee::ApplyController)],
        background: &[Step::Call(Callee::LoadBackground(|m| {
            of_player!(
                m.texts,
                N,
                [
                    player1_controller_picture,
                    player2_controller_picture,
                    player3_controller_picture,
                    player4_controller_picture
                ]
            )
        }))],
        after: &[Step::Call(Callee::ReleaseController)],
        frames: Frames {
            first_frame_lists: Some(&[
                List {
                    pointer: |m| &m.screens.flow.scene_list,
                    table: |_| DRAW_LISTS.player_controller_scene[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.draw_list,
                    table: |_| DRAW_LISTS.player_controller_draw[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.overlay_list,
                    table: |_| DRAW_LISTS.player_controller_overlay[N].id(),
                    run: false,
                },
            ]),
            ..Frames::DEFAULT
        },
        ..Screen::new(
            NAMES[1][N],
            |m| {
                of_player!(
                    m.tables.menus,
                    N,
                    [
                        player1_controller,
                        player2_controller,
                        player3_controller,
                        player4_controller
                    ]
                )
                .id()
            },
            [
                slot::PLAYER1_CONTROLLER,
                slot::PLAYER2_CONTROLLER,
                slot::PLAYER3_CONTROLLER,
                slot::PLAYER4_CONTROLLER,
            ][N],
        )
    };

    /// The button binding page: what each control is bound to.
    pub const BUTTONS: Screen = Screen {
        lists: &[
            List {
                pointer: |m| &m.screens.flow.unread_list,
                table: |_| DRAW_LISTS.unread_draw_list.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.player_buttons_draw[N].id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.setup_list,
                // The fourth player's page has no set-up list of its own: it is given the
                // menus' function list.
                table: |_| match DRAW_LISTS.player_buttons_setup.get(N) {
                    Some(list) => list.id(),
                    None => FnList::Menus,
                },
                run: true,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.player_buttons_overlay[N].id(),
                run: false,
            },
        ],
        set_up_first: &[
            Step::Call(Callee::ApplyController),
            Step::Call(Callee::AxisMoved(0x0)),
        ],
        background: &[Step::Call(Callee::LoadBackground(|m| {
            of_player!(
                m.texts,
                N,
                [
                    player1_buttons_picture,
                    player2_buttons_picture,
                    player3_buttons_picture,
                    player4_buttons_picture
                ]
            )
        }))],
        after: &[Step::Call(Callee::ReleaseController)],
        frames: Frames {
            first_frame_lists: Some(&[
                List {
                    pointer: |m| &m.screens.flow.scene_list,
                    table: |_| DRAW_LISTS.player_buttons_scene[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.draw_list,
                    table: |_| DRAW_LISTS.player_buttons_draw[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.overlay_list,
                    table: |_| DRAW_LISTS.player_buttons_overlay[N].id(),
                    run: false,
                },
            ]),
            ..Frames::DEFAULT
        },
        ..Screen::new(
            NAMES[2][N],
            |m| {
                of_player!(
                    m.tables.menus,
                    N,
                    [
                        player1_buttons,
                        player2_buttons,
                        player3_buttons,
                        player4_buttons
                    ]
                )
                .id()
            },
            [
                slot::PLAYER1_BUTTONS,
                slot::PLAYER2_BUTTONS,
                slot::PLAYER3_BUTTONS,
                slot::PLAYER4_BUTTONS,
            ][N],
        )
    };

    /// The keyboard assignment page "REDEFINE KEYBOARD".
    pub(super) const REDEFINE_KEYS: Screen = Screen {
        lists: &[
            List {
                pointer: |m| &m.screens.flow.unread_list,
                table: |_| DRAW_LISTS.unread_draw_list.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.redefine_keys_draw[N].id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.redefine_keys_overlay[N].id(),
                run: false,
            },
        ],
        before: &[Step::DoMenus(redefine_keys_before::<N>)],
        background: &[Step::Call(Callee::LoadBackground(|m| {
            of_player!(
                m.texts,
                N,
                [
                    redefine_keys_player1_picture,
                    redefine_keys_player2_picture,
                    redefine_keys_player3_picture,
                    redefine_keys_player4_picture
                ]
            )
        }))],
        after_first_frame: &[Step::DoSystemFlow(redefine_keys_after_first_frame)],
        frames: Frames {
            first_frame_lists: Some(&[
                List {
                    pointer: |m| &m.screens.flow.scene_list,
                    table: |_| DRAW_LISTS.redefine_keys_scene[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.draw_list,
                    table: |_| DRAW_LISTS.redefine_keys_draw[N].id(),
                    run: false,
                },
                List {
                    pointer: |m| &m.screens.flow.overlay_list,
                    table: |_| DRAW_LISTS.redefine_keys_overlay[N].id(),
                    run: false,
                },
            ]),
            each_frame: &[Step::DoScreensFlow(redefine_keys_each_frame)],
            ..Frames::DEFAULT
        },
        ..Screen::new(
            NAMES[3][N],
            |m| {
                of_player!(
                    m.tables.menus,
                    N,
                    [
                        redefine_keys_player1,
                        redefine_keys_player2,
                        redefine_keys_player3,
                        redefine_keys_player4
                    ]
                )
                .id()
            },
            [
                slot::REDEFINE_KEYS_PLAYER1,
                slot::REDEFINE_KEYS_PLAYER2,
                slot::REDEFINE_KEYS_PLAYER3,
                slot::REDEFINE_KEYS_PLAYER4,
            ][N],
        )
    };
}

/// The names the screens log under, by page kind (details, controller, buttons, redefine keys)
/// and player; the registry's test requires them to be distinct.
const NAMES: [[&str; 4]; 4] = [
    [
        "rc2k_decomp::menu::player_pages::player1_details",
        "rc2k_decomp::menu::player_pages::player2_details",
        "rc2k_decomp::menu::player_pages::player3_details",
        "rc2k_decomp::menu::player_pages::player4_details",
    ],
    [
        "rc2k_decomp::menu::player_pages::player1_controller",
        "rc2k_decomp::menu::player_pages::player2_controller",
        "rc2k_decomp::menu::player_pages::player3_controller",
        "rc2k_decomp::menu::player_pages::player4_controller",
    ],
    [
        "rc2k_decomp::menu::player_pages::player1_buttons",
        "rc2k_decomp::menu::player_pages::player2_buttons",
        "rc2k_decomp::menu::player_pages::player3_buttons",
        "rc2k_decomp::menu::player_pages::player4_buttons",
    ],
    [
        "rc2k_decomp::menu::player_pages::redefine_keys_player1",
        "rc2k_decomp::menu::player_pages::redefine_keys_player2",
        "rc2k_decomp::menu::player_pages::redefine_keys_player3",
        "rc2k_decomp::menu::player_pages::redefine_keys_player4",
    ],
];

/// While a name is being typed the frame is spent editing it; the fourth player's page first
/// checks the name already typed for a cheat.
fn details_each_frame<const N: usize>(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if N == 3 {
        Callee::CheckCheatName.run(entry)?;
    }
    if screens.entry.read().typing {
        Callee::EditPlayerName.run(entry)?;
        if screens.entry.read().typing {
            return Ok(Flow::NextFrame);
        }
    }
    Ok(Flow::On)
}

/// Copies player `N`'s twelve keys to the table the page edits and makes the player current.
fn redefine_keys_before<const N: usize>(menus: &Menus) -> Result<(), Refused> {
    let keys = menus.play.slots.records[N].get().keys;
    menus.keys.tables_mut().redefined_keys[..keys.len()].copy_from_slice(&keys);
    menus.screens.event.write().current_player = N as u32;
    Ok(())
}

fn redefine_keys_after_first_frame(system: &System) -> Result<Flow, Refused> {
    if quitting(system) as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

/// While a key is being chosen the page waits.
fn redefine_keys_each_frame(screens: &Screens) -> Result<Flow, Refused> {
    if screens.entry.read().redefining_key {
        return Ok(Flow::NextFrame);
    }
    Ok(Flow::On)
}
