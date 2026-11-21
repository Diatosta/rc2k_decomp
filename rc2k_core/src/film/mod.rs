//! The films' decoders in plain Rust, giving what FFmpeg's decoders give bit for bit: MPEG-1
//! video and MPEG audio layer II (the two `.MPG` films), and Indeo 4 (the `.AVI` logo). Packets of
//! a film's streams in, pictures and samples out.

pub mod indeo4;
pub mod mp2;
mod mp2_tables;
pub mod mpeg1_video;
mod mpeg1_video_tables;
mod mpeg_bits;
pub mod mpeg_film;

pub use mpeg_bits::FilmError;
