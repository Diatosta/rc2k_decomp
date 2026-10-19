//! `input::begin_force` (0x40945D): a device's force feedback set up, when its controller
//! has it (the controller's entry, `g_controllers` + 0x50 its number, +0x1C 1): "INP:
//! using force feedback" logged (level 2); the `DIPROPDWORD` (`dword_4F6AA4`: 0x14, 0x10,
//! the device, 0); with the record's strength (+0xC) 0 only the self-centring turned on
//! (`DIPROP_AUTOCENTER` 1); else the self-centring off and the gain the strength x 100
//! (`DIPROP_FFGAIN`), each failure a fatal error (0x80238421, 0x80238422), the device's
//! effects listed (`EnumEffects` with `input::effect_found`, the record as context), the
//! six made (`input::make_effect`, ecx the number, edi the effect) and +0x28 1 (effects
//! on).
//!
//! In:  esi = the device's record (+4 its DirectInput device)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::acquire_device`. The controller's number is not checked (as the
//! original's). The effects' functions through `make_effect`'s `part`; the methods the
//! device's own. Shadow mode runs the original alone (it calls DirectInput). Self-test: the
//! original and the Rust version on DirectInput objects made in Rust (`input::di_harness`)
//! and a copy of the first device's record: a controller with no force feedback, the wheel
//! with strength 0 and 50.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::input::devices::{RECORD_LEN, RECORDS, at, put};
use crate::input::di_harness::{
    DWORD_PROP_RVA, EFFECT_LEN, EFFECTS_AT, a, compare, entry_of, fatal, method, regs_text,
    shared_regions, slot,
};
use crate::input::types::{Controller, DiPropDword, DiPropHeader, InputDevice};
use std::mem::size_of;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::begin_force", rva(0x40945D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const USING_TEXT_RVA: usize = rva(0x4DA5D8); // aInpUsingForceF
const EFFECT_FOUND: usize = 0x40953B; // input::effect_found
const DIPROP_FFGAIN: u32 = 7;
const DIPROP_AUTOCENTER: u32 = 9;

/// The `DIPROPDWORD` for the whole device with `data`: its address.
pub fn dword_property(data: u32) -> u32 {
    let prop = a(DWORD_PROP_RVA);
    put(
        prop,
        field!(DiPropDword, header.size),
        size_of::<DiPropDword>() as u32,
    );
    put(
        prop,
        field!(DiPropDword, header.header_size),
        size_of::<DiPropHeader>() as u32,
    );
    put(prop, field!(DiPropDword, header.object), 0);
    put(prop, field!(DiPropDword, header.how), 0);
    put(prop, field!(DiPropDword, data), data);
    prop
}

/// The Rust version (see the header).
pub fn begin_force(record: u32, part: crate::input::make_effect::Part) -> Result<(), Refused> {
    let entry = entry_of(record);
    if at(entry, field!(Controller, force_feedback)) != 1 {
        return Ok(());
    }
    unsafe {
        crate::logfile::log_write::log_write(game_addr(USING_TEXT_RVA) as *const u8, true, 2)
    };
    let device = at(record, field!(InputDevice, device));
    if at(record, field!(InputDevice, strength)) == 0 {
        method(
            device,
            slot::SET_PROPERTY,
            &[DIPROP_AUTOCENTER, dword_property(1)],
        );
        return Ok(());
    }
    if method(
        device,
        slot::SET_PROPERTY,
        &[DIPROP_AUTOCENTER, dword_property(0)],
    ) != 0
    {
        return Err(fatal(0x8023_8421).into());
    }
    let gain = dword_property(at(record, field!(InputDevice, strength)).wrapping_mul(100));
    if method(device, slot::SET_PROPERTY, &[DIPROP_FFGAIN, gain]) != 0 {
        return Err(fatal(0x8023_8422).into());
    }
    let found = game_addr(rva(EFFECT_FOUND)) as u32;
    method(device, slot::ENUM_EFFECTS, &[found, record, 0]);
    for n in 0..6 {
        let r = Regs {
            ecx: n,
            esi: record,
            edi: record.wrapping_add(EFFECTS_AT + n * EFFECT_LEN),
            ..Regs::default()
        };
        crate::input::make_effect::make_effect(&r, part)?;
    }
    put(record, field!(InputDevice, effects_on), 1);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = begin_force(r.esi, &mut crate::input::make_effect::game_part) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// A test controller's entry (number 2 of the table) with force feedback or not, and a
/// record of `start` for it with the device and the strength given.
pub fn test_controller(rec: u32, start: &[u8], device: usize, force: bool, strength: u32) {
    unsafe { std::ptr::copy_nonoverlapping(start.as_ptr(), rec as *mut u8, RECORD_LEN) };
    put(rec, field!(InputDevice, controller), 2);
    put(
        entry_of(rec),
        field!(Controller, force_feedback),
        force as u32,
    );
    put(
        rec,
        field!(InputDevice, device),
        crate::input::di_objects::test_device(device),
    );
    put(rec, field!(InputDevice, strength), strength);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(
        crate::input::di_objects::test_devices(),
        Default::default(),
    );
    let start =
        unsafe { std::slice::from_raw_parts(game_addr(RECORDS[0]) as *const u8, RECORD_LEN) }
            .to_vec();
    let mut record = start.clone();
    let rec = record.as_mut_ptr() as u32;
    let mut regions = shared_regions();
    regions.push((rec, RECORD_LEN));
    for (device, force, strength) in [(0usize, false, 50u32), (1, true, 0), (1, true, 50)] {
        let entry = Regs {
            esi: rec,
            ..Regs::SENTINEL
        };
        compare(
            &mut t,
            &format!("device {device}, force feedback {force}, strength {strength}"),
            &world,
            &regions,
            &|| test_controller(rec, &start, device, force, strength),
            &mut || {
                let mut r = entry;
                unsafe { call_original(&mut r) };
                regs_text(&r)
            },
            &mut || {
                let done = begin_force(rec, &mut crate::input::make_effect::game_part);
                format!(
                    "{}{}",
                    regs_text(&entry),
                    if done.is_ok() { "" } else { " refused" }
                )
            },
        );
    }
    drop(world);
    drop(record);
    t.finish()
}
