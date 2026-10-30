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
        .init_resource::<types::PauseState>()
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
        .add_systems(
            Update,
            (toggle_background_music, handle_background_music_match_start),
        )
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 3rd-person Orbit Follow Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 35.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        camera::OrbitCamera::default(),
    ));

    // Spawn looping background soundtrack ("Breach at Sector Nine"), paused until match begins
    commands.spawn((
        BackgroundMusic,
        AudioPlayer::new(asset_server.load("music/Breach_at_Sector_Nine.mp3")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.55),
            paused: true,
            ..default()
        },
    ));
}

fn handle_background_music_match_start(
    koth: Res<koth::KothManager>,
    pause_state: Res<types::PauseState>,
    music_query: Query<&AudioSink, With<BackgroundMusic>>,
    voice_query: Query<&AudioSink, (With<koth::ActiveVoiceAudio>, Without<BackgroundMusic>)>,
) {
    let should_pause_all = !koth.match_started || pause_state.is_paused || pause_state.exited;

    for sink in music_query.iter() {
        if should_pause_all || pause_state.music_muted {
            if !sink.is_paused() {
                sink.pause();
            }
        } else if sink.is_paused() {
            sink.play();
        }
    }

    for sink in voice_query.iter() {
        if should_pause_all {
            if !sink.is_paused() {
                sink.pause();
            }
        } else if sink.is_paused() {
            sink.play();
        }
    }
}

fn toggle_background_music(
    keys: Res<ButtonInput<KeyCode>>,
    mut pause_state: ResMut<types::PauseState>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        pause_state.music_muted = !pause_state.music_muted;
    }
}
