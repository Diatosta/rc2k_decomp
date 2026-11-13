//! The service's owner: the car set-ups' stores in one struct, reached through `service()` by
//! the code that does not take it as a parameter.

use crate::service::read_setup_slot::{SetUps, template};
use crate::session::{GameThread, PlainBytes};

/// The service's state.
pub struct Service {
    /// The set-ups' buffers (file head, names, a kept value).
    pub set_ups: SetUps,
    /// The set-up file's name, the number's two digits last.
    pub template: PlainBytes<27>,
}

impl Service {
    pub const fn new() -> Self {
        Service {
            set_ups: SetUps::new(),
            template: template(),
        }
    }
}

/// The one instance, reached through `service()`.
pub static SERVICE: GameThread<Service> = GameThread::new(Service::new());

/// The service's state, for a caller that does not take it as a parameter yet.
pub fn service() -> &'static Service {
    SERVICE.get()
}
