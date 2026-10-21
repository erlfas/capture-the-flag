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
        Vec3::new(-18.5, 1.8, -1.5),
        4.8,
    ),
    (
        2,
        "Rød Base (Østplattform)",
        Vec3::new(17.5, 1.8, -3.0),
        4.8,
    ),
];

fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // =========================================================================
    // 1. POST-APOCALYPTIC STORMY DUSK ATMOSPHERE & LIGHTING
    //    (Matching art/scene/post_apocalyptic_scene.jpg)
    // =========================================================================
    commands.insert_resource(ClearColor(Color::srgb(0.18, 0.19, 0.21)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.62, 0.64, 0.68),
        brightness: 520.0,
    });

    // Pale amber dusk horizon sunlight breaking through the storm clouds from the North
    commands.spawn((
        DirectionalLight {
            illuminance: 14000.0,
            shadows_enabled: false,
            color: Color::srgb(0.95, 0.78, 0.56),
            ..default()
        },
        Transform::from_xyz(-10.0, 38.0, -55.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Moody cool overcast storm fill from above/South
    commands.spawn((
        DirectionalLight {
            illuminance: 8500.0,
            shadows_enabled: false,
            color: Color::srgb(0.50, 0.56, 0.65),
            ..default()
        },
        Transform::from_xyz(15.0, 50.0, 45.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Vivid Blue PointLight pools on the West Blue Platform (matching the intense blue glow in art)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.15, 0.68, 1.0),
            intensity: 1_800_000.0,
            range: 24.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-18.5, 4.5, -1.5),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.20, 0.75, 1.0),
            intensity: 950_000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-18.5, 6.4, -1.5),
    ));

    // Vivid Red PointLight pools on the East Red Platform (matching the intense red glow in art)
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.22, 0.14),
            intensity: 1_800_000.0,
            range: 24.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(17.5, 4.5, -3.0),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.30, 0.18),
            intensity: 950_000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(17.5, 6.4, -3.0),
    ));

    // =========================================================================
    // 2. POST-APOCALYPTIC PALETTE & MATERIALS
    // =========================================================================
    let asphalt_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.28, 0.27),
        perceptual_roughness: 0.94,
        metallic: 0.02,
        ..default()
    });

    let crack_void_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.09, 0.09, 0.10),
        perceptual_roughness: 0.98,
        metallic: 0.0,
        ..default()
    });

    let crack_rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.21, 0.20),
        perceptual_roughness: 0.92,
        metallic: 0.02,
        ..default()
    });

    let puddle_water_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.27, 0.36),
        perceptual_roughness: 0.15,
        metallic: 0.35,
        ..default()
    });

    let paint_line_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.52, 0.50),
        perceptual_roughness: 0.9,
        ..default()
    });

    let concrete_weathered_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.36, 0.35, 0.33),
        perceptual_roughness: 0.88,
        metallic: 0.04,
        ..default()
    });

    let concrete_slab_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.41, 0.38),
        perceptual_roughness: 0.86,
        metallic: 0.04,
        ..default()
    });

    let dark_outline_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.14),
        perceptual_roughness: 0.6,
        metallic: 0.5,
        ..default()
    });

    let steel_dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.32, 0.34),
        perceptual_roughness: 0.5,
        metallic: 0.75,
        ..default()
    });

    let fence_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.32, 0.34, 0.35, 0.46),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.5,
        metallic: 0.75,
        cull_mode: None,
        ..default()
    });

    // Overgrown post-apocalyptic olive/mossy grass
    let weed_grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.34, 0.17),
        perceptual_roughness: 0.95,
        ..default()
    });

    let wasteland_ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.26, 0.16),
        perceptual_roughness: 0.96,
        ..default()
    });

    // Containers & Rusted Metal
    let container_olive_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.33, 0.22),
        perceptual_roughness: 0.75,
        metallic: 0.25,
        ..default()
    });

    let container_rust_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.24, 0.14),
        perceptual_roughness: 0.78,
        metallic: 0.25,
        ..default()
    });

    let container_blue_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.28, 0.42),
        perceptual_roughness: 0.72,
        metallic: 0.3,
        ..default()
    });

    let rust_car_brown_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.23, 0.14),
        perceptual_roughness: 0.82,
        metallic: 0.2,
        ..default()
    });

    let rust_car_orange_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.54, 0.27, 0.15),
        perceptual_roughness: 0.80,
        metallic: 0.2,
        ..default()
    });

    let sandbag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.42, 0.30),
        perceptual_roughness: 0.95,
        ..default()
    });

    let hazard_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.52, 0.16),
        perceptual_roughness: 0.78,
        ..default()
    });

    let blue_rail_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.42, 0.78),
        perceptual_roughness: 0.5,
        metallic: 0.6,
        ..default()
    });

    let red_rail_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.18, 0.16),
        perceptual_roughness: 0.5,
        metallic: 0.6,
        ..default()
    });

    let blue_zone_disc_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.18, 0.72, 1.0, 0.36),
        emissive: LinearRgba::new(0.4, 2.2, 5.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let red_zone_disc_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.22, 0.14, 0.36),
        emissive: LinearRgba::new(5.0, 0.4, 0.2, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let searchlight_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.96, 0.82, 0.95),
        emissive: LinearRgba::new(8.0, 7.2, 4.5, 1.0),
        unlit: true,
        ..default()
    });

    let searchlight_beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.78, 0.09),
        emissive: LinearRgba::new(0.9, 0.82, 0.55, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // =========================================================================
    // 3. CRACKED ASPHALT COURTYARD, PARKING LINES & FLOODED CENTRAL FISSURE
    // =========================================================================
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(58.0, 1.0, 44.0))),
        MeshMaterial3d(asphalt_mat),
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(58.0, 1.0, 44.0),
    ));

    // Surrounding post-apocalyptic wasteland ground
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(320.0, 1.0, 320.0))),
        MeshMaterial3d(wasteland_ground_mat),
        Transform::from_xyz(0.0, -0.68, 0.0),
    ));

    // Faded Parking Lot Lines & Central Crosshair (matching art/scene/post_apocalyptic_scene.jpg)
    // Left & Right longitudinal lane lines
    for lx in [-9.2, 8.8] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.18, 0.02, 22.0))),
            MeshMaterial3d(paint_line_mat.clone()),
            Transform::from_xyz(lx, 0.01, 0.5),
        ));
        // Perpendicular parking stall ticks
        for pz in [-8.0, -3.5, 1.0, 5.5, 10.0] {
            let tick_x = if lx < 0.0 { lx - 1.8 } else { lx + 1.8 };
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(3.6, 0.02, 0.16))),
                MeshMaterial3d(paint_line_mat.clone()),
                Transform::from_xyz(tick_x, 0.01, pz),
            ));
        }
    }
    // Central white crosshair lines in the courtyard
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.20, 0.02, 20.0))),
        MeshMaterial3d(paint_line_mat.clone()),
        Transform::from_xyz(0.0, 0.01, -1.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 0.02, 0.20))),
        MeshMaterial3d(paint_line_mat),
        Transform::from_xyz(0.0, 0.01, -3.2),
    ));

    // Deep Jagged Tectonic Crack slicing through the courtyard center
    for (cx, cz, cw, cd, cyaw) in [
        (-1.2, -14.0, 1.4, 7.5, 0.18),
        (0.4, -7.5, 1.8, 6.8, -0.25),
        (-0.8, -1.2, 2.2, 7.2, 0.32),
        (-2.2, 4.8, 2.5, 6.5, -0.22),
        (2.8, -2.5, 1.2, 5.5, -0.65),
        (-4.5, 1.5, 1.1, 5.0, 0.58),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(cw, 0.04, cd))),
            MeshMaterial3d(crack_void_mat.clone()),
            Transform::from_xyz(cx, 0.015, cz).with_rotation(Quat::from_rotation_y(cyaw)),
        ));
        // Broken asphalt lip chunks along the fissure edges
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(cw + 0.55, 0.08, cd * 0.85))),
            MeshMaterial3d(crack_rock_mat.clone()),
            Transform::from_xyz(cx, 0.005, cz).with_rotation(Quat::from_rotation_y(cyaw + 0.06)),
        ));
    }

    // Flooded Reflective Water Puddle in the Foreground Trench (bottom-center of art)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(3.6, 0.03))),
        MeshMaterial3d(puddle_water_mat.clone()),
        Transform::from_xyz(-0.2, 0.02, 12.8).with_scale(Vec3::new(0.85, 1.0, 1.65)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(2.8, 0.03))),
        MeshMaterial3d(puddle_water_mat),
        Transform::from_xyz(0.5, 0.02, 17.5).with_scale(Vec3::new(0.75, 1.0, 1.45)),
    ));

    // Overgrown Weed & Mossy Grass Patches around cracks and obstacles
    for (gx, gz, gw, gd, gyaw) in [
        (-2.2, -1.5, 4.2, 2.4, 0.35),
        (1.8, 1.2, 4.8, 2.6, -0.28),
        (-4.5, 7.5, 5.5, 3.2, 0.22),
        (3.8, 8.2, 4.5, 2.8, -0.35),
        (-13.5, 8.5, 6.5, 3.8, -0.18),
        (12.5, 8.8, 6.2, 3.4, 0.15),
        (-9.5, -9.5, 5.2, 3.0, 0.12),
        (8.5, -9.8, 5.0, 2.8, -0.15),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(gw, 0.03, gd))),
            MeshMaterial3d(weed_grass_mat.clone()),
            Transform::from_xyz(gx, 0.012, gz).with_rotation(Quat::from_rotation_y(gyaw)),
        ));
    }

    // =========================================================================
    // 4. WEST BLUE PLATFORM (Zone 1) + TWIN RAMPS, BLUE GUARDRAILS & SANDBAGS
    // =========================================================================
    let (beta_idx, beta_name, beta_center, beta_radius) = PLATEAU_COORDS[1];
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.8, 1.8, 12.2))),
        MeshMaterial3d(concrete_weathered_mat.clone()),
        Transform::from_xyz(beta_center.x, 0.9, beta_center.z),
        RigidBody::Static,
        Collider::cuboid(11.8, 1.8, 12.2),
        Plateau {
            index: beta_idx,
            name: beta_name,
            center: beta_center,
            radius: beta_radius,
        },
    ));
    // Platform top concrete slab trim
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(12.1, 0.14, 12.5))),
        MeshMaterial3d(concrete_slab_mat.clone()),
        Transform::from_xyz(beta_center.x, 1.76, beta_center.z),
    ));

    // Glowing Blue Capture Floor Disc & Thin Ring
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(4.0, 0.03))),
        MeshMaterial3d(blue_zone_disc_mat),
        Transform::from_xyz(beta_center.x, 1.83, beta_center.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Torus {
            minor_radius: 0.055,
            major_radius: 4.0,
        })),
        MeshMaterial3d(blue_rail_mat.clone()),
        Transform::from_xyz(beta_center.x, 1.84, beta_center.z),
    ));

    // Blue Flag Base Pedestal + Sandbag Ring around Flagpole Base (matching art)
    spawn_flagpole(
        &mut commands,
        &mut meshes,
        &steel_dark_mat,
        &blue_rail_mat,
        beta_center,
    );
    spawn_sandbag_cluster(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        beta_center + Vec3::new(-0.8, 0.0, -1.2),
        0.3,
    );
    // Corner sandbags on Blue Platform
    spawn_sandbag_cluster(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(beta_center.x - 4.2, 1.8, beta_center.z + 4.5),
        0.0,
    );

    // Blue Metal Guardrails along the back, left, and front corners of Blue Platform
    spawn_platform_guardrail(
        &mut commands,
        &mut meshes,
        &blue_rail_mat,
        Vec3::new(beta_center.x, 1.8, beta_center.z - 5.8),
        11.2,
        true,
    );
    spawn_platform_guardrail(
        &mut commands,
        &mut meshes,
        &blue_rail_mat,
        Vec3::new(beta_center.x - 5.6, 1.8, beta_center.z),
        11.4,
        false,
    );
    spawn_platform_guardrail(
        &mut commands,
        &mut meshes,
        &blue_rail_mat,
        Vec3::new(beta_center.x + 3.2, 1.8, beta_center.z + 5.8),
        4.2,
        true,
    );

    // Lower East Concrete Slab Ramp on Blue Platform (matching foreground-left ramp in art)
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        Vec3::new(-10.4, 0.72, 1.8),
        Vec3::new(5.4, 0.45, 3.2),
        -0.34,
        Vec3::Z,
    );
    // Upper East Stepped Ramp with Handrails on Blue Platform (matching upper-left stairs in art)
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        Vec3::new(-10.4, 0.72, -3.8),
        Vec3::new(5.4, 0.45, 2.8),
        -0.34,
        Vec3::Z,
    );
    // Front South Ramp for smooth tactical retreat/entry
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        Vec3::new(-16.5, 0.72, 6.8),
        Vec3::new(5.2, 0.45, 6.4),
        0.30,
        Vec3::X,
    );

    // =========================================================================
    // 5. EAST RED PLATFORM (Zone 2) + STAIRS, RAMP, RED GUARDRAILS & SANDBAGS
    // =========================================================================
    let (gamma_idx, gamma_name, gamma_center, gamma_radius) = PLATEAU_COORDS[2];
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.8, 1.8, 11.8))),
        MeshMaterial3d(concrete_weathered_mat.clone()),
        Transform::from_xyz(gamma_center.x, 0.9, gamma_center.z),
        RigidBody::Static,
        Collider::cuboid(11.8, 1.8, 11.8),
        Plateau {
            index: gamma_idx,
            name: gamma_name,
            center: gamma_center,
            radius: gamma_radius,
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(12.1, 0.14, 12.1))),
        MeshMaterial3d(concrete_slab_mat.clone()),
        Transform::from_xyz(gamma_center.x, 1.76, gamma_center.z),
    ));

    // Glowing Red Capture Floor Disc & Thin Ring
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(4.0, 0.03))),
        MeshMaterial3d(red_zone_disc_mat),
        Transform::from_xyz(gamma_center.x, 1.83, gamma_center.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Torus {
            minor_radius: 0.055,
            major_radius: 4.0,
        })),
        MeshMaterial3d(red_rail_mat.clone()),
        Transform::from_xyz(gamma_center.x, 1.84, gamma_center.z),
    ));

    // Red Flag Base Pedestal + Sandbags on Red Platform
    spawn_flagpole(
        &mut commands,
        &mut meshes,
        &steel_dark_mat,
        &red_rail_mat,
        gamma_center,
    );
    spawn_sandbag_cluster(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(gamma_center.x - 3.6, 1.8, gamma_center.z - 4.2),
        -0.3,
    );
    spawn_sandbag_cluster(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(gamma_center.x + 4.0, 1.8, gamma_center.z + 4.2),
        0.2,
    );

    // Red Metal Guardrails on East Red Platform
    spawn_platform_guardrail(
        &mut commands,
        &mut meshes,
        &red_rail_mat,
        Vec3::new(gamma_center.x, 1.8, gamma_center.z - 5.6),
        11.0,
        true,
    );
    spawn_platform_guardrail(
        &mut commands,
        &mut meshes,
        &red_rail_mat,
        Vec3::new(gamma_center.x + 5.6, 1.8, gamma_center.z),
        11.0,
        false,
    );
    spawn_platform_guardrail(
        &mut commands,
        &mut meshes,
        &red_rail_mat,
        Vec3::new(gamma_center.x - 3.2, 1.8, gamma_center.z + 5.6),
        4.2,
        true,
    );

    // Stepped Concrete Staircase with Metal Handrails on Front/South of Red Platform
    spawn_staircase_with_rails(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        &steel_dark_mat,
        Vec3::new(16.5, 0.9, 5.2),
        3.6,
        1.8,
        5.0,
    );
    // Crumbled Concrete Ramp & Steps on West side of Red Platform (matching right-center of art)
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        Vec3::new(9.5, 0.72, -3.5),
        Vec3::new(5.2, 0.45, 4.6),
        0.34,
        Vec3::Z,
    );

    // =========================================================================
    // 6. TWIN GUARD WATCHTOWERS WITH SEARCHLIGHTS (North-West & North-East)
    // =========================================================================
    spawn_guard_watchtower(
        &mut commands,
        &mut meshes,
        &steel_dark_mat,
        &concrete_weathered_mat,
        &searchlight_glow_mat,
        &searchlight_beam_mat,
        Vec3::new(-14.5, 0.0, -14.8),
        1.0,
    );
    spawn_guard_watchtower(
        &mut commands,
        &mut meshes,
        &steel_dark_mat,
        &concrete_weathered_mat,
        &searchlight_glow_mat,
        &searchlight_beam_mat,
        Vec3::new(14.5, 0.0, -14.8),
        -1.0,
    );

    // =========================================================================
    // 7. SHIPPING CONTAINERS, CURVED SANDBAG BUNKERS & JERSEY BARRIERS
    // =========================================================================
    // Left-North Weathered Olive/Rust Shipping Container
    spawn_shipping_container(
        &mut commands,
        &mut meshes,
        &container_olive_mat,
        &dark_outline_mat,
        Vec3::new(-7.5, 1.65, -11.8),
        Vec3::new(8.5, 3.3, 3.4),
        0.28,
    );
    // Right-North Rust-Brown Shipping Container
    spawn_shipping_container(
        &mut commands,
        &mut meshes,
        &container_rust_mat,
        &dark_outline_mat,
        Vec3::new(4.2, 1.65, -11.5),
        Vec3::new(8.5, 3.3, 3.4),
        -0.25,
    );

    // Foreground Corner Containers (Blue on Far-Bottom-Left, Rust-Red on Far-Bottom-Right)
    spawn_shipping_container(
        &mut commands,
        &mut meshes,
        &container_blue_mat,
        &dark_outline_mat,
        Vec3::new(-25.5, 1.8, 16.5),
        Vec3::new(7.5, 3.6, 3.6),
        0.12,
    );
    spawn_shipping_container(
        &mut commands,
        &mut meshes,
        &container_rust_mat,
        &dark_outline_mat,
        Vec3::new(25.5, 1.8, 16.5),
        Vec3::new(7.5, 3.6, 3.6),
        -0.12,
    );

    // 4 Central Tactical Cover Clusters (Curved Sandbag Walls + Concrete Jersey Barriers)
    // 1) Upper-Left Sandbag Bunker + Concrete Barrier
    spawn_sandbag_pile(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-4.5, 0.0, -5.2),
        0.35,
    );
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        Vec3::new(-6.5, 0.6, -2.5),
        3.6,
        1.15,
    );

    // 2) Upper-Right Sandbag Bunker + Concrete Barrier
    spawn_sandbag_pile(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(4.8, 0.0, -5.0),
        -0.45,
    );
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &concrete_slab_mat,
        Vec3::new(6.2, 0.6, -2.2),
        3.6,
        -1.15,
    );

    // 3) Lower-Left Sandbag Bunker + Hazard Jersey Barrier + Tipped Drum
    spawn_sandbag_pile(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-5.2, 0.0, 4.2),
        -0.55,
    );
    spawn_hazard_barrier(
        &mut commands,
        &mut meshes,
        &hazard_yellow_mat,
        &dark_outline_mat,
        Vec3::new(-6.8, 0.6, 6.8),
        3.8,
        0.48,
    );

    // 4) Lower-Right Sandbag Bunker + Hazard Jersey Barrier
    spawn_sandbag_pile(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(4.8, 0.0, 5.2),
        0.38,
    );
    spawn_hazard_barrier(
        &mut commands,
        &mut meshes,
        &hazard_yellow_mat,
        &dark_outline_mat,
        Vec3::new(7.5, 0.6, 7.2),
        3.6,
        -0.48,
    );

    // =========================================================================
    // 8. WRECKED POST-APOCALYPTIC VEHICLES & OIL DRUMS
    // =========================================================================
    // Lower-Left Rusted SUV Wreck (inside south-west fence)
    spawn_wrecked_vehicle(
        &mut commands,
        &mut meshes,
        &rust_car_brown_mat,
        &dark_outline_mat,
        Vec3::new(-12.2, 0.0, 12.8),
        0.32,
        true,
    );
    // Lower-Right Rusted Pickup Truck Wreck (inside south-east fence)
    spawn_wrecked_vehicle(
        &mut commands,
        &mut meshes,
        &rust_car_orange_mat,
        &dark_outline_mat,
        Vec3::new(12.2, 0.0, 13.0),
        -0.18,
        false,
    );
    // Upper-Right Abandoned Sedan Wreck (near Red Container)
    spawn_wrecked_vehicle(
        &mut commands,
        &mut meshes,
        &rust_car_brown_mat,
        &dark_outline_mat,
        Vec3::new(10.8, 0.0, -12.2),
        -0.45,
        false,
    );

    // Scattered Rusted & Colored Oil Drums
    for (dx, dz, mat) in [
        (-13.8, 3.8, &container_blue_mat),
        (13.5, 4.2, &red_rail_mat),
        (-3.6, 5.2, &container_olive_mat),
        (9.2, -7.8, &container_rust_mat),
        (-10.5, -8.5, &container_rust_mat),
    ] {
        spawn_oil_drum(&mut commands, &mut meshes, mat, Vec3::new(dx, 0.55, dz));
    }

    // =========================================================================
    // 9. BREACHED PERIMETER CHAIN-LINK FENCES, RAZOR WIRE & BROKEN WALL RUINS
    // =========================================================================
    // Invisible full-perimeter physics boundary so players don't wander out of the compound
    for (bx, bz, bw, bd) in [
        (0.0, -19.5, 56.0, 0.6),
        (0.0, 19.5, 56.0, 0.6),
        (-26.5, 0.0, 0.6, 40.0),
        (26.5, 0.0, 0.6, 40.0),
    ] {
        commands.spawn((
            Transform::from_xyz(bx, 2.5, bz),
            RigidBody::Static,
            Collider::cuboid(bw, 5.0, bd),
        ));
    }

    // Visual Chain-Link Fence Segments with Razor Wire (leaving breached gaps in North & South center)
    spawn_chainlink_fence_segment(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_dark_mat,
        &concrete_weathered_mat,
        Vec3::new(-15.5, 0.0, -19.5),
        22.0,
        true,
    );
    spawn_chainlink_fence_segment(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_dark_mat,
        &concrete_weathered_mat,
        Vec3::new(15.5, 0.0, -19.5),
        22.0,
        true,
    );
    spawn_chainlink_fence_segment(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_dark_mat,
        &concrete_weathered_mat,
        Vec3::new(-26.5, 0.0, 0.0),
        38.0,
        false,
    );
    spawn_chainlink_fence_segment(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_dark_mat,
        &concrete_weathered_mat,
        Vec3::new(26.5, 0.0, 0.0),
        38.0,
        false,
    );
    spawn_chainlink_fence_segment(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_dark_mat,
        &concrete_weathered_mat,
        Vec3::new(-14.5, 0.0, 19.5),
        21.0,
        true,
    );
    spawn_chainlink_fence_segment(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_dark_mat,
        &concrete_weathered_mat,
        Vec3::new(14.5, 0.0, 19.5),
        21.0,
        true,
    );

    // Jagged Broken Concrete Wall Slabs flanking the North Fence Breach (matching art)
    for (wx, wz, ww, wh, wyaw) in [
        (-4.2, -19.2, 2.8, 2.8, 0.15),
        (-2.4, -19.4, 2.2, 1.9, -0.22),
        (5.2, -19.2, 3.2, 3.1, -0.12),
        (7.4, -19.3, 2.5, 2.2, 0.08),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(ww, wh, 0.45))),
            MeshMaterial3d(concrete_slab_mat.clone()),
            Transform::from_xyz(wx, wh * 0.5, wz).with_rotation(Quat::from_rotation_y(wyaw)),
        ));
    }

    // =========================================================================
    // 10. POST-APOCALYPTIC SKYBOX, RUINED CITY SKYLINE, OVERPASS & CRAWLER CRANE
    // =========================================================================
    spawn_post_apocalyptic_vista(&mut commands, &mut meshes, &mut materials);
}

fn spawn_flagpole(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    pole_mat: &Handle<StandardMaterial>,
    flag_mat: &Handle<StandardMaterial>,
    base_pos: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.48, 0.22))),
        MeshMaterial3d(flag_mat.clone()),
        Transform::from_translation(base_pos + Vec3::new(0.0, 0.11, 0.0)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.18, 0.32))),
        MeshMaterial3d(pole_mat.clone()),
        Transform::from_translation(base_pos + Vec3::new(0.0, 0.16, 0.0)),
    ));
}

fn spawn_platform_guardrail(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    rail_mat: &Handle<StandardMaterial>,
    base_center: Vec3,
    length: f32,
    along_x: bool,
) {
    let height = 1.05;
    let bar_size = if along_x {
        Vec3::new(length, 0.07, 0.07)
    } else {
        Vec3::new(0.07, 0.07, length)
    };
    // Top and middle horizontal bars
    for ry in [height * 0.55, height] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(bar_size.x, bar_size.y, bar_size.z))),
            MeshMaterial3d(rail_mat.clone()),
            Transform::from_translation(base_center + Vec3::Y * ry),
        ));
    }
    // Vertical posts
    let post_mesh = meshes.add(Cylinder::new(0.045, height));
    let count = (length / 2.6).ceil() as i32;
    for i in 0..=count {
        let offset = -length * 0.5 + (i as f32) * (length / count.max(1) as f32);
        let p_pos = if along_x {
            base_center + Vec3::new(offset, height * 0.5, 0.0)
        } else {
            base_center + Vec3::new(0.0, height * 0.5, offset)
        };
        commands.spawn((
            Mesh3d(post_mesh.clone()),
            MeshMaterial3d(rail_mat.clone()),
            Transform::from_translation(p_pos),
        ));
    }
}

fn spawn_guard_watchtower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    steel_mat: &Handle<StandardMaterial>,
    cabin_mat: &Handle<StandardMaterial>,
    glow_mat: &Handle<StandardMaterial>,
    beam_mat: &Handle<StandardMaterial>,
    base_pos: Vec3,
    aim_dir_x: f32,
) {
    commands
        .spawn(Transform::from_translation(base_pos))
        .with_children(|tower| {
            // 4 tall steel lattice legs
            let leg_mesh = meshes.add(Cylinder::new(0.10, 6.2));
            for lx in [-1.1, 1.1] {
                for lz in [-1.1, 1.1] {
                    tower.spawn((
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(steel_mat.clone()),
                        Transform::from_xyz(lx, 3.1, lz),
                    ));
                }
            }
            // Cross-bracing horizontal struts
            for by in [2.0, 4.0, 6.0] {
                tower.spawn((
                    Mesh3d(meshes.add(Cuboid::new(2.4, 0.10, 2.4))),
                    MeshMaterial3d(steel_mat.clone()),
                    Transform::from_xyz(0.0, by, 0.0),
                ));
            }
            // Watchtower cabin platform & low walls
            tower.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.9, 1.1, 2.9))),
                MeshMaterial3d(cabin_mat.clone()),
                Transform::from_xyz(0.0, 6.55, 0.0),
            ));
            // Cabin roof supported by corner posts
            for lx in [-1.2, 1.2] {
                for lz in [-1.2, 1.2] {
                    tower.spawn((
                        Mesh3d(meshes.add(Cylinder::new(0.06, 1.6))),
                        MeshMaterial3d(steel_mat.clone()),
                        Transform::from_xyz(lx, 7.8, lz),
                    ));
                }
            }
            tower.spawn((
                Mesh3d(meshes.add(Cone {
                    radius: 2.3,
                    height: 0.9,
                })),
                MeshMaterial3d(cabin_mat.clone()),
                Transform::from_xyz(0.0, 8.9, 0.0),
            ));

            // Glowing Searchlight Lamp & Volumetric Light Cone pointing into the courtyard
            let lamp_pos = Vec3::new(aim_dir_x * 1.35, 7.3, 0.8);
            tower.spawn((
                Mesh3d(meshes.add(Sphere::new(0.28))),
                MeshMaterial3d(glow_mat.clone()),
                Transform::from_translation(lamp_pos),
            ));
            tower.spawn((
                Mesh3d(meshes.add(Cone {
                    radius: 2.8,
                    height: 9.5,
                })),
                MeshMaterial3d(beam_mat.clone()),
                Transform::from_translation(lamp_pos + Vec3::new(aim_dir_x * 3.8, -2.2, 1.8))
                    .with_rotation(Quat::from_euler(
                        EulerRot::YXZ,
                        0.0,
                        0.35,
                        -aim_dir_x * 1.1,
                    )),
            ));
        });
}

fn spawn_wrecked_vehicle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    body_mat: &Handle<StandardMaterial>,
    dark_mat: &Handle<StandardMaterial>,
    pos: Vec3,
    yaw: f32,
    is_suv: bool,
) {
    let rot = Quat::from_rotation_y(yaw);
    commands
        .spawn((
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(4.6, 1.6, 2.2),
        ))
        .with_children(|car| {
            // Lower rusted chassis body
            car.spawn((
                Mesh3d(meshes.add(Cuboid::new(4.6, 0.85, 2.1))),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 0.55, 0.0),
            ));
            // Upper cabin roof
            let cabin_len = if is_suv { 2.8 } else { 1.9 };
            let cabin_x = if is_suv { -0.3 } else { 0.1 };
            car.spawn((
                Mesh3d(meshes.add(Cuboid::new(cabin_len, 0.72, 1.85))),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(cabin_x, 1.30, 0.0),
            ));
            // Hollowed dark window frames
            car.spawn((
                Mesh3d(meshes.add(Cuboid::new(cabin_len * 0.88, 0.48, 1.92))),
                MeshMaterial3d(dark_mat.clone()),
                Transform::from_xyz(cabin_x, 1.32, 0.0),
            ));
            // 4 Flat/sunken rubber tires
            let wheel_mesh = meshes.add(Cylinder::new(0.42, 0.32));
            for wx in [-1.45, 1.45] {
                for wz in [-1.05, 1.05] {
                    car.spawn((
                        Mesh3d(wheel_mesh.clone()),
                        MeshMaterial3d(dark_mat.clone()),
                        Transform::from_xyz(wx, 0.38, wz)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    ));
                }
            }
        });
}

fn spawn_sandbag_cluster(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
    yaw: f32,
) {
    let bag_mesh = meshes.add(Capsule3d::new(0.20, 0.55));
    commands
        .spawn(Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)))
        .with_children(|parent| {
            for (bx, by, bz) in [
                (-0.4, 0.18, 0.0),
                (0.4, 0.18, 0.0),
                (0.0, 0.45, 0.0),
            ] {
                parent.spawn((
                    Mesh3d(bag_mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(bx, by, bz)
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ));
            }
        });
}

fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    angle: f32,
    axis: Vec3,
) {
    let rotation = Quat::from_axis_angle(axis, angle);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_staircase_with_rails(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    concrete_mat: &Handle<StandardMaterial>,
    steel_mat: &Handle<StandardMaterial>,
    center: Vec3,
    width: f32,
    total_height: f32,
    depth: f32,
) {
    let slope_len = (depth * depth + total_height * total_height).sqrt();
    let pitch = (total_height / depth).atan();
    commands.spawn((
        Transform::from_translation(center).with_rotation(Quat::from_rotation_x(pitch)),
        RigidBody::Static,
        Collider::cuboid(width, 0.4, slope_len),
    ));

    let steps = 6;
    let step_h = total_height / steps as f32;
    let step_d = depth / steps as f32;
    for i in 0..steps {
        let h = step_h * (i + 1) as f32;
        let z_offset = (depth / 2.0) - step_d * (i as f32 + 0.5);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(width, h, step_d))),
            MeshMaterial3d(concrete_mat.clone()),
            Transform::from_xyz(center.x, h / 2.0, center.z + z_offset),
        ));
    }

    for side in [-1.0, 1.0] {
        let rx = center.x + side * (width / 2.0 - 0.1);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.08, 0.08, slope_len))),
            MeshMaterial3d(steel_mat.clone()),
            Transform::from_xyz(rx, center.y + 0.85, center.z)
                .with_rotation(Quat::from_rotation_x(pitch)),
        ));
        for pz in [-depth * 0.4, 0.0, depth * 0.4] {
            let py = center.y * (1.0 - (pz / (depth * 0.5))) + 0.45;
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.04, 0.9))),
                MeshMaterial3d(steel_mat.clone()),
                Transform::from_xyz(rx, py, center.z + pz),
            ));
        }
    }
}

fn spawn_shipping_container(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    trim_mat: &Handle<StandardMaterial>,
    pos: Vec3,
    size: Vec3,
    yaw: f32,
) {
    let rot = Quat::from_rotation_y(yaw);
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
        ))
        .with_children(|parent| {
            let rib_mesh = meshes.add(Cuboid::new(0.12, size.y * 0.92, size.z + 0.08));
            let num_ribs = (size.x / 0.85) as i32;
            for i in 0..num_ribs {
                let x = -size.x * 0.44 + (i as f32) * 0.85;
                parent.spawn((
                    Mesh3d(rib_mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(x, 0.0, 0.0),
                ));
            }
            let top_beam = meshes.add(Cuboid::new(size.x + 0.06, 0.14, size.z + 0.06));
            parent.spawn((
                Mesh3d(top_beam),
                MeshMaterial3d(trim_mat.clone()),
                Transform::from_xyz(0.0, size.y * 0.5, 0.0),
            ));
        });
}

fn spawn_jersey_barrier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
    length: f32,
    yaw: f32,
) {
    let rot = Quat::from_rotation_y(yaw);
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(length, 0.95, 0.55))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(length, 1.3, 0.85),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(length + 0.04, 0.38, 0.92))),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(0.0, -0.30, 0.0),
            ));
        });
}

fn spawn_hazard_barrier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    yellow_mat: &Handle<StandardMaterial>,
    black_mat: &Handle<StandardMaterial>,
    pos: Vec3,
    length: f32,
    yaw: f32,
) {
    let rot = Quat::from_rotation_y(yaw);
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(length, 1.15, 0.60))),
            MeshMaterial3d(yellow_mat.clone()),
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(length, 1.2, 0.75),
        ))
        .with_children(|parent| {
            let stripe_mesh = meshes.add(Cuboid::new(0.28, 1.05, 0.66));
            for i in -2..=2 {
                parent.spawn((
                    Mesh3d(stripe_mesh.clone()),
                    MeshMaterial3d(black_mat.clone()),
                    Transform::from_xyz(i as f32 * 0.68, 0.0, 0.0)
                        .with_rotation(Quat::from_rotation_z(0.38)),
                ));
            }
        });
}

fn spawn_sandbag_pile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
    yaw: f32,
) {
    let rot = Quat::from_rotation_y(yaw);
    commands
        .spawn((
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(3.8, 1.3, 1.8),
        ))
        .with_children(|parent| {
            let bag_mesh = meshes.add(Capsule3d::new(0.24, 0.65));
            for layer in 0..4 {
                let y = 0.22 + layer as f32 * 0.32;
                let count = 3 - (layer / 2);
                for i in 0..count {
                    let x = (i as f32 - (count as f32 - 1.0) * 0.5) * 1.12;
                    for z_row in [-0.26, 0.26] {
                        parent.spawn((
                            Mesh3d(bag_mesh.clone()),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_xyz(x, y, z_row).with_rotation(
                                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                                    * Quat::from_rotation_x((i as f32 + layer as f32) * 0.12),
                            ),
                        ));
                    }
                }
            }
        });
}

fn spawn_oil_drum(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.46, 1.15))),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(pos),
        RigidBody::Static,
        Collider::cylinder(0.46, 1.15),
    ));
}

fn spawn_chainlink_fence_segment(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    fence_mat: &Handle<StandardMaterial>,
    steel_mat: &Handle<StandardMaterial>,
    curb_mat: &Handle<StandardMaterial>,
    center: Vec3,
    length: f32,
    along_x: bool,
) {
    let height = 3.1;
    let size = if along_x {
        Vec3::new(length, height, 0.10)
    } else {
        Vec3::new(0.10, height, length)
    };

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(fence_mat.clone()),
        Transform::from_translation(center + Vec3::Y * (height * 0.5)),
    ));

    let curb_size = if along_x {
        Vec3::new(length, 0.35, 0.45)
    } else {
        Vec3::new(0.45, 0.35, length)
    };
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(curb_size.x, curb_size.y, curb_size.z))),
        MeshMaterial3d(curb_mat.clone()),
        Transform::from_translation(center + Vec3::Y * 0.175),
    ));

    for wire_y in [height, height + 0.22, height + 0.42] {
        let rail_size = if along_x {
            Vec3::new(length, 0.04, 0.04)
        } else {
            Vec3::new(0.04, 0.04, length)
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(rail_size.x, rail_size.y, rail_size.z))),
            MeshMaterial3d(steel_mat.clone()),
            Transform::from_translation(center + Vec3::Y * wire_y),
        ));
    }

    let post_mesh = meshes.add(Cylinder::new(0.08, height + 0.55));
    let count = (length / 5.0).ceil() as i32;
    for i in 0..=count {
        let offset = -length * 0.5 + (i as f32) * (length / count.max(1) as f32);
        let post_pos = if along_x {
            center + Vec3::new(offset, (height + 0.55) * 0.5, 0.0)
        } else {
            center + Vec3::new(0.0, (height + 0.55) * 0.5, offset)
        };
        commands.spawn((
            Mesh3d(post_mesh.clone()),
            MeshMaterial3d(steel_mat.clone()),
            Transform::from_translation(post_pos),
        ));
    }
}

fn spawn_post_apocalyptic_vista(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // 1. Open-Top Panoramic Stormy Post-Apocalyptic Sky Ring (No horizontal ceiling caps!)
    let horizon_amber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.52, 0.36),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let mid_storm_slate = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.28, 0.30),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let upper_charcoal_cloud = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.15, 0.17),
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let num_sky_segments = 24;
    let sky_radius = 175.0_f32;
    let seg_width = 2.0 * std::f32::consts::PI * sky_radius / (num_sky_segments as f32) + 3.0;
    let band_low_mesh = meshes.add(Cuboid::new(seg_width, 26.0, 1.0));
    let band_mid_mesh = meshes.add(Cuboid::new(seg_width, 32.0, 1.0));
    let band_high_mesh = meshes.add(Cuboid::new(seg_width, 55.0, 1.0));

    for i in 0..num_sky_segments {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_sky_segments as f32);
        let sx = sky_radius * angle.sin();
        let sz = sky_radius * angle.cos();
        let rot = Quat::from_rotation_y(angle);

        commands.spawn((
            Mesh3d(band_low_mesh.clone()),
            MeshMaterial3d(horizon_amber.clone()),
            Transform::from_xyz(sx, 10.0, sz).with_rotation(rot),
        ));
        commands.spawn((
            Mesh3d(band_mid_mesh.clone()),
            MeshMaterial3d(mid_storm_slate.clone()),
            Transform::from_xyz(sx, 37.0, sz).with_rotation(rot),
        ));
        commands.spawn((
            Mesh3d(band_high_mesh.clone()),
            MeshMaterial3d(upper_charcoal_cloud.clone()),
            Transform::from_xyz(sx, 78.0, sz).with_rotation(rot),
        ));
    }

    // 2. Ruined Skyscraper City Skyline, Smokestacks & Collapsed Truss Bridge (Left/North-West)
    let ruin_dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.21, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });
    let smokestack_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.24, 0.20),
        perceptual_roughness: 0.88,
        ..default()
    });
    let smoke_plume_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.16, 0.16, 0.17, 0.65),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let moss_vine_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.32, 0.15),
        perceptual_roughness: 0.95,
        ..default()
    });

    // Jagged Ruined Skyscrapers on the North-West skyline (matching art/scene/post_apocalyptic_scene.jpg)
    for (bx, bh, bz, bw) in [
        (-52.0, 26.0, -78.0, 9.5),
        (-38.0, 42.0, -82.0, 11.5), // Tallest central ruined tower
        (-25.0, 28.0, -80.0, 10.5),
        (-13.0, 24.0, -78.0, 7.5),
        (16.0, 18.0, -76.0, 6.0),
        (26.0, 20.0, -74.0, 14.0),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(bw, bh, 8.5))),
            MeshMaterial3d(ruin_dark_mat.clone()),
            Transform::from_xyz(bx, bh * 0.5, bz),
        ));
        // Jagged broken upper crown on top of each ruined skyscraper
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(bw * 0.55, 5.5, 6.5))),
            MeshMaterial3d(ruin_dark_mat.clone()),
            Transform::from_xyz(bx - bw * 0.2, bh + 2.5, bz),
        ));
    }

    // Factory Smokestacks (3 on the far Left, 2 on the Right) with dark drifting smoke plumes
    for (sx, sz, sh) in [
        (-82.0, -68.0, 24.0),
        (-74.0, -66.0, 26.0),
        (-64.0, -64.0, 32.0),
        (36.0, -72.0, 26.0),
        (45.0, -70.0, 29.0),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(1.2, sh))),
            MeshMaterial3d(smokestack_mat.clone()),
            Transform::from_xyz(sx, sh * 0.5, sz),
        ));
        // Drifting dark smoke clouds rising from the stacks
        for i in 1..=3 {
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(2.2 + i as f32 * 0.9))),
                MeshMaterial3d(smoke_plume_mat.clone()),
                Transform::from_xyz(
                    sx + i as f32 * 2.8,
                    sh + i as f32 * 2.4,
                    sz,
                ),
            ));
        }
    }

    // Stacked Tilted Containers outside the Left Fence (matching far-left of art)
    let ext_rust_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.44, 0.22, 0.14),
        perceptual_roughness: 0.85,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 3.4, 3.6))),
        MeshMaterial3d(ruin_dark_mat.clone()),
        Transform::from_xyz(-42.0, 1.7, -18.0).with_rotation(Quat::from_rotation_y(0.35)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 3.4, 3.6))),
        MeshMaterial3d(ext_rust_mat.clone()),
        Transform::from_xyz(-38.0, 4.6, -20.0)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.25, 0.0, -0.32)),
    ));

    // 3. Collapsed Elevated Concrete Highway Overpass on the Right Horizon (matching art)
    let overpass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.34, 0.34, 0.32),
        perceptual_roughness: 0.9,
        ..default()
    });
    for i in 0..5 {
        let ox = 20.0 + i as f32 * 13.0;
        let oz = -48.0 + i as f32 * 4.5;
        // Twin concrete support piers
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(1.1, 11.0))),
            MeshMaterial3d(overpass_mat.clone()),
            Transform::from_xyz(ox, 5.5, oz),
        ));
        // Elevated highway deck span (with a collapsed tilted span on segment 0)
        let (deck_y, deck_pitch) = if i == 0 { (5.2, 0.38) } else { (11.2, 0.0) };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(13.5, 1.4, 6.5))),
            MeshMaterial3d(overpass_mat.clone()),
            Transform::from_xyz(ox, deck_y, oz).with_rotation(
                Quat::from_rotation_y(-0.32) * Quat::from_rotation_z(deck_pitch),
            ),
        ));
        // Overgrown moss/vines hanging from the highway deck
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(8.5, 1.8, 6.8))),
            MeshMaterial3d(moss_vine_mat.clone()),
            Transform::from_xyz(ox + 1.0, deck_y - 0.6, oz)
                .with_rotation(Quat::from_rotation_y(-0.32)),
        ));
    }

    // 4. Tracked Crawler Crane on the Far-Right Outside the Fence (matching art)
    let crane_body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.56, 0.36, 0.16),
        perceptual_roughness: 0.75,
        metallic: 0.3,
        ..default()
    });
    let crane_pos = Vec3::new(42.0, 0.0, -14.0);
    commands
        .spawn(Transform::from_translation(crane_pos))
        .with_children(|crane| {
            // Heavy crawler tracks
            crane.spawn((
                Mesh3d(meshes.add(Cuboid::new(6.5, 1.4, 4.8))),
                MeshMaterial3d(ruin_dark_mat.clone()),
                Transform::from_xyz(0.0, 0.7, 0.0),
            ));
            // Rusted yellow-brown crane cab
            crane.spawn((
                Mesh3d(meshes.add(Cuboid::new(5.2, 2.8, 3.8))),
                MeshMaterial3d(crane_body_mat.clone()),
                Transform::from_xyz(0.5, 2.6, 0.0),
            ));
            // Tall angled lattice boom reaching up into the stormy sky
            crane.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.1, 32.0, 1.1))),
                MeshMaterial3d(ruin_dark_mat.clone()),
                Transform::from_xyz(-4.5, 16.5, 0.0)
                    .with_rotation(Quat::from_rotation_z(0.32)),
            ));
            // Dangling hook cable from boom tip
            crane.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.06, 9.5))),
                MeshMaterial3d(ruin_dark_mat.clone()),
                Transform::from_xyz(-9.5, 25.5, 0.0),
            ));
        });
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
