//! `screens::service_box_frame_2` (0x491987): the frame (yellow 0xFFFF00) round box `dword_75AC90` (0 .. 4) of the first row, shown while it is the chosen one or (with `dword_75AA9C`) the cursor is on item 0xE.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `screens::draw_service_boxes`. The body is the original's code written out by
//! `service_boxes`' translator.

use crate::errors::Refused;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::state_store::MENU_STATE;
use crate::screens::service_boxes::quad;

/// The Rust version (see the header): the registers it leaves.
#[allow(unused_assignments, clippy::needless_late_init)] // the translator's flags, one per `cmp`
pub fn service_box_frame_2() -> Result<(), Refused> {
    // The original's eax: an item number or a value compared (its repair flags indexed by
    // it: the original's esi their address).
    let mut value = 0u32;
    let mut c: (u32, u32);
    c = (MENU_STATE.service_ui.box_shown.get(), 0x1);
    'l492528: {
        'l4919cf: {
            'l4919ad: {
                if c.0 != c.1 {
                    break 'l4919ad;
                }
                value = 0xE;
                c = (LOOP_FLAGS.item.get(), value);
                if c.0 != c.1 {
                    break 'l4919ad;
                }
                c = (DRAW_WORK.blink.get(), 0x1);
                if c.0 != c.1 {
                    break 'l492528;
                }
                break 'l4919cf;
            }
            value = MENU_STATE.driver_stats.setups[0].chosen.get();
            c = (MENU_STATE.driver_stats.setups[0].installed.get(), value);
            if c.0 != c.1 {
                break 'l4919cf;
            }
            value = 0xE;
            c = (MENU_LISTS.inspection_flags_a[value as usize].get(), 0x1);
            if c.0 != c.1 {
                break 'l492528;
            }
        }
        c = (MENU_STATE.driver_stats.setups[0].chosen.get(), 0x0);
        'l491c14: {
            if c.0 != c.1 {
                break 'l491c14;
            }
            quad(0x44, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x44, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF)?;
            quad(0x85, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF)?;
            quad(0x44, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x45, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x45, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF)?;
            quad(0x84, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF)?;
            quad(0x45, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
        }
        c = (MENU_STATE.driver_stats.setups[0].chosen.get(), 0x1);
        'l491e59: {
            if c.0 != c.1 {
                break 'l491e59;
            }
            quad(0x86, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x86, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF)?;
            quad(0xC7, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF)?;
            quad(0x86, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x87, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x87, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF)?;
            quad(0xC6, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF)?;
            quad(0x87, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
        }
        c = (MENU_STATE.driver_stats.setups[0].chosen.get(), 0x2);
        'l49209e: {
            if c.0 != c.1 {
                break 'l49209e;
            }
            quad(0xC8, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0xC8, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF)?;
            quad(0x109, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF)?;
            quad(0xC8, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0xC9, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
            quad(0xC9, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF)?;
            quad(0x108, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF)?;
            quad(0xC9, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
        }
        c = (MENU_STATE.driver_stats.setups[0].chosen.get(), 0x3);
        'l4922e3: {
            if c.0 != c.1 {
                break 'l4922e3;
            }
            quad(0x10A, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x10A, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF)?;
            quad(0x14B, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF)?;
            quad(0x10A, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x10B, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
            quad(0x10B, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF)?;
            quad(0x14A, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF)?;
            quad(0x10B, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
        }
        c = (MENU_STATE.driver_stats.setups[0].chosen.get(), 0x4);
        if c.0 != c.1 {
            break 'l492528;
        }
        quad(0x14C, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF)?;
        quad(0x14C, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF)?;
        quad(0x18D, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF)?;
        quad(0x14C, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF)?;
        quad(0x14D, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
        quad(0x14D, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF)?;
        quad(0x18C, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF)?;
        quad(0x14D, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF)?;
    }
    Ok(())
}
