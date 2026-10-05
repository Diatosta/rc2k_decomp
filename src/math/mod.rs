use crate::framework::GameHook;

mod celsius_to_fahrenheit;

pub static HOOKS: &[&dyn GameHook] = &[&celsius_to_fahrenheit::HOOK];
