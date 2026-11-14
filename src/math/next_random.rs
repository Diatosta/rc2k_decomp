//! `math::next_random`: the game's random numbers.
//!
//! One step of the generator on its state: the low byte is changed by three bits of the
//! number (bit 22, then bit 2 and bit 1 of the byte as it then is, each flipping bit 0),
//! the whole is turned right by one, and what is mixed in is `xor`ed on.

/// The generator's state, starting at the game's first value until `seed_random`.
pub static STATE: crate::session::Dword = crate::session::Dword::new(0x9570_1114);

/// The generator loaded from `STATE`, `f` run on it, its state stored back.
pub fn with_random<R>(f: impl FnOnce(&mut rc2k_core::math::random::Random) -> R) -> R {
    let mut random = rc2k_core::math::random::Random { state: STATE.get() };
    let value = f(&mut random);
    STATE.set(random.state);
    value
}

/// The thread that owns the generator while it draws a sequence that must stay together
/// (0: none). Another thread asking for a number meanwhile waits.
static OWNER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn this_thread() -> u32 {
    crate::sync::thread_id()
}

/// Waits while another thread owns the generator (`own_random`).
pub fn wait_for_random() {
    use std::sync::atomic::Ordering;
    loop {
        let owner = OWNER.load(Ordering::SeqCst);
        if owner == 0 || owner == this_thread() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// The numbers drawn so far (for tests that compare how many each side drew).
pub static DRAWS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn next_random() -> u32 {
    // On a drawing thread, after the runs before its own (the numbers in the list's order).
    crate::workers::wait_for_earlier_runs();
    wait_for_random();
    DRAWS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    with_random(rc2k_core::math::random::Random::draw)
}

/// A generator of its own for one part of the game, so how many numbers one part takes no
/// longer shifts every number the others draw (wiki: "The race's sounds steer the race's
/// random numbers", "The pictures steer the race's random numbers"). The same generator
/// as `STATE`'s, started from the game's state (`seed_streams`).
pub struct Stream {
    state: crate::session::Dword,
}

impl Stream {
    const fn new() -> Self {
        Stream {
            state: crate::session::Dword::new(0x9570_1114),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut rc2k_core::math::random::Random) -> R) -> R {
        let mut random = rc2k_core::math::random::Random {
            state: self.state.get(),
        };
        let value = f(&mut random);
        self.state.set(random.state);
        value
    }

    /// The next number, as `next_random` on this stream.
    pub fn next(&self) -> u32 {
        self.with(rc2k_core::math::random::Random::draw)
    }

    /// A number below `limit`, as `math::random_below` on this stream.
    pub fn below(&self, limit: u32) -> Result<u32, crate::errors::BadInput> {
        if limit == 0 {
            return Err(crate::errors::BadInput(
                "a random number under 0: the original divides by it",
            ));
        }
        self.with(|r| r.below(limit))
            .ok_or(crate::errors::BadInput("unreachable: the limit is not 0"))
    }
}

/// The race sounds' numbers: drawn only on the game's thread (the sound turn, the race's sound
/// levels), so which sounds played no longer changes what the race and the drawing draw.
pub static SOUNDS: Stream = Stream::new();

/// The drawing's numbers (the mud thrown on the textures, the objects' shimmer), so the race
/// draws the same numbers whatever the frame rate. Drawn in the drawing's order: on a drawing
/// thread after the runs before its own.
static DRAWING: Stream = Stream::new();

/// A number for the drawing (`DRAWING`).
pub fn next_drawing_random() -> u32 {
    crate::workers::wait_for_earlier_runs();
    DRAWING.next()
}

/// The streams started from the game's state (after `math::seed_random`).
pub fn seed_streams() {
    for stream in [&SOUNDS, &DRAWING] {
        stream.state.set(STATE.get());
    }
}
