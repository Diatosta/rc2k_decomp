//! The network game's players: their blocks' order, this machine's among them, and a player's
//! record's state (16 bytes of DirectPlay player data: +0 the state, +8 the race's time, +0xC the
//! total).

/// A record's raw state byte equals a state of that value.
impl PartialEq<u8> for State {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

/// The most players a session has.
pub const MOST: u32 = 8;

/// A player's state: the bits of a record's first byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State(u8);

impl State {
    /// Ready for the race (the lobby's "ready").
    pub const READY: State = State(1);
    /// In a race, waiting for the others to finish.
    pub const WAITING: State = State(2);
    /// Its race is done.
    pub const RACED: State = State(4);
    /// The session's host.
    pub const HOST: State = State(0x10);
    /// A player left out of "everyone is ..." (`all_in_state`).
    pub const LEFT_OUT: State = State(0x80);

    /// The state of a record's first byte.
    pub const fn from_byte(byte: u8) -> State {
        State(byte)
    }

    /// The byte a record holds.
    pub const fn byte(self) -> u8 {
        self.0
    }

    /// Whether every bit of `bits` is set.
    pub fn has(self, bits: State) -> bool {
        self.0 & bits.0 == bits.0
    }

    pub fn is_ready(self) -> bool {
        self.has(State::READY)
    }

    pub fn is_host(self) -> bool {
        self.has(State::HOST)
    }

    /// The bits both states have.
    pub fn and(self, other: State) -> State {
        State(self.0 & other.0)
    }

    /// The state with the bits of `bits` also set.
    pub const fn with(self, bits: State) -> State {
        State(self.0 | bits.0)
    }

    /// The state with the bits of `bits` cleared.
    pub fn without(self, bits: State) -> State {
        State(self.0 & !bits.0)
    }

    /// The state after the lobby's "ready": ready, no longer waiting or raced.
    pub fn made_ready(self) -> State {
        self.without(State::WAITING.with(State::RACED))
            .with(State::READY)
    }

    /// The state after the lobby's "ready" is pressed again: ready off.
    pub fn made_not_ready(self) -> State {
        self.without(State::READY)
    }

    /// The state once its race is done: waiting and ready off, raced on.
    pub fn raced(self) -> State {
        self.without(State::READY.with(State::WAITING))
            .with(State::RACED)
    }

    /// The state while it waits for the race to start: ready off, waiting on.
    pub fn waiting(self) -> State {
        self.without(State::READY).with(State::WAITING)
    }

    /// The state when this machine is made the host: the host, not ready.
    pub fn made_host(self) -> State {
        self.with(State::HOST).without(State::READY)
    }
}

/// No time in a race.
pub const NO_TIME: u32 = u32::MAX;

/// What the lobby's table sorts a player by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standing {
    pub total: u32,
    pub time: u32,
    pub id: u32,
}

/// Whether `a` goes after `b` in the lobby's table: one with no time after one with a
/// time; else by the total, the race's time, the DirectPlay id, unsigned, the smaller
/// first.
pub fn goes_after(a: Standing, b: Standing) -> bool {
    match (a.time == NO_TIME, b.time == NO_TIME) {
        (true, false) => return true,
        (false, true) => return false,
        _ => {}
    }
    (a.total, a.time, a.id) > (b.total, b.time, b.id)
}

/// Bubble sort of `count` blocks: `count` - 1 passes (signed: none for one or none), pass p
/// comparing the first p pairs in order (`after(k)`: block k goes after block k + 1), `swap(k)`
/// where it does.
pub fn bubble(count: u32, mut after: impl FnMut(u32) -> bool, mut swap: impl FnMut(u32)) {
    let mut pass = count.wrapping_sub(1) as i32;
    while pass > 0 {
        for k in 0..pass as u32 {
            if after(k) {
                swap(k);
            }
        }
        pass -= 1;
    }
}

/// This machine's player among the `listed` blocks by its DirectPlay id (`id_of`, asked in
/// order; block 0 looked at even with none listed): its number, or where the search
/// stopped.
pub fn find_own(own: u32, listed: u32, id_of: impl Fn(u32) -> u32) -> Result<u32, u32> {
    let mut n = 0u32;
    loop {
        if id_of(n) == own {
            return Ok(n);
        }
        n += 1;
        if n >= listed {
            return Err(n);
        }
    }
}

/// Whether every player (`state_of`, asked of each of `count`, wrapping: 0 is 2^32) has
/// all of `bits`, those with `LEFT_OUT` left out: the bits they all have.
pub fn all_in_state(bits: State, count: u32, state_of: impl Fn(u32) -> State) -> State {
    let mut all = bits;
    let mut n = 0u32;
    let mut left = count;
    loop {
        let state = state_of(n);
        if !state.has(State::LEFT_OUT) {
            all = all.and(state);
        }
        n = n.wrapping_add(1);
        left = left.wrapping_sub(1);
        if left == 0 {
            return all;
        }
    }
}

/// Whether none of the players (`state_of`, asked of each of `count`) is still in a race
/// (`State::WAITING`), those with `LEFT_OUT` left out. The lobby waits for this after a race: waiting
/// for every player to have `State::RACED` would hang on a player who joined since.
pub fn none_racing(count: u32, state_of: impl Fn(u32) -> State) -> bool {
    (0..count)
        .map(state_of)
        .all(|state| state.has(State::LEFT_OUT) || !state.has(State::WAITING))
}

/// How far through its stage a car is, in percent: its checkpoint of the stage's `count`
/// (held to 100; none counted is 0).
pub fn stage_share(checkpoint: u32, count: u32) -> u8 {
    if count == 0 {
        return 0;
    }
    (u64::from(checkpoint) * 100 / u64::from(count)).min(100) as u8
}

/// DirectPlay's system messages the game handles, by their first dword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    /// 3, DPSYS_CREATEPLAYERORGROUP: the sync table cleared again.
    Created,
    /// 5, DPSYS_DESTROYPLAYERORGROUP: the same, and a player (+4 is 1) taken out of the
    /// group.
    Destroyed,
    /// 0x31, DPSYS_SESSIONLOST: the connection destroyed.
    SessionLost,
    /// 0x101, DPSYS_HOST: this machine the host.
    Host,
    /// Any other system message: nothing.
    Other,
}

/// DirectPlay's player type for a player (`DPPLAYERTYPE_PLAYER`).
pub const A_PLAYER: u32 = 1;

impl System {
    pub fn of(kind: u32) -> System {
        match kind {
            3 => System::Created,
            5 => System::Destroyed,
            0x31 => System::SessionLost,
            0x101 => System::Host,
            _ => System::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_racing_waits_for_racers_only() {
        const RACED_HOST: State = State::RACED.with(State::HOST);
        fn of(states: &[State]) -> impl Fn(u32) -> State + '_ {
            move |n| states[n as usize]
        }
        assert!(none_racing(2, of(&[RACED_HOST, State::RACED])));
        // a player who joined since the race
        assert!(none_racing(
            3,
            of(&[RACED_HOST, State::RACED, State::default()])
        ));
        assert!(none_racing(
            3,
            of(&[RACED_HOST, State::RACED, State::READY])
        ));
        assert!(!none_racing(2, of(&[RACED_HOST, State::WAITING])));
        assert!(none_racing(
            2,
            of(&[RACED_HOST, State::WAITING.with(State::LEFT_OUT)])
        ));
    }

    #[test]
    fn a_stage_share_is_the_checkpoint_over_the_count() {
        assert_eq!(stage_share(0, 0), 0);
        assert_eq!(stage_share(0, 400), 0);
        assert_eq!(stage_share(100, 400), 25);
        assert_eq!(stage_share(399, 400), 99);
        assert_eq!(stage_share(400, 400), 100);
        assert_eq!(stage_share(900, 400), 100);
        assert_eq!(stage_share(u32::MAX, 3), 100);
    }

    #[test]
    fn order() {
        let s = |total, time, id| Standing { total, time, id };
        assert!(goes_after(s(0, NO_TIME, 0), s(9, 5, 9)));
        assert!(!goes_after(s(9, 5, 9), s(0, NO_TIME, 0)));
        assert!(goes_after(s(2, 5, 1), s(1, 9, 9)));
        assert!(goes_after(s(1, 6, 1), s(1, 5, 9)));
        assert!(goes_after(s(1, 5, 2), s(1, 5, 1)));
        assert!(!goes_after(s(1, 5, 1), s(1, 5, 1)));
        let mut v = vec![5u32, 3, 9, 1, 7];
        let cell = std::cell::RefCell::new(&mut v);
        bubble(
            5,
            |k| cell.borrow()[k as usize] > cell.borrow()[k as usize + 1],
            |k| cell.borrow_mut().swap(k as usize, k as usize + 1),
        );
        assert_eq!(v, [1, 3, 5, 7, 9]);
        let ids = [4u32, 7, 9];
        assert_eq!(find_own(7, 3, |n| ids[n as usize]), Ok(1));
        assert_eq!(find_own(8, 3, |n| ids[n as usize]), Err(3));
        assert_eq!(find_own(8, 0, |n| ids[n as usize]), Err(1));
    }

    #[test]
    fn states() {
        let s = State::from_byte;
        assert_eq!(s(0x13).raced(), s(0x14));
        assert_eq!(s(0x11).waiting(), s(0x12));
        assert_eq!(s(0x03).made_host(), s(0x12));
        assert_eq!(s(0x16).made_ready(), s(0x11));
        assert_eq!(s(0).made_ready(), State::READY);
        assert_eq!(s(0).made_ready().made_not_ready(), s(0));
        assert!(s(0x11).is_host() && s(0x11).is_ready() && !s(0x10).is_ready());
        let states = [2u8, 0x82, 6];
        assert_eq!(
            all_in_state(State::WAITING, 3, |n| s(states[n as usize])),
            State::WAITING
        );
        let states = [2u8, 0x80, 4];
        assert_eq!(
            all_in_state(State::WAITING, 3, |n| s(states[n as usize])),
            s(0)
        );
        assert_eq!(System::of(0x101), System::Host);
        assert_eq!(System::of(4), System::Other);
    }
}
