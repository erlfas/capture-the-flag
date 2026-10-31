use avian3d::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::arena::{playable_max_x, playable_min_x, PLAYABLE_MAX_Z, PLAYABLE_MIN_Z};
use crate::player::Player;
use crate::terrain_heights::sample_terrain_height;
use crate::types::{Character, PauseState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    TacticalIsometric,
    #[default]
    ThirdPersonAction,
}

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Option<Entity>,
    pub mode: CameraMode,
    // Active yaw and pitch exposed for systems (e.g. player movement)
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub height: f32,
    pub sensitivity: f32,
    // Third-person action mode parameters
    pub tp_yaw: f32,
    pub tp_pitch: f32,
    pub tp_distance: f32,
    pub tp_height: f32,
    // Tactical isometric (high-distance overview) mode parameters
    pub iso_yaw: f32,
    pub iso_pitch: f32,
    pub iso_distance: f32,
    pub iso_height: f32,
    // Smooth damping states
    pub current_pos: Vec3,
    pub current_look_at: Vec3,
    pub initialized: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        // High-distance tactical overview angle matching the reference art:
        // Positioned on the South (+Z) looking North (-Z) with a slight isometric angle,
        // high up so the entire compound and platforms are clearly visible while following the player.
        let iso_yaw = -0.18; // Slight isometric angle (~ -10 deg)
        let iso_pitch = 0.88; // ~50 degrees downward angle from high altitude
        let iso_dist = 42.0; // High vantage distance giving a broad overview of the compound
        let iso_height = 1.5;

        let tp_yaw = -std::f32::consts::FRAC_PI_2; // Directly behind Anders facing East (+X) from West Blue base
        let tp_pitch = 0.18; // Low over-the-shoulder pitch so both the arena ahead and dark storm skies above are visible
        let tp_dist = 6.5;
        let tp_height = 1.65;

        Self {
            target: None,
            mode: CameraMode::ThirdPersonAction,
            yaw: tp_yaw,
            pitch: tp_pitch,
            distance: tp_dist,
            height: tp_height,
            sensitivity: 0.004,
            tp_yaw,
            tp_pitch,
            tp_distance: tp_dist,
            tp_height,
            iso_yaw,
            iso_pitch,
            iso_distance: iso_dist,
            iso_height,
            current_pos: Vec3::ZERO,
            current_look_at: Vec3::ZERO,
            initialized: false,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_cursor_lock, update_camera));
    }
}

fn handle_cursor_lock(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    pause_state: Res<PauseState>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<&OrbitCamera>,
) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        return;
    };

    if pause_state.is_paused || pause_state.exited {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        return;
    }

    let is_third_person = camera_query
        .get_single()
        .map(|c| c.mode == CameraMode::ThirdPersonAction)
        .unwrap_or(false);

    // Only lock cursor on Left-Click in Third-Person mode; in Tactical Overview mode keep cursor free
    if mouse_buttons.just_pressed(MouseButton::Left) && is_third_person {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

    if keys.just_pressed(KeyCode::Escape) || !is_third_person {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn update_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    pause_state: Res<PauseState>,
    spatial_query: SpatialQuery,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    player_query: Query<Entity, With<Player>>,
    character_entities: Query<Entity, With<Character>>,
    transforms: Query<&Transform, Without<OrbitCamera>>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        return;
    };

    // Sum up mouse movement this frame
    let mut mouse_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        mouse_delta += motion.delta;
    }

    // Sum up mouse scroll this frame
    let mut scroll_delta = 0.0;
    for scroll in mouse_wheel.read() {
        scroll_delta += scroll.y;
    }

    if pause_state.is_paused || pause_state.exited {
        return;
    }

    let dt = time.delta_secs();
    let is_locked = window.cursor_options.grab_mode == CursorGrabMode::Locked;
    // Allow steering camera by holding Right Mouse Button or Middle Mouse Button
    let is_dragging =
        mouse_buttons.pressed(MouseButton::Right) || mouse_buttons.pressed(MouseButton::Middle);

    for (mut orbit, mut cam_transform) in camera_query.iter_mut() {
        // 1. Ensure camera target is attached to the player
        if orbit.target.is_none() {
            if let Ok(player_entity) = player_query.get_single() {
                orbit.target = Some(player_entity);
            }
        }

        let Some(target_entity) = orbit.target else {
            continue;
        };
        let Ok(target_transform) = transforms.get(target_entity) else {
            continue;
        };

        let char_forward = (target_transform.rotation * Vec3::Z).normalize_or(Vec3::X);
        let target_behind_yaw = (-char_forward.x).atan2(-char_forward.z);

        // 2. Toggle camera mode with KeyV or Tab
        if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::Tab) {
            orbit.mode = match orbit.mode {
                CameraMode::TacticalIsometric => {
                    orbit.tp_yaw = target_behind_yaw;
                    CameraMode::ThirdPersonAction
                }
                CameraMode::ThirdPersonAction => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                    CameraMode::TacticalIsometric
                }
            };
        }

        // 3. Reset camera angle with R
        if keys.just_pressed(KeyCode::KeyR) {
            orbit.iso_yaw = -0.18;
            orbit.iso_pitch = 0.88;
            orbit.iso_distance = 42.0;
            orbit.tp_yaw = target_behind_yaw;
            orbit.tp_pitch = 0.18;
            orbit.tp_distance = 6.5;
        }

        // 4. Keyboard Steering (Q / E and Arrow Keys rotate & tilt camera)
        let mut key_yaw_delta = 0.0;
        let mut key_pitch_delta = 0.0;
        let rot_speed = 2.2 * dt;

        if keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::ArrowLeft) {
            key_yaw_delta += rot_speed;
        }
        if keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::ArrowRight) {
            key_yaw_delta -= rot_speed;
        }
        if keys.pressed(KeyCode::ArrowUp) {
            key_pitch_delta += rot_speed * 0.6;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            key_pitch_delta -= rot_speed * 0.6;
        }

        // 5. Mouse Wheel Zoom
        if scroll_delta.abs() > 0.01 {
            match orbit.mode {
                CameraMode::TacticalIsometric => {
                    orbit.iso_distance =
                        (orbit.iso_distance - scroll_delta * 3.0).clamp(18.0, 68.0);
                }
                CameraMode::ThirdPersonAction => {
                    orbit.tp_distance = (orbit.tp_distance - scroll_delta * 1.0).clamp(3.5, 16.0);
                }
            }
        }

        // 6. Update Angles based on Mode & Input
        match orbit.mode {
            CameraMode::TacticalIsometric => {
                // High-distance overview mode: steer with RMB/MMB drag or Q/E/Arrows
                if is_dragging {
                    orbit.iso_yaw -= mouse_delta.x * orbit.sensitivity;
                    orbit.iso_pitch =
                        (orbit.iso_pitch + mouse_delta.y * orbit.sensitivity).clamp(0.45, 1.35);
                }
                orbit.iso_yaw += key_yaw_delta;
                orbit.iso_pitch = (orbit.iso_pitch + key_pitch_delta).clamp(0.45, 1.35);

                orbit.yaw = orbit.iso_yaw;
                orbit.pitch = orbit.iso_pitch;
                orbit.distance = orbit.iso_distance;
                orbit.height = orbit.iso_height;
            }
            CameraMode::ThirdPersonAction => {
                let manual_mouse = (is_locked || is_dragging) && mouse_delta.length_squared() > 0.01;
                let manual_key = key_yaw_delta.abs() > 0.0001 || key_pitch_delta.abs() > 0.0001;

                if is_locked || is_dragging {
                    orbit.tp_yaw -= mouse_delta.x * orbit.sensitivity;
                    orbit.tp_pitch -= mouse_delta.y * orbit.sensitivity;
                }
                orbit.tp_yaw += key_yaw_delta;
                orbit.tp_pitch = (orbit.tp_pitch + key_pitch_delta).clamp(-0.15, 1.15);

                // Smoothly align camera behind the character's back when moving/facing forward
                // (paused while holding S to backpedal or while actively rotating with mouse/keys)
                let is_backpedaling = keys.pressed(KeyCode::KeyS)
                    && !keys.pressed(KeyCode::KeyW)
                    && !keys.pressed(KeyCode::KeyA)
                    && !keys.pressed(KeyCode::KeyD);
                if !manual_mouse && !manual_key && !is_backpedaling {
                    let mut diff = (target_behind_yaw - orbit.tp_yaw) % std::f32::consts::TAU;
                    if diff > std::f32::consts::PI {
                        diff -= std::f32::consts::TAU;
                    } else if diff < -std::f32::consts::PI {
                        diff += std::f32::consts::TAU;
                    }
                    let align_rate = if keys.pressed(KeyCode::KeyW)
                        || keys.pressed(KeyCode::KeyA)
                        || keys.pressed(KeyCode::KeyD)
                    {
                        3.4
                    } else {
                        2.0
                    };
                    orbit.tp_yaw += diff * (align_rate * dt).min(1.0);
                }

                orbit.yaw = orbit.tp_yaw;
                orbit.pitch = orbit.tp_pitch;
                orbit.distance = orbit.tp_distance;
                orbit.height = orbit.tp_height;
            }
        }

        // 7. Smooth Damping, Wall Avoidance & Overhead Look-Down Adjustment
        let mut focus_point = match orbit.mode {
            CameraMode::TacticalIsometric => {
                let player_pos = target_transform.translation;
                Vec3::new(player_pos.x * 0.78, player_pos.y, player_pos.z * 0.78)
                    + Vec3::Y * orbit.height
            }
            CameraMode::ThirdPersonAction => {
                target_transform.translation + Vec3::Y * orbit.height
            }
        };

        let rotation = Quat::from_axis_angle(Vec3::Y, orbit.yaw)
            * Quat::from_axis_angle(Vec3::X, -orbit.pitch);

        let mut desired_cam_pos = focus_point + rotation * Vec3::new(0.0, 0.0, orbit.distance);

        if orbit.mode == CameraMode::ThirdPersonAction {
            let ideal_offset = desired_cam_pos - focus_point;
            let ideal_horiz = Vec2::new(ideal_offset.x, ideal_offset.z);
            let ideal_horiz_dist = ideal_horiz.length().max(0.001);
            let horiz_dir = ideal_horiz / ideal_horiz_dist;

            // A) Physics Raycast Spring-Arm: pull camera in front of interior walls / shipping containers
            let excluded: Vec<Entity> = character_entities.iter().collect();
            let filter = SpatialQueryFilter::from_excluded_entities(excluded);
            let ray_vec = desired_cam_pos - focus_point;
            let ray_len = ray_vec.length();
            if ray_len > 0.1 {
                if let Ok(ray_dir) = Dir3::new(ray_vec / ray_len) {
                    if let Some(hit) =
                        spatial_query.cast_ray(focus_point, ray_dir, ray_len, true, &filter)
                    {
                        let safe_dist = (hit.distance - 0.55).max(1.4);
                        if safe_dist < ray_len {
                            desired_cam_pos = focus_point + (ray_vec / ray_len) * safe_dist;
                        }
                    }
                }
            }

            // B) Inner-Perimeter Fence Clamping so camera never goes behind the tall outer barbed-wire fences
            //    while staying at least 0.25m on the behind-side of the player even when flat against a wall
            let fence_allowance = 0.25;
            let cam_min_z = PLAYABLE_MIN_Z - fence_allowance;
            let cam_max_z = PLAYABLE_MAX_Z + fence_allowance;
            desired_cam_pos.z = desired_cam_pos.z.clamp(cam_min_z, cam_max_z);

            let cam_min_x = playable_min_x(desired_cam_pos.z) - fence_allowance;
            let cam_max_x = playable_max_x(desired_cam_pos.z) + fence_allowance;
            desired_cam_pos.x = desired_cam_pos.x.clamp(cam_min_x, cam_max_x);

            // C) Compute how much the wall/fence compressed the horizontal camera distance,
            //    and smoothly transition to an elevated "look-down on character" vantage when near walls!
            let clamped_horiz = Vec2::new(
                desired_cam_pos.x - focus_point.x,
                desired_cam_pos.z - focus_point.z,
            );
            let actual_horiz_dist = clamped_horiz.length();
            let wall_compression =
                (1.0 - (actual_horiz_dist / ideal_horiz_dist)).clamp(0.0, 1.0);

            if wall_compression > 0.01 {
                // Smoothly rise above the wall/fence and angle downward onto the character
                // while shifting the look target slightly ahead of the character so the camera
                // never flips backward even when pressed flat against a wall.
                let forward_xz = -horiz_dir;
                let lift_y = wall_compression * 5.2;
                let forward_look_shift = wall_compression * 1.85;
                desired_cam_pos.y += lift_y;
                focus_point.x += forward_xz.x * forward_look_shift;
                focus_point.z += forward_xz.y * forward_look_shift;
                focus_point.y -= wall_compression * 0.75;
            }

            // D) Terrain & Ledge Clearance along the camera arm so ledges/stairs behind the player
            //    never bury the camera or block line of sight
            for sample_t in [0.45_f32, 0.75_f32, 1.0_f32] {
                let sx = target_transform.translation.x
                    + (desired_cam_pos.x - target_transform.translation.x) * sample_t;
                let sz = target_transform.translation.z
                    + (desired_cam_pos.z - target_transform.translation.z) * sample_t;
                let terr_y = sample_terrain_height(sx, sz);
                let min_cam_y = terr_y + 1.45;
                if desired_cam_pos.y < min_cam_y {
                    desired_cam_pos.y = min_cam_y;
                }
            }
        }

        if !orbit.initialized {
            orbit.current_pos = desired_cam_pos;
            orbit.current_look_at = focus_point;
            orbit.initialized = true;
        } else {
            let follow_speed = match orbit.mode {
                CameraMode::TacticalIsometric => 8.0,
                CameraMode::ThirdPersonAction => 14.0,
            };
            orbit.current_pos = orbit
                .current_pos
                .lerp(desired_cam_pos, (follow_speed * dt).min(1.0));
            orbit.current_look_at = orbit
                .current_look_at
                .lerp(focus_point, (follow_speed * dt).min(1.0));

            if orbit.mode == CameraMode::ThirdPersonAction {
                let fence_allowance = 0.25;
                orbit.current_pos.z = orbit
                    .current_pos
                    .z
                    .clamp(PLAYABLE_MIN_Z - fence_allowance, PLAYABLE_MAX_Z + fence_allowance);
                let min_x = playable_min_x(orbit.current_pos.z) - fence_allowance;
                let max_x = playable_max_x(orbit.current_pos.z) + fence_allowance;
                orbit.current_pos.x = orbit.current_pos.x.clamp(min_x, max_x);
            }
        }

        cam_transform.translation = orbit.current_pos;
        cam_transform.look_at(orbit.current_look_at, Vec3::Y);
    }
}
