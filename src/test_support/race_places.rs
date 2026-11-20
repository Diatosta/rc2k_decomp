//! Where the race block's places the recorded blocks read and write live in the registry's stores:
//! the block's offset (the recorded instruction's displacement), the store's registry name and the
//! field in the store, by its path (`ByteView::field_path`). The blocks' tests
//! (`vectors::block_vectors`) reach the places through the stores by name, a field through `paths`
//! and not by its byte in the record.

use crate::drive::finish_store::Finish;
use crate::physics::camera_store::CameraWork;
use crate::physics::ground_query::GroundSearch;
use std::sync::OnceLock;

/// `(offset, store, field path)`, sorted by offset.
#[rustfmt::skip]
const FIELDS: &[(usize, &str, &str)] = &[
    (0x1C2C64, "owners.race.replay.viewer_and_kinds", ".viewer_panel.texel_scale"),
    (0x1C2C68, "owners.race.replay.viewer_and_kinds", ".viewer_panel.reciprocal_width_800"),
    (0x1C2C6C, "owners.race.replay.viewer_and_kinds", ".viewer_panel.reciprocal_height_600"),
    (0x1C5CFC, "owners.race.course.numbers", ".hundredth_a"),
    (0x1C5D00, "owners.race.course.numbers", ".two_and_a_quarter"),
    (0x1C5D14, "owners.race.course.numbers", ".hundred"),
    (0x1C5D18, "owners.race.course.numbers", ".thousand"),
    (0x1C5D38, "owners.race.course.numbers", ".one"),
    (0x1C5D3C, "owners.race.course.numbers", ".two"),
    (0x1C5D40, "owners.race.course.numbers", ".three"),
    (0x1C5D48, "owners.race.course.numbers", ".five"),
    (0x1C5D60, "owners.race.course.numbers", ".sixty_four"),
    (0x1C5D98, "owners.race.course.numbers", ".k65536"),
    (0x1C5DA0, "owners.race.course.numbers", ".one_over_65536"),
    (0x1C5DA4, "owners.race.course.numbers", ".quarter"),
    (0x1C5DB4, "owners.race.course.numbers", ".one_and_a_half"),
    (0x1C5DD4, "owners.race.course.numbers", ".one_over_two_pi"),
    (0x1C5DDC, "owners.race.course.numbers", ".tenths[0]"),
    (0x1C5E08, "owners.race.course.numbers", ".kilometres_in_mile"),
    (0x1C5E28, "owners.race.course.numbers", ".k4096"),
    (0x1C5E2C, "owners.race.course.numbers", ".k256"),
    (0x1C5E34, "owners.race.course.numbers", ".k10000"),
    (0x1C5E40, "owners.race.course.numbers", ".half"),
    (0x1C5E48, "owners.race.course.numbers", ".grip_change"),
    (0x1CB958, "owners.race.frame.lights_and_gear", ".gear.shift_up_point"),
    (0x1CB95C, "owners.race.frame.lights_and_gear", ".gear.shift_down_point"),
    (0x1CB960, "owners.race.frame.lights_and_gear", ".gear.shift_down_clutch_point"),
    (0x1CB964, "owners.race.frame.lights_and_gear", ".gear.shift_ratio_figure"),
    (0x1CB968, "owners.race.frame.lights_and_gear", ".gear.drive_torque_figure"),
    (0x1CB96C, "owners.race.frame.lights_and_gear", ".gear.figures[0]"),
    (0x1CB970, "owners.race.frame.lights_and_gear", ".gear.figures[1]"),
    (0x1CB974, "owners.race.frame.lights_and_gear", ".gear.figures[2]"),
    (0x1CB978, "owners.race.frame.lights_and_gear", ".gear.figures[3]"),
    (0x1CB97C, "owners.race.frame.lights_and_gear", ".gear.test_marks[0]"),
    (0x1CB980, "owners.race.frame.lights_and_gear", ".gear.test_marks[1]"),
    (0x1CB984, "owners.race.frame.lights_and_gear", ".gear.test_marks[2]"),
    (0x1CCD80, "owners.race.course.course", ".flat_depth"),
    (0x205260, "owners.race.shared.tables", ".view_share"),
    (0x20628C, "owners.race.frame.view_and_records", ".view_vars.camera_bias_scale"),
    (0x206290, "owners.race.frame.view_and_records", ".view_vars.camera_bias_limit"),
    (0x2062D8, "owners.race.frame.view_and_records", ".view_vars.camera_vector[0]"),
    (0x2062DC, "owners.race.frame.view_and_records", ".view_vars.camera_vector[1]"),
    (0x2062E0, "owners.race.frame.view_and_records", ".view_vars.camera_vector[2]"),
    (0x20631C, "owners.race.frame.view_and_records", ".view_vars.camera_distance"),
    (0x206570, "owners.race.frame.view_and_records", ".records[1].focus[1]"),
    (0x206574, "owners.race.frame.view_and_records", ".records[1].focus[2]"),
    (0x2065C8, "owners.race.frame.view_and_records", ".records[2].focus[0]"),
    (0x2065CC, "owners.race.frame.view_and_records", ".records[2].focus[1]"),
    (0x2065D0, "owners.race.frame.view_and_records", ".records[2].focus[2]"),
    (0x206624, "owners.race.frame.view_and_records", ".records[3].focus[0]"),
    (0x206628, "owners.race.frame.view_and_records", ".records[3].focus[1]"),
    (0x20662C, "owners.race.frame.view_and_records", ".records[3].focus[2]"),
    (0x206730, "owners.race.frame.view_and_records", ".records[6].offset[1]"),
    (0x206734, "owners.race.frame.view_and_records", ".records[6].offset[2]"),
    (0x206744, "owners.race.frame.view_and_records", ".records[6].angles[0]"),
    (0x206748, "owners.race.frame.view_and_records", ".records[6].angles[1]"),
    (0x208B20, "owners.race.physics.camera", ".view_shift_a"),
    (0x208B24, "owners.race.physics.camera", ".view_shift_b"),
    (0x208B28, "owners.race.physics.camera", ".view_shift_c"),
    (0x208B2C, "owners.race.physics.camera", ".view_shift_d"),
    (0x208B30, "owners.race.physics.camera", ".camera_offset_x"),
    (0x208B34, "owners.race.physics.camera", ".camera_offset_y"),
    (0x208B38, "owners.race.physics.camera", ".camera_offset_z"),
    (0x208B50, "owners.race.physics.camera", ".start_heading"),
    (0x208B54, "owners.race.physics.camera", ".start_roll"),
    (0x208B58, "owners.race.physics.camera", ".start_pitch"),
    (0x208B5C, "owners.race.physics.camera", ".view_a_heading"),
    (0x208B60, "owners.race.physics.camera", ".view_a_roll"),
    (0x208B64, "owners.race.physics.camera", ".view_a_pitch"),
    (0x208B68, "owners.race.physics.camera", ".view_b_heading"),
    (0x208B6C, "owners.race.physics.camera", ".view_b_roll"),
    (0x208B70, "owners.race.physics.camera", ".view_b_pitch"),
    (0x208B98, "owners.race.physics.camera", ".view_a_offset[0]"),
    (0x208B9C, "owners.race.physics.camera", ".view_a_offset[1]"),
    (0x208BA0, "owners.race.physics.camera", ".view_a_offset[2]"),
    (0x208BB0, "owners.race.physics.camera", ".view_b_offset[0]"),
    (0x208BB4, "owners.race.physics.camera", ".view_b_offset[1]"),
    (0x208BB8, "owners.race.physics.camera", ".view_b_offset[2]"),
    (0x208BF0, "owners.race.physics.camera", ".view_blend_aux_2"),
    (0x21388C, "owners.race.shared.physics_tables", ".step_time"),
    (0x213E78, "owners.race.shared.physics_tables", ".grip_slide_scale"),
    (0x213E7C, "owners.race.shared.physics_tables", ".grip_slip_limit"),
    (0x213E80, "owners.race.shared.physics_tables", ".grip_force_limit"),
    (0x213E88, "owners.race.shared.physics_tables", ".spec_scale_a4"),
    (0x213EB8, "owners.race.shared.physics_tables", ".rev_limit_scale"),
    (0x213EC4, "owners.race.shared.physics_tables", ".drive_speed_scale"),
    (0x213ECC, "owners.race.shared.physics_tables", ".side_force_scale"),
    (0x213ED0, "owners.race.shared.physics_tables", ".slip_blend"),
    (0x213ED4, "owners.race.shared.physics_tables", ".slip_share"),
    (0x213ED8, "owners.race.shared.physics_tables", ".wheel_speed_margin"),
    (0x213EDC, "owners.race.shared.physics_tables", ".wheel_speed_gap"),
    (0x213EE0, "owners.race.shared.physics_tables", ".sound_speed_scale"),
    (0x213F04, "owners.race.shared.physics_tables", ".spec_default_bc0[0]"),
    (0x213F08, "owners.race.shared.physics_tables", ".spec_default_bc0[1]"),
    (0x213F0C, "owners.race.shared.physics_tables", ".spec_default_bc0[2]"),
    (0x213F10, "owners.race.shared.physics_tables", ".spec_scale_1b8"),
    (0x213F34, "owners.race.shared.physics_tables", ".grip_ramp"),
    (0x213F38, "owners.race.shared.physics_tables", ".grip_ramp_share"),
    (0x213F5C, "owners.race.shared.physics_tables", ".surface_blend[0]"),
    (0x213F60, "owners.race.shared.physics_tables", ".surface_blend[1]"),
    (0x213F64, "owners.race.shared.physics_tables", ".surface_blend[2]"),
    (0x213F68, "owners.race.shared.physics_tables", ".surface_blend[3]"),
    (0x213FAC, "owners.race.shared.physics_tables", ".inertia[1]"),
    (0x213FB8, "owners.race.shared.physics_tables", ".rev_limits[0]"),
    (0x213FBC, "owners.race.shared.physics_tables", ".rev_limits[1]"),
    (0x213FC0, "owners.race.shared.physics_tables", ".camera_drop"),
    (0x213FC8, "owners.race.shared.physics_tables", ".gravity"),
    (0x213FCC, "owners.race.shared.physics_tables", ".air_drag"),
    (0x213FD0, "owners.race.shared.physics_tables", ".rolling_drag"),
    (0x213FD4, "owners.race.shared.physics_tables", ".free_rev_add"),
    (0x213FD8, "owners.race.shared.physics_tables", ".free_rev_scale"),
    (0x213FE4, "owners.race.shared.physics_tables", ".rev_top_scale"),
    (0x213FEC, "owners.race.shared.physics_tables", ".wheel_load_scale"),
    (0x214098, "owners.race.shared.physics_tables", ".camera_frame_scale"),
    (0x2140B0, "owners.race.shared.physics_tables", ".drive_shares[0]"),
    (0x2140B4, "owners.race.shared.physics_tables", ".drive_shares[1]"),
    (0x2140B8, "owners.race.shared.physics_tables", ".drive_shares[2]"),
    (0x2140C0, "owners.race.shared.physics_tables", ".gear_ratios[1]"),
    (0x2140F8, "owners.race.shared.physics_tables", ".idle_engine_speed"),
    (0x2140FC, "owners.race.shared.physics_tables", ".stall_engine_speed"),
    (0x214104, "owners.race.shared.physics_tables", ".rev_limit"),
    (0x21411C, "owners.race.shared.physics_tables", ".surface_scales[0]"),
    (0x214120, "owners.race.shared.physics_tables", ".surface_scales[1]"),
    (0x214138, "owners.race.shared.physics_tables", ".vertical_speed"),
    (0x214148, "owners.race.shared.physics_tables", ".contact_step"),
    (0x21414C, "owners.race.shared.physics_tables", ".contact_steps"),
    (0x214164, "owners.race.shared.physics_tables", ".grip_drive_word"),
    (0x214170, "owners.race.shared.physics_tables", ".weight"),
    (0x2141D4, "owners.race.shared.physics_tables", ".wheel_spin_push[0]"),
    (0x2141D8, "owners.race.shared.physics_tables", ".wheel_spin_push[1]"),
    (0x21422C, "owners.race.shared.physics_tables", ".wheel_speed_step"),
    (0x214230, "owners.race.shared.physics_tables", ".surface_softness"),
    (0x214234, "owners.race.shared.physics_tables", ".wheel_slips[0]"),
    (0x214238, "owners.race.shared.physics_tables", ".wheel_slips[1]"),
    (0x21423C, "owners.race.shared.physics_tables", ".drive_limits[0]"),
    (0x214240, "owners.race.shared.physics_tables", ".drive_limits[1]"),
    (0x214244, "owners.race.shared.physics_tables", ".drive_limits[2]"),
    (0x214248, "owners.race.shared.physics_tables", ".drive_forces[0]"),
    (0x21424C, "owners.race.shared.physics_tables", ".drive_forces[1]"),
    (0x214250, "owners.race.shared.physics_tables", ".drive_slip[0]"),
    (0x214254, "owners.race.shared.physics_tables", ".drive_slip[1]"),
    (0x214260, "owners.race.shared.physics_tables", ".grip_limit"),
    (0x214264, "owners.race.shared.physics_tables", ".slip_limit"),
    (0x214268, "owners.race.shared.physics_tables", ".wheel_load"),
    (0x214284, "owners.race.shared.physics_tables", ".wheel_loads[0]"),
    (0x214288, "owners.race.shared.physics_tables", ".wheel_loads[1]"),
    (0x21428C, "owners.race.shared.physics_tables", ".wheel_loads[2]"),
    (0x214290, "owners.race.shared.physics_tables", ".wheel_loads[3]"),
    (0x214294, "owners.race.shared.physics_tables", ".wheel_loads[4]"),
    (0x214298, "owners.race.shared.physics_tables", ".wheel_grips[0]"),
    (0x21429C, "owners.race.shared.physics_tables", ".wheel_grips[1]"),
    (0x2142A0, "owners.race.shared.physics_tables", ".wheel_grips[2]"),
    (0x2142A4, "owners.race.shared.physics_tables", ".wheel_grips[3]"),
    (0x2142A8, "owners.race.shared.physics_tables", ".wheel_grips[4]"),
    (0x2142AC, "owners.race.shared.physics_tables", ".wheel_grips_b[0]"),
    (0x2142B0, "owners.race.shared.physics_tables", ".wheel_grips_b[1]"),
    (0x2142B4, "owners.race.shared.physics_tables", ".wheel_grips_b[2]"),
    (0x2142B8, "owners.race.shared.physics_tables", ".wheel_grips_b[3]"),
    (0x2142BC, "owners.race.shared.physics_tables", ".wheel_grips_b[4]"),
    (0x2142C0, "owners.race.shared.physics_tables", ".point_grips[0]"),
    (0x2142C4, "owners.race.shared.physics_tables", ".point_grips[1]"),
    (0x2142D0, "owners.race.shared.physics_tables", ".point_grips[4]"),
    (0x2146C0, "owners.race.shared.physics_tables", ".surface_blends[0]"),
    (0x2146C4, "owners.race.shared.physics_tables", ".surface_blends[1]"),
    (0x2146D0, "owners.race.shared.physics_tables", ".surface_blends[4]"),
    (0x2146D4, "owners.race.shared.physics_tables", ".surface_blends[5]"),
    (0x2146D8, "owners.race.shared.physics_tables", ".surface_blends[6]"),
    (0x2146E4, "owners.race.shared.physics_tables", ".surface_blends[9]"),
    (0x2146E8, "owners.race.shared.physics_tables", ".surface_blends[10]"),
    (0x2146EC, "owners.race.shared.physics_tables", ".surface_blends[11]"),
    (0x2146F8, "owners.race.shared.physics_tables", ".surface_blends[14]"),
    (0x2146FC, "owners.race.shared.physics_tables", ".surface_blends[15]"),
    (0x214700, "owners.race.shared.physics_tables", ".surface_blends[16]"),
    (0x21470C, "owners.race.shared.physics_tables", ".surface_blends[19]"),
    (0x214740, "owners.race.shared.physics_tables", ".drive_ratio"),
    (0x214748, "owners.race.shared.physics_tables", ".engine_force"),
    (0x214750, "owners.race.shared.physics_tables", ".drive_torque"),
    (0x214760, "owners.race.shared.physics_tables", ".wheel_drag[0]"),
    (0x214764, "owners.race.shared.physics_tables", ".wheel_drag[1]"),
    (0x214768, "owners.race.shared.physics_tables", ".lever_share"),
    (0x214774, "owners.race.shared.physics_tables", ".forces[0].force[1]"),
    (0x214794, "owners.race.shared.physics_tables", ".forces[1].force[1]"),
    (0x2147B4, "owners.race.shared.physics_tables", ".forces[2].force[1]"),
    (0x2147D4, "owners.race.shared.physics_tables", ".forces[3].force[1]"),
    (0x2148B0, "owners.race.shared.physics_tables", ".forces[10].force[0]"),
    (0x2148B8, "owners.race.shared.physics_tables", ".forces[10].force[2]"),
    (0x2148D0, "owners.race.shared.physics_tables", ".forces[11].force[0]"),
    (0x2148F0, "owners.race.shared.physics_tables", ".forces[12].force[0]"),
    (0x214910, "owners.race.shared.physics_tables", ".forces[13].force[0]"),
    (0x214930, "owners.race.shared.physics_tables", ".forces[14].force[0]"),
    (0x2149D0, "owners.race.shared.physics_tables", ".forces[19].force[0]"),
    (0x2149E4, "owners.race.shared.physics_tables", ".forces[19].point[2]"),
    (0x2149F0, "owners.race.shared.physics_tables", ".forces[20].force[0]"),
    (0x214A04, "owners.race.shared.physics_tables", ".forces[20].point[2]"),
    (0x214B74, "owners.race.shared.physics_tables", ".grip_scale"),
    (0x214B78, "owners.race.shared.physics_tables", ".side_grip_scale"),
    (0x214BA0, "owners.race.shared.physics_tables", ".spec_grip"),
    (0x214BA8, "owners.race.shared.physics_tables", ".spec_wheel"),
    (0x214BAC, "owners.race.shared.physics_tables", ".spec_drive[0]"),
    (0x214BB0, "owners.race.shared.physics_tables", ".spec_drive[1]"),
    (0x214BB4, "owners.race.shared.physics_tables", ".spec_surface[0]"),
    (0x214BB8, "owners.race.shared.physics_tables", ".spec_surface[1]"),
    (0x214BBC, "owners.race.shared.physics_tables", ".spec_surface[2]"),
    (0x214BC0, "owners.race.shared.physics_tables", ".spec_ints[0]"),
    (0x214BC4, "owners.race.shared.physics_tables", ".spec_ints[1]"),
    (0x214BC8, "owners.race.shared.physics_tables", ".spec_ints[2]"),
    (0x214BF0, "owners.race.shared.physics_tables", ".handling[1]"),
    (0x214BFC, "owners.race.shared.physics_tables", ".handling[4]"),
    (0x214C04, "owners.race.shared.physics_tables", ".handling[6]"),
    (0x214C08, "owners.race.shared.physics_tables", ".handling[7]"),
    (0x214C0C, "owners.race.shared.physics_tables", ".handling[8]"),
    (0x214D04, "owners.race.shared.physics_tables", ".split_point_distance"),
    (0x223AE4, "owners.race.shared.buffers.reads", ".engine_tables.sines[0]"),
    (0x223AE8, "owners.race.shared.buffers.reads", ".engine_tables.sines[1]"),
    (0x223AF4, "owners.race.shared.buffers.reads", ".engine_tables.sines[4]"),
    (0x223DA8, "owners.race.shared.vars", ".mud_scale_a[0]"),
    (0x223DAC, "owners.race.shared.vars", ".mud_scale_a[1]"),
    (0x22400C, "owners.scene.drawing.scene", ".scratch"),
    (0x22401C, "owners.scene.drawing.scene", ".work_float_a"),
    (0x224020, "owners.scene.drawing.scene", ".work_float_b"),
    (0x224040, "owners.scene.drawing.scene", ".surface_point[0]"),
    (0x224044, "owners.scene.drawing.scene", ".surface_point[1]"),
    (0x224048, "owners.scene.drawing.scene", ".surface_point[2]"),
    (0x237638, "owners.scene.lights.state", ".camera_shake_x"),
    (0x23763C, "owners.scene.lights.state", ".camera_shake_y"),
    (0x237640, "owners.scene.lights.state", ".camera_shake_z"),
    (0x237644, "owners.scene.lights.state", ".camera_shake_listener"),
    (0x239824, "owners.race.physics.ground_search", ".x"),
    (0x23982C, "owners.race.physics.ground_search", ".z"),
    (0x24BB38, "owners.race.replay.finish", ".car_lift"),
    (0x24BB44, "owners.race.replay.finish", ".limit_a"),
    (0x24BB48, "owners.race.replay.finish", ".limit_b"),
    (0x24BB4C, "owners.race.replay.finish", ".camera_yaw_rate"),
    (0x24BB50, "owners.race.replay.finish", ".camera_distance_start"),
    (0x24BB54, "owners.race.replay.finish", ".camera_height_start"),
    (0x24BB58, "owners.race.replay.finish", ".camera_distance_rate"),
    (0x24BB5C, "owners.race.replay.finish", ".camera_height_rate"),
    (0x24BB64, "owners.race.replay.finish", ".time"),
    (0x24BB68, "owners.race.replay.finish", ".eased"),
    (0x24BB6C, "owners.race.replay.finish", ".share"),
    (0x24BB70, "owners.race.replay.finish", ".limit_c"),
    (0x24BB74, "owners.race.replay.finish", ".limit_d"),
    (0x24BB78, "owners.race.replay.finish", ".camera_pitch_per_share"),
    (0x24BB7C, "owners.race.replay.finish", ".camera_height_per_share"),
    (0x2D1DF4, "owners.race.physics.contact", ".contact_share_numerator"),
    (0x2D1DF8, "owners.race.physics.contact", ".contact_normal_factor"),
    (0x2D2A14, "owners.race.physics.contact", ".probe_share_u[0]"),
    (0x2D2A18, "owners.race.physics.contact", ".probe_share_u[1]"),
    (0x2D2A24, "owners.race.physics.contact", ".probe_share_u[4]"),
    (0x2D2B14, "owners.race.physics.contact", ".probe_share_v[0]"),
    (0x2D2B18, "owners.race.physics.contact", ".probe_share_v[1]"),
    (0x2D2B24, "owners.race.physics.contact", ".probe_share_v[4]"),
    (0x2D3014, "owners.race.physics.contact", ".crash_ground_scale"),
    (0x2D307C, "owners.race.physics.contact", ".contact_normal[1]"),
    (0x2D3090, "owners.race.physics.contact", ".contact_point[0]"),
    (0x2D3094, "owners.race.physics.contact", ".contact_point[1]"),
    (0x2D3098, "owners.race.physics.contact", ".contact_point[2]"),
    (0x2D30A8, "owners.race.physics.contact", ".contact_car_speed[0]"),
    (0x2D30AC, "owners.race.physics.contact", ".contact_car_speed[1]"),
    (0x2D30B0, "owners.race.physics.contact", ".contact_car_speed[2]"),
    (0x2D30C0, "owners.race.physics.contact", ".contact_turn_cross[0]"),
    (0x2D30C4, "owners.race.physics.contact", ".contact_turn_cross[1]"),
    (0x2D30C8, "owners.race.physics.contact", ".contact_turn_cross[2]"),
    (0x2D30CC, "owners.race.physics.contact", ".contact_point_speed[0]"),
    (0x2D30D0, "owners.race.physics.contact", ".contact_point_speed[1]"),
    (0x2D30D4, "owners.race.physics.contact", ".contact_point_speed[2]"),
    (0x2D30E4, "owners.race.physics.contact", ".contact_normal_speed[0]"),
    (0x2D30E8, "owners.race.physics.contact", ".contact_normal_speed[1]"),
    (0x2D30EC, "owners.race.physics.contact", ".contact_normal_speed[2]"),
    (0x2D30F0, "owners.race.physics.contact", ".contact_side_speed[0]"),
    (0x2D30F4, "owners.race.physics.contact", ".contact_side_speed[1]"),
    (0x2D30F8, "owners.race.physics.contact", ".contact_side_speed[2]"),
    (0x2D30FC, "owners.race.physics.contact", ".contact_slide_limit_held"),
    (0x2D3100, "owners.race.physics.contact", ".contact_slide_limit"),
    (0x2D3130, "owners.race.physics.contact", ".contact_normal_push[0]"),
    (0x2D3134, "owners.race.physics.contact", ".contact_normal_push[1]"),
    (0x2D3138, "owners.race.physics.contact", ".contact_normal_push[2]"),
    (0x2D3140, "owners.race.physics.contact", ".contact_slide[0]"),
    (0x2D3144, "owners.race.physics.contact", ".contact_slide[1]"),
    (0x2D3148, "owners.race.physics.contact", ".contact_slide[2]"),
    (0x2D3150, "owners.race.physics.contact", ".contact_total[0]"),
    (0x2D3154, "owners.race.physics.contact", ".contact_total[1]"),
    (0x2D3158, "owners.race.physics.contact", ".contact_total[2]"),
    (0x2D3160, "owners.race.physics.contact", ".contact_extra[0]"),
    (0x2D3164, "owners.race.physics.contact", ".contact_extra[1]"),
    (0x2D3168, "owners.race.physics.contact", ".contact_extra[2]"),
    (0x2D31D0, "owners.race.physics.contact", ".crash_up_dot"),
    (0x2D31D8, "owners.race.physics.contact", ".contact_turn_push"),
    (0x2D31DC, "owners.race.physics.contact", ".contact_depth"),
    (0x2D31E0, "owners.race.physics.contact", ".contact_share"),
    (0x2D3204, "owners.race.physics.contact", ".contact_factor"),
    (0x2D3B4C, "owners.race.physics.contact", ".object_point_offset[0]"),
    (0x2D3B50, "owners.race.physics.contact", ".object_point_offset[1]"),
    (0x2D3B54, "owners.race.physics.contact", ".object_point_offset[2]"),
    (0x2D3B58, "owners.race.physics.contact", ".object_point_turned[0]"),
    (0x2D3B5C, "owners.race.physics.contact", ".object_point_turned[1]"),
    (0x2D3B60, "owners.race.physics.contact", ".object_point_turned[2]"),
    (0x2D3B70, "owners.race.physics.contact", ".face_angle"),
    (0x2D3B74, "owners.race.physics.contact", ".face_speed"),
    (0x2D3B78, "owners.race.physics.contact", ".object_point_scaled[0]"),
    (0x2D3B7C, "owners.race.physics.contact", ".object_point_scaled[1]"),
    (0x2D3B80, "owners.race.physics.contact", ".object_point_scaled[2]"),
    (0x2D3B88, "owners.race.physics.contact", ".bump_push_scale"),
    (0x2D3B9C, "owners.race.physics.contact", ".bump_push[0]"),
    (0x2D3BA0, "owners.race.physics.contact", ".bump_push[1]"),
    (0x2D3BA4, "owners.race.physics.contact", ".bump_push[2]"),
    (0x2D3BAC, "owners.race.physics.contact", ".bump_share_a"),
    (0x2D3BB0, "owners.race.physics.contact", ".bump_share_b"),
    (0x2D3CD8, "owners.race.physics.contact", ".bump_relative_speed[0]"),
    (0x2D3CDC, "owners.race.physics.contact", ".bump_relative_speed[1]"),
    (0x2D3CE0, "owners.race.physics.contact", ".bump_relative_speed[2]"),
    (0x30F524, "owners.race.shared.car_blocks", ".cars[0].position[1]"),
    (0x319FE0, "owners.race.shared.car", ".step_clock"),
    (0x319FF8, "owners.race.shared.car", ".brake_balance"),
    (0x31A008, "owners.race.shared.car", ".ride_height"),
    (0x31A00C, "owners.race.shared.car", ".suspension_stiffness"),
    (0x31A018, "owners.race.shared.car", ".drive_kind"),
    (0x31A038, "owners.race.shared.car", ".ground_bump_scale"),
    (0x31A0A0, "owners.race.shared.car", ".crash_matrix[5]"),
    (0x31A10C, "owners.race.shared.car", ".position[0]"),
    (0x31A110, "owners.race.shared.car", ".position[1]"),
    (0x31A114, "owners.race.shared.car", ".position[2]"),
    (0x31A158, "owners.race.shared.car", ".drive_slip_speed"),
    (0x31A170, "owners.race.shared.car", ".velocity[0]"),
    (0x31A174, "owners.race.shared.car", ".velocity[1]"),
    (0x31A178, "owners.race.shared.car", ".velocity[2]"),
    (0x31A17C, "owners.race.shared.car", ".local_velocity[0]"),
    (0x31A180, "owners.race.shared.car", ".local_velocity[1]"),
    (0x31A184, "owners.race.shared.car", ".local_velocity[2]"),
    (0x31A190, "owners.race.shared.car", ".drive_speed_a"),
    (0x31A194, "owners.race.shared.car", ".drive_speed_b"),
    (0x31A198, "owners.race.shared.car", ".mean_drive_speed"),
    (0x31A1A0, "owners.race.shared.car", ".side_speed_held"),
    (0x31A1B0, "owners.race.shared.car", ".engine_speed"),
    (0x31A1B4, "owners.race.shared.car", ".engine_speed_limit"),
    (0x31A1B8, "owners.race.shared.car", ".spec_scale_copy"),
    (0x31A1CC, "owners.race.shared.car", ".wall_hit_timer"),
    (0x31A1D8, "owners.race.shared.car", ".eased_yaw_spin"),
    (0x31A1DC, "owners.race.shared.car", ".grip_force_sum"),
    (0x31A1E0, "owners.race.shared.car", ".steer_trim"),
    (0x31A1E8, "owners.race.shared.car", ".grip_forces[0]"),
    (0x31A1EC, "owners.race.shared.car", ".grip_forces[1]"),
    (0x31A1F0, "owners.race.shared.car", ".grip_forces[2]"),
    (0x31A1F4, "owners.race.shared.car", ".grip_forces[3]"),
    (0x31A1F8, "owners.race.shared.car", ".option_bits"),
    (0x31A1FC, "owners.race.shared.car", ".contact_point[0]"),
    (0x31A200, "owners.race.shared.car", ".contact_point[1]"),
    (0x31A204, "owners.race.shared.car", ".contact_point[2]"),
    (0x31A208, "owners.race.shared.car", ".contact_depth"),
    (0x31A258, "owners.race.shared.car", ".surface_blend"),
    (0x31A260, "owners.race.shared.car", ".slip_speed"),
    (0x31A264, "owners.race.shared.car", ".sound_slip_share"),
    (0x31A268, "owners.race.shared.car", ".sound_level"),
    (0x31A26C, "owners.race.shared.car", ".sound_speed"),
    (0x31A278, "owners.race.shared.car", ".brake_fade"),
    (0x31A27C, "owners.race.shared.car", ".damper_wear[0]"),
    (0x31A280, "owners.race.shared.car", ".damper_wear[1]"),
    (0x31A284, "owners.race.shared.car", ".brake_heat[0]"),
    (0x31A288, "owners.race.shared.car", ".brake_heat[1]"),
    (0x31A28C, "owners.race.shared.car", ".engine_heat"),
    (0x31A290, "owners.race.shared.car", ".engine_damage"),
    (0x31A294, "owners.race.shared.car", ".exhaust_damage"),
    (0x31A298, "owners.race.shared.car", ".turbo_damage"),
    (0x31A29C, "owners.race.shared.car", ".cooling_damage"),
    (0x31A2A0, "owners.race.shared.car", ".clutch_damage"),
    (0x31A2A4, "owners.race.shared.car", ".differential_damage"),
    (0x31A2A8, "owners.race.shared.car", ".gear_wear[0]"),
    (0x31A2AC, "owners.race.shared.car", ".gear_wear[1]"),
    (0x31A2B0, "owners.race.shared.car", ".gear_wear[2]"),
    (0x31A2B4, "owners.race.shared.car", ".gear_wear[3]"),
    (0x31A2B8, "owners.race.shared.car", ".gear_wear[4]"),
    (0x31A2BC, "owners.race.shared.car", ".gear_wear[5]"),
    (0x31A2C0, "owners.race.shared.car", ".gear_wear[6]"),
    (0x31A2C4, "owners.race.shared.car", ".gear_wear[7]"),
    (0x31A2C8, "owners.race.shared.car", ".gear_wear[8]"),
    (0x31A2CC, "owners.race.shared.car", ".gearbox_damage"),
    (0x31A2D0, "owners.race.shared.car", ".wheel_hit_damage[0]"),
    (0x31A2D4, "owners.race.shared.car", ".wheel_hit_damage[1]"),
    (0x31A2D8, "owners.race.shared.car", ".wheel_hit_damage[2]"),
    (0x31A2DC, "owners.race.shared.car", ".wheel_hit_damage[3]"),
    (0x31A2E0, "owners.race.shared.car", ".body_hit_damage"),
    (0x31A2E4, "owners.race.shared.car", ".body_damage"),
    (0x31A2E8, "owners.race.shared.car", ".electrics_damage"),
    (0x31A2EC, "owners.race.shared.car", ".light_damage[0]"),
    (0x31A2F0, "owners.race.shared.car", ".light_damage[1]"),
    (0x31A2F4, "owners.race.shared.car", ".light_damage[2]"),
    (0x31A2F8, "owners.race.shared.car", ".light_damage[3]"),
    (0x31A2FC, "owners.race.shared.car", ".light_damage[4]"),
    (0x31A300, "owners.race.shared.car", ".light_damage[5]"),
    (0x31A304, "owners.race.shared.car", ".lights_damage"),
    (0x31A308, "owners.race.shared.car", ".steering_damage"),
    (0x31A30C, "owners.race.shared.car", ".suspension_damage[0]"),
    (0x31A310, "owners.race.shared.car", ".suspension_damage[1]"),
    (0x31A314, "owners.race.shared.car", ".suspension_damage[2]"),
    (0x31A318, "owners.race.shared.car", ".suspension_damage[3]"),
    (0x31A31C, "owners.race.shared.car", ".overall_suspension_damage"),
    (0x31A320, "owners.race.shared.car", ".brake_wear_a"),
    (0x31A324, "owners.race.shared.car", ".brake_wear_b"),
    (0x31A328, "owners.race.shared.car", ".brake_damage"),
    (0x31A32C, "owners.race.shared.car", ".tyre_wears[0]"),
    (0x31A330, "owners.race.shared.car", ".tyre_wears[1]"),
    (0x31A334, "owners.race.shared.car", ".tyre_wears[2]"),
    (0x31A338, "owners.race.shared.car", ".tyre_wears[3]"),
    (0x31A33C, "owners.race.shared.car", ".tyre_wear"),
    (0x31A3D0, "owners.race.shared.car", ".brake_efficiency[0]"),
    (0x31A3D4, "owners.race.shared.car", ".brake_efficiency[1]"),
    (0x31A3D8, "owners.race.shared.car", ".drive_efficiency[0]"),
    (0x31A3DC, "owners.race.shared.car", ".drive_efficiency[1]"),
    (0x31A3E0, "owners.race.shared.car", ".steering_efficiency"),
    (0x31A410, "owners.race.shared.car", ".wheel_ride_height[0]"),
    (0x31A414, "owners.race.shared.car", ".wheel_ride_height[1]"),
    (0x31A418, "owners.race.shared.car", ".wheel_ride_height[2]"),
    (0x31A41C, "owners.race.shared.car", ".wheel_ride_height[3]"),
    (0x31A420, "owners.race.shared.car", ".wheel_stiffness[0]"),
    (0x31A424, "owners.race.shared.car", ".wheel_stiffness[1]"),
    (0x31A428, "owners.race.shared.car", ".wheel_stiffness[2]"),
    (0x31A42C, "owners.race.shared.car", ".wheel_stiffness[3]"),
    (0x31A430, "owners.race.shared.car", ".stiffness_mean"),
    (0x31A434, "owners.race.shared.car", ".ride_height_mean"),
    (0x31A610, "owners.race.shared.car", ".throttle"),
    (0x31A614, "owners.race.shared.car", ".brake"),
    (0x31A618, "owners.race.shared.car", ".handbrake"),
    (0x31A61C, "owners.race.shared.car", ".steering"),
    (0x31A668, "owners.race.shared.car", ".camera_switch_clock"),
    (0x31A670, "owners.race.shared.car", ".camera_switch_length"),
    (0x31A674, "owners.race.shared.car", ".camera_switch_share"),
    (0x31A688, "owners.race.shared.car", ".camera_heading"),
    (0x31A690, "owners.race.shared.car", ".camera_pitch"),
    (0x31A6A0, "owners.race.shared.car", ".camera_zoom"),
    (0x31A8E4, "owners.race.shared.car", ".vertical_speed_change"),
    (0x31A920, "owners.race.shared.car", ".point_wear_factor[0]"),
    (0x31A924, "owners.race.shared.car", ".point_wear_factor[1]"),
    (0x31A930, "owners.race.shared.car", ".point_wear_factor[4]"),
    (0x31A984, "owners.race.shared.car", ".wheel_surface_blend[0]"),
    (0x31A988, "owners.race.shared.car", ".wheel_surface_blend[1]"),
    (0x31A994, "owners.race.shared.car", ".wheel_surface_blend[4]"),
    (0x31AA88, "owners.race.shared.car", ".engine_sound_speed_int"),
    (0x31ACC8, "owners.race.shared.car", ".probe_point_z[0]"),
    (0x31ACD4, "owners.race.shared.car", ".probe_point_z[3]"),
    (0x31AEE0, "owners.race.shared.car", ".engine_speed_scaled"),
    (0x31AF18, "owners.race.shared.car", ".skid_grip[0]"),
    (0x31AF1C, "owners.race.shared.car", ".skid_grip[1]"),
    (0x31C708, "owners.race.shared.work", ".patch_factors[1]"),
];

/// Every place of the race block: its offset, the store and the field in the store.
fn table() -> &'static [(usize, &'static str, &'static str)] {
    static TABLE: OnceLock<Vec<(usize, &'static str, &'static str)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = FIELDS.to_vec();
        table.sort_by_key(|p| p.0);
        table
    })
}

/// The store and the field in it of the race block's byte `off`.
fn place(off: usize) -> (&'static str, &'static str) {
    let table = table();
    match table.binary_search_by_key(&off, |p| p.0) {
        Ok(i) => (table[i].1, table[i].2),
        Err(_) => panic!("race block place {off:#X} is not in test_support::race_places"),
    }
}

/// The finish state (plain fields, not cells): its registry name.
const FINISH: &str = "owners.race.replay.finish";

/// How a field of the finish state is read and written as bits.
type FinishBits = (fn(&Finish) -> u32, fn(&mut Finish, u32));

/// The finish state's field at `path`.
fn finish_field(path: &str) -> Option<FinishBits> {
    macro_rules! float {
        ($f:ident) => {
            Some((|f| f.$f.to_bits(), |f, v| f.$f = f32::from_bits(v)))
        };
    }
    match path {
        ".car_lift" => float!(car_lift),
        ".limit_a" => float!(limit_a),
        ".limit_b" => float!(limit_b),
        ".camera_yaw_rate" => float!(camera_yaw_rate),
        ".camera_distance_start" => float!(camera_distance_start),
        ".camera_height_start" => float!(camera_height_start),
        ".camera_distance_rate" => float!(camera_distance_rate),
        ".camera_height_rate" => float!(camera_height_rate),
        ".time" => float!(time),
        ".eased" => float!(eased),
        ".share" => float!(share),
        ".limit_c" => float!(limit_c),
        ".limit_d" => float!(limit_d),
        ".camera_pitch_per_share" => float!(camera_pitch_per_share),
        ".camera_height_per_share" => float!(camera_height_per_share),
        _ => None,
    }
}

/// The camera work (plain fields, not cells): its registry name.
const CAMERA: &str = "owners.race.physics.camera";

/// How a field of the camera work is read and written as bits.
type CameraBits = (fn(&CameraWork) -> u32, fn(&mut CameraWork, u32));

/// The camera work's field at `path`.
fn camera_field(path: &str) -> Option<CameraBits> {
    macro_rules! float {
        ($($f:ident)+ $([$i:literal])?) => {
            Some((|c| c.$($f)+$([$i])?.to_bits(), |c, v| c.$($f)+$([$i])? = f32::from_bits(v)))
        };
    }
    macro_rules! whole {
        ($($f:ident)+ $([$i:literal])?) => {
            Some((|c| c.$($f)+$([$i])? as u32, |c, v| c.$($f)+$([$i])? = v as _))
        };
    }
    match path {
        ".view_shift_a" => float!(view_shift_a),
        ".view_shift_b" => float!(view_shift_b),
        ".view_shift_c" => float!(view_shift_c),
        ".view_shift_d" => float!(view_shift_d),
        ".camera_offset_x" => whole!(camera_offset_x),
        ".camera_offset_y" => whole!(camera_offset_y),
        ".camera_offset_z" => whole!(camera_offset_z),
        ".start_heading" => whole!(start_heading),
        ".start_roll" => whole!(start_roll),
        ".start_pitch" => whole!(start_pitch),
        ".view_a_heading" => whole!(view_a_heading),
        ".view_a_roll" => whole!(view_a_roll),
        ".view_a_pitch" => whole!(view_a_pitch),
        ".view_b_heading" => whole!(view_b_heading),
        ".view_b_roll" => whole!(view_b_roll),
        ".view_b_pitch" => whole!(view_b_pitch),
        ".view_a_offset[0]" => float!(view_a_offset[0]),
        ".view_a_offset[1]" => float!(view_a_offset[1]),
        ".view_a_offset[2]" => float!(view_a_offset[2]),
        ".view_b_offset[0]" => float!(view_b_offset[0]),
        ".view_b_offset[1]" => float!(view_b_offset[1]),
        ".view_b_offset[2]" => float!(view_b_offset[2]),
        ".view_blend_aux_2" => whole!(view_blend_aux_2),
        _ => None,
    }
}

/// The ground search's record (plain fields, not cells): its registry name.
const GROUND_SEARCH: &str = "owners.race.physics.ground_search";

/// How a field of the ground search's record is read and written as bits.
type SearchBits = (fn(&GroundSearch) -> u32, fn(&mut GroundSearch, u32));

/// The ground search record's field at `path`.
fn search_field(path: &str) -> Option<SearchBits> {
    match path {
        ".x" => Some((|g| g.x, |g, v| g.x = v)),
        ".z" => Some((|g| g.z, |g, v| g.z = v)),
        _ => None,
    }
}

/// The bytes of the field at `path` in the store `name` (the registry's, or a plain one).
fn store_bytes(name: &str, path: &str) -> Option<Vec<u8>> {
    if name == FINISH {
        let (get, _) = finish_field(path)?;
        return Some(
            get(&super::own::race().replay.finish())
                .to_le_bytes()
                .to_vec(),
        );
    }
    if name == GROUND_SEARCH {
        let (get, _) = search_field(path)?;
        return Some(
            get(&super::own::race().physics.ground_search().borrow())
                .to_le_bytes()
                .to_vec(),
        );
    }
    if name == CAMERA {
        let (get, _) = camera_field(path)?;
        return Some(
            get(&super::own::race().physics.camera())
                .to_le_bytes()
                .to_vec(),
        );
    }
    super::state::walk(name).path_bytes(path)
}

/// `bytes` put into the field at `path` in the store `name`; false when it is not there.
fn set_store_bytes(name: &str, path: &str, bytes: &[u8]) -> bool {
    if name == FINISH {
        let (Some((_, set)), Ok(word)) = (finish_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::race().replay.finish_mut(),
            u32::from_le_bytes(word),
        );
        return true;
    }
    if name == GROUND_SEARCH {
        let (Some((_, set)), Ok(word)) = (search_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::race().physics.ground_search().borrow_mut(),
            u32::from_le_bytes(word),
        );
        return true;
    }
    if name == CAMERA {
        let (Some((_, set)), Ok(word)) = (camera_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::race().physics.camera_mut(),
            u32::from_le_bytes(word),
        );
        return true;
    }
    super::state::walk(name).set_path_bytes(path, bytes)
}

/// `len` bytes of the race block from `off`: the place's own cells, and where the read is
/// wider than they are, the places that follow it.
pub fn read(off: usize, len: usize) -> Vec<u8> {
    let (name, path) = place(off);
    let mut bytes = store_bytes(name, path).unwrap_or_else(|| panic!("no field {path} in {name}"));
    if bytes.len() < len {
        let rest = read(off + bytes.len(), len - bytes.len());
        bytes.extend(rest);
    }
    bytes.truncate(len);
    bytes
}

/// `bytes` put at the race block's `off`, as `read` finds the places.
pub fn write(off: usize, bytes: &[u8]) {
    let (name, path) = place(off);
    // A write that covers the place's cells exactly needs no read of them first: one walk of
    // the store instead of two (the vectors write places in every case).
    if known_len(off) == Some(bytes.len()) {
        assert!(set_store_bytes(name, path, bytes), "{path} in {name}");
        return;
    }
    let mut cells = store_bytes(name, path).unwrap_or_else(|| panic!("no field {path} in {name}"));
    note_len(off, cells.len());
    let here = cells.len().min(bytes.len());
    cells[..here].copy_from_slice(&bytes[..here]);
    assert!(set_store_bytes(name, path, &cells), "{path} in {name}");
    if bytes.len() > here {
        write(off + here, &bytes[here..]);
    }
}

/// The byte lengths of the places' cells as `write` has found them (a field's length is its
/// type's: it never changes).
static LENGTHS: std::sync::Mutex<Vec<(usize, usize)>> = std::sync::Mutex::new(Vec::new());

fn known_len(off: usize) -> Option<usize> {
    let lengths = LENGTHS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lengths
        .iter()
        .find(|&&(o, _)| o == off)
        .map(|&(_, len)| len)
}

fn note_len(off: usize, len: usize) {
    let mut lengths = LENGTHS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !lengths.iter().any(|&(o, _)| o == off) {
        lengths.push((off, len));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_places_are_found_once_each() {
        let _g = crate::test_support::test_lock();
        let table = super::table();
        assert_eq!(table.len(), super::FIELDS.len());
        for pair in table.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "place {:#X} twice or out of order",
                pair[1].0
            );
        }
        for &(block, name, path) in table {
            let cells = super::store_bytes(name, path);
            assert!(
                cells.is_some_and(|c| !c.is_empty()),
                "place {block:#X}: no field {path} in {name}"
            );
        }
    }
}
