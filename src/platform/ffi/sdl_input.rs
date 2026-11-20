//! SDL3's joysticks, gamepads, haptics and event injection (`sdl3-sys`), for
//! `platform::input` (the game's DirectInput devices) and `remote` (test events).
//!
//! The handles are opaque newtypes over SDL's pointers: they are made only by the `open`
//! functions here and used only by the methods here, which pass them back to SDL. A handle is
//! `Copy`; the owner closes it once (`close`) and does not use it after. SDL takes its own
//! lock in these calls, so a handle may go to another thread (`Send`). Every function is a
//! single SDL call (or the few that always go together), arguments and results as SDL's.

#![allow(unsafe_code)]

use sdl3_sys::everything::*;
use std::ffi::CStr;
#[cfg(any(feature = "remote", handheld))]
use std::ffi::CString;
#[cfg(handheld)]
use std::ffi::c_char;

/// SDL's last error text.
pub fn last_error() -> String {
    // SAFETY: SDL's NUL-ended error text, copied at once.
    unsafe { CStr::from_ptr(SDL_GetError()) }
        .to_string_lossy()
        .into_owned()
}

/// The handheld's joystick hints set (before the subsystem starts): events while the window
/// has no focus, and the joystick node SDL reads its pad from.
#[cfg(handheld)]
pub fn set_handheld_joystick_hints() {
    // SAFETY: SDL's own hint names and NUL-ended values, read during the calls.
    unsafe {
        SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, c"1".as_ptr());
        SDL_SetHint(SDL_HINT_JOYSTICK_DEVICE, c"/dev/input/js0".as_ptr());
    }
}

/// The joystick, gamepad and haptic subsystems started; false when SDL refuses.
pub fn init_joystick_subsystems() -> bool {
    // SAFETY: plain flags; SDL itself is started by then.
    unsafe { SDL_InitSubSystem(SDL_INIT_JOYSTICK | SDL_INIT_GAMEPAD | SDL_INIT_HAPTIC) }
}

/// The ids of SDL's joysticks now, or none when SDL gives no list.
pub fn joystick_ids() -> Option<Vec<SDL_JoystickID>> {
    let mut count = 0;
    // SAFETY: SDL's array of `count` ids (null checked), copied and freed here.
    unsafe {
        let ids = SDL_GetJoysticks(&raw mut count);
        if ids.is_null() {
            return None;
        }
        let copied = std::slice::from_raw_parts(ids, count.max(0) as usize).to_vec();
        SDL_free(ids.cast());
        Some(copied)
    }
}

/// A joystick's name by id, or `fallback` when SDL has none.
pub fn joystick_name_for_id(id: SDL_JoystickID, fallback: &str) -> String {
    // SAFETY: the name SDL returns (null checked) lives until its next call; copied here.
    unsafe {
        let name = SDL_GetJoystickNameForID(id);
        if name.is_null() {
            fallback.to_string()
        } else {
            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    }
}

/// A joystick's GUID by id, as SDL's hex text.
#[cfg(handheld)]
pub fn joystick_guid_text_for_id(id: SDL_JoystickID) -> String {
    let mut text = [0 as c_char; 33];
    // SAFETY: a buffer of the 33 bytes given, which SDL ends with a NUL.
    unsafe {
        SDL_GUIDToString(SDL_GetJoystickGUIDForID(id), text.as_mut_ptr(), 33);
        CStr::from_ptr(text.as_ptr()).to_string_lossy().into_owned()
    }
}

/// The family of button pictures SDL's gamepad `id` is drawn with when the player has not chosen
/// one: by its make and USB ids; Steam Deck too when the program runs on a Deck (`SteamDeck=1`,
/// which Steam sets for what it launches there).
pub fn pad_family(id: SDL_JoystickID) -> rc2k_core::input::icons::Family {
    use rc2k_core::input::icons::PadMake;
    // SAFETY: plain queries by instance id.
    let (kind, vendor, product) = unsafe {
        (
            SDL_GetGamepadTypeForID(id),
            SDL_GetJoystickVendorForID(id),
            SDL_GetJoystickProductForID(id),
        )
    };
    let make = match kind {
        SDL_GAMEPAD_TYPE_XBOX360 | SDL_GAMEPAD_TYPE_XBOXONE => PadMake::Xbox,
        SDL_GAMEPAD_TYPE_PS3 | SDL_GAMEPAD_TYPE_PS4 | SDL_GAMEPAD_TYPE_PS5 => PadMake::PlayStation,
        SDL_GAMEPAD_TYPE_STEAM => PadMake::Steam,
        _ => PadMake::Other,
    };
    let on_deck = std::env::var_os("SteamDeck").is_some_and(|v| v == "1");
    rc2k_core::input::icons::pad_family(make, vendor, product, on_deck)
}

/// A gamepad mapping added (SDL's mapping string); false when the text has a NUL.
#[cfg(handheld)]
pub fn add_gamepad_mapping(mapping: String) -> bool {
    let Ok(mapping) = CString::new(mapping) else {
        return false;
    };
    // SAFETY: a NUL-ended text, read during the call.
    unsafe { SDL_AddGamepadMapping(mapping.as_ptr()) };
    true
}

/// Whether SDL knows the joystick as a gamepad.
pub fn is_gamepad(id: SDL_JoystickID) -> bool {
    // SAFETY: a plain id; SDL answers false for one it does not have.
    unsafe { SDL_IsGamepad(id) }
}

/// An open joystick.
#[derive(Clone, Copy)]
pub struct Joystick(*mut SDL_Joystick);

// SAFETY: SDL's joystick functions take SDL's own lock.
unsafe impl Send for Joystick {}

impl Joystick {
    /// The joystick opened, or none.
    pub fn open(id: SDL_JoystickID) -> Option<Self> {
        // SAFETY: an id SDL listed; null (failure) checked.
        let joystick = unsafe { SDL_OpenJoystick(id) };
        (!joystick.is_null()).then_some(Self(joystick))
    }

    /// Closed; the handle is not used after.
    pub fn close(self) {
        // SAFETY: the handle `open` made, closed once by its owner.
        unsafe { SDL_CloseJoystick(self.0) };
    }

    pub fn name(self) -> String {
        // SAFETY: the name SDL returns (null checked) lives until its next call; copied.
        unsafe {
            let name = SDL_GetJoystickName(self.0);
            if name.is_null() {
                "Joystick".to_string()
            } else {
                CStr::from_ptr(name).to_string_lossy().into_owned()
            }
        }
    }

    /// The product GUID's 16 bytes.
    pub fn guid(self) -> [u8; 16] {
        // SAFETY: an open joystick.
        unsafe { SDL_GetJoystickGUID(self.0).data }
    }

    pub fn axis_count(self) -> i32 {
        // SAFETY: an open joystick.
        unsafe { SDL_GetNumJoystickAxes(self.0) }
    }

    /// Whether SDL's type for the joystick is a steering wheel.
    pub fn is_wheel(self) -> bool {
        // SAFETY: an open joystick.
        unsafe { SDL_GetJoystickType(self.0) == SDL_JOYSTICK_TYPE_WHEEL }
    }

    pub fn button_count(self) -> i32 {
        // SAFETY: an open joystick.
        unsafe { SDL_GetNumJoystickButtons(self.0) }
    }

    pub fn hat_count(self) -> i32 {
        // SAFETY: an open joystick.
        unsafe { SDL_GetNumJoystickHats(self.0) }
    }

    pub fn connected(self) -> bool {
        // SAFETY: an open joystick.
        unsafe { SDL_JoystickConnected(self.0) }
    }

    pub fn axis(self, n: i32) -> i16 {
        // SAFETY: an open joystick; SDL answers 0 for an axis it does not have.
        unsafe { SDL_GetJoystickAxis(self.0, n) }
    }

    pub fn button(self, n: i32) -> bool {
        // SAFETY: as `axis`.
        unsafe { SDL_GetJoystickButton(self.0, n) }
    }

    /// A hat's SDL bits (up 1, right 2, down 4, left 8).
    pub fn hat(self, n: i32) -> u8 {
        // SAFETY: as `axis`.
        unsafe { SDL_GetJoystickHat(self.0, n) }
    }

    /// The haptic device of this joystick, opened, when it has one.
    /// Its serial number, when it reports one (a DualSense: its Bluetooth address).
    pub fn serial(self) -> Option<String> {
        // SAFETY: the text SDL returns (null checked) lives with the joystick; copied.
        unsafe {
            let serial = SDL_GetJoystickSerial(self.0);
            if serial.is_null() {
                return None;
            }
            let text = CStr::from_ptr(serial).to_string_lossy().into_owned();
            (!text.is_empty()).then_some(text)
        }
    }

    /// Whether SDL can rumble it (a gamepad's two motors).
    pub fn has_rumble(self) -> bool {
        // SAFETY: an open joystick; its properties' id (0 on failure) is only read here.
        unsafe {
            let props = SDL_GetJoystickProperties(self.0);
            props != 0 && SDL_GetBooleanProperty(props, SDL_PROP_JOYSTICK_CAP_RUMBLE_BOOLEAN, false)
        }
    }

    /// Its motors at `low` and `high` (0 .. 65535: the slow, heavy one and the fast, light one)
    /// for `ms` milliseconds, or until the next call.
    pub fn rumble(self, low: u16, high: u16, ms: u32) {
        // SAFETY: an open joystick; SDL answers false (ignored) when it cannot.
        unsafe {
            SDL_RumbleJoystick(self.0, low, high, ms);
        }
    }

    pub fn open_haptic(self) -> Option<Haptic> {
        // SAFETY: an open joystick; null (no haptic device) checked.
        unsafe {
            if !SDL_IsJoystickHaptic(self.0) {
                return None;
            }
            let haptic = SDL_OpenHapticFromJoystick(self.0);
            (!haptic.is_null()).then_some(Haptic(haptic))
        }
    }

    /// A virtual joystick's axis set (the remote driver's stand-in stick).
    #[cfg(any(feature = "remote", phone))]
    pub fn set_virtual_axis(self, axis: i32, value: i16) -> bool {
        // SAFETY: an open joystick; SDL refuses one that is not virtual.
        unsafe { SDL_SetJoystickVirtualAxis(self.0, axis, value) }
    }

    /// A virtual joystick's button set.
    #[cfg(any(feature = "remote", phone))]
    pub fn set_virtual_button(self, button: i32, down: bool) -> bool {
        // SAFETY: an open joystick; SDL refuses one that is not virtual.
        unsafe { SDL_SetJoystickVirtualButton(self.0, button, down) }
    }
}

/// A plain virtual joystick (not a gamepad) of `axes`, `buttons` and `hats`, with the USB ids
/// given, attached as if plugged in (the remote driver's stand-in stick): its id, or none.
#[cfg(feature = "remote")]
pub fn attach_virtual_stick(
    axes: u16,
    buttons: u16,
    hats: u16,
    vendor: u16,
    product: u16,
) -> Option<SDL_JoystickID> {
    let desc = SDL_VirtualJoystickDesc {
        r#type: SDL_JOYSTICK_TYPE_FLIGHT_STICK.0 as u16,
        naxes: axes,
        nbuttons: buttons,
        nhats: hats,
        vendor_id: vendor,
        product_id: product,
        name: c"Remote stand-in stick".as_ptr(),
        Rumble: Some(virtual_rumble),
        ..SDL_VirtualJoystickDesc::default()
    };
    // SAFETY: a description made by SDL's own initialiser; the name is a constant C text.
    let id = unsafe { SDL_AttachVirtualJoystick(&raw const desc) };
    (id.0 != 0).then_some(id)
}

/// A virtual gamepad in SDL's standard layout (all its axes and buttons), attached as if
/// plugged in: the touch screen's controls (`platform::touch`). Its id, or none.
#[cfg(phone)]
pub fn attach_virtual_gamepad(
    vendor: u16,
    product: u16,
    name: &'static CStr,
) -> Option<SDL_JoystickID> {
    let desc = SDL_VirtualJoystickDesc {
        r#type: SDL_JOYSTICK_TYPE_GAMEPAD.0 as u16,
        naxes: SDL_GAMEPAD_AXIS_COUNT.0 as u16,
        nbuttons: SDL_GAMEPAD_BUTTON_COUNT.0 as u16,
        vendor_id: vendor,
        product_id: product,
        name: name.as_ptr(),
        // The game's rumble for this pad goes to the phone's motor (`platform::ffi::vibrate`).
        Rumble: Some(phone_rumble),
        ..SDL_VirtualJoystickDesc::default()
    };
    // SAFETY: a description made by SDL's own initialiser; the name is a constant C text.
    let id = unsafe { SDL_AttachVirtualJoystick(&raw const desc) };
    (id.0 != 0).then_some(id)
}

/// SDL's call to rumble the touch screen's virtual gamepad: the phone pulses for it when the
/// player has switched that on.
#[cfg(phone)]
unsafe extern "C" fn phone_rumble(_: *mut std::ffi::c_void, low: u16, high: u16) -> bool {
    crate::platform::ffi::vibrate::rumble(low, high);
    true
}

/// The stand-in stick's motors as SDL last set them (heavy in the high half, light in the low).
#[cfg(feature = "remote")]
pub static VIRTUAL_RUMBLE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// SDL's call to rumble the stand-in stick: the levels noted.
#[cfg(feature = "remote")]
unsafe extern "C" fn virtual_rumble(_: *mut std::ffi::c_void, low: u16, high: u16) -> bool {
    VIRTUAL_RUMBLE.store(
        (low as u32) << 16 | high as u32,
        std::sync::atomic::Ordering::Relaxed,
    );
    true
}

/// A virtual joystick detached, as if unplugged.
#[cfg(feature = "remote")]
pub fn detach_virtual_stick(id: SDL_JoystickID) -> bool {
    // SAFETY: a plain id; SDL refuses one that is not a virtual joystick.
    unsafe { SDL_DetachVirtualJoystick(id) }
}

/// An open gamepad (its joystick is its own, closed with it).
#[derive(Clone, Copy)]
pub struct Gamepad(*mut SDL_Gamepad);

// SAFETY: SDL's gamepad functions take SDL's own lock.
unsafe impl Send for Gamepad {}

impl Gamepad {
    /// The gamepad opened, or none.
    pub fn open(id: SDL_JoystickID) -> Option<Self> {
        // SAFETY: an id SDL listed; null (failure) checked. SDL counts the opens, so each
        // needs its own close.
        let pad = unsafe { SDL_OpenGamepad(id) };
        (!pad.is_null()).then_some(Self(pad))
    }

    /// Closed; the handle (and its joystick) is not used after.
    pub fn close(self) {
        // SAFETY: the handle `open` made, closed once by its owner.
        unsafe { SDL_CloseGamepad(self.0) };
    }

    /// The gamepad's joystick, which is closed with the gamepad (never by itself).
    pub fn joystick(self) -> Option<Joystick> {
        // SAFETY: an open gamepad; null checked.
        let joystick = unsafe { SDL_GetGamepadJoystick(self.0) };
        (!joystick.is_null()).then_some(Joystick(joystick))
    }

    pub fn connected(self) -> bool {
        // SAFETY: an open gamepad.
        unsafe { SDL_GamepadConnected(self.0) }
    }

    pub fn axis(self, axis: SDL_GamepadAxis) -> i16 {
        // SAFETY: an open gamepad.
        unsafe { SDL_GetGamepadAxis(self.0, axis) }
    }

    pub fn button(self, button: SDL_GamepadButton) -> bool {
        // SAFETY: an open gamepad.
        unsafe { SDL_GetGamepadButton(self.0, button) }
    }
}

/// An open haptic (force feedback) device.
#[derive(Clone, Copy)]
pub struct Haptic(*mut SDL_Haptic);

// SAFETY: SDL's haptic functions take SDL's own lock.
unsafe impl Send for Haptic {}

/// A blank haptic effect, all zero (every member of its union is plain integers).
pub fn blank_haptic_effect() -> SDL_HapticEffect {
    // SAFETY: a C union of plain integers: all zero is a valid value.
    unsafe { std::mem::zeroed() }
}

impl Haptic {
    /// Closed; the handle is not used after.
    pub fn close(self) {
        // SAFETY: the handle `open_haptic` made, closed once by its owner.
        unsafe { SDL_CloseHaptic(self.0) };
    }

    /// SDL's bit set of the effects and settings the device has.
    pub fn features(self) -> u32 {
        // SAFETY: an open haptic.
        unsafe { SDL_GetHapticFeatures(self.0) }
    }

    /// An effect made on the device, or SDL's error text.
    pub fn create_effect(self, effect: &SDL_HapticEffect) -> Result<SDL_HapticEffectID, String> {
        // SAFETY: an open haptic and a valid effect, read during the call.
        let id = unsafe { SDL_CreateHapticEffect(self.0, effect) };
        if id.0 < 0 { Err(last_error()) } else { Ok(id) }
    }

    pub fn update_effect(self, id: SDL_HapticEffectID, effect: &SDL_HapticEffect) {
        // SAFETY: an open haptic, an effect made on it, a valid effect read during the call.
        unsafe { SDL_UpdateHapticEffect(self.0, id, effect) };
    }

    pub fn run_effect(self, id: SDL_HapticEffectID, iterations: u32) {
        // SAFETY: an open haptic and an effect made on it (SDL refuses others).
        unsafe { SDL_RunHapticEffect(self.0, id, iterations) };
    }

    pub fn stop_effect(self, id: SDL_HapticEffectID) {
        // SAFETY: as `run_effect`.
        unsafe { SDL_StopHapticEffect(self.0, id) };
    }

    pub fn destroy_effect(self, id: SDL_HapticEffectID) {
        // SAFETY: as `run_effect`.
        unsafe { SDL_DestroyHapticEffect(self.0, id) };
    }

    pub fn stop_effects(self) {
        // SAFETY: an open haptic.
        unsafe { SDL_StopHapticEffects(self.0) };
    }

    pub fn set_gain(self, gain: i32) {
        // SAFETY: an open haptic.
        unsafe { SDL_SetHapticGain(self.0, gain) };
    }

    pub fn set_autocenter(self, percent: i32) {
        // SAFETY: an open haptic.
        unsafe { SDL_SetHapticAutocenter(self.0, percent) };
    }
}

/// A scancode by SDL's name, none when SDL does not know it.
#[cfg(feature = "remote")]
pub fn scancode_from_name(name: &str) -> Option<SDL_Scancode> {
    let name = CString::new(name).ok()?;
    // SAFETY: a NUL-ended text, read during the call.
    let code = unsafe { SDL_GetScancodeFromName(name.as_ptr()) };
    (code != SDL_SCANCODE_UNKNOWN).then_some(code)
}

/// A gamepad button by SDL's name, none when SDL does not know it.
#[cfg(feature = "remote")]
pub fn gamepad_button_from_name(name: &str) -> Option<SDL_GamepadButton> {
    let name = CString::new(name).ok()?;
    // SAFETY: as `scancode_from_name`.
    let code = unsafe { SDL_GetGamepadButtonFromString(name.as_ptr()) };
    (code != SDL_GAMEPAD_BUTTON_INVALID).then_some(code)
}

/// A key press or release pushed on SDL's event queue.
#[cfg(feature = "remote")]
pub fn push_key_event(scancode: SDL_Scancode, down: bool) {
    // SAFETY: an all-zero event is valid; SDL copies it.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        event.r#type = if down {
            SDL_EVENT_KEY_DOWN.0
        } else {
            SDL_EVENT_KEY_UP.0
        };
        event.key.scancode = scancode;
        event.key.down = down;
        SDL_PushEvent(&raw mut event);
    }
}

/// A gamepad button press or release pushed on SDL's event queue.
#[cfg(feature = "remote")]
pub fn push_gamepad_button_event(button: SDL_GamepadButton, down: bool) {
    // SAFETY: as `push_key_event`.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        event.r#type = if down {
            SDL_EVENT_GAMEPAD_BUTTON_DOWN.0
        } else {
            SDL_EVENT_GAMEPAD_BUTTON_UP.0
        };
        event.gbutton.button = button.0 as u8;
        event.gbutton.down = down;
        SDL_PushEvent(&raw mut event);
    }
}

/// A gamepad axis moved to `value`, pushed on SDL's event queue.
#[cfg(feature = "remote")]
pub fn push_gamepad_axis_event(axis: SDL_GamepadAxis, value: i16) {
    // SAFETY: as `push_key_event`.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        event.r#type = SDL_EVENT_GAMEPAD_AXIS_MOTION.0;
        event.gaxis.axis = axis.0 as u8;
        event.gaxis.value = value;
        SDL_PushEvent(&raw mut event);
    }
}

/// A quit request pushed on SDL's event queue.
#[cfg(feature = "remote")]
pub fn push_quit_event() {
    // SAFETY: as `push_key_event`.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        event.r#type = SDL_EVENT_QUIT.0;
        SDL_PushEvent(&raw mut event);
    }
}

/// A virtual joystick standing for a browser gamepad (`platform::web_pads`): the page's Gamepad
/// API cannot be read from the game's worker, so its pads are copied into SDL's virtual ones,
/// which the rest of the game meets as ordinary joysticks and gamepads.
#[cfg(web)]
pub struct VirtualPad {
    id: SDL_JoystickID,
    joystick: *mut SDL_Joystick,
}

// SAFETY: the joystick handle is used by one thread at a time (the bridge's lock); SDL's joystick
// functions take SDL's own lock.
#[cfg(web)]
unsafe impl Send for VirtualPad {}

#[cfg(web)]
impl VirtualPad {
    /// A virtual gamepad of SDL's layout (six axes: the sticks, then the two triggers; the fifteen
    /// buttons from the south one to the pad's right), attached as if plugged in.
    pub fn attach_gamepad(name: &CStr, vendor: u16, product: u16) -> Option<VirtualPad> {
        let mask = (0..15).fold(0u32, |mask, button| mask | 1 << button);
        let axes = (1 << SDL_GAMEPAD_AXIS_LEFTX.0)
            | (1 << SDL_GAMEPAD_AXIS_LEFTY.0)
            | (1 << SDL_GAMEPAD_AXIS_RIGHTX.0)
            | (1 << SDL_GAMEPAD_AXIS_RIGHTY.0)
            | (1 << SDL_GAMEPAD_AXIS_LEFT_TRIGGER.0)
            | (1 << SDL_GAMEPAD_AXIS_RIGHT_TRIGGER.0);
        let desc = SDL_VirtualJoystickDesc {
            r#type: SDL_JOYSTICK_TYPE_GAMEPAD.0 as u16,
            naxes: 6,
            nbuttons: 15,
            vendor_id: vendor,
            product_id: product,
            button_mask: mask,
            axis_mask: axes as u32,
            name: name.as_ptr(),
            ..SDL_VirtualJoystickDesc::default()
        };
        Self::attach(&desc)
    }

    /// A plain virtual joystick of `axes` and `buttons` (a pad the browser has no standard layout
    /// for), attached as if plugged in.
    pub fn attach_stick(
        name: &CStr,
        vendor: u16,
        product: u16,
        axes: u16,
        buttons: u16,
    ) -> Option<VirtualPad> {
        let desc = SDL_VirtualJoystickDesc {
            r#type: SDL_JOYSTICK_TYPE_UNKNOWN.0 as u16,
            naxes: axes,
            nbuttons: buttons,
            vendor_id: vendor,
            product_id: product,
            name: name.as_ptr(),
            ..SDL_VirtualJoystickDesc::default()
        };
        Self::attach(&desc)
    }

    fn attach(desc: &SDL_VirtualJoystickDesc) -> Option<VirtualPad> {
        // SAFETY: a description made by SDL's own initialiser whose name is a C text alive for the
        // call (SDL copies it); the id it returns is opened at once, null (failure) checked.
        unsafe {
            let id = SDL_AttachVirtualJoystick(desc);
            if id.0 == 0 {
                return None;
            }
            let joystick = SDL_OpenJoystick(id);
            if joystick.is_null() {
                SDL_DetachVirtualJoystick(id);
                return None;
            }
            Some(VirtualPad { id, joystick })
        }
    }

    /// An axis set (SDL refuses an index the joystick does not have).
    pub fn set_axis(&self, axis: i32, value: i16) {
        // SAFETY: the open virtual joystick this value made; SDL checks the index.
        unsafe { SDL_SetJoystickVirtualAxis(self.joystick, axis, value) };
    }

    /// A button set.
    pub fn set_button(&self, button: i32, down: bool) {
        // SAFETY: the open virtual joystick this value made; SDL checks the index.
        unsafe { SDL_SetJoystickVirtualButton(self.joystick, button, down) };
    }

    /// Closed and detached, as if unplugged.
    pub fn detach(self) {
        // SAFETY: the joystick opened in `attach`, closed once; the id is the one attached.
        unsafe {
            SDL_CloseJoystick(self.joystick);
            SDL_DetachVirtualJoystick(self.id);
        }
    }
}
