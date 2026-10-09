use crate::framework::GameHook;

pub mod celsius_to_fahrenheit;
pub mod miles_to_km;
pub mod mmss_to_seconds;

pub static HOOKS: &[&dyn GameHook] = &[
    &celsius_to_fahrenheit::HOOK,
    &miles_to_km::HOOK,
    &mmss_to_seconds::HOOK,
];
