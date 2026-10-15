//! A 3D buffer's property sets (IDA, 2026-10-15): DirectSound's voice manager ("DSV",
//! setting `ignoredsv`) and EAX 1's reverb ("EAX", `ignoreeax`), each opened once through
//! the first 3D buffer made (`IKsPropertySet` from its `QueryInterface`), and the EAX room.

use crate::audio::com::method;
use crate::audio::voices::{EAX_RVA, VOICE_MANAGER_RVA, read, write};
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};

pub const IID_PROPERTY_SET_RVA: usize = rva(0x506A78); // g_iidKsPropertySet
pub const VOICE_MANAGER_SET_RVA: usize = rva(0x506CC8); // g_voiceManagerSet
pub const EAX_REVERB_SET_RVA: usize = rva(0x506DAC); // g_eaxReverbSet
pub const EAX_ROOMS_RVA: usize = rva(0x506DCC); // g_eaxRooms: 10 of 0x10 bytes
pub const IGNORE_DSV_RVA: usize = rva(0x506C50); // g_ignoreDsvSetting
pub const IGNORE_EAX_RVA: usize = rva(0x506C6C); // g_ignoreEaxSetting
pub const VOICE_MANAGER_ON_RVA: usize = rva(0x506C54); // g_voiceManagerOn
pub const HAS_EAX_RVA: usize = rva(0x506C70); // g_hasEax
const USING_DSV_TEXT: usize = rva(0x4DA73E); // "AUD: Using DSV"
const USING_EAX_TEXT: usize = rva(0x4DA7AA); // "AUD: Using EAX"
const ROOMS: u32 = 10;
/// `IKsPropertySet`'s `Set` and `QuerySupport`.
const SET: u32 = 0x10;
const QUERY_SUPPORT: u32 = 0x14;
/// `KSPROPERTY_SUPPORT_GET | KSPROPERTY_SUPPORT_SET`.
const GET_AND_SET: u32 = 3;
/// `DSPROPERTY_VMANAGER_MODE`'s value 2 (`DSPROPERTY_VMANAGER_MODE_REPORT`).
const VOICE_MANAGER_MODE: u32 = 2;

/// A text to the log (level 1, with a line's end), as `sound_device::Part`'s place 6.
pub type Log<'a> = &'a mut dyn FnMut(u32);

pub fn game_log(text: u32) {
    let _ = crate::audio::sound_device::game_part(6, text);
}

/// Where one opening's things are.
#[derive(Clone, Copy)]
pub struct Opening {
    /// The `ignore...` setting.
    pub setting: u32,
    /// Where the property set goes.
    pub object: u32,
    /// Set to 1 when it is there.
    pub found: u32,
    /// Which set; the text logged.
    pub set: usize,
    pub text: usize,
    /// The voice manager's mode is set; EAX's nothing.
    pub mode: bool,
}

impl Opening {
    pub fn voice_manager() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            setting: a(IGNORE_DSV_RVA),
            object: a(VOICE_MANAGER_RVA),
            found: a(VOICE_MANAGER_ON_RVA),
            set: VOICE_MANAGER_SET_RVA,
            text: USING_DSV_TEXT,
            mode: true,
        }
    }

    pub fn eax() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            setting: a(IGNORE_EAX_RVA),
            object: a(EAX_RVA),
            found: a(HAS_EAX_RVA),
            set: EAX_REVERB_SET_RVA,
            text: USING_EAX_TEXT,
            mode: false,
        }
    }
}

/// `audio::open_voice_manager` and `audio::open_eax`: true for the carry clear. `holder`
/// (ebx) holds the 3D buffer. Nothing with the setting on or the set already opened;
/// else the set from the 3D buffer's `QueryInterface`, its `QuerySupport` (get and set
/// both), for the voice manager its mode set to 2; then logged and `found` 1. A failure
/// leaves the set where it went (it is released at the close). Refuses a holder that is
/// no pointer (the original reads through it: `make_buffer`'s EAX bug, `notes/bugs.md`).
pub unsafe fn open_property_set(o: &Opening, holder: u32, log: Log) -> Result<bool, Refused> {
    unsafe {
        if read(o.setting) != 0 || read(o.object) != 0 {
            return Ok(false);
        }
        if holder < 0x1_0000 {
            return Err(BadInput("ebx holds no 3D buffer (make_buffer's EAX bug)").into());
        }
        let buffer = read(holder);
        if buffer == 0 {
            return Ok(false);
        }
        let query: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(method(buffer, 0));
        if query(buffer, game_addr(IID_PROPERTY_SET_RVA) as u32, o.object) != 0 {
            return Ok(false);
        }
        let p = read(o.object);
        let set = game_addr(o.set) as u32;
        let mut support = 0u32;
        let query_support: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(method(p, QUERY_SUPPORT));
        if query_support(p, set, 0, &mut support as *mut u32 as u32) != 0
            || support & GET_AND_SET != GET_AND_SET
        {
            return Ok(false);
        }
        if o.mode {
            let value = VOICE_MANAGER_MODE;
            if set_property(p, set, &value as *const u32 as u32, 4) != 0 {
                return Ok(false);
            }
        }
        log(game_addr(o.text) as u32);
        write(o.found, 1);
        Ok(true)
    }
}

/// `IKsPropertySet::Set` of property 0, no instance data.
unsafe fn set_property(p: u32, set: u32, data: u32, bytes: u32) -> i32 {
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(method(p, SET));
        f(p, set, 0, 0, 0, data, bytes)
    }
}

/// `audio::set_eax_room`: the room's reverb (`g_eaxRooms`) set when EAX is open; eax the
/// room's entry. Refuses a room past the table (the original reads past it).
pub unsafe fn set_eax_room(room: u32, properties: u32) -> Result<u32, Refused> {
    if room >= ROOMS {
        return Err(BadInput("an EAX room past the table").into());
    }
    let at = game_addr(EAX_ROOMS_RVA) as u32 + room * 0x10;
    unsafe {
        let p = read(properties);
        if p != 0 {
            set_property(p, game_addr(EAX_REVERB_SET_RVA) as u32, at, 0x10);
        }
    }
    Ok(at)
}

// ---- for the self-tests ----

static LOGGED: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

fn note_log(_: u32, r: &mut crate::framework::Regs) {
    LOGGED.lock().unwrap().push(r.eax);
}

/// The self-test of the two openings: a redirected copy of `copy` whose setting, set and
/// found flag are the test's and whose log call goes to a stand-in noting eax, against the
/// Rust version on the same, each run on the test DirectSound.
pub fn test_opening(
    mut t: crate::framework::SelfTest,
    hook: &crate::framework::RawHook,
    copy: &crate::audio::test_copy::Copy,
    game: &Opening,
    variables: [usize; 3],
    references: usize,
) -> (usize, usize) {
    use crate::audio::test_sound::{Results, TestSound};
    use crate::framework::{Regs, call_game};
    use crate::install::stand_ins::{set_handler, stand_in};
    // [0] the setting, [1] the set, [2] found; then the holder.
    let mut vars = Box::new([0u32; 4]);
    let at = vars.as_mut_ptr() as u32;
    let o = Opening {
        setting: at,
        object: at + 4,
        found: at + 8,
        ..*game
    };
    let c = match crate::audio::test_copy::make(
        hook,
        copy,
        &|_| Some(stand_in(0)),
        &[
            (variables[0], o.setting),
            (variables[1], o.object),
            (variables[2], o.found),
        ],
        references,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(note_log);
    let sound = TestSound::new();
    let fail = 0x8000_4005u32 as i32;
    let ok = Results {
        support: 3,
        ..Results::default()
    };
    let mut cases = vec![
        (1u32, 0u32, true, ok),
        (0, 0x77, true, ok),
        (0, 0, false, ok),
    ];
    for r in [
        ok,
        Results { query: fail, ..ok },
        Results {
            query_support: fail,
            ..ok
        },
        Results { support: 1, ..ok },
        Results { support: 7, ..ok },
        Results { set: fail, ..ok },
    ] {
        cases.push((0, 0, true, r));
    }
    for (setting, preset, there, results) in cases {
        let run = |rust: bool| {
            let holder = if there { sound.three_d(0) } else { 0 };
            unsafe { (at as *mut [u32; 4]).write_volatile([setting, preset, 0x55, holder]) };
            LOGGED.lock().unwrap().clear();
            sound.begin(results);
            let before = Regs {
                ebx: at + 12,
                ..Regs::SENTINEL
            };
            let carry = if rust {
                unsafe {
                    open_property_set(&o, at + 12, &mut |e| {
                        note_log(
                            0,
                            &mut Regs {
                                eax: e,
                                ..Regs::default()
                            },
                        )
                    })
                }
                .ok()
                .map(|clear| !clear)
            } else {
                let mut ro = before;
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0).then(|| ro.carry())
            };
            let logged = std::mem::take(&mut *LOGGED.lock().unwrap());
            (carry, sound.noted(), logged, unsafe {
                (at as *const [u32; 4]).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0.is_some() && theirs == mine, || {
            format!(
                "setting {setting}, set {preset:#x}, buffer there {there}, query {:#x}, support {:#x} {:#x}, set {:#x}: original {theirs:x?}; rust {mine:x?}",
                results.query, results.support, results.query_support, results.set
            )
        });
    }
    drop(vars);
    t.finish()
}
