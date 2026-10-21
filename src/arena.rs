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
        "Sentralsonen (Courtyard)",
        Vec3::new(-0.5, 0.1, 1.0),
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
    // 1. GOLDEN-HOUR SUNSET ATMOSPHERE & LIGHTING (Matching art/skybox.jpg)
    // =========================================================================
    commands.insert_resource(ClearColor(Color::srgb(0.92, 0.70, 0.48)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.96, 0.88, 0.80),
        brightness: 1500.0,
    });

    // Golden-hour sun from the West/North-West casting warm highlights
    commands.spawn((
        DirectionalLight {
            illuminance: 58000.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.91, 0.76),
            ..default()
        },
        Transform::from_xyz(-45.0, 48.0, -35.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Cool twilight sky fill light from the South-East
    commands.spawn((
        DirectionalLight {
            illuminance: 22000.0,
            shadows_enabled: false,
            color: Color::srgb(0.62, 0.76, 0.95),
            ..default()
        },
        Transform::from_xyz(35.0, 45.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // =========================================================================
    // 2. PALETTE & MATERIALS
    // =========================================================================
    // Warm dusty concrete courtyard floor
    let courtyard_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.60, 0.56),
        perceptual_roughness: 0.92,
        metallic: 0.02,
        ..default()
    });

    // Lighter concrete for platforms, ramps, stairs, and walls
    let concrete_light_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.74, 0.73, 0.70),
        perceptual_roughness: 0.85,
        metallic: 0.05,
        ..default()
    });

    // Medium gray concrete for jersey barriers & curbs
    let barrier_gray_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.56, 0.57, 0.58),
        perceptual_roughness: 0.82,
        metallic: 0.05,
        ..default()
    });

    // Dark outline / trim metal for comic edge definition
    let dark_trim_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.24, 0.26),
        perceptual_roughness: 0.5,
        metallic: 0.6,
        ..default()
    });

    let steel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.50, 0.53, 0.56),
        perceptual_roughness: 0.4,
        metallic: 0.8,
        ..default()
    });

    // Translucent chain-link mesh
    let fence_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.45, 0.48, 0.50, 0.48),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.4,
        metallic: 0.8,
        cull_mode: None,
        ..default()
    });

    // Grass patches inside & outside the compound
    let grass_patch_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.58, 0.26),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    let exterior_grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.36, 0.52, 0.24),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    // Lush green hedge bushes
    let hedge_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.48, 0.20),
        perceptual_roughness: 0.92,
        metallic: 0.0,
        ..default()
    });

    let hedge_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.58, 0.25),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    // Shipping containers (Rust-Red on Left, Industrial Blue on Right)
    let container_red_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.24, 0.16),
        perceptual_roughness: 0.55,
        metallic: 0.3,
        ..default()
    });

    let container_blue_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.36, 0.58),
        perceptual_roughness: 0.55,
        metallic: 0.3,
        ..default()
    });

    // Sandbags (warm burlap tan)
    let sandbag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.62, 0.44),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    // Barrels & Crates
    let drum_green_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.44, 0.24),
        perceptual_roughness: 0.45,
        metallic: 0.55,
        ..default()
    });

    let drum_brown_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.32, 0.18),
        perceptual_roughness: 0.5,
        metallic: 0.5,
        ..default()
    });

    let drum_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.64, 0.22),
        perceptual_roughness: 0.45,
        metallic: 0.5,
        ..default()
    });

    let drum_blue_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.40, 0.65),
        perceptual_roughness: 0.45,
        metallic: 0.5,
        ..default()
    });

    let crate_purple_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.58, 0.26, 0.68),
        perceptual_roughness: 0.65,
        metallic: 0.1,
        ..default()
    });

    let crate_green_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.36, 0.66, 0.24),
        perceptual_roughness: 0.65,
        metallic: 0.1,
        ..default()
    });

    let orange_barrier_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.38, 0.18),
        perceptual_roughness: 0.6,
        metallic: 0.05,
        ..default()
    });

    let hazard_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.76, 0.16),
        perceptual_roughness: 0.6,
        metallic: 0.05,
        ..default()
    });

    // Platform zone circles & flags
    let blue_zone_disc_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 0.62, 1.0, 0.32),
        emissive: LinearRgba::new(0.1, 1.2, 2.8, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let blue_flag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.45, 0.92),
        emissive: LinearRgba::new(0.05, 0.4, 1.2, 1.0),
        perceptual_roughness: 0.6,
        cull_mode: None,
        ..default()
    });

    let red_zone_disc_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.25, 0.18, 0.32),
        emissive: LinearRgba::new(2.5, 0.3, 0.1, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let red_flag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.16, 0.14),
        emissive: LinearRgba::new(1.2, 0.1, 0.05, 1.0),
        perceptual_roughness: 0.6,
        cull_mode: None,
        ..default()
    });

    // =========================================================================
    // 3. COMPOUND COURTYARD FLOOR & GRASSY PATCHES (54m x 40m)
    // =========================================================================
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(56.0, 1.0, 42.0))),
        MeshMaterial3d(courtyard_mat),
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(56.0, 1.0, 42.0),
    ));

    // Exterior surrounding grass ground plane
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(260.0, 1.0, 260.0))),
        MeshMaterial3d(exterior_grass_mat),
        Transform::from_xyz(0.0, -0.65, 0.0),
    ));

    // Mossy / grassy patches inside the courtyard (matching art/skybox.jpg)
    for (gx, gz, gw, gd, gyaw) in [
        (-13.5, 9.5, 6.5, 3.2, -0.35),
        (-6.2, 5.5, 9.5, 2.6, -0.48),
        (3.8, -0.8, 6.2, 2.5, -0.48),
        (11.5, 9.5, 3.5, 8.5, 0.25),
        (-22.0, -9.5, 5.0, 3.0, 0.1),
        (18.0, -11.5, 6.0, 2.8, -0.2),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(gw, 0.03, gd))),
            MeshMaterial3d(grass_patch_mat.clone()),
            Transform::from_xyz(gx, 0.01, gz).with_rotation(Quat::from_rotation_y(gyaw)),
        ));
    }

    // =========================================================================
    // 4. WEST BLUE PLATFORM (Zone Beta) + FRONT RAMP & FLAG
    // =========================================================================
    let (beta_idx, beta_name, beta_center, beta_radius) = PLATEAU_COORDS[1];
    // Main concrete platform block
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.5, 1.8, 12.0))),
        MeshMaterial3d(concrete_light_mat.clone()),
        Transform::from_xyz(beta_center.x, 0.9, beta_center.z),
        RigidBody::Static,
        Collider::cuboid(11.5, 1.8, 12.0),
        Plateau {
            index: beta_idx,
            name: beta_name,
            center: beta_center,
            radius: beta_radius,
        },
    ));
    // Dark top trim border on Blue Platform
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.65, 0.08, 12.15))),
        MeshMaterial3d(dark_trim_mat.clone()),
        Transform::from_xyz(beta_center.x, 1.78, beta_center.z),
    ));

    // Glowing Blue Capture Circle on West Platform
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(4.2, 0.04))),
        MeshMaterial3d(blue_zone_disc_mat),
        Transform::from_xyz(beta_center.x, 1.82, beta_center.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Torus {
            minor_radius: 0.06,
            major_radius: 4.2,
        })),
        MeshMaterial3d(blue_flag_mat.clone()),
        Transform::from_xyz(beta_center.x, 1.83, beta_center.z),
    ));

    // Blue Flagpole & Waving Blue Flag (matching left side of art/skybox.jpg)
    spawn_flagpole(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &blue_flag_mat,
        Vec3::new(beta_center.x - 2.2, 1.8, beta_center.z - 1.2),
        1.0,
    );

    // Wide Front Concrete Ramp sloping down from Blue Platform towards South-East
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        Vec3::new(-15.5, 0.72, 6.8),
        Vec3::new(6.8, 0.5, 7.2),
        0.30,
        Vec3::X,
    );
    // Secondary Side Ramp on North-East edge of Blue Platform
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        Vec3::new(-10.6, 0.72, -4.2),
        Vec3::new(5.2, 0.5, 3.6),
        -0.34,
        Vec3::Z,
    );

    // =========================================================================
    // 5. EAST RED PLATFORM (Zone Gamma) + STAIRS, HANDRAILS & FLAG
    // =========================================================================
    let (gamma_idx, gamma_name, gamma_center, gamma_radius) = PLATEAU_COORDS[2];
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.5, 1.8, 11.5))),
        MeshMaterial3d(concrete_light_mat.clone()),
        Transform::from_xyz(gamma_center.x, 0.9, gamma_center.z),
        RigidBody::Static,
        Collider::cuboid(11.5, 1.8, 11.5),
        Plateau {
            index: gamma_idx,
            name: gamma_name,
            center: gamma_center,
            radius: gamma_radius,
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.65, 0.08, 11.65))),
        MeshMaterial3d(dark_trim_mat.clone()),
        Transform::from_xyz(gamma_center.x, 1.78, gamma_center.z),
    ));

    // Glowing Red Capture Circle on East Platform
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(4.2, 0.04))),
        MeshMaterial3d(red_zone_disc_mat),
        Transform::from_xyz(gamma_center.x, 1.82, gamma_center.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Torus {
            minor_radius: 0.06,
            major_radius: 4.2,
        })),
        MeshMaterial3d(red_flag_mat.clone()),
        Transform::from_xyz(gamma_center.x, 1.83, gamma_center.z),
    ));

    // Red Flagpole & Waving Red Flag (matching right side of art/skybox.jpg)
    spawn_flagpole(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &red_flag_mat,
        Vec3::new(gamma_center.x + 1.2, 1.8, gamma_center.z - 1.0),
        -1.0,
    );

    // Stepped Concrete Staircase with Metal Handrails on Front/South of Red Platform
    spawn_staircase_with_rails(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        &steel_mat,
        Vec3::new(16.5, 0.9, 5.2),
        3.8,
        1.8,
        5.0,
    );
    // Smooth West-facing Ramp onto Red Platform for additional access
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        Vec3::new(9.6, 0.72, -3.5),
        Vec3::new(5.0, 0.5, 4.2),
        0.34,
        Vec3::Z,
    );

    // =========================================================================
    // 6. ZONE ALPHA: CENTRAL COURTYARD OBJECTIVE
    // =========================================================================
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

    // =========================================================================
    // 7. NORTH SHIPPING CONTAINERS (Red on Left, Blue on Right)
    // =========================================================================
    // Red Corrugated Shipping Container
    spawn_shipping_container(
        &mut commands,
        &mut meshes,
        &container_red_mat,
        &dark_trim_mat,
        Vec3::new(-6.8, 1.75, -13.5),
        Vec3::new(10.5, 3.5, 3.6),
        -0.12,
    );
    // Blue Corrugated Shipping Container right next to it
    spawn_shipping_container(
        &mut commands,
        &mut meshes,
        &container_blue_mat,
        &dark_trim_mat,
        Vec3::new(3.8, 1.75, -13.8),
        Vec3::new(10.0, 3.5, 3.6),
        -0.12,
    );

    // =========================================================================
    // 8. CENTER COURTYARD BARRIERS, SANDBAGS, HEDGE BUSHES, CRATES & DRUMS
    // =========================================================================
    // Concrete Jersey Barriers in front of Red Container
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &barrier_gray_mat,
        Vec3::new(-7.5, 0.65, -5.8),
        4.2,
        -0.38,
    );
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &barrier_gray_mat,
        Vec3::new(-3.2, 0.65, -7.4),
        4.2,
        -0.38,
    );

    // White/Light Concrete Barrier in front of Blue Container
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        Vec3::new(5.2, 0.65, -6.5),
        4.2,
        0.45,
    );

    // Stacked Sandbag Bunker in Center-Left (matching art/skybox.jpg)
    spawn_sandbag_pile(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-4.2, 0.0, -2.6),
        -0.42,
    );
    // Yellow/tan barrel right beside the sandbag bunker
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_yellow_mat,
        Vec3::new(-2.8, 0.55, -0.8),
    );

    // Row of 3 Trimmed Green Hedge Bushes (Left-Center Lane)
    for (hx, hz) in [(-8.2, 7.2), (-5.8, 4.8), (-3.5, 2.5)] {
        spawn_hedge_bush(
            &mut commands,
            &mut meshes,
            &hedge_mat,
            &hedge_highlight_mat,
            Vec3::new(hx, 0.65, hz),
            Vec3::new(2.8, 1.3, 1.6),
            -0.65,
        );
    }

    // Pair of 2 Trimmed Green Hedge Bushes (Upper-Right Center)
    for (hx, hz) in [(2.2, -1.8), (4.6, 0.2)] {
        spawn_hedge_bush(
            &mut commands,
            &mut meshes,
            &hedge_mat,
            &hedge_highlight_mat,
            Vec3::new(hx, 0.65, hz),
            Vec3::new(2.6, 1.3, 1.6),
            0.55,
        );
    }

    // Lower-Right Center Concrete Barrier + Orange Barrier + Yellow/Black Hazard Barrier
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        Vec3::new(4.2, 0.75, 4.0),
        5.5,
        -0.65,
    );
    // Orange plastic jersey barrier
    spawn_jersey_barrier(
        &mut commands,
        &mut meshes,
        &orange_barrier_mat,
        Vec3::new(-0.2, 0.55, 7.6),
        3.2,
        -0.65,
    );
    // Yellow & black diagonal hazard-striped barrier next to it
    spawn_hazard_barrier(
        &mut commands,
        &mut meshes,
        &hazard_yellow_mat,
        &dark_trim_mat,
        Vec3::new(3.2, 0.65, 7.8),
        3.8,
        0.52,
    );

    // Foreground Crates (Purple & Bright Green in bottom-left of art/skybox.jpg)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.2, 2.0))),
        MeshMaterial3d(crate_purple_mat),
        Transform::from_xyz(-4.8, 0.6, 14.5).with_rotation(Quat::from_rotation_y(-0.55)),
        RigidBody::Static,
        Collider::cuboid(2.0, 1.2, 2.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 2.0))),
        MeshMaterial3d(crate_green_mat),
        Transform::from_xyz(-2.8, 0.5, 16.0).with_rotation(Quat::from_rotation_y(-0.55)),
        RigidBody::Static,
        Collider::cuboid(2.0, 1.0, 2.0),
    ));

    // Foreground Pair of Green Barrels (bottom-center of art/skybox.jpg)
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_green_mat,
        Vec3::new(1.8, 0.55, 15.2),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_green_mat,
        Vec3::new(3.1, 0.55, 14.6),
    );

    // Additional barrels around platforms & containers (matching art/skybox.jpg)
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_brown_mat,
        Vec3::new(-11.2, 0.55, -0.5),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        & steel_mat,
        Vec3::new(-11.8, 0.55, -1.8),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_brown_mat,
        Vec3::new(9.2, 0.55, -7.8),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_green_mat,
        Vec3::new(10.5, 0.55, -7.2),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_blue_mat,
        Vec3::new(-0.8, 0.55, -10.8),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_green_mat,
        Vec3::new(-13.5, 0.55, -11.5),
    );
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_green_mat,
        Vec3::new(-15.0, 0.55, -11.2),
    );

    // =========================================================================
    // 9. PERIMETER WALLS & BARBED-WIRE CHAIN-LINK FENCES
    // =========================================================================
    // North (Back) Chain-Link Fence with Barbed Wire
    spawn_chainlink_fence_with_barbed_wire(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        &concrete_light_mat,
        Vec3::new(0.0, 0.0, -19.5),
        54.0,
        true,
    );
    // West (Left) Chain-Link Fence with Barbed Wire
    spawn_chainlink_fence_with_barbed_wire(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        &concrete_light_mat,
        Vec3::new(-26.5, 0.0, 0.0),
        39.0,
        false,
    );
    // South-West Foreground Chain-Link Fence
    spawn_chainlink_fence_with_barbed_wire(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        &concrete_light_mat,
        Vec3::new(-14.0, 0.0, 19.5),
        25.0,
        true,
    );
    // East / South-East Solid Paneled Concrete Perimeter Wall (matching bottom-right of art/skybox.jpg)
    spawn_paneled_concrete_wall(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        &dark_trim_mat,
        Vec3::new(12.0, 0.0, 19.5),
        28.0,
        true,
    );
    spawn_paneled_concrete_wall(
        &mut commands,
        &mut meshes,
        &concrete_light_mat,
        &dark_trim_mat,
        Vec3::new(26.0, 0.0, 0.0),
        39.0,
        false,
    );

    // Bushes tucked along the inside of the right concrete wall
    for (bx, bz) in [(8.5, 16.5), (12.5, 14.5), (16.5, 12.5)] {
        spawn_hedge_bush(
            &mut commands,
            &mut meshes,
            &hedge_mat,
            &hedge_highlight_mat,
            Vec3::new(bx, 0.55, bz),
            Vec3::new(2.5, 1.1, 1.5),
            -0.4,
        );
    }

    // Green Dumpster & Barrel outside the right wall (matching far-right of art/skybox.jpg)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 1.8, 3.6))),
        MeshMaterial3d(drum_green_mat.clone()),
        Transform::from_xyz(28.8, 0.9, 5.5),
    ));
    spawn_oil_drum(
        &mut commands,
        &mut meshes,
        &drum_green_mat,
        Vec3::new(28.2, 0.55, 8.2),
    );

    // =========================================================================
    // 10. EXTERIOR TREES, INDUSTRIAL SKYLINE, CRANES, PYLONS & SUNSET SKYBOX
    // =========================================================================
    spawn_exterior_skybox_and_vista(&mut commands, &mut meshes, &mut materials);
}

fn spawn_flagpole(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    pole_mat: &Handle<StandardMaterial>,
    flag_mat: &Handle<StandardMaterial>,
    base_pos: Vec3,
    dir_sign: f32,
) {
    // Round base pedestal
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.42, 0.22))),
        MeshMaterial3d(flag_mat.clone()),
        Transform::from_translation(base_pos + Vec3::new(0.0, 0.11, 0.0)),
    ));
    // Tall steel flagpole
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.07, 5.2))),
        MeshMaterial3d(pole_mat.clone()),
        Transform::from_translation(base_pos + Vec3::new(0.0, 2.6, 0.0)),
    ));
    // Waving cloth banner
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.2, 1.25, 0.06))),
        MeshMaterial3d(flag_mat.clone()),
        Transform::from_translation(base_pos + Vec3::new(dir_sign * 1.1, 4.4, 0.0)),
    ));
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
    // Smooth underlying ramp collider so characters run up effortlessly
    let slope_len = (depth * depth + total_height * total_height).sqrt();
    let pitch = (total_height / depth).atan();
    commands.spawn((
        Transform::from_translation(center).with_rotation(Quat::from_rotation_x(pitch)),
        RigidBody::Static,
        Collider::cuboid(width, 0.4, slope_len),
    ));

    // Visual steps (6 steps)
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

    // Twin steel handrails on left and right sides of the stairs
    for side in [-1.0, 1.0] {
        let rx = center.x + side * (width / 2.0 - 0.1);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.08, 0.08, slope_len))),
            MeshMaterial3d(steel_mat.clone()),
            Transform::from_xyz(rx, center.y + 0.85, center.z)
                .with_rotation(Quat::from_rotation_x(pitch)),
        ));
        // Vertical rail posts
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
            // Corrugated vertical ribs along the front & back walls
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
            // Corner frame beams
            let top_beam = meshes.add(Cuboid::new(size.x + 0.06, 0.14, size.z + 0.06));
            parent.spawn((
                Mesh3d(top_beam.clone()),
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
            // Wide flared concrete base of the New Jersey barrier
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
            Mesh3d(meshes.add(Cuboid::new(length, 1.25, 0.65))),
            MeshMaterial3d(yellow_mat.clone()),
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(length, 1.25, 0.75),
        ))
        .with_children(|parent| {
            // Diagonal black hazard stripes
            let stripe_mesh = meshes.add(Cuboid::new(0.28, 1.15, 0.70));
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
            Collider::cuboid(3.8, 1.5, 1.8),
        ))
        .with_children(|parent| {
            let bag_mesh = meshes.add(Capsule3d::new(0.24, 0.65));
            for layer in 0..4 {
                let y = 0.22 + layer as f32 * 0.34;
                let count = 3 - (layer / 2);
                for i in 0..count {
                    let x = (i as f32 - (count as f32 - 1.0) * 0.5) * 1.15;
                    for z_row in [-0.28, 0.28] {
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

fn spawn_hedge_bush(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    base_mat: &Handle<StandardMaterial>,
    top_mat: &Handle<StandardMaterial>,
    pos: Vec3,
    size: Vec3,
    yaw: f32,
) {
    let rot = Quat::from_rotation_y(yaw);
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(base_mat.clone()),
            Transform::from_translation(pos).with_rotation(rot),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
        ))
        .with_children(|parent| {
            // Leafy rounded top puffs for organic comic-book bush silhouette
            let puff = meshes.add(Sphere::new(size.z * 0.52));
            for dx in [-size.x * 0.3, 0.0, size.x * 0.3] {
                parent.spawn((
                    Mesh3d(puff.clone()),
                    MeshMaterial3d(top_mat.clone()),
                    Transform::from_xyz(dx, size.y * 0.32, 0.0),
                ));
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

fn spawn_chainlink_fence_with_barbed_wire(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    fence_mat: &Handle<StandardMaterial>,
    steel_mat: &Handle<StandardMaterial>,
    curb_mat: &Handle<StandardMaterial>,
    center: Vec3,
    length: f32,
    along_x: bool,
) {
    let height = 3.2;
    let size = if along_x {
        Vec3::new(length, height, 0.12)
    } else {
        Vec3::new(0.12, height, length)
    };

    // Main fence mesh & physics collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(fence_mat.clone()),
        Transform::from_translation(center + Vec3::Y * (height * 0.5)),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y + 1.5, size.z.max(0.4)),
    ));

    // Low concrete base curb
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

    // Top horizontal rail + 2 strands of barbed wire
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

    // Steel fence posts every 5.5m
    let post_mesh = meshes.add(Cylinder::new(0.08, height + 0.55));
    let count = (length / 5.5) as i32;
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

fn spawn_paneled_concrete_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    wall_mat: &Handle<StandardMaterial>,
    trim_mat: &Handle<StandardMaterial>,
    center: Vec3,
    length: f32,
    along_x: bool,
) {
    let height = 2.6;
    let size = if along_x {
        Vec3::new(length, height, 0.55)
    } else {
        Vec3::new(0.55, height, length)
    };

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_translation(center + Vec3::Y * (height * 0.5)),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y + 1.5, size.z),
    ));

    // Concrete pillars every 5.0m with top capstones (matching right wall in art/skybox.jpg)
    let pillar_mesh = meshes.add(Cuboid::new(0.85, height + 0.25, 0.85));
    let cap_mesh = meshes.add(Cuboid::new(0.95, 0.12, 0.95));
    let count = (length / 5.0) as i32;
    for i in 0..=count {
        let offset = -length * 0.5 + (i as f32) * (length / count.max(1) as f32);
        let p_pos = if along_x {
            center + Vec3::new(offset, (height + 0.25) * 0.5, 0.0)
        } else {
            center + Vec3::new(0.0, (height + 0.25) * 0.5, offset)
        };
        commands.spawn((
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_translation(p_pos),
        ));
        commands.spawn((
            Mesh3d(cap_mesh.clone()),
            MeshMaterial3d(trim_mat.clone()),
            Transform::from_translation(p_pos + Vec3::Y * ((height + 0.25) * 0.5)),
        ));
    }
}

fn spawn_exterior_skybox_and_vista(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // 1. Golden-Hour Sunset Sky Backdrop Bands (Unlit panoramic horizon cylinder bands)
    let horizon_gold = materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.76, 0.50),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let mid_peach = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.64, 0.52),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let upper_sky_blue = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.56, 0.78),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let cloud_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.85, 0.68),
        unlit: true,
        ..default()
    });

    // Panoramic sky backdrop panels arranged around a 165m perimeter circle
    // (Using vertical wall panels instead of closed Cylinders so there are no horizontal
    // top/bottom cylinder end-caps obstructing high-altitude overview cameras!)
    let num_sky_segments = 24;
    let sky_radius = 165.0_f32;
    let seg_width = 2.0 * std::f32::consts::PI * sky_radius / (num_sky_segments as f32) + 2.5;
    let band_low_mesh = meshes.add(Cuboid::new(seg_width, 28.0, 1.0));
    let band_mid_mesh = meshes.add(Cuboid::new(seg_width, 30.0, 1.0));
    let band_high_mesh = meshes.add(Cuboid::new(seg_width, 50.0, 1.0));

    for i in 0..num_sky_segments {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_sky_segments as f32);
        let sx = sky_radius * angle.sin();
        let sz = sky_radius * angle.cos();
        let rot = Quat::from_rotation_y(angle);

        commands.spawn((
            Mesh3d(band_low_mesh.clone()),
            MeshMaterial3d(horizon_gold.clone()),
            Transform::from_xyz(sx, 10.0, sz).with_rotation(rot),
        ));
        commands.spawn((
            Mesh3d(band_mid_mesh.clone()),
            MeshMaterial3d(mid_peach.clone()),
            Transform::from_xyz(sx, 38.0, sz).with_rotation(rot),
        ));
        commands.spawn((
            Mesh3d(band_high_mesh.clone()),
            MeshMaterial3d(upper_sky_blue.clone()),
            Transform::from_xyz(sx, 76.0, sz).with_rotation(rot),
        ));
    }

    // Golden-hour clouds across the northern sky
    for (cx, cy, cz, cw) in [
        (-55.0, 28.0, -110.0, 26.0),
        (-15.0, 32.0, -115.0, 32.0),
        (35.0, 30.0, -110.0, 28.0),
        (75.0, 26.0, -105.0, 22.0),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(cw, 4.5, 6.0))),
            MeshMaterial3d(cloud_mat.clone()),
            Transform::from_xyz(cx, cy, cz),
        ));
    }

    // 2. Pine Trees & Round Leafy Trees Outside the Compound Fence (matching art/skybox.jpg)
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.25, 0.16),
        perceptual_roughness: 0.9,
        ..default()
    });
    let pine_dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.38, 0.18),
        perceptual_roughness: 0.9,
        ..default()
    });
    let leafy_green_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.50, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Coniferous Pine Trees
    for (px, pz, scale) in [
        (-31.0, -14.0, 1.15),
        (-28.0, -23.0, 1.0),
        (-13.5, -23.5, 1.25),
        (-2.5, -23.5, 1.15),
        (16.0, -24.0, 1.05),
        (-32.5, 4.0, 1.1),
    ] {
        commands
            .spawn((
                Transform::from_xyz(px, 0.0, pz).with_scale(Vec3::splat(scale)),
            ))
            .with_children(|tree| {
                tree.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.35, 3.0))),
                    MeshMaterial3d(trunk_mat.clone()),
                    Transform::from_xyz(0.0, 1.5, 0.0),
                ));
                for (tier_y, radius, h) in [(3.2, 2.4, 3.2), (5.0, 1.9, 2.8), (6.6, 1.3, 2.4)] {
                    tree.spawn((
                        Mesh3d(meshes.add(Cone { radius, height: h })),
                        MeshMaterial3d(pine_dark_mat.clone()),
                        Transform::from_xyz(0.0, tier_y, 0.0),
                    ));
                }
            });
    }

    // Round Leafy Deciduous Trees
    for (lx, lz, radius) in [
        (-20.0, -22.5, 2.8),
        (4.5, -22.0, 2.6),
        (27.5, -14.0, 2.5),
        (29.0, -5.0, 2.8),
        (-31.5, -5.0, 2.5),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.35, 2.6))),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_xyz(lx, 1.3, lz),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(leafy_green_mat.clone()),
            Transform::from_xyz(lx, 3.8, lz),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(radius * 0.78))),
            MeshMaterial3d(leafy_green_mat.clone()),
            Transform::from_xyz(lx + 1.1, 3.4, lz + 0.6),
        ));
    }

    // 3. Background Industrial Warehouses & City Skyline (matching art/skybox.jpg)
    let beige_building_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.60, 0.50),
        perceptual_roughness: 0.85,
        ..default()
    });
    let roof_gray_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.46, 0.48),
        perceptual_roughness: 0.7,
        ..default()
    });
    let blue_warehouse_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.44, 0.62),
        perceptual_roughness: 0.6,
        metallic: 0.25,
        ..default()
    });
    let distant_city_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.46, 0.50),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Large Blue Industrial Warehouse on the Right (behind the Red Flag Platform)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(28.0, 9.5, 16.0))),
        MeshMaterial3d(blue_warehouse_mat),
        Transform::from_xyz(34.0, 4.75, -29.0).with_rotation(Quat::from_rotation_y(-0.22)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(28.6, 1.2, 16.6))),
        MeshMaterial3d(roof_gray_mat.clone()),
        Transform::from_xyz(34.0, 9.8, -29.0).with_rotation(Quat::from_rotation_y(-0.22)),
    ));

    // Left / North-West Tan Industrial Warehouses & Multi-story Factory
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(24.0, 7.5, 12.0))),
        MeshMaterial3d(beige_building_mat.clone()),
        Transform::from_xyz(-32.0, 3.75, -32.0).with_rotation(Quat::from_rotation_y(0.15)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(14.0, 14.0, 12.0))),
        MeshMaterial3d(beige_building_mat.clone()),
        Transform::from_xyz(-16.0, 7.0, -42.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 9.0, 10.0))),
        MeshMaterial3d(beige_building_mat),
        Transform::from_xyz(-4.0, 4.5, -36.0),
    ));

    // Distant City Skyline Blocks & Factory Smokestacks
    for (bx, bh, bz, bw) in [
        (-52.0, 14.0, -65.0, 12.0),
        (-38.0, 18.0, -68.0, 9.0),
        (-25.0, 22.0, -70.0, 10.0),
        (8.0, 16.0, -65.0, 12.0),
        (22.0, 20.0, -68.0, 10.0),
        (55.0, 18.0, -62.0, 11.0),
        (68.0, 24.0, -66.0, 9.0),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(bw, bh, 8.0))),
            MeshMaterial3d(distant_city_mat.clone()),
            Transform::from_xyz(bx, bh * 0.5, bz),
        ));
    }

    // 4. Tall Yellow Tower Crane on Right Horizon (matching art/skybox.jpg)
    let crane_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.68, 0.18),
        perceptual_roughness: 0.5,
        metallic: 0.4,
        ..default()
    });
    // Vertical crane mast
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 34.0, 1.4))),
        MeshMaterial3d(crane_yellow_mat.clone()),
        Transform::from_xyz(42.0, 17.0, -52.0),
    ));
    // Horizontal crane jib boom
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(26.0, 1.1, 1.1))),
        MeshMaterial3d(crane_yellow_mat.clone()),
        Transform::from_xyz(35.0, 32.5, -52.0).with_rotation(Quat::from_rotation_y(0.25)),
    ));
    // Top crane peak
    commands.spawn((
        Mesh3d(meshes.add(Cone {
            radius: 1.2,
            height: 4.5,
        })),
        MeshMaterial3d(crane_yellow_mat),
        Transform::from_xyz(42.0, 36.0, -52.0),
    ));

    // 5. Electrical Power Pylons on Left Horizon (matching art/skybox.jpg)
    let pylon_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.36, 0.34, 0.35),
        perceptual_roughness: 0.6,
        metallic: 0.5,
        ..default()
    });
    for px in [-48.0, -32.0] {
        // Pylon tower body
        commands.spawn((
            Mesh3d(meshes.add(Cone {
                radius: 1.8,
                height: 24.0,
            })),
            MeshMaterial3d(pylon_mat.clone()),
            Transform::from_xyz(px, 12.0, -54.0),
        ));
        // Cross-arms
        for arm_y in [16.5, 20.5] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(6.5, 0.35, 0.35))),
                MeshMaterial3d(pylon_mat.clone()),
                Transform::from_xyz(px, arm_y, -54.0),
            ));
        }
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
