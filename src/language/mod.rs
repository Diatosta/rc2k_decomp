//! Loading the game's language: its words, units, keyboard and letter cases. The language file's
//! lexicon is compiled here (`compile_lexicon`, recompiled on a developer's key by
//! `maybe_compile_lexicon`), the locale and keyboard files are read (`load_language`) and the
//! built-in texts of the pages the game adds are picked (`own_texts`). The loaded state lives in
//! `strings`.

pub mod compile_lexicon;
pub mod load_language;
pub mod maybe_compile_lexicon;
pub mod own_texts;
