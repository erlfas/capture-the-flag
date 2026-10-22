use avian3d::prelude::*;
use bevy::prelude::*;

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
    //    (Calibrated for assets/models/post_apocalyptic_battlefield.glb)
    // =========================================================================
    commands.insert_resource(ClearColor(Color::srgb(0.16, 0.17, 0.19)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.85, 0.86, 0.90),
        brightness: 1100.0,
    });

    // Warm dusk horizon directional light from the North-West
    commands.spawn((
        DirectionalLight {
            illuminance: 24000.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.88, 0.72),
            ..default()
        },
        Transform::from_xyz(-15.0, 42.0, -45.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Cool overcast sky fill from the South-East
    commands.spawn((
        DirectionalLight {
            illuminance: 14000.0,
            shadows_enabled: false,
            color: Color::srgb(0.68, 0.76, 0.88),
            ..default()
        },
        Transform::from_xyz(20.0, 48.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Vivid Blue PointLight pools on the West Blue Platform (-20.2, 2.55, 1.8)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.15, 0.72, 1.0),
            intensity: 2_200_000.0,
            range: 24.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-20.2, 5.2, 1.8),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.82, 1.0),
            intensity: 1_200_000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-20.2, 11.4, 1.8),
    ));

    // Vivid Red PointLight pools on the East Red Platform (19.5, 2.55, 1.8)
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.22, 0.14),
            intensity: 2_200_000.0,
            range: 24.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(19.5, 5.2, 1.8),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.32, 0.18),
            intensity: 1_200_000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(19.5, 11.4, 1.8),
    ));

    // Watchtower Searchlight PointLights (North-West at (-16.5, 9.5, -7.8) & North-East at (16.8, 9.5, -7.8))
    for wx in [-14.0, 14.0] {
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.95, 0.78),
                intensity: 1_100_000.0,
                range: 22.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(wx, 8.5, -7.8),
        ));
    }

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
    // 3. PRECISION PHYSICS COLLIDERS ALIGNED TO THE 3D MODEL GEOMETRY
    // =========================================================================
    // Main Courtyard Ground Floor Collider (top at y = 0.0m)
    commands.spawn((
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(64.0, 1.0, 48.0),
    ));

    // West Blue Platform Deck Collider (x in [-24.8, -15.4], z in [-7.5, 10.2], top at y = 2.53m)
    let (beta_idx, beta_name, beta_center, beta_radius) = PLATEAU_COORDS[1];
    commands.spawn((
        Transform::from_xyz(-20.1, 1.265, 1.35),
        RigidBody::Static,
        Collider::cuboid(9.4, 2.53, 17.7),
        Plateau {
            index: beta_idx,
            name: beta_name,
            center: beta_center,
            radius: beta_radius,
        },
    ));

    // West Blue Platform — Upper East Stepped Ramp (connects x = -15.4, y = 2.53 down to x = -10.4, y = 0.0)
    commands.spawn((
        Transform::from_xyz(-12.9, 1.26, -0.5).with_rotation(Quat::from_rotation_z(-0.44)),
        RigidBody::Static,
        Collider::cuboid(5.8, 0.4, 3.6),
    ));

    // West Blue Platform — Lower East Concrete Slab Ramp (connects x = -15.4, y = 2.53 down to x = -10.2, y = 0.0)
    commands.spawn((
        Transform::from_xyz(-12.8, 1.26, 6.5).with_rotation(Quat::from_rotation_z(-0.44)),
        RigidBody::Static,
        Collider::cuboid(6.0, 0.4, 4.2),
    ));

    // East Red Platform Deck Collider (x in [15.0, 23.8], z in [-7.5, 10.2], top at y = 2.53m)
    let (gamma_idx, gamma_name, gamma_center, gamma_radius) = PLATEAU_COORDS[2];
    commands.spawn((
        Transform::from_xyz(19.4, 1.265, 1.35),
        RigidBody::Static,
        Collider::cuboid(8.8, 2.53, 17.7),
        Plateau {
            index: gamma_idx,
            name: gamma_name,
            center: gamma_center,
            radius: gamma_radius,
        },
    ));

    // East Red Platform — West Crumbled Ramp & Stairs (connects x = 15.0, y = 2.53 down to x = 10.2, y = 0.0)
    commands.spawn((
        Transform::from_xyz(12.6, 1.26, 2.5).with_rotation(Quat::from_rotation_z(0.44)),
        RigidBody::Static,
        Collider::cuboid(5.8, 0.4, 7.6),
    ));

    // East Red Platform — South Staircase (connects z = 10.2, y = 2.53 down to z = 14.6, y = 0.0)
    commands.spawn((
        Transform::from_xyz(19.5, 1.26, 12.4).with_rotation(Quat::from_rotation_x(0.46)),
        RigidBody::Static,
        Collider::cuboid(3.4, 0.4, 5.2),
    ));

    // Central Courtyard Objective Marker
    let (alpha_idx, alpha_name, alpha_center, alpha_radius) = PLATEAU_COORDS[0];
    commands.spawn((
        Transform::from_translation(alpha_center),
        Plateau {
            index: alpha_idx,
            name: alpha_name,
            center: alpha_center,
            radius: alpha_radius,
        },
    ));

    // Shipping Containers in the Upper Courtyard (matching 3D model)
    // Left-North Olive Container
    commands.spawn((
        Transform::from_xyz(-9.5, 1.9, -10.8).with_rotation(Quat::from_rotation_y(0.72)),
        RigidBody::Static,
        Collider::cuboid(7.6, 3.8, 3.1),
    ));
    // Right-North Rust Container
    commands.spawn((
        Transform::from_xyz(3.8, 1.9, -10.2).with_rotation(Quat::from_rotation_y(-0.22)),
        RigidBody::Static,
        Collider::cuboid(7.6, 3.8, 3.1),
    ));

    // Central Sandbag Walls & Concrete Jersey Barriers (matching 3D model)
    for (cx, cz, cw, cd, cyaw) in [
        (-4.2, -6.2, 4.2, 1.6, 0.08),
        (-6.8, -1.0, 1.2, 5.8, 0.05),
        (6.2, -1.0, 1.2, 6.2, 0.12),
        (-5.6, 8.0, 1.6, 5.2, -0.32),
        (4.8, 7.2, 4.4, 1.6, 0.05),
        (7.2, 9.5, 1.4, 4.8, 0.10),
    ] {
        commands.spawn((
            Transform::from_xyz(cx, 0.65, cz).with_rotation(Quat::from_rotation_y(cyaw)),
            RigidBody::Static,
            Collider::cuboid(cw, 1.3, cd),
        ));
    }

    // Wrecked Vehicles (matching 3D model)
    for (vx, vz, vw, vd, vyaw) in [
        (-10.5, 16.2, 5.2, 2.4, -0.22),
        (12.2, 16.2, 5.8, 2.4, 0.08),
        (11.8, -13.2, 4.0, 2.1, 0.35),
    ] {
        commands.spawn((
            Transform::from_xyz(vx, 0.85, vz).with_rotation(Quat::from_rotation_y(vyaw)),
            RigidBody::Static,
            Collider::cuboid(vw, 1.7, vd),
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
        base_color: Color::srgb(0.18, 0.19, 0.15),
        perceptual_roughness: 0.98,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(340.0, 1.0, 340.0))),
        MeshMaterial3d(dark_wasteland_mat),
        Transform::from_xyz(0.0, -5.65, 0.0),
    ));

    let horizon_amber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.54, 0.45, 0.32),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let mid_storm_slate = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.24, 0.26),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let upper_charcoal_cloud = materials.add(StandardMaterial {
        base_color: Color::srgb(0.13, 0.14, 0.16),
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
            MeshMaterial3d(horizon_amber.clone()),
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
        if transform.translation.y < -10.0 {
            transform.translation = char_info.spawn_point;
            lin_vel.0 = Vec3::ZERO;
            ang_vel.0 = Vec3::ZERO;
            stagger.duration = 0.5;
        }
    }
}
