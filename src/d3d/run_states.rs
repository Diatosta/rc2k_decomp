//! The render states across the drawing threads' parts (`workers::draw_parts`). The game sets a
//! state only when it changes and draws on with whatever the last object left: run n (n > 0)
//! starts from the states as the list began, not as run n - 1 ends, so its fans up to its own
//! first setting of a state would carry a value the game in one run would not have drawn with
//! (z-writes on after a shadow had turned them off). The runs each keep their own cache of the
//! states (`d3d::render_cache`), and what they left is put back in their order. Each run
//! therefore notes where it first sets each state; when the runs are joined, its fans before
//! that point get the value the earlier runs left (`patch`), and the states it set go into the
//! store whatever their value (`carry`).

use std::cell::Cell;
use std::sync::Arc;

use crate::render::{Command, STATES};
use crate::workers::PartScratch;

type States = [u32; STATES];

/// Where a run first set each state, counted in the commands it noted (`usize::MAX`:
/// never).
pub struct RunStates {
    first_set: [usize; STATES],
    /// Where it first set the texture, counted the same way.
    texture_set: usize,
    /// The cache the run left.
    left: crate::d3d::render_cache::Cache,
}

/// A drawing part's run under way (`workers::PartScratch::run`): where it first set each
/// state. Only a part other than the first has one; only the thread drawing it reaches it
/// (plain cells, `part_cell::PartCell`).
pub struct OwnRun {
    on: Cell<bool>,
    /// The noted commands' count as the run began.
    base: Cell<usize>,
    first_set: [Cell<usize>; STATES],
    texture_set: Cell<usize>,
}

impl OwnRun {
    pub const fn new() -> Self {
        Self {
            on: Cell::new(false),
            base: Cell::new(0),
            first_set: [const { Cell::new(usize::MAX) }; STATES],
            texture_set: Cell::new(usize::MAX),
        }
    }
}

impl Default for OwnRun {
    fn default() -> Self {
        Self::new()
    }
}

/// Part `part`'s run begins on this thread, its cache starting as `start`.
pub fn begin_run(part: &PartScratch, start: &crate::d3d::render_cache::Cache) {
    crate::d3d::render_cache::begin(part, start);
    let run = &part.run;
    run.base.set(crate::platform_3d::noted_len(part));
    for f in &run.first_set {
        f.set(usize::MAX);
    }
    run.texture_set.set(usize::MAX);
    run.on.set(true);
}

/// Part `part`'s run is over: where it set its states.
pub fn end_run(part: &PartScratch) -> RunStates {
    let run = &part.run;
    let (first_set, texture_set) = if run.on.replace(false) {
        (
            std::array::from_fn(|i| run.first_set[i].get()),
            run.texture_set.get(),
        )
    } else {
        ([usize::MAX; STATES], usize::MAX)
    };
    RunStates {
        first_set,
        texture_set,
        left: crate::d3d::render_cache::end(part),
    }
}

/// `state` set (whatever its value, and whether it changed or not) in `part`'s run, if one is
/// under way.
pub fn note_set(part: &PartScratch, state: usize) {
    let run = &part.run;
    if run.on.get() && state < STATES && run.first_set[state].get() == usize::MAX {
        run.first_set[state]
            .set(crate::platform_3d::noted_len(part).saturating_sub(run.base.get()));
    }
}

/// The texture set in `part`'s run, if one is under way (as `note_set`).
pub fn note_texture_set(part: &PartScratch) {
    let run = &part.run;
    if run.on.get() && run.texture_set.get() == usize::MAX {
        run.texture_set
            .set(crate::platform_3d::noted_len(part).saturating_sub(run.base.get()));
    }
}

/// Whether the run noted a fan before it set the texture (that fan took the texture as the
/// run began, which `patch` cannot change).
pub fn drew_before_texture(noted: &[Command], run: &RunStates) -> bool {
    noted
        .iter()
        .take(run.texture_set)
        .any(|c| matches!(c, Command::Fan { .. }))
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

/// The states a run set, as it left them, into the first part's cache `first`, and its texture
/// if it changed from `start`'s.
pub fn carry(
    first: &crate::d3d::render_cache::OwnCache,
    run: &RunStates,
    start: &crate::d3d::render_cache::Cache,
) {
    for (i, &f) in run.first_set.iter().enumerate() {
        if f != usize::MAX {
            crate::d3d::render_cache::put_kept(
                first,
                i,
                crate::d3d::render_cache::left_kept(&run.left, i),
            );
        }
    }
    crate::d3d::render_cache::merge_texture(first, &run.left, start);
}

/// The states and the texture a piece drawn aside set (`workers::draw_aside`), as it left
/// them, into its part's cache `first`: each it set, whatever its value (the piece was drawn
/// after everything in the part so far).
pub fn carry_set(first: &crate::d3d::render_cache::OwnCache, run: &RunStates) {
    for (i, &f) in run.first_set.iter().enumerate() {
        if f != usize::MAX {
            crate::d3d::render_cache::put_kept(
                first,
                i,
                crate::d3d::render_cache::left_kept(&run.left, i),
            );
        }
    }
    if run.texture_set != usize::MAX {
        crate::d3d::render_cache::put_texture(first, &run.left);
    }
}

/// The first part's (`first`) states now: what the runs merged so far left.
pub fn carried(first: &crate::d3d::render_cache::OwnCache) -> [u32; STATES] {
    crate::d3d::render_cache::stored_kept(first)
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
        let run = RunStates {
            first_set,
            texture_set: usize::MAX,
            left: crate::d3d::render_cache::current(&crate::workers::part_scratch(0).cache),
        };
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

    /// A run's fans before it set the texture are found; none when it set it first.
    #[test]
    fn fans_before_the_texture_are_found() {
        let start: Arc<[u32; STATES]> = Arc::new([1; STATES]);
        let noted = vec![fan(&start), fan(&start)];
        let run = |texture_set| RunStates {
            first_set: [usize::MAX; STATES],
            texture_set,
            left: crate::d3d::render_cache::current(&crate::workers::part_scratch(0).cache),
        };
        assert!(drew_before_texture(&noted, &run(usize::MAX)));
        assert!(drew_before_texture(&noted, &run(1)));
        assert!(!drew_before_texture(&noted, &run(0)));
        assert!(!drew_before_texture(&[], &run(usize::MAX)));
    }
}
