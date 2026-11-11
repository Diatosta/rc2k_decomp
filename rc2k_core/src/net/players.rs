//! The network game's players (from `net::sort_players` 0x418108,
//! `net::list_players` 0x419192, `net::race_over` 0x41732E, `net::race_wait` 0x4171FE and
//! `net::system_message` 0x41976F): their blocks' order, this machine's among them, and a
//! player's record's state (16 bytes of DirectPlay player data: +0 the state, +8 the
//! race's time, +0xC the total).

/// The most players a session has.
pub const MOST: u32 = 8;

/// A record's state bits.
pub const READY: u8 = 1;
pub const WAITING: u8 = 2;
pub const RACED: u8 = 4;
pub const HOST: u8 = 0x10;
/// A player left out of "everyone is ..." (`all_in_state`).
pub const LEFT_OUT: u8 = 0x80;

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

/// The original's bubble sort of `count` blocks: `count` - 1 passes (signed: none for one
/// or none), pass p comparing the first p pairs in order (`after(k)`: block k goes after
/// block k + 1, asked of the blocks as they are then), `swap(k)` where it does.
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

/// A record's state once its race is done: waiting and ready off, raced on.
pub fn raced(state: u8) -> u8 {
    state & !(READY | WAITING) | RACED
}

/// A record's state while it waits for the race to start: ready off, waiting on.
pub fn waiting(state: u8) -> u8 {
    state & !READY | WAITING
}

/// A record's state when DirectPlay makes this machine the host: the host, not ready.
pub fn made_host(state: u8) -> u8 {
    (state | HOST) & !READY
}

/// Whether every player (`state_of`, asked of each of `count`, wrapping: 0 is 2^32) has
/// all of `bits`, those with `LEFT_OUT` left out: the bits they all have.
pub fn all_in_state(bits: u8, count: u32, state_of: impl Fn(u32) -> u8) -> u8 {
    let mut all = bits;
    let mut n = 0u32;
    let mut left = count;
    loop {
        let state = state_of(n);
        if state & LEFT_OUT == 0 {
            all &= state;
        }
        n = n.wrapping_add(1);
        left = left.wrapping_sub(1);
        if left == 0 {
            return all;
        }
    }
}

/// Whether none of the players (`state_of`, asked of each of `count`) is still in a race
/// (`WAITING`), those with `LEFT_OUT` left out. The lobby's wait after a race: the original
/// waited for every player to have `RACED`, which a player who joined the session since the
/// race never has (the host lets players in again between races).
pub fn none_racing(count: u32, state_of: impl Fn(u32) -> u8) -> bool {
    (0..count)
        .map(state_of)
        .all(|state| state & LEFT_OUT != 0 || state & WAITING == 0)
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
    /// Any other: nothing.
    Other,
}

/// DPPLAYERTYPE_PLAYER.
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
        let of = |states: &'static [u8]| move |n: u32| states[n as usize];
        assert!(none_racing(2, of(&[RACED | HOST, RACED])));
        // a player who joined since the race
        assert!(none_racing(3, of(&[RACED | HOST, RACED, 0])));
        assert!(none_racing(3, of(&[RACED | HOST, RACED, READY])));
        assert!(!none_racing(2, of(&[RACED | HOST, WAITING])));
        assert!(none_racing(2, of(&[RACED | HOST, WAITING | LEFT_OUT])));
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
        assert_eq!(raced(0x13), 0x14);
        assert_eq!(waiting(0x11), 0x12);
        assert_eq!(made_host(0x03), 0x12);
        let states = [2u8, 0x82, 6];
        assert_eq!(all_in_state(2, 3, |n| states[n as usize]), 2);
        let states = [2u8, 0x80, 4];
        assert_eq!(all_in_state(2, 3, |n| states[n as usize]), 0);
        assert_eq!(System::of(0x101), System::Host);
        assert_eq!(System::of(4), System::Other);
    }
}
