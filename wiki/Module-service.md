# Module: service

*Category: [Module reference](Modules)*

The service area between stages, apart from its drawing: choosing which damaged parts to
repair within the time allowed, changing the car's set-up (tyres, gear ratios, ride height,
suspension stiffness, brake balance, steering sensitivity), and loading and saving set-ups.

## State

`Service` (`src/service/owner.rs`), a field of the menus' owner (`Menus::service`): the four
set-up items' chosen and installed values and the saved set-ups. Accessors for the current
player, the repair flags and the edit mode are in `src/service/state.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `service_area_loop.rs` | `src/service/service_area_loop.rs` | The service area as a screen (its description for `menu::screen_loop`). |
| `handle_edit_keys` | `src/service/handle_edit_keys.rs` | Keys while a set-up value is being edited. |
| `toggle_repair_choice` | `src/service/toggle_repair_choice.rs` | Marks a part for repair or not. |
| `update_repair_total` | `src/service/update_repair_total.rs` | Adds up the time the chosen repairs take. |
| `apply_repairs` | `src/service/apply_repairs.rs` | Carries the chosen repairs out. |
| `repair_car_in` | `src/service/repair_parts.rs` | Repairs a car's parts. |
| `edit_tyre`, `edit_gear_ratio`, `edit_ride_height` ... | `src/service/edit_*.rs` | One editor per set-up value. |
| `step_*_up`, `step_*_down` | `src/service/step_*.rs` | One step of a value. |
| `load_setup_values` | `src/service/load_setup_values.rs` | Puts a set-up's values onto the car. |
| `read_setup_slot`, `write_setup_slot` | files of the same names | A saved set-up, read and written. |

## How it is laid out

A small flat folder: `edit_*`, `step_*_up` / `step_*_down` and `toggle_*` per set-up value,
plus the repairs and the slots. The rules (repair times, value ranges) are in
`rc2k_core/src/service/`.

## Calls and callers

It uses [`menu`](Module-menu)'s players and screen loop, [`input`](Module-input) for keys and
[`files`](Module-files) for the slots. It is called from `menu`'s event flows and drawn by
[`screens`](Module-screens).

## Related pages

[The front end](The-front-end), [Car setups](Car-setups), [Car physics](Car-physics).
