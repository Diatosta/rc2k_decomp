//! `menu::championship_welcome`: the championship's welcome. Unless a saved game goes on
//! (`event.continuing_save` 1), the speech `;3files\%speech\rally\rally0.wav` is started
//! (`sound::start_stream`, at `options.menu_voices_volume`) and the commands waited for;
//! `loop_flags.event_left`, `loop_flags.leave_screen` and the yes-no answer are 0. The picture
//! `mob_welc.png` with the title bar (`gfx::load_picture_with_overlay`; a failed load is fatal
//! error 0x80712345, no message) is shown: a key is taken (dropped), one frame of it is run
//! (`screens::run_frame` with `menu::picture_frame`), then three seconds are given for a key
//! (`system::wait_or_key`). Without one, keys are taken until one comes while the speech plays
//! (`sound::stream_playing`) and the game is not quitting: the speech's end or the quitting
//! ends it there. A key other than Escape stops the speech and ends it; Escape stops it and
//! asks about leaving (`menu::confirm_leaving`): a yes stops it again and ends, a no starts
//! over.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self};

pub fn championship_welcome(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    Game::new(o).run(welcome::championship_welcome)
}
