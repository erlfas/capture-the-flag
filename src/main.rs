mod ai;
mod arena;
mod camera;
mod combat;
mod koth;
mod player;
mod terrain_heights;
mod types;
mod ui;

use avian3d::prelude::*;
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

#[derive(Component)]
pub struct BackgroundMusic;

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
        .add_systems(Update, toggle_background_music)
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 3rd-person Orbit Follow Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 35.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        camera::OrbitCamera::default(),
    ));

    // Spawn looping background soundtrack ("Breach at Sector Nine")
    commands.spawn((
        BackgroundMusic,
        AudioPlayer::new(asset_server.load("music/Breach_at_Sector_Nine.mp3")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.55),
            ..default()
        },
    ));
}

fn toggle_background_music(
    keys: Res<ButtonInput<KeyCode>>,
    music_query: Query<&AudioSink, With<BackgroundMusic>>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        for sink in music_query.iter() {
            sink.toggle();
        }
    }
}
