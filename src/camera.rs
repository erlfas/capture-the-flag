use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::player::Player;
use crate::types::PauseState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    #[default]
    TacticalIsometric,
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

        let tp_yaw = -std::f32::consts::FRAC_PI_2; // Facing East (+X) towards center from West Blue base
        let tp_pitch = 0.35;
        let tp_dist = 8.5;
        let tp_height = 2.0;

        Self {
            target: None,
            mode: CameraMode::TacticalIsometric,
            yaw: iso_yaw,
            pitch: iso_pitch,
            distance: iso_dist,
            height: iso_height,
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

fn update_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    pause_state: Res<PauseState>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    player_query: Query<Entity, With<Player>>,
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

        // 2. Toggle camera mode with KeyV or Tab
        if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::Tab) {
            orbit.mode = match orbit.mode {
                CameraMode::TacticalIsometric => CameraMode::ThirdPersonAction,
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
        }

        // 4. Keyboard Steering (Q / E and Arrow Keys rotate & tilt camera)
        let mut key_yaw_delta = 0.0;
        let mut key_pitch_delta = 0.0;
        let rot_speed = 2.0 * dt;

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
                    orbit.tp_distance = (orbit.tp_distance - scroll_delta * 1.0).clamp(4.0, 18.0);
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
                if is_locked || is_dragging {
                    orbit.tp_yaw -= mouse_delta.x * orbit.sensitivity;
                    orbit.tp_pitch -= mouse_delta.y * orbit.sensitivity;
                }
                orbit.tp_yaw += key_yaw_delta;
                orbit.tp_pitch = (orbit.tp_pitch + key_pitch_delta).clamp(0.1, 1.25);

                orbit.yaw = orbit.tp_yaw;
                orbit.pitch = orbit.tp_pitch;
                orbit.distance = orbit.tp_distance;
                orbit.height = orbit.tp_height;
            }
        }

        // 7. Smooth Damping & High-Distance Target Tracking
        let Some(target_entity) = orbit.target else {
            continue;
        };
        let Ok(target_transform) = transforms.get(target_entity) else {
            continue;
        };

        // In Tactical Overview mode, follow the character while blending 20% towards arena center
        // so the player always sees both their character and a great overview of the compound.
        let focus_point = match orbit.mode {
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

        let desired_cam_pos = focus_point + rotation * Vec3::new(0.0, 0.0, orbit.distance);

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
        }

        cam_transform.translation = orbit.current_pos;
        cam_transform.look_at(orbit.current_look_at, Vec3::Y);
    }
}
