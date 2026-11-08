//! Module service's owner (step 11, owned state): the car set-ups' stores in one struct,
//! reached through `service()` by the code that does not take it as a parameter.

use crate::service::read_setup_slot::{SetUps, template};
use crate::session::Bytes;

/// The service's state.
pub struct Service {
    /// The set-ups' buffers (file head, names, a kept value).
    pub set_ups: SetUps,
    /// `aSavedataCarset`: the set-up file's name, the number's two digits last.
    pub template: Bytes<27>,
}

impl Service {
    pub const fn new() -> Self {
        Service {
            set_ups: SetUps::new(),
            template: template(),
        }
    }
}

/// The one instance. Reached through `service()`; `moved` registers its stores' places.
pub static SERVICE: Service = Service::new();

/// The service's state, for a caller that does not take it as a parameter yet.
pub fn service() -> &'static Service {
    &SERVICE
}
