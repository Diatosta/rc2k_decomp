use crate::sound::queue::{Queue, send_command};
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use rc2k_core::sound::senders::Music;

/// Command `command` queued with what the audio thread reads of it (eax, ebx, ecx, edx).
pub fn send(command: u32, [eax, ebx, ecx, edx]: [u32; 4]) {
    unsafe { send_command(&Queue::game(), command, [eax, ebx, ecx, edx, 0, 0, 0]) };
}

/// `send`, then `sound::wait_for_commands`.
pub fn send_and_wait(command: u32, registers: [u32; 4]) {
    send(command, registers);
    wait_for_commands(&mut Game([0; 7]));
}

/// For the senders: a game variable's value.
pub fn variable(at: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(at) }
}

/// Command 0xB (converted: `rc2k_core::sound::senders::Music`), sent and waited for.
pub fn music(m: Music) {
    send_and_wait(0xB, [m.tune, m.then, m.volume, m.action]);
}

/// `sound::play_tune_n` (converted: `rc2k_core::sound::senders::tune_command`).
pub fn play_tune(n: u32) {
    if let Some(m) = rc2k_core::sound::senders::tune_command(
        n,
        variable(crate::sound::store::EffectsSlot::at(0x5180FC)),
        variable(crate::sound::store::EffectsSlot::at(0x518100)),
        variable(crate::sound::store::EffectsSlot::at(0x5180F4)) == 1,
    ) {
        music(m);
    }
}
