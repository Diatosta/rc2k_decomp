use crate::framework::{game_ptr, game_read, rva};

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const DRAWTRI_RVA: usize = rva(0x4DD9AD); // aDrawtri_0

/// The Rust version (see the header). What it leaves in the registers (eax the device's
/// result, the zero and carry flags) is read by none of its callers (register liveness,
/// 2026-10-23: the 24 sites through 0x44EA65 read only SF, which the original's `cmp`
/// leaves and the Rust version never set).
pub fn draw_fan() {
    if unsafe { game_read::<u32>(SWITCH_RVA) } == 0 {
        return;
    }
    let result = crate::d3d::draw_fan::draw_fan();
    if result != 0 {
        unsafe { crate::logfile::log_write::log_write(game_ptr(DRAWTRI_RVA), false, 1) };
        crate::logfile::log_hex8::log_hex8(result, false, 1);
    }
}
