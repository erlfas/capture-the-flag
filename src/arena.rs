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
            .add_systems(Update, (handle_fallout_respawns, animate_sewer_water));
    }
}

#[derive(Component)]
struct SewerStreamSegment {
    base_pos: Vec3,
    base_rot: Quat,
    base_scale: Vec3,
    phase: f32,
    is_foam_sheath: bool,
}

#[derive(Component)]
struct SewerWaterParticle {
    phase_offset: f32,
    cycle_duration: f32,
    initial_pos: Vec3,
    initial_vel: Vec3,
    is_splash: bool,
    base_radius: f32,
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

#[derive(Component)]
struct SewerWaterWave {
    start_pos: Vec3,
    end_pos: Vec3,
    phase_offset: f32,
    speed: f32,
    base_scale: Vec3,
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

    // =========================================================================
    // 5. REALISTIC POURING SEWER WATERFALL & ANIMATED TRENCH WATER EFFECTS
    // =========================================================================
    spawn_sewer_water_effects(&mut commands, &mut meshes, &mut materials);
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

fn spawn_sewer_water_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // 1. Realistic PBR Water, Aerated Whitewater Foam & Specular Ripple Materials
    let water_core_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.26, 0.48, 0.58, 0.86),
        emissive: LinearRgba::new(0.05, 0.14, 0.20, 1.0),
        metallic: 0.12,
        perceptual_roughness: 0.06,
        reflectance: 0.98,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let water_foam_sheath_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.88, 0.94, 0.56),
        emissive: LinearRgba::new(0.18, 0.28, 0.34, 1.0),
        metallic: 0.04,
        perceptual_roughness: 0.14,
        reflectance: 0.90,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let whitewater_splash_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.86, 0.95, 0.99, 0.82),
        emissive: LinearRgba::new(0.26, 0.38, 0.45, 1.0),
        metallic: 0.02,
        perceptual_roughness: 0.18,
        reflectance: 0.85,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let trench_surface_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.20, 0.39, 0.48, 0.72),
        emissive: LinearRgba::new(0.03, 0.09, 0.13, 1.0),
        metallic: 0.18,
        perceptual_roughness: 0.04,
        reflectance: 1.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let ripple_ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.78, 0.92, 0.98, 0.68),
        emissive: LinearRgba::new(0.16, 0.26, 0.32, 1.0),
        perceptual_roughness: 0.08,
        reflectance: 0.95,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Exact 3D Sewer Pipe Mouth coordinates extracted from post_apocalyptic_battlefield.glb:
    // Pipe runs from (-3.65, -0.08, 7.98) through the open concrete mouth at (-2.90, -0.28, 8.21),
    // pouring along outflow vector (2.15, -0.42, 0.68) m/s into the trench pool at (-1.72, -1.94, 8.58).
    let pipe_interior = Vec3::new(-3.42, -0.14, 8.05);
    let pipe_mouth = Vec3::new(-2.90, -0.28, 8.21);
    let outflow_vel = Vec3::new(2.15, -0.42, 0.68);
    let gravity = Vec3::new(0.0, -9.81, 0.0);

    // 2. Interior Pipe Water Tongue (water pooling inside the dark concrete pipe barrel before spilling out)
    let pipe_dir = (pipe_mouth - pipe_interior).normalize();
    let pipe_mid = (pipe_interior + pipe_mouth) * 0.5 + Vec3::new(0.0, -0.05, 0.0);
    let pipe_rot = Quat::from_rotation_arc(Vec3::Y, pipe_dir);
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.22, 0.52))),
        MeshMaterial3d(water_core_mat.clone()),
        Transform::from_translation(pipe_mid)
            .with_rotation(pipe_rot)
            .with_scale(Vec3::new(1.15, 1.0, 0.65)),
    ));

    // 3. Sculpted Parabolic Pouring Waterfall Arch (16 overlapping core + frothy sheath segments)
    // t_total = 0.54s brings the parabolic stream from y = -0.28m down to the trench pool at y = -1.94m
    let num_arch_segments = 16;
    let total_fall_time = 0.54_f32;
    let seg_mesh = meshes.add(Capsule3d::new(0.18, 0.24));

    for i in 0..num_arch_segments {
        let u = (i as f32) / ((num_arch_segments - 1) as f32);
        let t = u * total_fall_time;
        let pos = pipe_mouth + outflow_vel * t + 0.5 * gravity * t * t;
        let inst_vel = outflow_vel + gravity * t;
        let tangent = inst_vel.normalize();
        let rot = Quat::from_rotation_arc(Vec3::Y, tangent);

        // Stream narrows slightly as it accelerates, then flares into frothy whitewater near impact
        let width_profile = 1.05 - 0.22 * u + 0.35 * u * u;
        let core_scale = Vec3::new(width_profile * 0.95, 1.15 + 0.45 * u, width_profile * 0.78);

        commands.spawn((
            SewerStreamSegment {
                base_pos: pos,
                base_rot: rot,
                base_scale: core_scale,
                phase: u * std::f32::consts::TAU * 2.0,
                is_foam_sheath: false,
            },
            Mesh3d(seg_mesh.clone()),
            MeshMaterial3d(water_core_mat.clone()),
            Transform::from_translation(pos)
                .with_rotation(rot)
                .with_scale(core_scale),
        ));

        // Aerated whitewater outer sheath along the waterfall arch
        let sheath_scale = core_scale * Vec3::new(1.18, 1.08, 1.18);
        commands.spawn((
            SewerStreamSegment {
                base_pos: pos + Vec3::new(0.0, 0.02, 0.0),
                base_rot: rot,
                base_scale: sheath_scale,
                phase: u * std::f32::consts::TAU * 2.5 + 1.1,
                is_foam_sheath: true,
            },
            Mesh3d(seg_mesh.clone()),
            MeshMaterial3d(water_foam_sheath_mat.clone()),
            Transform::from_translation(pos + Vec3::new(0.0, 0.02, 0.0))
                .with_rotation(rot)
                .with_scale(sheath_scale),
        ));
    }

    // 4. Physics-Driven Pouring Water Parcels & Splash Droplets (72 dynamic fluid particles)
    let parcel_mesh = meshes.add(Sphere::new(1.0));
    let impact_center = pipe_mouth
        + outflow_vel * total_fall_time
        + 0.5 * gravity * total_fall_time * total_fall_time;

    // 48 falling water parcels & foam streaks pouring out of the sewer pipe
    for i in 0..48 {
        let frac = (i as f32) / 48.0;
        let angle = (i as f32) * 2.39996; // Golden angle distribution inside the pipe mouth
        let radial = ((i % 7) as f32 / 7.0).sqrt() * 0.19;
        let offset_y = angle.sin() * radial * 0.65;
        let offset_side = angle.cos() * radial;
        // Lateral vector perpendicular to outflow in XZ plane
        let side_dir = Vec3::new(-0.30, 0.0, 0.95);

        let spawn_pos = pipe_mouth + side_dir * offset_side + Vec3::new(0.0, offset_y, 0.0);
        let speed_jitter = 0.88 + ((i * 13) % 11) as f32 * 0.024;
        let spread = ((i as f32 * 1.7).sin()) * 0.28;
        let init_vel = outflow_vel * speed_jitter
            + side_dir * spread
            + Vec3::new(0.0, ((i % 5) as f32 - 2.0) * 0.08, 0.0);

        let is_foam_streak = i % 3 == 0;
        let mat = if is_foam_streak {
            whitewater_splash_mat.clone()
        } else {
            water_core_mat.clone()
        };

        commands.spawn((
            SewerWaterParticle {
                phase_offset: frac * 0.56,
                cycle_duration: 0.56,
                initial_pos: spawn_pos,
                initial_vel: init_vel,
                is_splash: false,
                base_radius: if is_foam_streak { 0.085 } else { 0.105 },
            },
            Mesh3d(parcel_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_translation(spawn_pos).with_scale(Vec3::splat(0.1)),
        ));
    }

    // 28 turbulent whitewater splash droplets & spray crowns erupting from the plunge zone
    for i in 0..28 {
        let frac = (i as f32) / 28.0;
        let crown_angle = (i as f32) * 2.39996;
        let out_speed = 0.65 + ((i * 7) % 9) as f32 * 0.14;
        let up_speed = 1.85 + ((i * 11) % 7) as f32 * 0.22;
        let splash_vel = Vec3::new(
            crown_angle.cos() * out_speed + 0.25,
            up_speed,
            crown_angle.sin() * out_speed + 0.18,
        );
        let start_offset = Vec3::new(
            crown_angle.cos() * 0.22,
            0.06,
            crown_angle.sin() * 0.22,
        );

        commands.spawn((
            SewerWaterParticle {
                phase_offset: frac * 0.48,
                cycle_duration: 0.48,
                initial_pos: impact_center + start_offset,
                initial_vel: splash_vel,
                is_splash: true,
                base_radius: 0.075 + ((i % 4) as f32) * 0.015,
            },
            Mesh3d(parcel_mesh.clone()),
            MeshMaterial3d(whitewater_splash_mat.clone()),
            Transform::from_translation(impact_center + start_offset)
                .with_scale(Vec3::splat(0.08)),
        ));
    }

    // 5. Boiling Plunge Pool Foam Cushion + 6 Expanding Concentric Water Ripples
    let plunge_surface = Vec3::new(impact_center.x, -1.91, impact_center.z);
    commands.spawn((
        SewerWaterRipple {
            center: plunge_surface,
            drift_vel: Vec3::ZERO,
            phase_offset: 0.0,
            period: 0.85,
            min_scale: 0.72,
            max_scale: 1.05,
        },
        Mesh3d(meshes.add(Cylinder::new(0.52, 0.08))),
        MeshMaterial3d(whitewater_splash_mat.clone()),
        Transform::from_translation(plunge_surface),
    ));

    let ripple_torus_mesh = meshes.add(Torus {
        minor_radius: 0.028,
        major_radius: 0.55,
    });
    for i in 0..6 {
        let phase = (i as f32) / 6.0 * 1.5;
        commands.spawn((
            SewerWaterRipple {
                center: plunge_surface + Vec3::new(0.0, 0.02, 0.0),
                drift_vel: Vec3::new(0.24, -0.03, 0.42),
                phase_offset: phase,
                period: 1.5,
                min_scale: 0.35,
                max_scale: 2.35,
            },
            Mesh3d(ripple_torus_mesh.clone()),
            MeshMaterial3d(ripple_ring_mat.clone()),
            Transform::from_translation(plunge_surface),
        ));
    }

    // 6. Glossy Specular Trench Water Surface & Animated Downstream Current Waves
    let trench_patches = [
        (Vec3::new(-1.95, -1.91, 8.15), Vec3::new(1.65, 0.04, 1.55), -0.35),
        (Vec3::new(-1.55, -1.96, 9.10), Vec3::new(1.85, 0.04, 1.75), -0.42),
        (Vec3::new(-1.10, -2.02, 10.15), Vec3::new(1.90, 0.04, 1.85), -0.40),
        (Vec3::new(-0.65, -2.09, 11.20), Vec3::new(1.80, 0.04, 1.80), -0.35),
        (Vec3::new(-0.25, -2.22, 12.25), Vec3::new(1.65, 0.04, 1.70), -0.25),
    ];
    let patch_mesh = meshes.add(Cylinder::new(0.58, 1.0));
    for (pos, scale, yaw) in trench_patches {
        commands.spawn((
            Mesh3d(patch_mesh.clone()),
            MeshMaterial3d(trench_surface_mat.clone()),
            Transform::from_translation(pos)
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(scale),
        ));
    }

    // 14 animated downstream foam/wave crests gliding down the water trench from the plunge pool
    let wave_mesh = meshes.add(Capsule3d::new(0.045, 0.48));
    for i in 0..14 {
        let frac = (i as f32) / 14.0;
        let lane_offset = ((i % 3) as f32 - 1.0) * 0.28;
        let start = Vec3::new(-1.82 + lane_offset * 0.7, -1.90, 8.35 - lane_offset * 0.3);
        let end = Vec3::new(-0.25 + lane_offset * 0.8, -2.22, 12.45 - lane_offset * 0.3);
        commands.spawn((
            SewerWaterWave {
                start_pos: start,
                end_pos: end,
                phase_offset: frac * 2.6,
                speed: 1.0,
                base_scale: Vec3::new(0.85 + (i % 3) as f32 * 0.22, 1.0, 0.65),
            },
            Mesh3d(wave_mesh.clone()),
            MeshMaterial3d(water_foam_sheath_mat.clone()),
            Transform::from_translation(start),
        ));
    }
}

fn animate_sewer_water(
    time: Res<Time>,
    mut stream_segments: Query<(&SewerStreamSegment, &mut Transform), (Without<SewerWaterParticle>, Without<SewerWaterRipple>, Without<SewerWaterWave>)>,
    mut particles: Query<(&SewerWaterParticle, &mut Transform), (Without<SewerStreamSegment>, Without<SewerWaterRipple>, Without<SewerWaterWave>)>,
    mut ripples: Query<(&SewerWaterRipple, &mut Transform), (Without<SewerStreamSegment>, Without<SewerWaterParticle>, Without<SewerWaterWave>)>,
    mut waves: Query<(&SewerWaterWave, &mut Transform), (Without<SewerStreamSegment>, Without<SewerWaterParticle>, Without<SewerWaterRipple>)>,
) {
    let t = time.elapsed_secs();
    let gravity = Vec3::new(0.0, -9.81, 0.0);

    // 1. Animate the parabolic waterfall arch segments with turbulent pulsing & flow oscillation
    for (seg, mut transform) in stream_segments.iter_mut() {
        let freq = if seg.is_foam_sheath { 14.0 } else { 10.5 };
        let pulse_x = 1.0 + 0.14 * (t * freq - seg.phase).sin();
        let pulse_z = 1.0 + 0.14 * (t * (freq * 1.18) - seg.phase).cos();
        let sway = Vec3::new(
            0.025 * (t * 9.0 - seg.phase).sin(),
            0.018 * (t * 12.0 - seg.phase).cos(),
            0.025 * (t * 8.5 - seg.phase).cos(),
        );
        transform.translation = seg.base_pos + sway;
        transform.rotation = seg.base_rot;
        transform.scale = seg.base_scale * Vec3::new(pulse_x, 1.0, pulse_z);
    }

    // 2. Animate physics-driven pouring water parcels & splash crown droplets
    for (p, mut transform) in particles.iter_mut() {
        let local_t = (t + p.phase_offset) % p.cycle_duration;
        let u = local_t / p.cycle_duration;
        let pos = p.initial_pos + p.initial_vel * local_t + 0.5 * gravity * local_t * local_t;
        let vel = p.initial_vel + gravity * local_t;
        let speed = vel.length().max(0.1);
        let dir = vel / speed;

        transform.translation = pos;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, dir);

        if p.is_splash {
            // Splash droplets swell at apex and taper as they fall back into the water
            let envelope = (u * std::f32::consts::PI).sin().max(0.15);
            let r = p.base_radius * envelope;
            transform.scale = Vec3::new(r, r * (1.1 + speed * 0.18), r);
        } else {
            // Pouring stream parcels stretch along their velocity vector as gravity accelerates them
            let stretch = (1.15 + speed * 0.34).clamp(1.1, 3.2);
            let r = p.base_radius * (1.05 - 0.25 * u);
            transform.scale = Vec3::new(r, r * stretch, r);
        }
    }

    // 3. Animate expanding concentric water ripples & boiling plunge cushion
    for (r, mut transform) in ripples.iter_mut() {
        let local_t = (t + r.phase_offset) % r.period;
        let u = local_t / r.period;
        let scale_xz = r.min_scale + (r.max_scale - r.min_scale) * u;
        let thickness = (1.0 - u * 0.72).max(0.15);
        transform.translation = r.center + r.drift_vel * local_t;
        transform.scale = Vec3::new(scale_xz, thickness, scale_xz);
    }

    // 4. Animate downstream current wave crests flowing along the trench
    for (w, mut transform) in waves.iter_mut() {
        let cycle = 2.6_f32;
        let local_t = (t * w.speed + w.phase_offset) % cycle;
        let u = local_t / cycle;
        let pos = w.start_pos.lerp(w.end_pos, u);
        let flow_dir = (w.end_pos - w.start_pos).normalize();
        // Orient wave crest perpendicular to downstream flow
        let crest_dir = Vec3::new(-flow_dir.z, 0.0, flow_dir.x).normalize();
        let envelope = (u * std::f32::consts::PI).sin().max(0.2);

        transform.translation = pos + Vec3::new(0.0, 0.015 * (t * 6.0 + w.phase_offset).sin(), 0.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, crest_dir);
        transform.scale = w.base_scale * Vec3::new(envelope, 0.7 + 0.5 * envelope, envelope);
    }
}
