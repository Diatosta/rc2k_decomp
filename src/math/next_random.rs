//! `math::next_random`: the game's random numbers.
//!
//! One step of the generator on its state: the low byte is changed by three bits of the
//! number (bit 22, then bit 2 and bit 1 of the byte as it then is, each flipping bit 0),
//! the whole is turned right by one, and what is mixed in is `xor`ed on.

/// The game's random numbers: the main generator and the two streams of their own for the race
/// sounds and the drawing. Held by the race (`RaceShared::random`) and handed to what draws.
pub struct Randoms {
    /// The main generator's state, starting at the game's first value until `seed_random`.
    pub state: crate::memory::session::Dword,
    /// The race sounds' numbers: drawn only on the game's thread (the sound turn, the race's sound
    /// levels), so which sounds played no longer changes what the race and the drawing draw.
    pub sounds: Stream,
    /// The drawing's numbers (the mud thrown on the textures, the objects' shimmer), so the race
    /// draws the same numbers whatever the frame rate. Drawn in the drawing's order: on a drawing
    /// thread after the runs before its own (`next_drawing`).
    drawing: Stream,
}

impl Randoms {
    pub const fn new() -> Self {
        Randoms {
            state: crate::memory::session::Dword::new(0x9570_1114),
            sounds: Stream::new(),
            drawing: Stream::new(),
        }
    }

    /// The generator loaded from the state, `f` run on it, its state stored back.
    pub fn with_random<R>(&self, f: impl FnOnce(&mut rc2k_core::math::random::Random) -> R) -> R {
        let mut random = rc2k_core::math::random::Random {
            state: self.state.get(),
        };
        let value = f(&mut random);
        self.state.set(random.state);
        value
    }

    /// The next number. On a drawing thread it comes after the runs before its own, so the
    /// numbers follow the list's order.
    pub fn next_random(&self) -> u32 {
        crate::scene::workers::wait_for_earlier_runs();
        self.with_random(rc2k_core::math::random::Random::draw)
    }

    /// A number under `limit`; a limit of 0 is refused.
    pub fn random_below(&self, limit: u32) -> Result<u32, crate::errors::BadInput> {
        if limit == 0 {
            return Err(crate::errors::BadInput(
                "a random number under 0: the original divides by it",
            ));
        }
        self.with_random(|r| r.below(limit))
            .ok_or(crate::errors::BadInput("unreachable: the limit is not 0"))
    }

    /// A number for the drawing, after the runs before this thread's.
    pub fn next_drawing(&self) -> u32 {
        crate::scene::workers::wait_for_earlier_runs();
        self.drawing.next()
    }

    /// The streams started from the main generator's state (after `math::seed_random`).
    pub fn seed_streams(&self) {
        for stream in [&self.sounds, &self.drawing] {
            stream.state.set(self.state.get());
        }
    }
}

/// A generator of its own for one part of the game, so how many numbers one part takes no
/// longer shifts every number the others draw (wiki: "The race's sounds steer the race's
/// random numbers", "The pictures steer the race's random numbers"). The same generator
/// as the main one, started from the game's state (`Randoms::seed_streams`).
pub struct Stream {
    state: crate::memory::session::Dword,
}

impl Stream {
    const fn new() -> Self {
        Stream {
            state: crate::memory::session::Dword::new(0x9570_1114),
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

    /// A number below `limit`, as `Randoms::random_below` on this stream.
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
