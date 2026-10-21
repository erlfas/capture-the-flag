use bevy::prelude::*;

use crate::arena::PLATEAU_COORDS;
use crate::types::{Character, Team};

pub const ROTATION_INTERVAL: f32 = 45.0;
pub const WIN_SCORE: f32 = 100.0;
pub const INITIAL_MATCH_TIME: f32 = 462.0; // 07:42 in seconds, matching art reference

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneContestState {
    Neutral,
    Contested,
    BlueControlling,
    RedControlling,
}

#[derive(Resource)]
pub struct KothManager {
    pub active_index: usize,
    pub rotation_timer: f32,
    pub match_timer: f32,
    pub blue_score: f32,
    pub red_score: f32,
    pub contest_state: ZoneContestState,
    pub blue_occupants: usize,
    pub red_occupants: usize,
    pub winner: Option<Team>,
}

impl Default for KothManager {
    fn default() -> Self {
        Self {
            active_index: 0,
            rotation_timer: ROTATION_INTERVAL,
            match_timer: INITIAL_MATCH_TIME,
            blue_score: 1.0, // Matches initial "Blå Flagg: 1" from art
            red_score: 0.0,  // Matches initial "Rød Flagg: 0" from art
            contest_state: ZoneContestState::Neutral,
            blue_occupants: 0,
            red_occupants: 0,
            winner: None,
        }
    }
}

#[derive(Component)]
pub struct HologramZone;

pub struct KothPlugin;

impl Plugin for KothPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KothManager>()
            .add_systems(Startup, setup_hologram_beacon)
            .add_systems(Update, (update_koth_zone, update_hologram_visuals));
    }
}

fn setup_hologram_beacon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (_idx, _, center, radius) = PLATEAU_COORDS[0];

    // Low-profile glowing ground capture disc matching the reference art
    let cylinder_mesh = meshes.add(Cylinder::new(radius, 0.14));
    let holo_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.9, 1.0, 0.25),
        emissive: LinearRgba::new(0.0, 3.0, 4.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(cylinder_mesh),
        MeshMaterial3d(holo_mat),
        Transform::from_translation(center + Vec3::new(0.0, 0.08, 0.0)),
        HologramZone,
    ));
}

fn update_koth_zone(
    time: Res<Time>,
    mut koth: ResMut<KothManager>,
    characters: Query<(&Transform, &Character)>,
) {
    let dt = time.delta_secs();

    // Match countdown timer
    if koth.match_timer > 0.0 {
        koth.match_timer = (koth.match_timer - dt).max(0.0);
    }

    if koth.winner.is_some() {
        return;
    }

    // 1. Zone Rotation Timer
    koth.rotation_timer -= dt;
    if koth.rotation_timer <= 0.0 {
        koth.active_index = (koth.active_index + 1) % PLATEAU_COORDS.len();
        koth.rotation_timer = ROTATION_INTERVAL;
    }

    // 2. Scan occupants on the active plateau
    let (_, _, center, radius) = PLATEAU_COORDS[koth.active_index];
    let mut blue_count = 0;
    let mut red_count = 0;

    for (transform, char_info) in characters.iter() {
        let pos = transform.translation;
        let horiz_dist_sq =
            (pos.x - center.x) * (pos.x - center.x) + (pos.z - center.z) * (pos.z - center.z);
        let vert_dist = (pos.y - center.y).abs();

        if horiz_dist_sq <= radius * radius && vert_dist <= 2.5 {
            match char_info.team {
                Team::Blue => blue_count += 1,
                Team::Red => red_count += 1,
            }
        }
    }

    koth.blue_occupants = blue_count;
    koth.red_occupants = red_count;

    // 3. Resolve contest status & points
    if blue_count > 0 && red_count > 0 {
        koth.contest_state = ZoneContestState::Contested;
    } else if blue_count > 0 {
        koth.contest_state = ZoneContestState::BlueControlling;
        koth.blue_score += dt * 1.5;
        if koth.blue_score >= WIN_SCORE {
            koth.blue_score = WIN_SCORE;
            koth.winner = Some(Team::Blue);
        }
    } else if red_count > 0 {
        koth.contest_state = ZoneContestState::RedControlling;
        koth.red_score += dt * 1.5;
        if koth.red_score >= WIN_SCORE {
            koth.red_score = WIN_SCORE;
            koth.winner = Some(Team::Red);
        }
    } else {
        koth.contest_state = ZoneContestState::Neutral;
    }
}

fn update_hologram_visuals(
    koth: Res<KothManager>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<HologramZone>>,
) {
    let (_, _, target_center, _radius) = PLATEAU_COORDS[koth.active_index];

    for (mut transform, mat_handle) in query.iter_mut() {
        // Position ground capture disc over active plateau
        transform.translation = target_center + Vec3::new(0.0, 0.08, 0.0);

        if let Some(mat) = materials.get_mut(mat_handle) {
            match koth.contest_state {
                ZoneContestState::Neutral => {
                    mat.base_color = Color::srgba(0.0, 0.8, 1.0, 0.22);
                    mat.emissive = LinearRgba::new(0.0, 1.5, 2.5, 1.0);
                }
                ZoneContestState::Contested => {
                    mat.base_color = Color::srgba(1.0, 0.8, 0.0, 0.35);
                    mat.emissive = LinearRgba::new(3.5, 2.5, 0.0, 1.0);
                }
                ZoneContestState::BlueControlling => {
                    mat.base_color = Color::srgba(0.0, 0.5, 1.0, 0.35);
                    mat.emissive = LinearRgba::new(0.0, 3.5, 6.0, 1.0);
                }
                ZoneContestState::RedControlling => {
                    mat.base_color = Color::srgba(1.0, 0.2, 0.15, 0.35);
                    mat.emissive = LinearRgba::new(6.0, 0.8, 0.8, 1.0);
                }
            }
        }
    }
}
