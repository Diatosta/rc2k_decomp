//! The HUD batch's variables (see `hud`), shared by its hooks.

use crate::framework::{game_addr, rva};
use crate::service::harness::{Region, at};

pub const VERTICES_RVA: usize = rva(0x75C324); // unk_75C324: 0x400 TLVertex
pub const VERTEX_ROOM: u32 = 0x400;
pub const INDICES_RVA: usize = rva(0x764324); // unk_764324: 0x600 u16
pub const INDEX_ROOM: u32 = 0x600;
pub const VERTEX_COUNT_RVA: usize = rva(0x764F24); // dword_764F24
pub const INDEX_COUNT_RVA: usize = rva(0x764F28); // dword_764F28
pub const TEXTURE_RVA: usize = rva(0x75C034); // dword_75C034: the texture asked for
pub const BATCH_TEXTURE_RVA: usize = rva(0x75C038); // dword_75C038: the batch's (-1 none)
pub const SHOWN_TEXTURE_RVA: usize = rva(0x6D14FE); // dword_6D14FE
pub const POLYGON_RVA: usize = rva(0x9605A0); // g_polygonVertices
pub const POLYGON_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount

/// Everything a batch step may write.
pub fn regions() -> Vec<Region> {
    vec![
        (
            game_addr(VERTICES_RVA) as u32,
            (VERTEX_ROOM * 0x20) as usize,
        ),
        (game_addr(INDICES_RVA) as u32, (INDEX_ROOM * 2) as usize),
        at(VERTEX_COUNT_RVA),
        at(INDEX_COUNT_RVA),
        at(BATCH_TEXTURE_RVA),
        at(SHOWN_TEXTURE_RVA),
        (game_addr(POLYGON_RVA) as u32, 0x80),
        at(POLYGON_COUNT_RVA),
    ]
}

/// `race::mud::run_both` for a converted float function: each case at the tests' control
/// words (`service::floats::at_test_precisions`), the registers and the calls the same,
/// `regions` the same but for the places of `floats` (exact at double precision, within
/// their tolerances at extended).
#[allow(clippy::too_many_arguments)] // the case, its copy, what is compared and how
pub fn run_both_floats(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &crate::redirect::RedirectedCopy,
    regions: &[Region],
    floats: &[(Region, crate::service::floats::Tolerance)],
    prepare: &dyn Fn(),
    before: crate::framework::Regs,
    rust: &dyn Fn(
        &crate::framework::Regs,
    ) -> Result<crate::framework::Regs, crate::errors::Refused>,
) {
    use crate::race::level_init::take_called;
    use crate::service::floats::{Agreement, Floats, at_test_precisions, compare_memory};
    use crate::service::harness::capture;
    let regs = |r: &crate::framework::Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let tolerant = Floats {
        memory: floats,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(regions));
        prepare();
        let mine = rust(&before);
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(regions),
        );
        let memory = compare_memory(regions, &theirs.2, &mine.2, held);
        t.check(
            theirs.0 == mine.0 && theirs.1 == mine.1 && memory != Agreement::Differ,
            || {
                format!(
                    "control word {word:#x}, {what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory {memory:?}",
                    theirs.0, theirs.1, mine.0, mine.1
                )
            },
        );
    });
}
