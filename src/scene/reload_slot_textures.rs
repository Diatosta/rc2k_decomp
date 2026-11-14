//! `scene::reload_slot_textures`: every texture slot is loaded anew ("reload video plates").
//!
//! Two lines in the game's log (a rule and "reload video plates - start");
//! `scene::clear_slot_textures`; `scene::load_wanted_textures`; two lines more ("reload video
//! plates - finis" and a rule). Two callers (the front end's 3D set-up and a race's).

use crate::errors::BadInput;
use crate::race::owner_frame::Frame;
use crate::scene::clear_slot_textures::clear_slot_textures;
use crate::scene::load_wanted_textures::load_wanted_textures;

/// Its calls in order: two log lines, the slots cleared, the wanted textures loaded, two
/// log lines.
pub fn reload_slot_textures(
    frame: &Frame,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    crate::logfile::log_write::log_nul(c"------------------------------------", true, 1);
    crate::logfile::log_write::log_nul(c"reload video plates - start", true, 1);
    clear_slot_textures(scene);
    crate::race::load_times::mark("load_wanted_textures");
    load_wanted_textures(frame, scene)?;
    crate::logfile::log_write::log_nul(c"reload video plates - finis", true, 1);
    crate::logfile::log_write::log_nul(c"------------------------------------", true, 1);
    Ok(())
}
