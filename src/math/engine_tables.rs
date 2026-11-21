//! The engine's lookup tables (in the race's buffers, `race::buffers_store`), made by
//! `scene::fill_engine_tables` at start: the sines (the cosines are the same table 0x400 places
//! on), the arctangents, the noise bytes and the shuffled words. About 0x28000 bytes, written once
//! at start, read by the physics on the game's thread and by every drawing part
//! (`session::DrawArc`, read through `EngineTablesRef`).

#[derive(Clone)]
pub struct EngineTables {
    /// 0x1400 floats, the sine of n times pi over 0x800; the cosines are this table from entry
    /// 0x400 (`math::rotate_scratch_about_y`, `scene::matrix`, `effects::`, `hud::draw_disc`).
    pub sines: [f32; 0x1400],
    /// 0x1000 words, the angle of a tangent (`scene::fill_engine_tables`).
    pub arctangents: [u16; 0x1000],
    /// 0x1070 bytes of noise; the noise maps are in it, at 0x20 and 0x50
    /// (`physics::probe_ground`, `physics::surface_noise`, `race::frame_draw`).
    pub noise_bytes: [u8; 0x1070],
    /// The numbers 0 .. 0xFFFF shuffled (`race::muddy_texel`).
    pub shuffled_words: [u16; 0x10000],
}

impl EngineTables {
    /// The tables, zeroed.
    pub const fn new() -> Self {
        Self {
            sines: [0.0; 0x1400],
            arctangents: [0; 0x1000],
            noise_bytes: [0; 0x1070],
            shuffled_words: [0; 0x10000],
        }
    }
}

/// The tables as one reader finds them (the game's thread's own, or a drawing part's copy).
pub type EngineTablesRef<'a> = crate::memory::drawn_table::DrawnTable<'a, EngineTables>;

/// The sine table's entry `n`: the callers index with an angle's top bits (below 0x1000), and the
/// cosine is the entry 0x400 on.
pub fn sine(tables: EngineTablesRef, n: usize) -> f32 {
    tables.read(|t| t.sines[n])
}

/// The cosine of table place `n` (the sine table's entry `n + 0x400`).
pub fn cosine(tables: EngineTablesRef, n: usize) -> f32 {
    sine(tables, n.wrapping_add(0x400))
}

/// The shuffled word `n` (below 0x10000).
pub fn shuffled_word(tables: EngineTablesRef, n: usize) -> u16 {
    tables.read(|t| t.shuffled_words[n])
}

/// The noise byte `n`; one past the 0x1070 bytes is refused, and logged (`what` names it).
pub fn noise_byte(
    tables: EngineTablesRef,
    n: usize,
    what: &str,
) -> Result<u8, crate::errors::BadInput> {
    tables
        .read(|t| t.noise_bytes.get(n).copied())
        .ok_or_else(|| {
            crate::memory::host_hooks::log(&format!(
                "session: {what}: 1 bytes at {n:#x} of a 0x1070-byte buffer"
            ));
            crate::errors::BadInput("past the end of a buffer")
        })
}
