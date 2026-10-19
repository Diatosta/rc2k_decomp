//! The physics patches a replay was recorded with (IDA 2026-10-19), and the self-test of
//! the two hooks:
//! - `race::apply_replay_patches` (0x41F2C9): when the replay's header has them
//!   (`dword_719C54`) and they are not in force (`dword_5C2C94` 0): in force, the game's
//!   switches (`g_patchSwitch`, 13 dwords) kept in `unk_5C2C98`, the replay's
//!   (`unk_719C58`) put in their place, `g_patchesApplied` 0 and the patches applied
//!   (`physics::apply_patches`).
//! - `race::restore_replay_patches` (0x41F31D): when the header has them and they are in
//!   force: no longer, the kept switches back, applied again.
//!
//! Both keep every register (`pusha` / `popa`). Their call through `part` (1
//! `physics::apply_patches`; in the game its Rust version).

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{Part, Saved, dword, fill, get, move_dwords, put, region, run_both};
use crate::service::harness::Region;

const IN_HEADER: usize = 0x719C54; // dword_719C54
const IN_FORCE: usize = 0x5C2C94; // dword_5C2C94
const SWITCHES: usize = 0x71C6D0; // g_patchSwitch, 13 dwords
const KEPT: usize = 0x5C2C98; // unk_5C2C98
const REPLAYS: usize = 0x719C58; // unk_719C58
const APPLIED: usize = 0x71C72C; // g_patchesApplied

/// `apply_replay_patches`.
pub fn apply_replay_patches(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if get(IN_HEADER) != 0 && get(IN_FORCE) == 0 {
        put(IN_FORCE, 1);
        move_dwords(KEPT, SWITCHES, 13);
        move_dwords(SWITCHES, REPLAYS, 13);
        put(APPLIED, 0);
        let mut r = Regs {
            ecx: 0,
            esi: crate::physics::flow::a(REPLAYS) + 0x34,
            edi: crate::physics::flow::a(SWITCHES) + 0x34,
            ..*entry
        };
        part(1, &mut r)?;
    }
    Ok(*entry)
}

/// `restore_replay_patches`.
pub fn restore_replay_patches(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if get(IN_HEADER) != 0 && get(IN_FORCE) != 0 {
        put(IN_FORCE, 0);
        move_dwords(SWITCHES, KEPT, 13);
        put(APPLIED, 0);
        let mut r = Regs {
            ecx: 0,
            esi: crate::physics::flow::a(KEPT) + 0x34,
            edi: crate::physics::flow::a(SWITCHES) + 0x34,
            ..*entry
        };
        part(1, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: `physics::apply_patches`.
pub fn game_part(_place: u32, _r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::physics::apply_patches::apply_patches() };
    Ok(())
}

pub fn regions() -> Vec<Region> {
    vec![
        dword(IN_FORCE),
        region(SWITCHES, 0x34),
        region(KEPT, 0x34),
        dword(APPLIED),
    ]
}

/// The self-test of either: a copy whose call goes to a stand-in, against the Rust
/// version; in the header or not, in force or not.
pub fn test_patches(
    hook: &crate::framework::RawHook,
    address: usize,
    len: usize,
    call: usize,
    rust: &dyn Fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    let Some(mut t) = crate::framework::SelfTest::begin(hook) else {
        return (0, 0);
    };
    // pusha; cmp dword_719C54, 0
    let copy = match crate::race::viewer::copy(
        address,
        len,
        &[0x60, 0x83, 0x3D, 0x54, 0x9C, 0x71, 0x00, 0x00],
        &[call],
        &[(0x4648BD, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let mut all = regions();
    all.extend([dword(IN_HEADER), region(REPLAYS, 0x34)]);
    let saved = Saved::new(&all);
    for (header, force) in [(0u32, 0u32), (0, 1), (5, 0), (5, 1)] {
        run_both(
            &mut t,
            &format!("in the header {header}, in force {force}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions(),
            0,
            &|| {
                put(IN_HEADER, header);
                put(IN_FORCE, force);
                fill(SWITCHES, 0x34, 14);
                fill(KEPT, 0x34, 15);
                fill(REPLAYS, 0x34, 16);
                put(APPLIED, 1);
            },
            &|r| rust(r, &mut crate::race::viewer::test_part),
        );
    }
    drop(saved);
    t.finish()
}
