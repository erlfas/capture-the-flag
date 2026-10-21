use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    #[default]
    ThirdPersonAction,
    TacticalIsometric,
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
    // Tactical isometric mode parameters
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
        let iso_yaw = -std::f32::consts::FRAC_PI_4; // -45 degrees isometric angle
        let iso_pitch = 0.82; // ~47 degrees downward angle
        let iso_dist = 26.0; // High-angle tactical vantage
        let iso_height = 1.2;

        let tp_yaw = std::f32::consts::PI; // Facing forward towards arena center
        let tp_pitch = 0.22;
        let tp_dist = 6.0;
        let tp_height = 2.0;

        Self {
            target: None,
            mode: CameraMode::ThirdPersonAction,
            yaw: tp_yaw,
            pitch: tp_pitch,
            distance: tp_dist,
            height: tp_height,
            sensitivity: 0.0035,
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
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<&OrbitCamera>,
) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        return;
    };

    let is_third_person = camera_query
        .get_single()
        .map(|c| c.mode == CameraMode::ThirdPersonAction)
        .unwrap_or(true);

    // Left click locks mouse in third person
    if mouse_buttons.just_pressed(MouseButton::Left) && is_third_person {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

    if keys.just_pressed(KeyCode::Escape) {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn update_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
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

    let dt = time.delta_secs();
    let is_locked = window.cursor_options.grab_mode == CursorGrabMode::Locked;
    let is_dragging = mouse_buttons.pressed(MouseButton::Right) || (mouse_buttons.pressed(MouseButton::Left) && !is_locked);

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

    for (mut orbit, mut cam_transform) in camera_query.iter_mut() {
        // 1. Ensure camera target is attached to the player (resolves startup race condition)
        if orbit.target.is_none() {
            if let Ok(player_entity) = player_query.get_single() {
                orbit.target = Some(player_entity);
            }
        }

        // 2. Toggle camera mode with KeyV or Tab
        if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::Tab) {
            orbit.mode = match orbit.mode {
                CameraMode::TacticalIsometric => {
                    CameraMode::ThirdPersonAction
                }
                CameraMode::ThirdPersonAction => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                    CameraMode::TacticalIsometric
                }
            };
        }

        // 3. Keyboard Steering (Q / E and Arrow Keys rotate camera)
        let mut key_yaw_delta = 0.0;
        let mut key_pitch_delta = 0.0;
        let rot_speed = 2.4 * dt;

        if keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::ArrowLeft) {
            key_yaw_delta += rot_speed;
        }
        if keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::ArrowRight) {
            key_yaw_delta -= rot_speed;
        }
        if keys.pressed(KeyCode::ArrowUp) {
            key_pitch_delta += rot_speed * 0.7;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            key_pitch_delta -= rot_speed * 0.7;
        }

        // 4. Mouse Wheel Zoom
        if scroll_delta.abs() > 0.01 {
            match orbit.mode {
                CameraMode::TacticalIsometric => {
                    orbit.iso_distance = (orbit.iso_distance - scroll_delta * 2.5).clamp(14.0, 45.0);
                }
                CameraMode::ThirdPersonAction => {
                    orbit.tp_distance = (orbit.tp_distance - scroll_delta * 0.8).clamp(3.0, 15.0);
                }
            }
        }

        // 5. Update Angles based on Mode & Input
        match orbit.mode {
            CameraMode::ThirdPersonAction => {
                // Steer if cursor locked OR dragging with mouse
                if is_locked || is_dragging {
                    orbit.tp_yaw -= mouse_delta.x * orbit.sensitivity;
                    orbit.tp_pitch -= mouse_delta.y * orbit.sensitivity;
                }
                orbit.tp_yaw += key_yaw_delta;
                orbit.tp_pitch = (orbit.tp_pitch + key_pitch_delta).clamp(-0.85, 1.3);

                orbit.yaw = orbit.tp_yaw;
                orbit.pitch = orbit.tp_pitch;
                orbit.distance = orbit.tp_distance;
                orbit.height = orbit.tp_height;
            }
            CameraMode::TacticalIsometric => {
                // In Isometric, steer via dragging or Q/E
                if is_dragging {
                    orbit.iso_yaw -= mouse_delta.x * orbit.sensitivity * 0.8;
                }
                orbit.iso_yaw += key_yaw_delta;
                orbit.iso_pitch = (orbit.iso_pitch + key_pitch_delta).clamp(0.4, 1.4);

                orbit.yaw = orbit.iso_yaw;
                orbit.pitch = orbit.iso_pitch;
                orbit.distance = orbit.iso_distance;
                orbit.height = orbit.iso_height;
            }
        }

        // 6. Smooth Damping & Target Tracking
        let Some(target_entity) = orbit.target else {
            continue;
        };
        let Ok(target_transform) = transforms.get(target_entity) else {
            continue;
        };

        let target_pos = target_transform.translation + Vec3::Y * orbit.height;

        let rotation = Quat::from_axis_angle(Vec3::Y, orbit.yaw)
            * Quat::from_axis_angle(Vec3::X, -orbit.pitch);

        let desired_cam_pos = target_pos + rotation * Vec3::new(0.0, 0.0, orbit.distance);

        if !orbit.initialized {
            orbit.current_pos = desired_cam_pos;
            orbit.current_look_at = target_pos;
            orbit.initialized = true;
        } else {
            let follow_speed = match orbit.mode {
                CameraMode::TacticalIsometric => 10.0,
                CameraMode::ThirdPersonAction => 16.0,
            };
            orbit.current_pos = orbit
                .current_pos
                .lerp(desired_cam_pos, (follow_speed * dt).min(1.0));
            orbit.current_look_at = orbit
                .current_look_at
                .lerp(target_pos, (follow_speed * dt).min(1.0));
        }

        cam_transform.translation = orbit.current_pos;
        cam_transform.look_at(orbit.current_look_at, Vec3::Y);
    }
}
