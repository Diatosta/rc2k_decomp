//! The game's options, kept in `var/rc2k.toml`, and the old file they came from, `var\ral.dat`
//! (`saves::load_options`), whose byte format never changes.
//!
//! The model (`Options`) holds everything the old file holds, by area. `old_file` is the
//! old file's codec (bit-exact both ways), `toml_file` the new file's text, `files` the rule
//! for which file is read and written when:
//! - the new file wins whenever it exists;
//! - the old file is read only when the new one does not exist yet (once, to bring its
//!   values over), or when asked (a command-line switch, the settings page's button);
//! - the options are saved to the new file; to the old one only when asked.

pub mod files;
pub mod model;
pub mod old_file;
pub mod toml_file;

pub use model::Options;

/// What went wrong with an options file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionsError {
    /// The old file has its head but not its "####" at the end (the game's fatal error
    /// 0x80ACFF01, "ral.dat out of date"), or is too short for its pieces.
    OutOfDate,
    /// The new file is not TOML, or a value in it has the wrong type, length or text.
    Text(String),
    /// The new file's `version` is newer than this program knows.
    TooNew(i64),
    /// A file could not be read or written.
    Io(String),
}

impl std::fmt::Display for OptionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfDate => f.write_str("ral.dat out of date"),
            Self::Text(why) => write!(f, "rc2k.toml: {why}"),
            Self::TooNew(v) => write!(f, "rc2k.toml has version {v}, newer than this program's"),
            Self::Io(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for OptionsError {}
