//! The event's state, out of RAL.EXE's image (IDA 0x7558C4 .. 0x7559A4): the stage
//! chosen, the classes, the times and flags of the event's players, the service, the event's
//! type, kind and sections, the current player and the car shown. A `#[repr(C)]` struct with the
//! image's layout, one static.

use crate::session::{PlainBytes, PlainDword, PlainFlag, with_runs};

/// The event's state (IDA 0x7558C4 .. 0x7559A4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Event {
    /// +0 (0x7558C4) `dword_7558C4`: the controller (0 the keyboard, n the input device n - 1) of the player picked on the controller page, written by `screens::pick_player_n` (player +0x2C); read by no code.
    pub picked_controller: PlainDword,
    /// `dword_7558C8`: The stage chosen: its group, 0 or 1 (`menu::stage_choice`).
    pub stage_group: PlainDword,
    /// `dword_7558CC`: The stage chosen: its number in its group, 1 .. 6.
    pub stage_number: PlainDword,
    /// `dword_7558D0`: The stage chosen: its sub-group, 0 .. 5.
    pub stage_sub: PlainDword,
    /// `| Not worked out.`:
    pub dword_7558d4: PlainDword,
    /// `g_clearsEscapeFlag`: 1 clears the Escape flag with the highlight (`menu::screen_loop`).
    pub clears_escape_flag: PlainDword,
    /// `dword_7558DC`: The `sparksmode` setting of `var\ral.cfg`.
    pub sparks_mode: PlainDword,
    /// `dword_7558E0`: Not worked out.
    pub dword_7558e0: PlainDword,
    /// `dword_7558E4`: Not worked out.
    pub dword_7558e4: PlainDword,
    /// `dword_7558E8`: Not worked out.
    pub dword_7558e8: PlainDword,
    /// `dword_7558EC`: Not worked out.
    pub dword_7558ec: PlainDword,
    /// `dword_7558F0`: Not worked out.
    pub dword_7558f0: PlainDword,
    /// `dword_7558F4`: The class shown on the class pages.
    pub class: PlainDword,
    /// `dword_7558F8`: The number of classes (`menu::calc_cpu_times`).
    pub classes: PlainDword,
    /// `| Not worked out.`:
    pub dword_7558fc: PlainDword,
    /// `dword_755900`: Not worked out.
    pub dword_755900: PlainDword,
    /// `dword_755904`: Not worked out.
    pub dword_755904: PlainDword,
    /// `dword_755908`: Not worked out.
    pub dword_755908: PlainDword,
    /// `dword_75590C`: Not worked out.
    pub dword_75590c: PlainDword,
    /// `dword_755910`: Not worked out.
    pub dword_755910: PlainDword,
    /// `g_playerTime`: The player's time (`menu::time_trial_qualifies`).
    pub player_time: PlainDword,
    /// `g_timeQualifies`: 1 when the time qualifies / beats the target.
    pub time_qualifies: PlainDword,
    /// `| Not worked out.`:
    pub dword_75591c: PlainDword,
    /// `g_anyTimeQualified`: 1 when any player's time qualified.
    pub any_time_qualified: PlainFlag,
    /// `dword_755924`: The welcome page's figure (`menu::run_championship`).
    pub welcome_figure: PlainDword,
    /// `g_championshipEnd`: The championship is over.
    pub championship_end: PlainDword,
    /// `g_eventGoesOn`: 1 while the event goes on.
    pub event_goes_on: PlainDword,
    /// `dword_755930`: Not worked out.
    pub dword_755930: PlainDword,
    /// `dword_755934`: 1 once the welcome page was shown.
    pub welcomed: PlainDword,
    /// `g_serviceDue`: 1 when the service is due.
    pub service_due: PlainDword,
    /// `g_serviceMinutes`: The service time left.
    pub service_minutes: PlainDword,
    /// +0x7C (0x755940) `dword_755940`: the +0xC word of the service record the championship has reached (`menu::run_championship`); written there, read by no code.
    pub service_record_c: PlainDword,
    /// +0x80 (0x755944) `dword_755944`: 1 when the next computer drivers' times start a batch (set by a service record and the condition-reset step), 0 once `menu::calc_cpu_times` has drawn the first spread: with `cpu_spread_step` the state of `rc2k_core::menu::event::Spread`.
    pub cpu_spread_first: PlainDword,
    /// +0x84 (0x755948) `dword_755948`: how many spreads of the batch `menu::calc_cpu_times` has drawn after the first: the multiplier of its second random number (under 1000); 0 while the batch starts.
    pub cpu_spread_step: PlainDword,
    /// `dword_75594C`: The next service.
    pub service_next: PlainDword,
    /// `g_stagesDone`: The stages done in the event.
    pub stages_done: PlainDword,
    /// `g_eventType`: 3 time trial, 4 and 5 the championships (`menu::event_flow`).
    pub event_type: PlainDword,
    /// `index`: The event's stage in its section.
    pub index: PlainDword,
    /// `section`: The event's section.
    pub section: PlainDword,
    /// `dword_755960`: The first stage of the page shown.
    pub first_page_stage: PlainDword,
    /// +0xA0 (0x755964) `dword_755964`: the section of the stage list being viewed (a copy of `section` made by `menu::view_stage_loop`); read by no code.
    pub viewed_section: PlainDword,
    /// `g_stagePages`: The stage pages.
    pub stage_pages: PlainDword,
    /// `g_stagePage`: The stage page shown.
    pub stage_page: PlainDword,
    /// `g_continuingSave`: 1 when a saved game goes on.
    pub continuing_save: PlainDword,
    /// Unreferenced bytes (IDA 0x755974 .. 0x755978).
    pub _gap_755974: PlainBytes<4>,
    /// `dword_755978`: 1 when the player is out of the event.
    pub event_out: PlainDword,
    /// `dword_75597C`: The count of the standings (`menu::number_all_a`).
    pub count_a: PlainDword,
    /// `dword_755980`: 1 when an event is over.
    pub event_over: PlainFlag,
    /// `dword_755984`: The players in the event.
    pub player_count: PlainDword,
    /// `g_eventKind`: The event's kind (9 .. 0x18, `menu::run_event_races`).
    pub event_kind: PlainDword,
    /// `g_currentPlayer`: The player whose turn it is.
    pub current_player: PlainDword,
    /// `dword_755990`: The car shown (an entry of the carousel).
    pub shown_car: PlainDword,
    /// `dword_755994`: The slideshow's picture, 0 .. 99.
    pub picture: PlainDword,
    /// `dword_755998`: The car page shown.
    pub car_page: PlainDword,
    /// Unreferenced bytes (IDA 0x75599C .. 0x7559A4).
    pub _gap_75599c: PlainBytes<8>,
}

impl Event {
    /// First values the image's (`tests::event_state_starts_as_the_games`).
    pub const fn new() -> Self {
        Event {
            picked_controller: PlainDword::new(0x0),
            stage_group: PlainDword::new(0x0),
            stage_number: PlainDword::new(0x1),
            stage_sub: PlainDword::new(0x0),
            dword_7558d4: PlainDword::new(0x0),
            clears_escape_flag: PlainDword::new(0x0),
            sparks_mode: PlainDword::new(0x0),
            dword_7558e0: PlainDword::new(0x0),
            dword_7558e4: PlainDword::new(0x0),
            dword_7558e8: PlainDword::new(0x0),
            dword_7558ec: PlainDword::new(0x0),
            dword_7558f0: PlainDword::new(0x0),
            class: PlainDword::new(0x0),
            classes: PlainDword::new(0x4),
            dword_7558fc: PlainDword::new(0x0),
            dword_755900: PlainDword::new(0x0),
            dword_755904: PlainDword::new(0x0),
            dword_755908: PlainDword::new(0x0),
            dword_75590c: PlainDword::new(0x0),
            dword_755910: PlainDword::new(0x2),
            player_time: PlainDword::new(0x0),
            time_qualifies: PlainDword::new(0x0),
            dword_75591c: PlainDword::new(0x0),
            any_time_qualified: PlainFlag::new(false),
            welcome_figure: PlainDword::new(0x3DC),
            championship_end: PlainDword::new(0x0),
            event_goes_on: PlainDword::new(0x1),
            dword_755930: PlainDword::new(0x0),
            welcomed: PlainDword::new(0x0),
            service_due: PlainDword::new(0x0),
            service_minutes: PlainDword::new(0x0),
            service_record_c: PlainDword::new(0x0),
            cpu_spread_first: PlainDword::new(0x0),
            cpu_spread_step: PlainDword::new(0x0),
            service_next: PlainDword::new(0x0),
            stages_done: PlainDword::new(0x0),
            event_type: PlainDword::new(0x1),
            index: PlainDword::new(0x0),
            section: PlainDword::new(0x0),
            first_page_stage: PlainDword::new(0x0),
            viewed_section: PlainDword::new(0x0),
            stage_pages: PlainDword::new(0x0),
            stage_page: PlainDword::new(0x0),
            continuing_save: PlainDword::new(0x0),
            _gap_755974: PlainBytes::new(with_runs::<4>(&[(0x0, &[0x05])])),
            event_out: PlainDword::new(0x0),
            count_a: PlainDword::new(0x0),
            event_over: PlainFlag::new(false),
            player_count: PlainDword::new(0x1),
            event_kind: PlainDword::new(0x18),
            current_player: PlainDword::new(0x0),
            shown_car: PlainDword::new(0x0),
            picture: PlainDword::new(0x0),
            car_page: PlainDword::new(0x0),
            _gap_75599c: PlainBytes::new([0; 8]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn event_state_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().screens.event,
        );
        assert_eq!(offset_of!(Event, picked_controller), 0x0);
        assert_eq!(offset_of!(Event, stage_group), 0x4);
        assert_eq!(offset_of!(Event, stage_number), 0x8);
        assert_eq!(offset_of!(Event, stage_sub), 0xc);
        assert_eq!(offset_of!(Event, dword_7558d4), 0x10);
        assert_eq!(offset_of!(Event, clears_escape_flag), 0x14);
        assert_eq!(offset_of!(Event, sparks_mode), 0x18);
        assert_eq!(offset_of!(Event, dword_7558e0), 0x1c);
        assert_eq!(offset_of!(Event, dword_7558e4), 0x20);
        assert_eq!(offset_of!(Event, dword_7558e8), 0x24);
        assert_eq!(offset_of!(Event, dword_7558ec), 0x28);
        assert_eq!(offset_of!(Event, dword_7558f0), 0x2c);
        assert_eq!(offset_of!(Event, class), 0x30);
        assert_eq!(offset_of!(Event, classes), 0x34);
        assert_eq!(offset_of!(Event, dword_7558fc), 0x38);
        assert_eq!(offset_of!(Event, dword_755900), 0x3c);
        assert_eq!(offset_of!(Event, dword_755904), 0x40);
        assert_eq!(offset_of!(Event, dword_755908), 0x44);
        assert_eq!(offset_of!(Event, dword_75590c), 0x48);
        assert_eq!(offset_of!(Event, dword_755910), 0x4c);
        assert_eq!(offset_of!(Event, player_time), 0x50);
        assert_eq!(offset_of!(Event, time_qualifies), 0x54);
        assert_eq!(offset_of!(Event, dword_75591c), 0x58);
        assert_eq!(offset_of!(Event, any_time_qualified), 0x5c);
        assert_eq!(offset_of!(Event, welcome_figure), 0x60);
        assert_eq!(offset_of!(Event, championship_end), 0x64);
        assert_eq!(offset_of!(Event, event_goes_on), 0x68);
        assert_eq!(offset_of!(Event, dword_755930), 0x6c);
        assert_eq!(offset_of!(Event, welcomed), 0x70);
        assert_eq!(offset_of!(Event, service_due), 0x74);
        assert_eq!(offset_of!(Event, service_minutes), 0x78);
        assert_eq!(offset_of!(Event, service_record_c), 0x7c);
        assert_eq!(offset_of!(Event, cpu_spread_first), 0x80);
        assert_eq!(offset_of!(Event, cpu_spread_step), 0x84);
        assert_eq!(offset_of!(Event, service_next), 0x88);
        assert_eq!(offset_of!(Event, stages_done), 0x8c);
        assert_eq!(offset_of!(Event, event_type), 0x90);
        assert_eq!(offset_of!(Event, index), 0x94);
        assert_eq!(offset_of!(Event, section), 0x98);
        assert_eq!(offset_of!(Event, first_page_stage), 0x9c);
        assert_eq!(offset_of!(Event, viewed_section), 0xa0);
        assert_eq!(offset_of!(Event, stage_pages), 0xa4);
        assert_eq!(offset_of!(Event, stage_page), 0xa8);
        assert_eq!(offset_of!(Event, continuing_save), 0xac);
        assert_eq!(offset_of!(Event, event_out), 0xb4);
        assert_eq!(offset_of!(Event, count_a), 0xb8);
        assert_eq!(offset_of!(Event, event_over), 0xbc);
        assert_eq!(offset_of!(Event, player_count), 0xc0);
        assert_eq!(offset_of!(Event, event_kind), 0xc4);
        assert_eq!(offset_of!(Event, current_player), 0xc8);
        assert_eq!(offset_of!(Event, shown_car), 0xcc);
        assert_eq!(offset_of!(Event, picture), 0xd0);
        assert_eq!(offset_of!(Event, car_page), 0xd4);
    }
}
