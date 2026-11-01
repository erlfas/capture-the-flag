use avian3d::prelude::*;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::terrain_heights::{
    sample_terrain_height, TERRAIN_HEIGHTS, TERRAIN_NX, TERRAIN_NZ, TERRAIN_STEP, TERRAIN_X_MIN,
    TERRAIN_Z_MIN,
};
use crate::types::{Character, StaggerState};

pub struct ArenaPlugin;

pub const PLAYABLE_MIN_Z: f32 = -14.6;
pub const PLAYABLE_MAX_Z: f32 = 16.0;

/// Exact Z-dependent inner West fence boundary matching the 3D compound's perspective-angled barbed-wire fence.
pub fn playable_min_x(z: f32) -> f32 {
    if z <= -6.5 {
        // North-West barbed-wire fence angles from x = -22.3 (at z = -15.0) to x = -23.1 (at z = -6.5)
        let u = ((z - (-15.0)) / (-6.5 - (-15.0))).clamp(0.0, 1.0);
        -21.65 - u * 0.55
    } else if z <= 11.0 {
        // West Blue Platform back fence & railing angles from x = -22.0 (at z = -6.5) to x = -22.8 (at z = 11.0)
        let u = ((z - (-6.5)) / (11.0 - (-6.5))).clamp(0.0, 1.0);
        -22.0 - u * 0.80
    } else {
        // South-West stairs & corner perimeter
        -21.65
    }
}

/// Exact Z-dependent inner East fence boundary matching the 3D compound's perspective-angled barbed-wire fence.
pub fn playable_max_x(z: f32) -> f32 {
    if z <= -6.5 {
        // North-East barbed-wire fence angles from x = 21.4 (at z = -15.0) to x = 22.2 (at z = -6.5)
        let u = ((z - (-15.0)) / (-6.5 - (-15.0))).clamp(0.0, 1.0);
        20.75 + u * 0.55
    } else if z <= 11.0 {
        // East Red Platform back fence & railing angles from x = 21.3 (at z = -6.5) to x = 22.1 (at z = 11.0)
        let u = ((z - (-6.5)) / (11.0 - (-6.5))).clamp(0.0, 1.0);
        21.30 + u * 0.80
    } else {
        // South-East staircase perimeter
        20.80
    }
}

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_arena)
            .add_systems(
                Update,
                (
                    animate_sewer_water,
                    animate_ominous_sky,
                ),
            )
            .add_systems(PostUpdate, handle_fallout_respawns.after(PhysicsSet::Sync));
    }
}

#[derive(Component)]
struct ScrollingSkyLayer {
    angular_speed: f32,
    base_y: f32,
    bob_amp: f32,
    bob_speed: f32,
    phase: f32,
}

#[derive(Component)]
struct SewerWaterPulse {
    base_pos: Vec3,
    freq: f32,
}

#[derive(Component)]
struct SewerWaterRipple {
    center: Vec3,
    drift_vel: Vec3,
    phase_offset: f32,
    period: f32,
    min_scale: f32,
    max_scale: f32,
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
        "Central Zone (Ruined Courtyard)",
        Vec3::new(0.0, 0.1, 0.0),
        5.2,
    ),
    (
        1,
        "Blue Base (West Platform)",
        Vec3::new(-20.2, 2.55, 1.8),
        4.6,
    ),
    (
        2,
        "Red Base (East Platform)",
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
    //    (Calibrated to match original_art.jpg & snapshot_from_3d_model.png)
    // =========================================================================
    commands.insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.88, 0.89, 0.92),
        brightness: 640.0,
    });

    // Warm sunset key directional light from the North-West
    commands.spawn((
        DirectionalLight {
            illuminance: 11000.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.92, 0.84),
            ..default()
        },
        Transform::from_xyz(-15.0, 42.0, -45.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Cool storm-slate sky fill from the South-East
    commands.spawn((
        DirectionalLight {
            illuminance: 5800.0,
            shadows_enabled: false,
            color: Color::srgb(0.74, 0.80, 0.90),
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
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));

    // Outer Safety Floor Collider well below the deepest trench (top at y = -3.8m)
    commands.spawn((
        Transform::from_xyz(0.0, -4.3, 0.0),
        RigidBody::Static,
        Collider::cuboid(68.0, 1.0, 52.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));

    // =========================================================================
    // SOLID SHIPPING CONTAINER COLLIDERS
    // Impassable, solid physics colliders aligned exactly with the 3D model's
    // shipping containers so players and bots cannot walk through broken panels
    // or stand inside hollow containers.
    // =========================================================================
    // Right-North Rust Shipping Container (near Red Base, x ≈ 3.82, z ≈ -10.63)
    commands.spawn((
        Transform::from_xyz(3.825, 2.30, -10.632)
            .with_rotation(Quat::from_rotation_y(-0.2134)),
        RigidBody::Static,
        Collider::cuboid(6.20, 3.60, 2.65),
        CollisionLayers::new(
            crate::types::GameLayer::Obstacle,
            [
                crate::types::GameLayer::Default,
                crate::types::GameLayer::Character,
            ],
        ),
    ));

    // Left-North Olive/Rust Shipping Container (near Blue Base, x ≈ -9.84, z ≈ -10.86)
    commands.spawn((
        Transform::from_xyz(-9.843, 2.30, -10.864)
            .with_rotation(Quat::from_rotation_y(0.7419)),
        RigidBody::Static,
        Collider::cuboid(6.25, 3.60, 2.60),
        CollisionLayers::new(
            crate::types::GameLayer::Obstacle,
            [
                crate::types::GameLayer::Default,
                crate::types::GameLayer::Character,
            ],
        ),
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

    // =========================================================================
    // 3. IMPENETRABLE PERIMETER BARRIER COLLIDERS (30m high, 20m+ deep solid walls)
    //    Angled to match the 3D model's perspective-tapered barbed-wire fences
    //    so no player or bot can ever tunnel, fly, or be pushed outside the fence perimeter
    // =========================================================================
    // West angled fence barrier (z from -20 to 11.5): inner face follows playable_min_x(z)
    commands.spawn((
        Transform::from_xyz(-32.25, 10.0, -2.0).with_rotation(Quat::from_rotation_y(-0.044)),
        RigidBody::Static,
        Collider::cuboid(20.0, 30.0, 42.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));
    // South-West corner barrier (z > 11.0): inner face at x = -21.65
    commands.spawn((
        Transform::from_xyz(-31.65, 10.0, 18.0),
        RigidBody::Static,
        Collider::cuboid(20.0, 30.0, 16.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));

    // East angled fence barrier (z from -20 to 11.5): inner face follows playable_max_x(z)
    commands.spawn((
        Transform::from_xyz(31.40, 10.0, -2.0).with_rotation(Quat::from_rotation_y(0.052)),
        RigidBody::Static,
        Collider::cuboid(20.0, 30.0, 42.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));
    // South-East corner barrier (z > 11.0): inner face at x = 20.80
    commands.spawn((
        Transform::from_xyz(30.80, 10.0, 18.0),
        RigidBody::Static,
        Collider::cuboid(20.0, 30.0, 16.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));

    // North barrier: inner face at z = PLAYABLE_MIN_Z (-14.6), 24m deep, 30m high, 75m wide
    commands.spawn((
        Transform::from_xyz(0.0, 10.0, PLAYABLE_MIN_Z - 12.0),
        RigidBody::Static,
        Collider::cuboid(75.0, 30.0, 24.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));

    // South barrier: inner face at z = PLAYABLE_MAX_Z (16.0), 24m deep, 30m high, 75m wide
    commands.spawn((
        Transform::from_xyz(0.0, 10.0, PLAYABLE_MAX_Z + 12.0),
        RigidBody::Static,
        Collider::cuboid(75.0, 30.0, 24.0),
        CollisionLayers::new(
            crate::types::GameLayer::Terrain,
            [crate::types::GameLayer::Character],
        ),
    ));

    // =========================================================================
    // 4. PAINTED POST-APOCALYPTIC STORM SKY DOME & MOVING CLOUD LAYERS
    // =========================================================================
    spawn_stormy_sky_ring(&mut commands, &asset_server, &mut meshes, &mut materials);

    // =========================================================================
    // 5. REALISTIC POURING SEWER WATERFALL & ANIMATED TRENCH WATER EFFECTS
    // =========================================================================
    spawn_sewer_water_effects(&mut commands, &mut meshes, &mut materials);
}

/// Builds a 360-degree UV-mapped sky dome/cylinder mesh that repeats a seamless
/// painted panorama `repeats` times around the horizon and curves inward at the upper dome.
/// Maps `v in [0.22, 0.85]` (the main cloud & sky body) across `[y_horizon, y_mid]` so the
/// painted storm clouds fill the 3rd-person camera's sky viewport directly above the fences.
fn build_sky_dome_mesh(
    radius: f32,
    repeats: usize,
    y_bottom: f32,
    y_horizon: f32,
    y_mid: f32,
    y_top: f32,
) -> Mesh {
    let segs_per_repeat = 32usize;
    let rings = 20usize;
    let span_angle = std::f32::consts::TAU / (repeats as f32);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for rep in 0..repeats {
        let base_vert = positions.len() as u32;
        let angle_offset = (rep as f32) * span_angle;

        for r_idx in 0..=rings {
            let v = (r_idx as f32) / (rings as f32); // 0.0 at top, 1.0 at bottom
            let (y, r) = if v >= 0.85 {
                // Below horizon skyline down to dark wasteland floor
                let t = (v - 0.85) / 0.15;
                let y = y_horizon + t * (y_bottom - y_horizon);
                (y, radius)
            } else if v >= 0.22 {
                // Main storm cloud bank & amber cloud break (fills camera sky viewport)
                let t = (0.85 - v) / (0.85 - 0.22);
                let y = y_horizon + t * (y_mid - y_horizon);
                let r = radius * (1.0 - 0.10 * t);
                (y, r)
            } else {
                // Upper storm vault curving inward overhead
                let t = (0.22 - v) / 0.22;
                let y = y_mid + t * (y_top - y_mid);
                let r = radius * (0.90 - 0.75 * t * t);
                (y, r)
            };

            for s_idx in 0..=segs_per_repeat {
                let u = (s_idx as f32) / (segs_per_repeat as f32);
                let theta = angle_offset + u * span_angle;
                let x = r * theta.sin();
                let z = r * theta.cos();
                positions.push([x, y, z]);
                normals.push([-theta.sin(), 0.0, -theta.cos()]);
                uvs.push([u, v]);
            }
        }

        let stride = (segs_per_repeat + 1) as u32;
        for r_idx in 0..(rings as u32) {
            for s_idx in 0..(segs_per_repeat as u32) {
                let i00 = base_vert + r_idx * stride + s_idx;
                let i01 = i00 + 1;
                let i10 = base_vert + (r_idx + 1) * stride + s_idx;
                let i11 = i10 + 1;
                // Wind inward toward the origin so standard backface culling culls the far exterior faces
                indices.push(i00);
                indices.push(i01);
                indices.push(i10);
                indices.push(i01);
                indices.push(i11);
                indices.push(i10);
            }
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn spawn_stormy_sky_ring(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // 1. Dark wasteland ground plane outside the compound perimeter
    let dark_wasteland_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.11),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(380.0, 1.0, 380.0))),
        MeshMaterial3d(dark_wasteland_mat),
        Transform::from_xyz(0.0, -6.5, 0.0),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 2. Overhead Zenith Storm Cap (matches top of sky_horizon_panorama.png)
    let zenith_cap_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.11),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(85.0, 1.0))),
        MeshMaterial3d(zenith_cap_mat),
        Transform::from_xyz(0.0, 59.5, 0.0),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 3. Outer 360-Degree Painted Dark Storm Sky & Low Horizon Silhouette Dome
    //    (Inward-wound dome uses standard backface culling to skip exterior backfaces)
    let pano_tex: Handle<Image> = asset_server.load("ui/sky_horizon_panorama.png");
    let pano_mat = materials.add(StandardMaterial {
        base_color_texture: Some(pano_tex),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(build_sky_dome_mesh(108.0, 2, -12.0, 2.6, 29.0, 60.0))),
        MeshMaterial3d(pano_mat),
        Transform::IDENTITY,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 4. Continuously Moving Painted Storm Cloud Layers (2 Consolidated Parallax Domes)
    //    With backface culling enabled and 2 layers, transparent sky passes drop from 8 to 2!
    let clouds_tex_1: Handle<Image> = asset_server.load("ui/storm_clouds_layer1.png");
    let clouds_tex_2: Handle<Image> = asset_server.load("ui/storm_clouds_layer2.png");

    // Layer A: Heavy rolling dark thunderheads sweeping steadily across the sky
    let cloud_mat_a = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.96, 1.0, 0.90),
        base_color_texture: Some(clouds_tex_1),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        ScrollingSkyLayer {
            angular_speed: 0.068,
            base_y: 0.0,
            bob_amp: 0.65,
            bob_speed: 0.38,
            phase: 0.0,
        },
        Mesh3d(meshes.add(build_sky_dome_mesh(100.0, 3, -2.0, 3.2, 27.0, 58.0))),
        MeshMaterial3d(cloud_mat_a),
        Transform::IDENTITY,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // Layer B: Faster low-to-mid scudding dark storm billows for strong parallax motion
    let cloud_mat_b = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.93, 0.98, 0.84),
        base_color_texture: Some(clouds_tex_2),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        ScrollingSkyLayer {
            angular_speed: 0.125,
            base_y: 0.2,
            bob_amp: 0.85,
            bob_speed: 0.52,
            phase: 1.7,
        },
        Mesh3d(meshes.add(build_sky_dome_mesh(92.0, 3, -1.5, 3.6, 25.5, 56.0))),
        MeshMaterial3d(cloud_mat_b),
        Transform::from_rotation(Quat::from_rotation_y(1.1)),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

fn animate_ominous_sky(
    time: Res<Time>,
    mut sky_layers: Query<(&ScrollingSkyLayer, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    for (layer, mut transform) in sky_layers.iter_mut() {
        let yaw = layer.phase + t * layer.angular_speed;
        let y = layer.base_y + (t * layer.bob_speed + layer.phase).sin() * layer.bob_amp;
        transform.translation.y = y;
        transform.rotation = Quat::from_rotation_y(yaw);
    }
}

fn handle_fallout_respawns(
    mut characters: Query<(
        &mut Transform,
        Option<&mut Position>,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut StaggerState,
        &Character,
    )>,
) {
    for (mut transform, mut maybe_pos, mut lin_vel, mut ang_vel, mut stagger, char_info) in characters.iter_mut()
    {
        // Always prevent any physics-induced angular velocity from spinning characters
        ang_vel.0 = Vec3::ZERO;

        // Fallout rescue: if fallen far below the arena floor, recover back to base
        if transform.translation.y < -10.0 {
            transform.translation = char_info.spawn_point;
            if let Some(ref mut pos) = maybe_pos {
                pos.0 = char_info.spawn_point;
            }
            lin_vel.0 = Vec3::ZERO;
            stagger.duration = 0.5;
            continue;
        }

        // Hard boundary clamping: characters can NEVER cross the angled compound fence perimeter
        let mut clamped_xz = false;
        if transform.translation.z < PLAYABLE_MIN_Z {
            transform.translation.z = PLAYABLE_MIN_Z;
            clamped_xz = true;
            if lin_vel.z < 0.0 {
                lin_vel.z = 0.0;
            }
        } else if transform.translation.z > PLAYABLE_MAX_Z {
            transform.translation.z = PLAYABLE_MAX_Z;
            clamped_xz = true;
            if lin_vel.z > 0.0 {
                lin_vel.z = 0.0;
            }
        }

        let min_x = playable_min_x(transform.translation.z);
        let max_x = playable_max_x(transform.translation.z);
        if transform.translation.x < min_x {
            transform.translation.x = min_x;
            clamped_xz = true;
            if lin_vel.x < 0.0 {
                lin_vel.x = 0.0;
            }
        } else if transform.translation.x > max_x {
            transform.translation.x = max_x;
            clamped_xz = true;
            if lin_vel.x > 0.0 {
                lin_vel.x = 0.0;
            }
        }

        // Only write to Avian3d Position when a boundary clamp actually occurred
        if clamped_xz {
            if let Some(ref mut pos) = maybe_pos {
                pos.0.x = transform.translation.x;
                pos.0.z = transform.translation.z;
            }
        }

        // Cap horizontal velocity so physics solver collisions can never fling characters across the map
        let horiz_speed_limit = if stagger.duration > 0.0 { 19.8 } else { 21.5 };
        let horiz_vel = Vec2::new(lin_vel.x, lin_vel.z);
        let horiz_speed = horiz_vel.length();
        if horiz_speed > horiz_speed_limit {
            let clamped = horiz_vel * (horiz_speed_limit / horiz_speed);
            lin_vel.x = clamped.x;
            lin_vel.z = clamped.y;
        }

        // Prevent physics slope/capsule deflections from ever launching characters high up into the air
        let max_upward_vel = if stagger.duration > 0.0 { 2.4 } else { 5.8 };
        if lin_vel.y > max_upward_vel {
            lin_vel.y = max_upward_vel;
        }

        // Eject characters out of shipping container interiors so nobody can ever walk through,
        // clip into, or stand inside hollow containers:
        for (cx, cz, yaw, half_len, half_wid, roof_y) in [
            (3.825_f32, -10.632_f32, -0.2134_f32, 6.20_f32 * 0.5, 2.65_f32 * 0.5, 4.08_f32),
            (-9.843_f32, -10.864_f32, 0.7419_f32, 6.25_f32 * 0.5, 2.60_f32 * 0.5, 4.08_f32),
        ] {
            if transform.translation.y < roof_y {
                let dx = transform.translation.x - cx;
                let dz = transform.translation.z - cz;
                let cos_y = yaw.cos();
                let sin_y = yaw.sin();
                let lx = cos_y * dx - sin_y * dz;
                let lz = sin_y * dx + cos_y * dz;
                let r = 0.38; // character capsule radius + safety margin
                let limit_x = half_len + r;
                let limit_z = half_wid + r;
                if lx.abs() < limit_x && lz.abs() < limit_z {
                    let pen_x = limit_x - lx.abs();
                    let pen_z = limit_z - lz.abs();
                    let (delta_lx, delta_lz) = if pen_x < pen_z {
                        let sign = if lx >= 0.0 { 1.0 } else { -1.0 };
                        (sign * limit_x - lx, 0.0)
                    } else {
                        let sign = if lz >= 0.0 { 1.0 } else { -1.0 };
                        (0.0, sign * limit_z - lz)
                    };
                    transform.translation.x += delta_lx * cos_y + delta_lz * sin_y;
                    transform.translation.z += -delta_lx * sin_y + delta_lz * cos_y;
                    if let Some(ref mut pos) = maybe_pos {
                        pos.0.x = transform.translation.x;
                        pos.0.z = transform.translation.z;
                    }
                    if lin_vel.x * (delta_lx * cos_y + delta_lz * sin_y) < 0.0 {
                        lin_vel.x = 0.0;
                    }
                    if lin_vel.z * (-delta_lx * sin_y + delta_lz * cos_y) < 0.0 {
                        lin_vel.z = 0.0;
                    }
                }
            }
        }

        // Guarantee characters never clip into the 3D model ground anywhere inside the compound:
        // Boots sole is at -0.90m below transform.translation.y (capsule center)
        let ground_y = sample_terrain_height(transform.translation.x, transform.translation.z);
        let min_center_y = ground_y + 0.90;
        if transform.translation.y < min_center_y {
            transform.translation.y = min_center_y;
            if let Some(ref mut pos) = maybe_pos {
                pos.0.y = min_center_y;
            }
            if lin_vel.y < 0.0 {
                lin_vel.y = 0.0;
            }
        }
    }
}

/// Builds a single combined parabolic waterfall tube mesh along the sewer pipe outflow trajectory,
/// replacing 80 individual stream segment capsules and falling particle spheres with 1 draw call.
fn build_waterfall_arch_mesh(
    pipe_mouth: Vec3,
    outflow_vel: Vec3,
    gravity: Vec3,
    total_fall_time: f32,
    radius_scale: f32,
) -> Mesh {
    let rings = 16usize;
    let sides = 10usize;
    let mut positions = Vec::with_capacity((rings + 1) * (sides + 1));
    let mut normals = Vec::with_capacity((rings + 1) * (sides + 1));
    let mut uvs = Vec::with_capacity((rings + 1) * (sides + 1));
    let mut indices = Vec::with_capacity(rings * sides * 6);

    for r_idx in 0..=rings {
        let u = (r_idx as f32) / (rings as f32);
        let t = u * total_fall_time;
        let center = pipe_mouth + outflow_vel * t + 0.5 * gravity * t * t - pipe_mouth;
        let tangent = (outflow_vel + gravity * t).normalize();
        let rot = Quat::from_rotation_arc(Vec3::Y, tangent);
        let width = (1.05 - 0.22 * u + 0.35 * u * u) * 0.19 * radius_scale;

        for s_idx in 0..=sides {
            let v = (s_idx as f32) / (sides as f32);
            let theta = v * std::f32::consts::TAU;
            let local_dir = Vec3::new(theta.cos(), 0.0, theta.sin());
            let world_norm = (rot * local_dir).normalize();
            let local_pos = center + world_norm * width;
            positions.push(local_pos.to_array());
            normals.push(world_norm.to_array());
            uvs.push([v, u]);
        }
    }

    let stride = (sides + 1) as u32;
    for r_idx in 0..(rings as u32) {
        for s_idx in 0..(sides as u32) {
            let i00 = r_idx * stride + s_idx;
            let i01 = i00 + 1;
            let i10 = (r_idx + 1) * stride + s_idx;
            let i11 = i10 + 1;
            indices.push(i00);
            indices.push(i10);
            indices.push(i01);
            indices.push(i01);
            indices.push(i10);
            indices.push(i11);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Builds a single combined trench water surface ribbon mesh along the sewer canal floor.
fn build_trench_water_ribbon_mesh() -> Mesh {
    let spine = [
        (Vec3::new(-1.95, -1.91, 8.05), 0.95_f32),
        (Vec3::new(-1.55, -1.96, 9.10), 1.05_f32),
        (Vec3::new(-1.10, -2.02, 10.15), 1.08_f32),
        (Vec3::new(-0.65, -2.09, 11.20), 1.02_f32),
        (Vec3::new(-0.25, -2.22, 12.35), 0.92_f32),
    ];
    let mut positions = Vec::with_capacity(spine.len() * 2);
    let mut normals = Vec::with_capacity(spine.len() * 2);
    let mut uvs = Vec::with_capacity(spine.len() * 2);
    let mut indices = Vec::with_capacity((spine.len() - 1) * 6);

    let side = Vec3::new(0.92, 0.0, -0.39).normalize();
    for (idx, (center, half_w)) in spine.iter().enumerate() {
        let v = (idx as f32) / ((spine.len() - 1) as f32);
        let left = *center - side * *half_w;
        let right = *center + side * *half_w;
        positions.push(left.to_array());
        positions.push(right.to_array());
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.0, v]);
        uvs.push([1.0, v]);
    }

    for idx in 0..((spine.len() - 1) as u32) {
        let b = idx * 2;
        indices.push(b);
        indices.push(b + 2);
        indices.push(b + 1);
        indices.push(b + 1);
        indices.push(b + 2);
        indices.push(b + 3);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn spawn_sewer_water_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Consolidated Sewer Waterfall & Trench Water (4 meshes total instead of 135 entities)
    let water_core_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.26, 0.48, 0.58, 0.88),
        emissive: LinearRgba::new(0.05, 0.14, 0.20, 1.0),
        metallic: 0.12,
        perceptual_roughness: 0.06,
        reflectance: 0.98,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let water_foam_sheath_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.76, 0.90, 0.96, 0.62),
        emissive: LinearRgba::new(0.20, 0.30, 0.36, 1.0),
        metallic: 0.04,
        perceptual_roughness: 0.14,
        reflectance: 0.90,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let trench_surface_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.20, 0.39, 0.48, 0.74),
        emissive: LinearRgba::new(0.03, 0.09, 0.13, 1.0),
        metallic: 0.18,
        perceptual_roughness: 0.04,
        reflectance: 1.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let ripple_ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.80, 0.93, 0.99, 0.70),
        emissive: LinearRgba::new(0.18, 0.28, 0.34, 1.0),
        perceptual_roughness: 0.08,
        reflectance: 0.95,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let pipe_mouth = Vec3::new(-2.90, -0.28, 8.21);
    let outflow_vel = Vec3::new(2.15, -0.42, 0.68);
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let total_fall_time = 0.54_f32;
    let impact_center = pipe_mouth
        + outflow_vel * total_fall_time
        + 0.5 * gravity * total_fall_time * total_fall_time;

    // 1. Consolidated Parabolic Waterfall Core Stream Mesh (1 draw call)
    commands.spawn((
        Mesh3d(meshes.add(build_waterfall_arch_mesh(
            pipe_mouth,
            outflow_vel,
            gravity,
            total_fall_time,
            1.0,
        ))),
        MeshMaterial3d(water_core_mat),
        Transform::from_translation(pipe_mouth),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 2. Consolidated Oscillating Whitewater Foam Sheath Mesh (1 draw call)
    let foam_pos = pipe_mouth + Vec3::new(0.0, 0.015, 0.0);
    commands.spawn((
        SewerWaterPulse {
            base_pos: foam_pos,
            freq: 12.5,
        },
        Mesh3d(meshes.add(build_waterfall_arch_mesh(
            pipe_mouth,
            outflow_vel,
            gravity,
            total_fall_time,
            1.22,
        ))),
        MeshMaterial3d(water_foam_sheath_mat),
        Transform::from_translation(foam_pos),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 3. Consolidated Trench Water Surface Ribbon Mesh (1 draw call)
    commands.spawn((
        Mesh3d(meshes.add(build_trench_water_ribbon_mesh())),
        MeshMaterial3d(trench_surface_mat),
        Transform::IDENTITY,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 4. Plunge Pool Expanding Ripple Ring (1 draw call)
    let plunge_surface = Vec3::new(impact_center.x, -1.90, impact_center.z);
    commands.spawn((
        SewerWaterRipple {
            center: plunge_surface,
            drift_vel: Vec3::new(0.22, -0.02, 0.38),
            phase_offset: 0.0,
            period: 1.35,
            min_scale: 0.45,
            max_scale: 2.25,
        },
        Mesh3d(meshes.add(Torus {
            minor_radius: 0.035,
            major_radius: 0.55,
        })),
        MeshMaterial3d(ripple_ring_mat),
        Transform::from_translation(plunge_surface),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

fn animate_sewer_water(
    time: Res<Time>,
    mut pulses: Query<(&SewerWaterPulse, &mut Transform), Without<SewerWaterRipple>>,
    mut ripples: Query<(&SewerWaterRipple, &mut Transform), Without<SewerWaterPulse>>,
) {
    let t = time.elapsed_secs();

    for (pulse, mut transform) in pulses.iter_mut() {
        let sx = 1.0 + 0.09 * (t * pulse.freq).sin();
        let sz = 1.0 + 0.09 * (t * (pulse.freq * 1.15)).cos();
        let sway_y = 0.015 * (t * 10.0).cos();
        transform.translation = pulse.base_pos + Vec3::new(0.0, sway_y, 0.0);
        transform.scale = Vec3::new(sx, 1.0, sz);
    }

    for (r, mut transform) in ripples.iter_mut() {
        let local_t = (t + r.phase_offset) % r.period;
        let u = local_t / r.period;
        let scale_xz = r.min_scale + (r.max_scale - r.min_scale) * u;
        let thickness = (1.0 - u * 0.70).max(0.18);
        transform.translation = r.center + r.drift_vel * local_t;
        transform.scale = Vec3::new(scale_xz, thickness, scale_xz);
    }
}
