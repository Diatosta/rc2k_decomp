use crate::framework::GameHook;

pub mod celsius_to_fahrenheit;
pub mod count_bits;
pub mod hhmm_to_minutes;
pub mod miles_to_km;
pub mod nearest_on_segment;
pub mod next_random;
pub mod normalise_vector;
pub mod random_below;
pub mod random_step;
pub mod seed_random;
pub mod stir_random;
pub mod turn_float_point_xz;
pub mod turn_point_xz;
pub mod turn_point_xz_ebx;
pub mod turn_point_xz_ecx;
pub mod turn_scratch_xz;
pub mod x87;
pub mod x87_load;
pub mod x87_run;

pub static HOOKS: &[&dyn GameHook] = &[
    &celsius_to_fahrenheit::HOOK,
    &count_bits::HOOK,
    &miles_to_km::HOOK,
    &hhmm_to_minutes::HOOK,
    &next_random::HOOK,
    &normalise_vector::HOOK,
    &random_below::HOOK,
    &seed_random::HOOK,
    &stir_random::HOOK,
    &turn_scratch_xz::HOOK,
    &turn_point_xz::HOOK,
    &turn_point_xz_ebx::HOOK,
    &turn_point_xz_ecx::HOOK,
    &turn_float_point_xz::HOOK,
    &nearest_on_segment::HOOK,
    &random_step::HOOK,
];
