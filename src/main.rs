mod ai;
mod arena;
mod camera;
mod combat;
mod koth;
mod player;
mod types;
mod ui;

use avian3d::prelude::*;
use bevy::prelude::*;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Cyber-Athletic Coliseum — 3v3 King of the Hill".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        // Avian3D ECS Physics Plugin with realistic Earth gravity (9.81 m/s²)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
        // Game Systems
        .add_plugins((
            camera::CameraPlugin,
            arena::ArenaPlugin,
            koth::KothPlugin,
            player::PlayerPlugin,
            combat::CombatPlugin,
            ai::AiPlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    // Spawn 3rd-person Orbit Follow Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 35.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        camera::OrbitCamera::default(),
    ));
}
