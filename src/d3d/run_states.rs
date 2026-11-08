//! The render states across the drawing threads' runs (`workers::draw_in_runs`). The game
//! sets a state only when it changes and draws on with whatever the last object left: run
//! n (n > 0) starts from the states as the list began, not as run n - 1 ends, so its fans
//! up to its own first setting of a state carry a value the game in one run would not
//! have drawn with (z-writes on after a shadow had turned them off, replay 01). And the
//! runs' copies are merged into copy 0 by the bytes that changed from the list's start, so
//! a state a later run sets back to its opening value would be lost. Each run therefore
//! notes where it first sets each state; when the runs are joined, its fans before that
//! point get the value the earlier runs left (`patch`), and the states it set go into
//! copy 0 whatever their value (`carry`).

use std::cell::RefCell;
use std::sync::Arc;

use crate::render::{Command, STATES};

type States = [u32; STATES];

/// Where a run first set each state, counted in the commands it noted (`usize::MAX`:
/// never).
pub struct RunStates {
    first_set: [usize; STATES],
}

struct Current {
    /// The noted commands' count as the run began.
    base: usize,
    first_set: [usize; STATES],
}

thread_local! {
    static CURRENT: RefCell<Option<Current>> = const { RefCell::new(None) };
}

/// A run begins on this thread.
pub fn begin_run() {
    let base = crate::platform_3d::noted_len();
    CURRENT.set(Some(Current {
        base,
        first_set: [usize::MAX; STATES],
    }));
}

/// The run on this thread is over: where it set its states.
pub fn end_run() -> RunStates {
    let first_set = CURRENT.take().map_or([usize::MAX; STATES], |c| c.first_set);
    RunStates { first_set }
}

/// `state` set (whatever its value, and whether it changed or not) in the run on this
/// thread, if one is under way.
pub fn note_set(state: usize) {
    CURRENT.with_borrow_mut(|c| {
        if let Some(c) = c
            && state < STATES
            && c.first_set[state] == usize::MAX
        {
            c.first_set[state] = crate::platform_3d::noted_len().saturating_sub(c.base);
        }
    });
}

/// The run's fans before its own setting of a state given `carried`'s value of it (the
/// states as the earlier runs left them). Fans that shared states share the patched ones.
pub fn patch(noted: &mut [Command], run: &RunStates, carried: &[u32; STATES]) {
    // (the states patched, how many states were still unset, the patched ones)
    let mut made: Vec<(*const States, usize, Arc<States>)> = Vec::new();
    for (at, c) in noted.iter_mut().enumerate() {
        let Command::Fan { states, .. } = c else {
            continue;
        };
        let stale = |i: usize| at < run.first_set[i] && states[i] != carried[i];
        if !(0..STATES).any(stale) {
            continue;
        }
        // The same shared states in the same span of unset states patch the same way.
        let unset = run.first_set.iter().filter(|&&f| at < f).count();
        let key = Arc::as_ptr(states);
        if let Some((_, _, done)) = made.iter().find(|(k, u, _)| *k == key && *u == unset) {
            *states = Arc::clone(done);
            continue;
        }
        let fixed: [u32; STATES] = std::array::from_fn(|i| {
            if at < run.first_set[i] {
                carried[i]
            } else {
                states[i]
            }
        });
        let fixed = Arc::new(fixed);
        made.push((key, unset, Arc::clone(&fixed)));
        *states = fixed;
    }
}

/// The states run `n` set, as it left them, into copy 0 (after the byte merge, which loses
/// a state set back to its opening value).
pub fn carry(run: &RunStates, n: usize) {
    let cards = crate::d3d::owner::d3d().cards.all();
    for (i, &f) in run.first_set.iter().enumerate() {
        if f != usize::MAX {
            cards[0].kept_states[i].set(cards[n].kept_states[i].get());
        }
    }
}

/// Copy 0's states now: what the runs merged so far left.
pub fn carried() -> [u32; STATES] {
    let cards = crate::d3d::owner::d3d().cards.all();
    std::array::from_fn(|i| cards[0].kept_states[i].get())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fan(states: &Arc<[u32; STATES]>) -> Command {
        Command::Fan {
            vertices: Default::default(),
            states: Arc::clone(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Fans before a run's own setting of a state take the carried value; fans after keep
    /// the run's.
    #[test]
    fn fans_before_a_set_take_the_carried_value() {
        let start: Arc<[u32; STATES]> = Arc::new([1; STATES]);
        let mut noted = vec![fan(&start), fan(&start), fan(&start)];
        let mut first_set = [usize::MAX; STATES];
        first_set[14] = 2; // set before the third fan
        let run = RunStates { first_set };
        let mut carried = [1; STATES];
        carried[14] = 0;
        carried[3] = 3;
        patch(&mut noted, &run, &carried);
        let states = |c: &Command| match c {
            Command::Fan { states, .. } => Arc::clone(states),
            Command::Clear(_) => unreachable!(),
        };
        let (a, b, c) = (states(&noted[0]), states(&noted[1]), states(&noted[2]));
        assert_eq!((a[14], a[3]), (0, 3));
        assert!(Arc::ptr_eq(&a, &b), "shared states stay shared");
        assert_eq!((c[14], c[3]), (1, 3));
    }
}
