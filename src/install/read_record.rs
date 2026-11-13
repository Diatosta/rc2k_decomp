//! Reads the installation's record `var\ral.zog`: a settings text with three keys, each taken if
//! it is there: `cdrom` becomes base directory A, `installation` the media mode (held to
//! 0 .. 3) and `language` what `%` stands for in a path. The two texts get a `\` after them
//! unless they are empty or end in `\` or `/`. Without the file nothing changes. Called by
//! `start`, `install::run_installer`, `install::check_clio_disc`. Refuses a text of 0xFE bytes
//! or more.

use crate::errors::BadInput;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

pub fn read_record(system: &crate::system::owner::System, path: &[u8]) -> Result<(), BadInput> {
    {
        let Some(text) = read_text(system, path)? else {
            return Ok(());
        };
        let record = config::read_record(&text, &case_table())?;
        for (directory, text) in [(0, record.base), (1, record.locale)] {
            if let Some(text) = text {
                crate::files::state::write_directory(directory, &text, false, "an installer path")?;
            }
        }
        if let Some(mode) = record.mode {
            crate::files::state::MEDIA_MODE.set(mode);
        }
        Ok(())
    }
}
