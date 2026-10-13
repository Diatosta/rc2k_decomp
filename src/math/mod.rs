use crate::framework::GameHook;

pub mod celsius_to_fahrenheit;
pub mod hhmm_to_minutes;
pub mod miles_to_km;
pub mod next_random;
pub mod normalise_vector;
pub mod random_below;
pub mod seed_random;
pub mod stir_random;
pub mod x87;

pub static HOOKS: &[&dyn GameHook] = &[
    &celsius_to_fahrenheit::HOOK,
    &miles_to_km::HOOK,
    &hhmm_to_minutes::HOOK,
    &next_random::HOOK,
    &normalise_vector::HOOK,
    &random_below::HOOK,
    &seed_random::HOOK,
    &stir_random::HOOK,
];
