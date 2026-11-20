//! The passes over the players after a stage (the inspection and the service area): who is looked
//! at, when the event is over, the service's minutes and money, and the pauses between the screens.

/// A player's +0xD0: taking part, and out (failed the inspection).
pub(crate) const TAKING_PART: u32 = 1;
pub const OUT: u32 = 2;

/// The pauses (ms): after the inspection's screen, after the fade in (service), after
/// an inspection page.
pub const SCREEN_PAUSE: u32 = 1000;
pub const FADE_PAUSE: u32 = 0xFA;
pub const INSPECTION_PAUSE: u32 = 0x4E2;

/// The keys the service area drops before its page.
pub const DROPPED_KEYS: u32 = 6;

/// The service's minutes: 900 more each service between stages; the
/// time trial's before its stage.
pub(crate) const SERVICE_MINUTES: u32 = 900;
pub const TIME_TRIAL_MINUTES: u32 = 0xA8C;

pub fn takes_part(kind: u32) -> bool {
    kind == TAKING_PART
}

pub fn is_out(kind: u32) -> bool {
    kind == OUT
}

/// `service_pass`: a player is serviced when taking part and the service is due
/// (`EventState.service.due` 1; `due` asked only for a player taking part).
pub fn serviced(kind: u32, due: impl FnOnce() -> u32) -> bool {
    takes_part(kind) && due() == 1
}

/// The service's minutes after one more service.
pub fn more_minutes(minutes: u32) -> u32 {
    minutes.wrapping_add(SERVICE_MINUTES)
}

/// A serviced player's +0x288: the money spent times 100.
pub fn spent(money: u32) -> u32 {
    money.wrapping_mul(100)
}

/// The event goes on (`EventState.event_goes_on` stays 1) unless every player is out.
pub fn goes_on(out: u32, players: u32) -> bool {
    out != players
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        assert!(takes_part(1) && !takes_part(2) && is_out(2) && !is_out(1));
        assert!(serviced(1, || 1));
        assert!(!serviced(1, || 0));
        assert!(!serviced(2, || unreachable!()));
        assert_eq!(more_minutes(100), 1000);
        assert_eq!(more_minutes(u32::MAX), 899);
        assert_eq!(spent(0x0200_0000), 0x0800_0000u32.wrapping_mul(25));
        assert!(goes_on(1, 2) && !goes_on(2, 2));
    }
}
