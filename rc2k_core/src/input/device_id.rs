//! Which controller a player chose, kept by what the controller is rather than by its place in
//! the list of those plugged in (which changes with what is plugged in, and in what order).
//!
//! A controller is known by its name and, when it reports one, its serial number (a DualSense
//! gives its Bluetooth address): two pads of the same model are then told apart exactly. One
//! without a serial number is known as the n-th of its name in the list, the best that can be
//! done for two identical pads that say nothing more.

/// A controller as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listed {
    pub name: String,
    pub serial: Option<String>,
}

/// A chosen controller, as it is kept (in the options file, each player's `controller_device`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceId {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    /// Which of the controllers of this name it was, from 1, in the list's order.
    pub nth: u32,
}

/// The controller at `index` of `list`, as it is kept; none past the list.
pub fn id_of(list: &[Listed], index: usize) -> Option<DeviceId> {
    let device = list.get(index)?;
    let nth = list[..=index]
        .iter()
        .filter(|d| d.name == device.name)
        .count() as u32;
    Some(DeviceId {
        name: device.name.clone(),
        serial: device.serial.clone(),
        nth,
    })
}

/// Where the kept controller is in `list` now, if it is plugged in: the one of its name and
/// serial number when it has one (none if that one is not there: another pad of the model may
/// be another player's), else the n-th of its name.
pub fn find(list: &[Listed], id: &DeviceId) -> Option<usize> {
    if id.serial.is_some() {
        return list
            .iter()
            .position(|d| d.name == id.name && d.serial == id.serial);
    }
    list.iter()
        .enumerate()
        .filter(|(_, d)| d.name == id.name)
        .nth(id.nth.checked_sub(1)? as usize)
        .map(|(n, _)| n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad(name: &str, serial: Option<&str>) -> Listed {
        Listed {
            name: name.to_string(),
            serial: serial.map(str::to_string),
        }
    }

    #[test]
    fn two_pads_of_a_model_with_serials_are_told_apart() {
        let list = [
            pad("DualSense", Some("aa")),
            pad("Wheel", None),
            pad("DualSense", Some("bb")),
        ];
        let second = id_of(&list, 2).unwrap();
        assert_eq!(second.serial.as_deref(), Some("bb"));
        assert_eq!(second.nth, 2);
        // plugged in the other way round, each is still found
        let swapped = [pad("DualSense", Some("bb")), pad("DualSense", Some("aa"))];
        assert_eq!(find(&swapped, &second), Some(0));
        // its own pad away: not the other one
        assert_eq!(find(&[pad("DualSense", Some("aa"))], &second), None);
    }

    #[test]
    fn without_serials_the_order_tells_them_apart() {
        let list = [pad("Pad", None), pad("Wheel", None), pad("Pad", None)];
        let second = id_of(&list, 2).unwrap();
        assert_eq!(second.nth, 2);
        assert_eq!(find(&list, &second), Some(2));
        assert_eq!(find(&[pad("Wheel", None), pad("Pad", None)], &second), None);
        let first = id_of(&list, 0).unwrap();
        assert_eq!(
            find(&[pad("Wheel", None), pad("Pad", None)], &first),
            Some(1)
        );
    }

    #[test]
    fn nothing_past_the_list() {
        assert_eq!(id_of(&[], 0), None);
        let id = DeviceId {
            name: "Pad".into(),
            serial: None,
            nth: 0,
        };
        assert_eq!(find(&[pad("Pad", None)], &id), None);
    }
}
