//! Where the race block's places the recorded blocks read and write live in the registry's stores:
//! the block's offset (the recorded instruction's displacement), the store's registry name and the
//! field in the store, by its path (`ByteView::field_path`). The blocks' tests
//! (`vectors::block_vectors`) reach the places through the stores by name, a field through `paths`
//! and not by its byte in the record.

use crate::drive::finish_store::Finish;
use crate::physics::camera_store::CameraWork;
use crate::physics::contact_store::{ContactWork, ProbeLists};
use crate::physics::ground_query::GroundSearch;
use crate::physics::wheel_work::WheelWork;
use crate::race::groups_store::ViewAndRecords;
use std::sync::OnceLock;

/// `(offset, store, field path)`, sorted by offset.
#[rustfmt::skip]
const FIELDS: &[(usize, &str, &str)] = &[
    (0x1C2C64, "owners.race.replay.viewer_panel.texel_scale", ""),
    (0x1C2C68, "owners.race.replay.viewer_panel.reciprocal_width_800", ""),
    (0x1C2C6C, "owners.race.replay.viewer_panel.reciprocal_height_600", ""),
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
    (0x1C5E48, "owners.race.course.grip_change", ""),
    (0x1CB958, "owners.race.physics.gear", ".shift_up_point"),
    (0x1CB95C, "owners.race.physics.gear", ".shift_down_point"),
    (0x1CB960, "owners.race.physics.gear", ".shift_down_clutch_point"),
    (0x1CB964, "owners.race.physics.gear", ".shift_ratio_figure"),
    (0x1CB968, "owners.race.physics.gear", ".drive_torque_figure"),
    (0x1CB96C, "owners.race.physics.gear", ".figures[0]"),
    (0x1CB970, "owners.race.physics.gear", ".figures[1]"),
    (0x1CB974, "owners.race.physics.gear", ".figures[2]"),
    (0x1CB978, "owners.race.physics.gear", ".figures[3]"),
    (0x1CB97C, "owners.race.physics.gear", ".test_marks[0]"),
    (0x1CB980, "owners.race.physics.gear", ".test_marks[1]"),
    (0x1CB984, "owners.race.physics.gear", ".test_marks[2]"),
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
    (0x21388C, "owners.race.course.numbers", ".step_time"),
    (0x213E78, "owners.race.course.numbers", ".grip_slide_scale"),
    (0x213E7C, "owners.race.course.numbers", ".grip_slip_limit"),
    (0x213E80, "owners.race.course.numbers", ".grip_force_limit"),
    (0x213E88, "owners.race.course.numbers", ".spec_scale_a4"),
    (0x213EB8, "owners.race.course.numbers", ".rev_limit_scale"),
    (0x213EC4, "owners.race.course.numbers", ".drive_speed_scale"),
    (0x213ECC, "owners.race.course.numbers", ".side_force_scale"),
    (0x213ED0, "owners.race.course.numbers", ".slip_blend"),
    (0x213ED4, "owners.race.course.numbers", ".slip_share"),
    (0x213ED8, "owners.race.course.numbers", ".wheel_speed_margin"),
    (0x213EDC, "owners.race.course.numbers", ".wheel_speed_gap"),
    (0x213EE0, "owners.race.course.numbers", ".sound_speed_scale"),
    (0x213F04, "owners.race.shared.physics_tables", ".spec_default_bc0[0]"),
    (0x213F08, "owners.race.shared.physics_tables", ".spec_default_bc0[1]"),
    (0x213F0C, "owners.race.shared.physics_tables", ".spec_default_bc0[2]"),
    (0x213F10, "owners.race.course.numbers", ".spec_scale_1b8"),
    (0x213F34, "owners.race.course.numbers", ".grip_ramp"),
    (0x213F38, "owners.race.course.numbers", ".grip_ramp_share"),
    (0x213F5C, "owners.race.course.numbers", ".surface_blend[0]"),
    (0x213F60, "owners.race.course.numbers", ".surface_blend[1]"),
    (0x213F64, "owners.race.course.numbers", ".surface_blend[2]"),
    (0x213F68, "owners.race.course.numbers", ".surface_blend[3]"),
    (0x213FAC, "owners.race.course.numbers", ".inertia[1]"),
    (0x213FB8, "owners.race.shared.physics_tables", ".rev_limits[0]"),
    (0x213FBC, "owners.race.shared.physics_tables", ".rev_limits[1]"),
    (0x213FC0, "owners.race.shared.physics_tables", ".camera_drop"),
    (0x213FC8, "owners.race.shared.physics_tables", ".gravity"),
    (0x213FCC, "owners.race.course.numbers", ".air_drag"),
    (0x213FD0, "owners.race.course.numbers", ".rolling_drag"),
    (0x213FD4, "owners.race.course.numbers", ".free_rev_add"),
    (0x213FD8, "owners.race.course.numbers", ".free_rev_scale"),
    (0x213FE4, "owners.race.course.numbers", ".rev_top_scale"),
    (0x213FEC, "owners.race.course.numbers", ".wheel_load_scale"),
    (0x214098, "owners.race.course.numbers", ".camera_frame_scale"),
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
    (0x214234, "owners.race.physics.wheel_work", ".wheel_slips[0]"),
    (0x214238, "owners.race.physics.wheel_work", ".wheel_slips[1]"),
    (0x21423C, "owners.race.shared.physics_tables", ".drive_limits[0]"),
    (0x214240, "owners.race.shared.physics_tables", ".drive_limits[1]"),
    (0x214244, "owners.race.shared.physics_tables", ".drive_limits[2]"),
    (0x214248, "owners.race.physics.wheel_work", ".drive_forces[0]"),
    (0x21424C, "owners.race.physics.wheel_work", ".drive_forces[1]"),
    (0x214250, "owners.race.physics.wheel_work", ".drive_slip[0]"),
    (0x214254, "owners.race.physics.wheel_work", ".drive_slip[1]"),
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
    (0x223AE4, "owners.race.shared.buffers.engine_tables", ".sines[0]"),
    (0x223AE8, "owners.race.shared.buffers.engine_tables", ".sines[1]"),
    (0x223AF4, "owners.race.shared.buffers.engine_tables", ".sines[4]"),
    (0x223DA8, "owners.race.shared.vars.mud_scale_a", "[0]"),
    (0x223DAC, "owners.race.shared.vars.mud_scale_a", "[1]"),
    (0x22400C, "owners.scene.draw_work", ".scratch"),
    (0x22401C, "owners.scene.draw_work", ".work_float_a"),
    (0x224020, "owners.scene.draw_work", ".work_float_b"),
    (0x224040, "owners.scene.draw_work", ".surface_point[0]"),
    (0x224044, "owners.scene.draw_work", ".surface_point[1]"),
    (0x224048, "owners.scene.draw_work", ".surface_point[2]"),
    (0x237638, "owners.scene.line_and_shake", ".camera_shake_x"),
    (0x23763C, "owners.scene.line_and_shake", ".camera_shake_y"),
    (0x237640, "owners.scene.line_and_shake", ".camera_shake_z"),
    (0x237644, "owners.scene.line_and_shake", ".camera_shake_listener"),
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
    (0x2D2A14, "owners.race.physics.probes", ".probe_share_u[0]"),
    (0x2D2A18, "owners.race.physics.probes", ".probe_share_u[1]"),
    (0x2D2A24, "owners.race.physics.probes", ".probe_share_u[4]"),
    (0x2D2B14, "owners.race.physics.probes", ".probe_share_v[0]"),
    (0x2D2B18, "owners.race.physics.probes", ".probe_share_v[1]"),
    (0x2D2B24, "owners.race.physics.probes", ".probe_share_v[4]"),
    (0x2D307C, "owners.race.physics.contact", ".contact_normal[1]"),
    (0x2D3090, "owners.race.physics.contact", ".contact_point[0]"),
    (0x2D3094, "owners.race.physics.contact", ".contact_point[1]"),
    (0x2D3098, "owners.race.physics.contact", ".contact_point[2]"),
    (0x2D30A8, "owners.race.physics.contact", ".contact_car_speed[0]"),
    (0x2D30AC, "owners.race.physics.contact", ".contact_car_speed[1]"),
    (0x2D30B0, "owners.race.physics.contact", ".contact_car_speed[2]"),
    (0x2D3100, "owners.race.physics.contact", ".contact_slide_limit"),
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

/// The course's counts and places (plain fields, not cells): their registry name.
const COURSE_WORK: &str = "owners.race.course.course";

/// The course record's field at `path` (the ones the recorded vectors touch).
fn course_word<'a>(
    work: &'a mut crate::race::course_store::CourseWork,
    path: &str,
) -> Option<&'a mut u32> {
    match path {
        ".flat_depth" => Some(&mut work.flat_depth),
        _ => None,
    }
}

/// The view variables and records (plain fields, not cells): its registry name.
const VIEW: &str = "owners.race.frame.view_and_records";

/// How a field of the view variables or records is read and written as bits.
type ViewBits = (fn(&ViewAndRecords) -> u32, fn(&mut ViewAndRecords, u32));

/// The view variables' or records' field at `path` (the ones the recorded vectors touch).
fn view_field(path: &str) -> Option<ViewBits> {
    macro_rules! float {
        ($($f:tt)+) => {
            Some((
                |v| v.$($f)+.to_bits(),
                |v, b| v.$($f)+ = f32::from_bits(b),
            ))
        };
    }
    macro_rules! word {
        ($($f:tt)+) => {
            Some((|v| v.$($f)+, |v, b| v.$($f)+ = b))
        };
    }
    match path {
        ".view_vars.camera_bias_scale" => float!(view_vars.camera_bias_scale),
        ".view_vars.camera_bias_limit" => float!(view_vars.camera_bias_limit),
        ".view_vars.camera_vector[0]" => float!(view_vars.camera_vector[0]),
        ".view_vars.camera_vector[1]" => float!(view_vars.camera_vector[1]),
        ".view_vars.camera_vector[2]" => float!(view_vars.camera_vector[2]),
        ".view_vars.camera_distance" => float!(view_vars.camera_distance),
        ".records[1].focus[1]" => word!(records[1].focus[1]),
        ".records[1].focus[2]" => word!(records[1].focus[2]),
        ".records[2].focus[0]" => word!(records[2].focus[0]),
        ".records[2].focus[1]" => word!(records[2].focus[1]),
        ".records[2].focus[2]" => word!(records[2].focus[2]),
        ".records[3].focus[0]" => word!(records[3].focus[0]),
        ".records[3].focus[1]" => word!(records[3].focus[1]),
        ".records[3].focus[2]" => word!(records[3].focus[2]),
        ".records[6].offset[1]" => word!(records[6].offset[1]),
        ".records[6].offset[2]" => word!(records[6].offset[2]),
        ".records[6].angles[0]" => word!(records[6].angles[0]),
        ".records[6].angles[1]" => word!(records[6].angles[1]),
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

/// The scene's work words (plain fields, not cells): their registry name.
const DRAW_WORK: &str = "owners.scene.draw_work";

/// How a field of the scene's work words is read and written as bits.
type WorkBits = (
    fn(&crate::scene::draw_store::DrawWork) -> u32,
    fn(&mut crate::scene::draw_store::DrawWork, u32),
);

/// The work words' field at `path`.
fn work_field(path: &str) -> Option<WorkBits> {
    match path {
        ".scratch" => Some((|w| w.scratch, |w, v| w.scratch = v)),
        ".work_float_a" => Some((
            |w| w.work_float_a.to_bits(),
            |w, v| w.work_float_a = f32::from_bits(v),
        )),
        ".work_float_b" => Some((
            |w| w.work_float_b.to_bits(),
            |w, v| w.work_float_b = f32::from_bits(v),
        )),
        ".surface_point[0]" => Some((|w| w.surface_point[0], |w, v| w.surface_point[0] = v)),
        ".surface_point[1]" => Some((|w| w.surface_point[1], |w, v| w.surface_point[1] = v)),
        ".surface_point[2]" => Some((|w| w.surface_point[2], |w, v| w.surface_point[2] = v)),
        _ => None,
    }
}

/// The start and tables work (not walked by path: its one place here is `view_share`).
const TABLES: &str = "owners.race.shared.tables";
/// The race's work variables (a plain record, not walked): its one place here is `patch_factors[1]`.
const WORK: &str = "owners.race.shared.work";

/// The best-line place and the camera shake (plain fields, not cells): their registry name.
const LINE_AND_SHAKE: &str = "owners.scene.line_and_shake";

/// How a field of the best-line place and camera shake is read and written as bits.
type ShakeBits = (
    fn(&crate::scene::line_shake_store::LineAndShake) -> u32,
    fn(&mut crate::scene::line_shake_store::LineAndShake, u32),
);

/// The best-line and shake record's field at `path`.
fn shake_field(path: &str) -> Option<ShakeBits> {
    match path {
        ".camera_shake_x" => Some((|w| w.camera_shake_x, |w, v| w.camera_shake_x = v)),
        ".camera_shake_y" => Some((|w| w.camera_shake_y, |w, v| w.camera_shake_y = v)),
        ".camera_shake_z" => Some((|w| w.camera_shake_z, |w, v| w.camera_shake_z = v)),
        ".camera_shake_listener" => Some((
            |w| w.camera_shake_listener,
            |w, v| w.camera_shake_listener = v,
        )),
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

/// The probe lists (plain fields, not cells): their registry name.
const PROBES: &str = "owners.race.physics.probes";

/// How a field of the probe lists is read and written as bits.
type ProbeBits = (fn(&ProbeLists) -> u32, fn(&mut ProbeLists, u32));

/// The probe lists' field at `path` (the shares the recorded vectors touch).
fn probes_field(path: &str) -> Option<ProbeBits> {
    macro_rules! share {
        ($list:ident, $i:expr) => {
            Some((
                |p| p.$list[$i].to_bits(),
                |p, v| p.$list[$i] = f32::from_bits(v),
            ))
        };
    }
    match path {
        ".probe_share_u[0]" => share!(probe_share_u, 0),
        ".probe_share_u[1]" => share!(probe_share_u, 1),
        ".probe_share_u[4]" => share!(probe_share_u, 4),
        ".probe_share_v[0]" => share!(probe_share_v, 0),
        ".probe_share_v[1]" => share!(probe_share_v, 1),
        ".probe_share_v[4]" => share!(probe_share_v, 4),
        _ => None,
    }
}

/// The contact work (plain fields, not cells): its registry name.
const CONTACT: &str = "owners.race.physics.contact";

/// The word of the contact work at `path` (`.name` or `.name[index]`).
fn contact_word<'a>(work: &'a mut ContactWork, path: &str) -> Option<&'a mut u32> {
    let (name, index) = match path.strip_suffix(']') {
        Some(head) => {
            let (name, index) = head.rsplit_once('[')?;
            (name, index.parse::<usize>().ok()?)
        }
        None => (path, 0),
    };
    let array = match name {
        ".contact_normal" => &mut work.contact_normal,
        ".contact_point" => &mut work.contact_point,
        ".contact_car_speed" => &mut work.contact_car_speed,
        ".object_point_offset" => &mut work.object_point_offset,
        ".object_point_turned" => &mut work.object_point_turned,
        ".object_point_scaled" => &mut work.object_point_scaled,
        ".contact_slide_limit" => return Some(&mut work.contact_slide_limit),
        ".face_angle" => return Some(&mut work.face_angle),
        ".face_speed" => return Some(&mut work.face_speed),
        _ => return None,
    };
    array.get_mut(index)
}

/// The drive's working figures (plain fields, not cells): their registry name.
const WHEEL_WORK: &str = "owners.race.physics.wheel_work";

/// The word of the drive's working figures at `path` (`.name[index]`).
fn wheel_word<'a>(work: &'a mut WheelWork, path: &str) -> Option<&'a mut u32> {
    let (name, index) = path.strip_suffix(']')?.rsplit_once('[')?;
    let array = match name {
        ".wheel_slips" => &mut work.wheel_slips[..],
        ".drive_forces" => &mut work.drive_forces[..],
        ".drive_slip" => &mut work.drive_slip[..],
        _ => return None,
    };
    array.get_mut(index.parse::<usize>().ok()?)
}

/// The gearbox figures (plain cells, not walked): their registry name.
const GEAR: &str = "owners.race.physics.gear";

/// The gearbox figures' cell at `path`.
fn gear_cell<'a>(
    gear: &'a crate::physics::gear_store::GearFigures,
    path: &str,
) -> Option<&'a dyn crate::memory::session::Cell32> {
    Some(match path {
        ".shift_up_point" => &gear.shift_up_point,
        ".shift_down_point" => &gear.shift_down_point,
        ".shift_down_clutch_point" => &gear.shift_down_clutch_point,
        ".shift_ratio_figure" => &gear.shift_ratio_figure,
        ".drive_torque_figure" => &gear.drive_torque_figure,
        ".figures[0]" => &gear.figures[0],
        ".figures[1]" => &gear.figures[1],
        ".figures[2]" => &gear.figures[2],
        ".figures[3]" => &gear.figures[3],
        ".test_marks[0]" => &gear.test_marks[0],
        ".test_marks[1]" => &gear.test_marks[1],
        ".test_marks[2]" => &gear.test_marks[2],
        _ => return None,
    })
}

/// The registry name the game's float constants are read under.
const NUMBERS: &str = "owners.race.course.numbers";

thread_local! {
    /// The values a test put into the constants' places (the constants themselves are fixed): the
    /// code under test is handed them (`number_constant`).
    static NUMBER_OVERRIDES: std::cell::RefCell<std::collections::HashMap<String, u32>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// The constant at `path` as a test last put it, else as the game holds it.
pub fn number_constant(path: &str) -> crate::memory::session::ConstFloat {
    crate::memory::session::ConstFloat::from_bits(number_bits(path).expect("a constant's path"))
}

/// The field of the constants at `path`.
fn number_field<'a>(
    n: &'a mut crate::race::numbers_store::Numbers,
    path: &str,
) -> Option<&'a mut crate::memory::session::ConstFloat> {
    Some(match path {
        ".hundredth_a" => &mut n.hundredth_a,
        ".two_and_a_quarter" => &mut n.two_and_a_quarter,
        ".hundred" => &mut n.hundred,
        ".thousand" => &mut n.thousand,
        ".one" => &mut n.one,
        ".two" => &mut n.two,
        ".three" => &mut n.three,
        ".five" => &mut n.five,
        ".sixty_four" => &mut n.sixty_four,
        ".k65536" => &mut n.k65536,
        ".one_over_65536" => &mut n.one_over_65536,
        ".quarter" => &mut n.quarter,
        ".one_and_a_half" => &mut n.one_and_a_half,
        ".one_over_two_pi" => &mut n.one_over_two_pi,
        ".tenths[0]" => &mut n.tenths[0],
        ".kilometres_in_mile" => &mut n.kilometres_in_mile,
        ".k4096" => &mut n.k4096,
        ".k256" => &mut n.k256,
        ".k10000" => &mut n.k10000,
        ".half" => &mut n.half,
        ".inertia[1]" => &mut n.inertia[1],
        ".surface_blend[0]" => &mut n.surface_blend[0],
        ".surface_blend[1]" => &mut n.surface_blend[1],
        ".surface_blend[2]" => &mut n.surface_blend[2],
        ".surface_blend[3]" => &mut n.surface_blend[3],
        ".grip_slide_scale" => &mut n.grip_slide_scale,
        ".grip_slip_limit" => &mut n.grip_slip_limit,
        ".grip_force_limit" => &mut n.grip_force_limit,
        ".spec_scale_a4" => &mut n.spec_scale_a4,
        ".rev_limit_scale" => &mut n.rev_limit_scale,
        ".drive_speed_scale" => &mut n.drive_speed_scale,
        ".side_force_scale" => &mut n.side_force_scale,
        ".slip_blend" => &mut n.slip_blend,
        ".slip_share" => &mut n.slip_share,
        ".wheel_speed_margin" => &mut n.wheel_speed_margin,
        ".wheel_speed_gap" => &mut n.wheel_speed_gap,
        ".sound_speed_scale" => &mut n.sound_speed_scale,
        ".spec_scale_1b8" => &mut n.spec_scale_1b8,
        ".grip_ramp" => &mut n.grip_ramp,
        ".grip_ramp_share" => &mut n.grip_ramp_share,
        ".air_drag" => &mut n.air_drag,
        ".rolling_drag" => &mut n.rolling_drag,
        ".free_rev_add" => &mut n.free_rev_add,
        ".free_rev_scale" => &mut n.free_rev_scale,
        ".rev_top_scale" => &mut n.rev_top_scale,
        ".wheel_load_scale" => &mut n.wheel_load_scale,
        ".camera_frame_scale" => &mut n.camera_frame_scale,
        ".step_time" => &mut n.step_time,
        _ => return None,
    })
}

/// The bits of the constant at `path` (the constants are fixed: a test's value is kept aside).
fn number_bits(path: &str) -> Option<u32> {
    if let Some(bits) = NUMBER_OVERRIDES.with(|o| o.borrow().get(path).copied()) {
        return Some(bits);
    }
    let mut n = super::own::race().course.numbers;
    number_field(&mut n, path).map(|c| c.bits())
}

/// The game's constants with the values a test put into their places.
pub fn numbers() -> crate::race::numbers_store::Numbers {
    let mut n = super::own::race().course.numbers;
    NUMBER_OVERRIDES.with(|o| {
        for (path, bits) in o.borrow().iter() {
            if let Some(c) = number_field(&mut n, path) {
                *c = crate::memory::session::ConstFloat::from_bits(*bits);
            }
        }
    });
    n
}

/// The physics' tables (a plain record): their registry name.
const PHYSICS_TABLES: &str = "owners.race.shared.physics_tables";

/// The engine tables (plain arrays, not cells): their registry name.
const ENGINE_TABLES: &str = "owners.race.shared.buffers.engine_tables";

/// The index of the sine table's entry a path `.sines[n]` names.
fn sine_index(path: &str) -> Option<usize> {
    path.strip_prefix(".sines[")?
        .strip_suffix(']')?
        .parse()
        .ok()
}

/// The bytes of the field at `path` in the store `name` (the registry's, or a plain one).
fn store_bytes(name: &str, path: &str) -> Option<Vec<u8>> {
    if name == CONTACT {
        let mut work = super::own::race().physics.contact().borrow().clone();
        return Some(contact_word(&mut work, path)?.to_le_bytes().to_vec());
    }
    if name == NUMBERS {
        return Some(number_bits(path)?.to_le_bytes().to_vec());
    }
    if name == COURSE_WORK {
        let mut work = *super::own::race().course.course.borrow();
        return Some(course_word(&mut work, path)?.to_le_bytes().to_vec());
    }
    if name == WHEEL_WORK {
        let mut work = super::own::race().physics.wheel_work().borrow().clone();
        return Some(wheel_word(&mut work, path)?.to_le_bytes().to_vec());
    }
    if name == GEAR {
        let cell = gear_cell(super::own::race().physics.gear(), path)?;
        return Some(cell.bits().to_le_bytes().to_vec());
    }
    if name == FINISH {
        let (get, _) = finish_field(path)?;
        return Some(
            get(&super::own::race().replay.finish())
                .to_le_bytes()
                .to_vec(),
        );
    }
    if name == DRAW_WORK {
        let (get, _) = work_field(path)?;
        return Some(get(&super::own::scene().draw_work()).to_le_bytes().to_vec());
    }
    if name == LINE_AND_SHAKE {
        let (get, _) = shake_field(path)?;
        return Some(
            get(&super::own::scene().line_and_shake())
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
    if name == PROBES {
        let (get, _) = probes_field(path)?;
        return Some(
            get(&super::own::race().physics.probes().borrow())
                .to_le_bytes()
                .to_vec(),
        );
    }
    if name == ENGINE_TABLES {
        let n = sine_index(path)?;
        let tables = super::own::race().shared.buffers.engine_tables.borrow();
        return Some(tables.sines.get(n)?.to_bits().to_le_bytes().to_vec());
    }
    if name == WORK && path == ".patch_factors[1]" {
        let factor = super::own::race().shared.work().patch_factors[1];
        return Some(factor.to_bits().to_le_bytes().to_vec());
    }
    if name == PHYSICS_TABLES {
        let mut tables = super::own::race().shared.physics_tables().clone();
        let word = tables.word_mut(path)?;
        return Some(word.bits().to_le_bytes().to_vec());
    }
    if name == TABLES && path == ".view_share" {
        let share = super::own::race().shared.tables().view_share.get();
        return Some(share.to_bits().to_le_bytes().to_vec());
    }
    if name == VIEW {
        let (get, _) = view_field(path)?;
        return Some(
            get(&super::own::race().frame.view_and_records.borrow())
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
    if name == CONTACT {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        let mut work = super::own::race().physics.contact().borrow_mut();
        let Some(slot) = contact_word(&mut work, path) else {
            return false;
        };
        *slot = u32::from_le_bytes(word);
        return true;
    }
    if name == NUMBERS {
        // The constants are fixed: a test's value is kept aside for the code under test.
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        if number_bits(path).is_none() {
            return false;
        }
        NUMBER_OVERRIDES.with(|o| {
            o.borrow_mut()
                .insert(path.to_string(), u32::from_le_bytes(word))
        });
        return true;
    }
    if name == COURSE_WORK {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        let mut work = super::own::race().course.course.borrow_mut();
        let Some(slot) = course_word(&mut work, path) else {
            return false;
        };
        *slot = u32::from_le_bytes(word);
        return true;
    }
    if name == WHEEL_WORK {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        let mut work = super::own::race().physics.wheel_work().borrow_mut();
        let Some(slot) = wheel_word(&mut work, path) else {
            return false;
        };
        *slot = u32::from_le_bytes(word);
        return true;
    }
    if name == GEAR {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        let Some(cell) = gear_cell(super::own::race().physics.gear(), path) else {
            return false;
        };
        cell.set_bits(u32::from_le_bytes(word));
        return true;
    }
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
    if name == DRAW_WORK {
        let (Some((_, set)), Ok(word)) = (work_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::scene().draw_work_mut(),
            u32::from_le_bytes(word),
        );
        return true;
    }
    if name == LINE_AND_SHAKE {
        let (Some((_, set)), Ok(word)) = (shake_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::scene().line_and_shake_mut(),
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
    if name == PROBES {
        let (Some((_, set)), Ok(word)) = (probes_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::race().physics.probes().borrow_mut(),
            u32::from_le_bytes(word),
        );
        return true;
    }
    if name == ENGINE_TABLES {
        let (Some(n), Ok(word)) = (sine_index(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        let mut tables = super::own::race().shared.buffers.engine_tables.borrow_mut();
        let Some(entry) = tables.sines.get_mut(n) else {
            return false;
        };
        *entry = f32::from_bits(u32::from_le_bytes(word));
        return true;
    }
    if name == WORK && path == ".patch_factors[1]" {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        super::own::race().shared.work_mut().patch_factors[1] =
            f32::from_bits(u32::from_le_bytes(word));
        return true;
    }
    if name == PHYSICS_TABLES {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        let mut tables = super::own::race().shared.physics_tables_mut();
        let Some(slot) = tables.word_mut(path) else {
            return false;
        };
        slot.set_bits(u32::from_le_bytes(word));
        return true;
    }
    if name == TABLES && path == ".view_share" {
        let Ok(word) = <[u8; 4]>::try_from(bytes) else {
            return false;
        };
        let share = &super::own::race().shared.tables().view_share;
        share.set(f32::from_bits(u32::from_le_bytes(word)));
        return true;
    }
    if name == VIEW {
        let (Some((_, set)), Ok(word)) = (view_field(path), <[u8; 4]>::try_from(bytes)) else {
            return false;
        };
        set(
            &mut super::own::race().frame.view_and_records.borrow_mut(),
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
