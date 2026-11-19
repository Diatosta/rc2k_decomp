//! Going back to the main menu after a trouble the game can recover from (`Refused::Recoverable`).
//!
//! The trouble travels up with `?` where the callers can carry it, and as an unwind (`back`)
//! where they cannot (a menu item's action, a flow's helper). `guarded` catches both where a
//! screen or a race is run: the race there tears itself down first, and the front end
//! (`menu::run_front_end`) asks the player, then runs the main menu again.
//!
//! The unwind only travels up the game's thread through the menus' and the races' own
//! frames; where no front end is running (`armed` is false) a trouble ends the program as an
//! error does.

use crate::errors::Refused;
use std::sync::atomic::{AtomicU32, Ordering};

/// The unwind's payload: why the game goes back.
struct BackToMenu(String);

/// How many front ends are running (guards up) on the game's thread.
static ARMED: AtomicU32 = AtomicU32::new(0);

/// Whether a `guarded` call is up, so that an unwind would be caught.
pub fn armed() -> bool {
    ARMED.load(Ordering::SeqCst) > 0
}

/// Leaves everything between here and the nearest `guarded` call; `reason` is what the player
/// is told. The program ends instead where nothing would catch it.
pub fn back(reason: &str) -> ! {
    crate::program_log::log(&format!("recover: back to the main menu: {reason}"));
    if !armed() || cfg!(test) {
        crate::standalone::finish(&format!("an error with nowhere to go back to: {reason}"), 1);
    }
    std::panic::resume_unwind(Box::new(BackToMenu(reason.to_string())))
}

/// Runs `f`; a trouble that can be gone back from (a `Recoverable` result, or a `back` from
/// anywhere inside) comes out as `Err(reason)`. Any other result, or a panic, passes through.
pub fn guarded<T>(f: impl FnOnce() -> Result<T, Refused>) -> Result<Result<T, Refused>, String> {
    struct Armed;
    impl Armed {
        fn up() -> Self {
            ARMED.fetch_add(1, Ordering::SeqCst);
            Armed
        }
    }
    impl Drop for Armed {
        fn drop(&mut self) {
            ARMED.fetch_sub(1, Ordering::SeqCst);
        }
    }
    let _armed = Armed::up();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Err(Refused::Recoverable(r))) => Err(r.0.to_string()),
        Ok(other) => Ok(other),
        Err(payload) => match payload.downcast::<BackToMenu>() {
            Ok(back) => Err(back.0),
            Err(other) => std::panic::resume_unwind(other),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Recover;

    #[test]
    fn a_result_or_an_unwind_comes_out_as_the_reason() {
        let result: Result<Result<(), Refused>, String> =
            guarded(|| Err(Recover("a bad stage").into()));
        assert_eq!(result.unwrap_err(), "a bad stage");
        let unwound: Result<Result<(), Refused>, String> = guarded(|| {
            std::panic::resume_unwind(Box::new(BackToMenu("deep inside".into())));
        });
        assert_eq!(unwound.unwrap_err(), "deep inside");
        assert!(!armed());
        let fine: Result<Result<u32, Refused>, String> = guarded(|| Ok(3));
        assert_eq!(fine.unwrap().unwrap(), 3);
    }
}
