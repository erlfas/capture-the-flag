use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

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
        let iso_dist = 28.0; // High-angle tactical vantage
        let iso_height = 1.2;

        Self {
            target: None,
            mode: CameraMode::TacticalIsometric,
            yaw: iso_yaw,
            pitch: iso_pitch,
            distance: iso_dist,
            height: iso_height,
            sensitivity: 0.003,
            tp_yaw: std::f32::consts::PI,
            tp_pitch: 0.25,
            tp_distance: 6.2,
            tp_height: 2.2,
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
        .unwrap_or(false);

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
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    transforms: Query<&Transform, Without<OrbitCamera>>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let is_locked = window.cursor_options.grab_mode == CursorGrabMode::Locked;

    for (mut orbit, mut cam_transform) in camera_query.iter_mut() {
        // 1. Toggle camera mode with KeyV or Tab
        if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::Tab) {
            orbit.mode = match orbit.mode {
                CameraMode::TacticalIsometric => {
                    // Unlock cursor when entering third-person until player clicks
                    CameraMode::ThirdPersonAction
                }
                CameraMode::ThirdPersonAction => {
                    // Unlock cursor in isometric mode for easy UI & overview interaction
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                    CameraMode::TacticalIsometric
                }
            };
        }

        // 2. Mouse Wheel Zoom Handling
        for scroll in mouse_wheel.read() {
            match orbit.mode {
                CameraMode::TacticalIsometric => {
                    orbit.iso_distance = (orbit.iso_distance - scroll.y * 2.5).clamp(16.0, 42.0);
                }
                CameraMode::ThirdPersonAction => {
                    orbit.tp_distance = (orbit.tp_distance - scroll.y * 0.8).clamp(3.5, 12.0);
                }
            }
        }

        // 3. Mouse Look & Perspective Alignment
        match orbit.mode {
            CameraMode::ThirdPersonAction => {
                if is_locked {
                    for motion in mouse_motion.read() {
                        orbit.tp_yaw -= motion.delta.x * orbit.sensitivity;
                        orbit.tp_pitch -= motion.delta.y * orbit.sensitivity;
                        orbit.tp_pitch = orbit.tp_pitch.clamp(-0.85, 1.3);
                    }
                }
                orbit.yaw = orbit.tp_yaw;
                orbit.pitch = orbit.tp_pitch;
                orbit.distance = orbit.tp_distance;
                orbit.height = orbit.tp_height;
            }
            CameraMode::TacticalIsometric => {
                // Tactical elevated angle
                orbit.yaw = orbit.iso_yaw;
                orbit.pitch = orbit.iso_pitch;
                orbit.distance = orbit.iso_distance;
                orbit.height = orbit.iso_height;
            }
        }

        // 4. Smooth Damping & Target Tracking
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
                CameraMode::TacticalIsometric => 8.0,
                CameraMode::ThirdPersonAction => 14.0,
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
