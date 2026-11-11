//! A screenshot taken, and the fatal error's own font loaded (from
//! `gfx::take_screenshot` 0x407897 and `gfx::load_jade` 0x4075F3).

/// What `gfx::take_screenshot` is made of.
pub trait Shooting {
    type Error;
    /// `g_screenshotsOn`.
    fn screenshots_on(&mut self) -> u32;
    /// `g_surfaceLocked`.
    fn surface_locked(&mut self) -> u32;
    /// `dword_4F52B8`: 1 while the lock is this function's.
    fn locked_here(&mut self) -> u32;
    fn set_locked_here(&mut self, value: u32);
    /// `gfx::lock_surface`: true for its carry (failed).
    fn lock(&mut self) -> Result<bool, Self::Error>;
    /// `gfx::write_screen_pcx`.
    fn write(&mut self) -> Result<(), Self::Error>;
    /// `gfx::unlock_surface`.
    fn unlock(&mut self) -> Result<(), Self::Error>;
}

/// With screenshots on: the surface locked unless it is (a failed lock ends it), written,
/// unlocked again if it was locked here; the "locked here" switch 0 at the end.
pub fn take_screenshot<S: Shooting>(s: &mut S) -> Result<(), S::Error> {
    if s.screenshots_on() == 0 {
        return Ok(());
    }
    if s.surface_locked() != 1 {
        if s.lock()? {
            s.set_locked_here(0);
            return Ok(());
        }
        s.set_locked_here(1);
    }
    s.write()?;
    if s.locked_here() == 1 {
        s.unlock()?;
    }
    s.set_locked_here(0);
    Ok(())
}

/// The jade font's room: its file must be smaller.
pub const JADE_ROOM: u32 = 0x1400;
/// The fatal errors: the file could not be read; it filled the room.
pub const JADE_UNREAD: u32 = 0x80F0_B0D1;
pub const JADE_TOO_BIG: u32 = 0x80F0_B0D2;
/// The jade fonts' colour (0x00RRGGBB), in the surface's format.
pub const JADE_COLOUR: u32 = 0xFF_FFFF;

/// `gfx::load_jade`: a read (whether it worked, the bytes it read) -> the fatal error it
/// is, if any.
pub fn jade_read(ok: bool, bytes: Option<u32>) -> Result<(), u32> {
    if !ok {
        Err(JADE_UNREAD)
    } else if bytes == Some(JADE_ROOM) {
        Err(JADE_TOO_BIG)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        on: u32,
        locked: u32,
        here: u32,
        lock_fails: bool,
        calls: String,
    }

    impl Shooting for Fake {
        type Error = ();
        fn screenshots_on(&mut self) -> u32 {
            self.on
        }
        fn surface_locked(&mut self) -> u32 {
            self.locked
        }
        fn locked_here(&mut self) -> u32 {
            self.here
        }
        fn set_locked_here(&mut self, value: u32) {
            self.here = value;
        }
        fn lock(&mut self) -> Result<bool, ()> {
            self.calls.push('l');
            Ok(self.lock_fails)
        }
        fn write(&mut self) -> Result<(), ()> {
            self.calls.push('w');
            Ok(())
        }
        fn unlock(&mut self) -> Result<(), ()> {
            self.calls.push('u');
            Ok(())
        }
    }

    fn run(on: u32, locked: u32, lock_fails: bool) -> (String, u32) {
        let mut f = Fake {
            on,
            locked,
            here: 7,
            lock_fails,
            calls: String::new(),
        };
        take_screenshot(&mut f).unwrap();
        (f.calls, f.here)
    }

    #[test]
    fn screenshots() {
        assert_eq!(run(0, 0, false), (String::new(), 7));
        assert_eq!(run(1, 0, false), ("lwu".into(), 0));
        assert_eq!(run(1, 0, true), ("l".into(), 0));
        assert_eq!(run(1, 1, false), ("w".into(), 0));
        assert_eq!(jade_read(false, None), Err(JADE_UNREAD));
        assert_eq!(jade_read(true, Some(0x1400)), Err(JADE_TOO_BIG));
        assert_eq!(jade_read(true, Some(0x13FF)), Ok(()));
    }
}
