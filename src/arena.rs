use avian3d::prelude::*;
use bevy::prelude::*;

use crate::terrain_heights::{
    sample_terrain_height, TERRAIN_HEIGHTS, TERRAIN_NX, TERRAIN_NZ, TERRAIN_STEP, TERRAIN_X_MIN,
    TERRAIN_Z_MIN,
};
use crate::types::{Character, StaggerState};

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_arena)
            .add_systems(Update, handle_fallout_respawns);
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Plateau {
    pub index: usize,
    pub name: &'static str,
    pub center: Vec3,
    pub radius: f32,
}

// Exact coordinates matching the 3D model (assets/models/post_apocalyptic_battlefield.glb)
pub const PLATEAU_COORDS: [(usize, &str, Vec3, f32); 3] = [
    (
        0,
        "Sentralsonen (Ruined Courtyard)",
        Vec3::new(0.0, 0.1, 0.0),
        5.2,
    ),
    (
        1,
        "Blå Base (Vestplattform)",
        Vec3::new(-20.2, 2.55, 1.8),
        4.6,
    ),
    (
        2,
        "Rød Base (Østplattform)",
        Vec3::new(19.5, 2.55, 1.8),
        4.6,
    ),
];

fn setup_arena(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // =========================================================================
    // 1. POST-APOCALYPTIC LIGHTING & ATMOSPHERE
    //    (Calibrated to match snapshot_from_3d_model.png in WebGL2)
    // =========================================================================
    commands.insert_resource(ClearColor(Color::srgb(0.11, 0.11, 0.13)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.90, 0.91, 0.94),
        brightness: 680.0,
    });

    // Warm key directional light from the North-West
    commands.spawn((
        DirectionalLight {
            illuminance: 11500.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.93, 0.86),
            ..default()
        },
        Transform::from_xyz(-15.0, 42.0, -45.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Subtle cool sky fill from the South-East
    commands.spawn((
        DirectionalLight {
            illuminance: 6200.0,
            shadows_enabled: false,
            color: Color::srgb(0.76, 0.82, 0.92),
            ..default()
        },
        Transform::from_xyz(20.0, 48.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // =========================================================================
    // 2. SPAWN THE 3D BATTLEFIELD MODEL (post_apocalyptic_battlefield.glb)
    // =========================================================================
    commands.spawn((
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0).from_asset("models/post_apocalyptic_battlefield.glb"),
            ),
        ),
        Transform::IDENTITY,
    ));

    // =========================================================================
    // 3. PRECISION INVISIBLE PHYSICS COLLIDERS ALIGNED TO THE 3D MODEL GEOMETRY
    //    (No visible Mesh3d on any collider so 100% of the 3D model's ground
    //     textures, trenches, moss, and platforms remain completely uncovered)
    // =========================================================================
    // Exact 0.5m-resolution 3D Terrain Trimesh Collider extracted directly from
    // assets/models/post_apocalyptic_battlefield.glb (121 x 85 grid covering [-30, 30] x [-21, 21])
    let mut terrain_verts = Vec::with_capacity(TERRAIN_NX * TERRAIN_NZ);
    for ix in 0..TERRAIN_NX {
        let x = TERRAIN_X_MIN + (ix as f32) * TERRAIN_STEP;
        for iz in 0..TERRAIN_NZ {
            let z = TERRAIN_Z_MIN + (iz as f32) * TERRAIN_STEP;
            let y = TERRAIN_HEIGHTS[ix * TERRAIN_NZ + iz];
            terrain_verts.push(Vec3::new(x, y, z));
        }
    }
    let mut terrain_indices = Vec::with_capacity((TERRAIN_NX - 1) * (TERRAIN_NZ - 1) * 2);
    for ix in 0..(TERRAIN_NX - 1) {
        for iz in 0..(TERRAIN_NZ - 1) {
            let i00 = (ix * TERRAIN_NZ + iz) as u32;
            let i01 = (ix * TERRAIN_NZ + iz + 1) as u32;
            let i10 = ((ix + 1) * TERRAIN_NZ + iz) as u32;
            let i11 = ((ix + 1) * TERRAIN_NZ + iz + 1) as u32;
            terrain_indices.push([i00, i01, i10]);
            terrain_indices.push([i10, i01, i11]);
        }
    }
    commands.spawn((
        Transform::IDENTITY,
        RigidBody::Static,
        Collider::trimesh(terrain_verts, terrain_indices),
    ));

    // Outer Safety Floor Collider well below the deepest trench (top at y = -3.8m)
    commands.spawn((
        Transform::from_xyz(0.0, -4.3, 0.0),
        RigidBody::Static,
        Collider::cuboid(68.0, 1.0, 52.0),
    ));

    // Plateau Objective Markers (all ground, platform, ramp, staircase, shipping container,
    // and sandbag collision is handled seamlessly by the 0.5m-resolution 3D Terrain Trimesh Collider above)
    for (idx, name, center, radius) in PLATEAU_COORDS {
        commands.spawn((
            Transform::from_translation(center),
            Plateau {
                index: idx,
                name,
                center,
                radius,
            },
        ));
    }

    // Perimeter Fence Physics Boundaries (keeps players & bots inside the compound)
    for (bx, bz, bw, bd) in [
        (0.0, -17.2, 52.0, 0.6),
        (0.0, 18.2, 52.0, 0.6),
        (-24.8, 0.0, 0.6, 36.0),
        (24.2, 0.0, 0.6, 36.0),
    ] {
        commands.spawn((
            Transform::from_xyz(bx, 3.0, bz),
            RigidBody::Static,
            Collider::cuboid(bw, 6.0, bd),
        ));
    }

    // =========================================================================
    // 4. OPEN-TOP STORMY POST-APOCALYPTIC SKY BACKDROP
    // =========================================================================
    spawn_stormy_sky_ring(&mut commands, &mut meshes, &mut materials);
}

fn spawn_stormy_sky_ring(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let dark_wasteland_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.11, 0.13),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(340.0, 1.0, 340.0))),
        MeshMaterial3d(dark_wasteland_mat),
        Transform::from_xyz(0.0, -6.5, 0.0),
    ));

    let horizon_charcoal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.12, 0.14),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let mid_storm_slate = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.11, 0.13),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let upper_charcoal_cloud = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.10, 0.12),
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let num_sky_segments = 24;
    let sky_radius = 165.0_f32;
    let seg_width = 2.0 * std::f32::consts::PI * sky_radius / (num_sky_segments as f32) + 3.0;
    let band_low_mesh = meshes.add(Cuboid::new(seg_width, 28.0, 1.0));
    let band_mid_mesh = meshes.add(Cuboid::new(seg_width, 34.0, 1.0));
    let band_high_mesh = meshes.add(Cuboid::new(seg_width, 55.0, 1.0));

    for i in 0..num_sky_segments {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_sky_segments as f32);
        let sx = sky_radius * angle.sin();
        let sz = sky_radius * angle.cos();
        let rot = Quat::from_rotation_y(angle);

        commands.spawn((
            Mesh3d(band_low_mesh.clone()),
            MeshMaterial3d(horizon_charcoal.clone()),
            Transform::from_xyz(sx, 8.0, sz).with_rotation(rot),
        ));
        commands.spawn((
            Mesh3d(band_mid_mesh.clone()),
            MeshMaterial3d(mid_storm_slate.clone()),
            Transform::from_xyz(sx, 36.0, sz).with_rotation(rot),
        ));
        commands.spawn((
            Mesh3d(band_high_mesh.clone()),
            MeshMaterial3d(upper_charcoal_cloud.clone()),
            Transform::from_xyz(sx, 78.0, sz).with_rotation(rot),
        ));
    }
}

fn handle_fallout_respawns(
    mut characters: Query<(
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut StaggerState,
        &Character,
    )>,
) {
    for (mut transform, mut lin_vel, mut ang_vel, mut stagger, char_info) in characters.iter_mut()
    {
        // Always prevent any physics-induced angular velocity from spinning characters
        ang_vel.0 = Vec3::ZERO;

        if transform.translation.y < -10.0 {
            transform.translation = char_info.spawn_point;
            lin_vel.0 = Vec3::ZERO;
            stagger.duration = 0.5;
            continue;
        }

        // Guarantee characters never clip into the 3D model ground anywhere on the map:
        // Boots sole is at -0.90m below transform.translation.y (capsule center)
        let ground_y = sample_terrain_height(transform.translation.x, transform.translation.z);
        let min_center_y = ground_y + 0.90;
        if transform.translation.y < min_center_y {
            transform.translation.y = min_center_y;
            if lin_vel.y < 0.0 {
                lin_vel.y = 0.0;
            }
        }
    }
}
