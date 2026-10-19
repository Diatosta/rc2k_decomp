//! The self-tests of the pure page actions (`install::mark_main_page` .. `finish_page`): a
//! list of the test's and the globals they read set to every mix that matters, the original
//! against the Rust version.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest};
use crate::install::installer::{
    IGNORE_ROOM, LIST_ANSWER, LIST_LEAVE, MEDIA_MODE, OFFER_REGISTRATION, PATH_PROBLEM,
    RUN_DXSETUP, SAME_VERSION, STATUS, a, p, pb, region,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original};
use crate::screens::harness::CallOriginal;
use crate::service::harness::Region;

/// The globals the page actions touch, as a region.
pub fn globals() -> Vec<Region> {
    [
        SAME_VERSION,
        STATUS,
        OFFER_REGISTRATION,
        MEDIA_MODE,
        PATH_PROBLEM,
        IGNORE_ROOM,
        RUN_DXSETUP,
        LIST_ANSWER,
        LIST_LEAVE,
        0x51BE21,
        0x51D407,
    ]
    .iter()
    .map(|&x| region(x, 4))
    .chain(std::iter::once(region(0x95EBF8, 2)))
    .collect()
}

/// Runs one page action on a list of two items, with `ebx` (the state) each of -1, 0, 1, and
/// the globals in several mixes; the registers, ZF, CF and the globals compared.
pub fn run(
    t: &mut SelfTest,
    call_original: CallOriginal,
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    let list = TestList::new();
    let mut regions = globals();
    regions.push(list.region());
    let saved = Saved::new(&regions);
    for (same, status, offer) in [(0u32, 0u32, 0u32), (1, 0, 1), (0, 1, 0), (1, 1, 1)] {
        for mode in [0u32, 1, 2, 3] {
            for problem in [0u32, 1, 2] {
                for answer in [0u32, 1, 2] {
                    for ebx in [0u32, 1, 0xFFFF_FFFF] {
                        let prepare = || {
                            list.set(0, &[1, 1, 1, 1, 1, 1, 1, 1], 0, 0, 0, 0, 0, b"");
                            // the main page reads [edi+2Ah] as a word
                            crate::install::installer::ww(list.base() + 0x2A, 0xBD);
                            p(SAME_VERSION, same);
                            pb(STATUS, status as u8);
                            p(OFFER_REGISTRATION, offer);
                            p(MEDIA_MODE, mode);
                            p(PATH_PROBLEM, problem);
                            pb(LIST_ANSWER, answer as u8);
                            pb(LIST_LEAVE, 0);
                            p(IGNORE_ROOM, 0);
                            p(RUN_DXSETUP, 0);
                            pb(0x51BE21, 0);
                            // the drive page reads byte_95EBF8 and the drive texts
                            pb(0x95EBF8, b'c');
                            for d in 0..26u32 {
                                pb(0x52A30F + 0x10 * d as usize, b'a' + d as u8);
                            }
                            let _ = a(0);
                        };
                        prepare();
                        let before = Regs {
                            ebx,
                            edi: list.base(),
                            ..Regs::SENTINEL
                        };
                        check_original(
                            t,
                            &format!(
                                "same {same}, status {status}, offer {offer}, mode {mode}, \
                                 problem {problem}, answer {answer}, ebx {ebx:#x}"
                            ),
                            call_original,
                            before,
                            &regions,
                            0,
                            &prepare,
                            rust,
                        );
                    }
                }
            }
        }
    }
    drop(saved);
}
