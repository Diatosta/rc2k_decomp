use crate::framework::GameHook;

pub mod celsius_to_fahrenheit;
pub mod hhmm_to_minutes;
pub mod miles_to_km;

pub static HOOKS: &[&dyn GameHook] = &[
    &celsius_to_fahrenheit::HOOK,
    &miles_to_km::HOOK,
    &hhmm_to_minutes::HOOK,
];
