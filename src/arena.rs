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
        "Zone Alpha (Central Depot)",
        Vec3::new(0.0, 0.8, 0.0),
        6.0,
    ),
    (
        1,
        "Zone Beta (West Blue Platform)",
        Vec3::new(-24.0, 2.0, 0.0),
        5.5,
    ),
    (
        2,
        "Zone Gamma (East Red Platform)",
        Vec3::new(24.0, 2.0, 0.0),
        5.5,
    ),
];

fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Crisp Daylight Atmosphere & Lighting
    // Sky clear color matching bright outdoor compound
    commands.insert_resource(ClearColor(Color::srgb(0.44, 0.68, 0.92)));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.78, 0.86, 0.96),
        brightness: 1600.0,
    });

    // Sun: Main warm directional sunlight casting sharp tactical shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 65000.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.97, 0.90),
            ..default()
        },
        Transform::from_xyz(38.0, 56.0, 32.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Secondary sky bounce light
    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: false,
            color: Color::srgb(0.55, 0.72, 0.95),
            ..default()
        },
        Transform::from_xyz(-32.0, 42.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Shared Materials
    let tarmac_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.22, 0.24),
        perceptual_roughness: 0.88,
        metallic: 0.05,
        ..default()
    });

    let concrete_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.40, 0.42),
        perceptual_roughness: 0.75,
        metallic: 0.1,
        ..default()
    });

    let steel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.56, 0.60),
        perceptual_roughness: 0.35,
        metallic: 0.85,
        ..default()
    });

    let fence_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.60, 0.65, 0.60),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.4,
        metallic: 0.8,
        cull_mode: None,
        ..default()
    });

    let marking_white = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.88, 0.88),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });

    let marking_yellow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.75, 0.10),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });

    let container_blue = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.34, 0.68),
        perceptual_roughness: 0.45,
        metallic: 0.35,
        ..default()
    });

    let container_red = materials.add(StandardMaterial {
        base_color: Color::srgb(0.74, 0.18, 0.14),
        perceptual_roughness: 0.45,
        metallic: 0.35,
        ..default()
    });

    let container_olive = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.34, 0.24),
        perceptual_roughness: 0.55,
        metallic: 0.25,
        ..default()
    });

    let sandbag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.60, 0.46),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    let hedge_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.42, 0.20),
        perceptual_roughness: 0.92,
        metallic: 0.0,
        ..default()
    });

    let drum_green = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.34, 0.20),
        perceptual_roughness: 0.4,
        metallic: 0.65,
        ..default()
    });

    let drum_brown = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.26, 0.16),
        perceptual_roughness: 0.5,
        metallic: 0.55,
        ..default()
    });

    let drum_red = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.22, 0.12),
        perceptual_roughness: 0.4,
        metallic: 0.6,
        ..default()
    });

    let crate_wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.46, 0.34, 0.22),
        perceptual_roughness: 0.85,
        metallic: 0.05,
        ..default()
    });

    let blue_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.7, 1.0),
        emissive: LinearRgba::new(0.0, 4.0, 8.0, 1.0),
        unlit: true,
        ..default()
    });

    let red_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.25, 0.05),
        emissive: LinearRgba::new(8.0, 1.5, 0.2, 1.0),
        unlit: true,
        ..default()
    });

    let depot_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.85, 0.75),
        emissive: LinearRgba::new(0.0, 3.5, 3.0, 1.0),
        unlit: true,
        ..default()
    });

    // 2. Concrete Road / Tarmac Ground (76m x 60m Compound)
    let ground_mesh = meshes.add(Cuboid::new(76.0, 1.0, 60.0));
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(tarmac_mat),
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(76.0, 1.0, 60.0),
    ));

    // Road Markings: Center line and perimeter cross lanes
    let center_line = meshes.add(Cuboid::new(0.35, 0.02, 56.0));
    commands.spawn((
        Mesh3d(center_line),
        MeshMaterial3d(marking_white.clone()),
        Transform::from_xyz(0.0, 0.01, 0.0),
    ));

    let transverse_line = meshes.add(Cuboid::new(70.0, 0.02, 0.35));
    commands.spawn((
        Mesh3d(transverse_line),
        MeshMaterial3d(marking_yellow.clone()),
        Transform::from_xyz(0.0, 0.01, 0.0),
    ));

    // Boundary Curbs (perimeter edges)
    spawn_curb(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(0.0, 0.1, -29.8),
        Vec3::new(76.0, 0.2, 0.4),
    );
    spawn_curb(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(0.0, 0.1, 29.8),
        Vec3::new(76.0, 0.2, 0.4),
    );
    spawn_curb(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(-37.8, 0.1, 0.0),
        Vec3::new(0.4, 0.2, 60.0),
    );
    spawn_curb(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(37.8, 0.1, 0.0),
        Vec3::new(0.4, 0.2, 60.0),
    );

    // 3. Chain-Link Perimeter Fences with Colliders
    spawn_perimeter_fence(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        Vec3::new(0.0, 1.8, -30.0),
        76.0,
        true,
    );
    spawn_perimeter_fence(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        Vec3::new(0.0, 1.8, 30.0),
        76.0,
        true,
    );
    spawn_perimeter_fence(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        Vec3::new(-38.0, 1.8, 0.0),
        60.0,
        false,
    );
    spawn_perimeter_fence(
        &mut commands,
        &mut meshes,
        &fence_mat,
        &steel_mat,
        Vec3::new(38.0, 1.8, 0.0),
        60.0,
        false,
    );

    // 4. Corner Floodlight Towers
    for (cx, cz) in [
        (-36.5, -28.5),
        (36.5, -28.5),
        (-36.5, 28.5),
        (36.5, 28.5),
    ] {
        spawn_floodlight_tower(
            &mut commands,
            &mut meshes,
            &steel_mat,
            Vec3::new(cx, 0.0, cz),
        );
    }

    // 5. Zone Alpha: Central Depot (center: 0, 0.8, 0)
    let (alpha_idx, alpha_name, alpha_center, alpha_radius) = PLATEAU_COORDS[0];
    let depot_mesh = meshes.add(Cuboid::new(13.0, 0.8, 13.0));
    commands.spawn((
        Mesh3d(depot_mesh),
        MeshMaterial3d(concrete_mat.clone()),
        Transform::from_xyz(alpha_center.x, alpha_center.y / 2.0, alpha_center.z),
        RigidBody::Static,
        Collider::cuboid(13.0, 0.8, 13.0),
        Plateau {
            index: alpha_idx,
            name: alpha_name,
            center: alpha_center,
            radius: alpha_radius,
        },
    ));

    // Central Depot Capture Ring
    let depot_ring_mesh = meshes.add(Torus::new(5.0, 0.12));
    commands.spawn((
        Mesh3d(depot_ring_mesh),
        MeshMaterial3d(depot_glow),
        Transform::from_xyz(alpha_center.x, alpha_center.y + 0.02, alpha_center.z),
    ));

    // Low entry ramps to Central Depot
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(0.0, 0.4, -7.5),
        Vec3::new(4.5, 0.4, 3.0),
        -0.26,
        Vec3::X,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(0.0, 0.4, 7.5),
        Vec3::new(4.5, 0.4, 3.0),
        0.26,
        Vec3::X,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(-7.5, 0.4, 0.0),
        Vec3::new(3.0, 0.4, 4.5),
        0.26,
        Vec3::Z,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(7.5, 0.4, 0.0),
        Vec3::new(3.0, 0.4, 4.5),
        -0.26,
        Vec3::Z,
    );

    // 6. Zone Beta: West Blue Platform (center: -24, 2.0, 0)
    let (beta_idx, beta_name, beta_center, beta_radius) = PLATEAU_COORDS[1];
    let west_plat_mesh = meshes.add(Cuboid::new(12.0, 2.0, 12.0));
    commands.spawn((
        Mesh3d(west_plat_mesh),
        MeshMaterial3d(concrete_mat.clone()),
        Transform::from_xyz(beta_center.x, 1.0, beta_center.z),
        RigidBody::Static,
        Collider::cuboid(12.0, 2.0, 12.0),
        Plateau {
            index: beta_idx,
            name: beta_name,
            center: beta_center,
            radius: beta_radius,
        },
    ));

    // Glowing Blue Capture Ring
    let west_ring_mesh = meshes.add(Torus::new(4.6, 0.14));
    commands.spawn((
        Mesh3d(west_ring_mesh),
        MeshMaterial3d(blue_glow.clone()),
        Transform::from_xyz(beta_center.x, beta_center.y + 0.02, beta_center.z),
    ));

    // Blue Team Flagpole and Banner
    spawn_flag(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &blue_glow,
        Vec3::new(-24.0, 2.0, -4.8),
        1.0,
    );

    // Access stairs/ramps to West Platform
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(-15.5, 1.0, 0.0),
        Vec3::new(5.5, 0.5, 4.5),
        -0.36,
        Vec3::Z,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(-24.0, 1.0, 8.5),
        Vec3::new(4.5, 0.5, 5.5),
        0.36,
        Vec3::X,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(-24.0, 1.0, -8.5),
        Vec3::new(4.5, 0.5, 5.5),
        -0.36,
        Vec3::X,
    );

    // 7. Zone Gamma: East Red Platform (center: 24, 2.0, 0)
    let (gamma_idx, gamma_name, gamma_center, gamma_radius) = PLATEAU_COORDS[2];
    let east_plat_mesh = meshes.add(Cuboid::new(12.0, 2.0, 12.0));
    commands.spawn((
        Mesh3d(east_plat_mesh),
        MeshMaterial3d(concrete_mat.clone()),
        Transform::from_xyz(gamma_center.x, 1.0, gamma_center.z),
        RigidBody::Static,
        Collider::cuboid(12.0, 2.0, 12.0),
        Plateau {
            index: gamma_idx,
            name: gamma_name,
            center: gamma_center,
            radius: gamma_radius,
        },
    ));

    // Glowing Red Capture Ring
    let east_ring_mesh = meshes.add(Torus::new(4.6, 0.14));
    commands.spawn((
        Mesh3d(east_ring_mesh),
        MeshMaterial3d(red_glow.clone()),
        Transform::from_xyz(gamma_center.x, gamma_center.y + 0.02, gamma_center.z),
    ));

    // Red Team Flagpole and Banner
    spawn_flag(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &red_glow,
        Vec3::new(24.0, 2.0, -4.8),
        -1.0,
    );

    // Access stairs/ramps to East Platform
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(15.5, 1.0, 0.0),
        Vec3::new(5.5, 0.5, 4.5),
        0.36,
        Vec3::Z,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(24.0, 1.0, 8.5),
        Vec3::new(4.5, 0.5, 5.5),
        0.36,
        Vec3::X,
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &concrete_mat,
        Vec3::new(24.0, 1.0, -8.5),
        Vec3::new(4.5, 0.5, 5.5),
        -0.36,
        Vec3::X,
    );

    // 8. Red and Blue Shipping Containers (Chokepoints and LoS Blockers)
    // Blue Side Containers
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_blue,
        Vec3::new(-11.5, 1.3, 11.5),
        0.18,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_blue,
        Vec3::new(-11.5, 1.3, -11.5),
        -0.18,
    );
    // Stacked blue container
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_blue,
        Vec3::new(-11.5, 3.9, -11.5),
        -0.18,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_blue,
        Vec3::new(-28.0, 1.3, 18.0),
        1.57,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_blue,
        Vec3::new(-28.0, 1.3, -18.0),
        1.57,
    );

    // Red Side Containers
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_red,
        Vec3::new(11.5, 1.3, 11.5),
        -0.18,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_red,
        Vec3::new(11.5, 1.3, -11.5),
        0.18,
    );
    // Stacked red container
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_red,
        Vec3::new(11.5, 3.9, 11.5),
        -0.18,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_red,
        Vec3::new(28.0, 1.3, 18.0),
        1.57,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_red,
        Vec3::new(28.0, 1.3, -18.0),
        1.57,
    );

    // Center Alley & Flank Tactical Containers
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_olive,
        Vec3::new(-5.5, 1.3, -20.5),
        0.0,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_olive,
        Vec3::new(5.5, 1.3, 20.5),
        0.0,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &container_olive,
        Vec3::new(0.0, 1.3, -25.0),
        1.57,
    );

    // Chokepoint LoS Blockers between Center Depot and Platforms
    spawn_container(
        &mut commands,
        &mut meshes,
        &steel_mat,
        Vec3::new(-17.0, 1.3, 0.0),
        1.57,
    );
    spawn_container(
        &mut commands,
        &mut meshes,
        &steel_mat,
        Vec3::new(17.0, 1.3, 0.0),
        1.57,
    );

    // 9. Concrete Jersey Barriers (Trapezoidal cover)
    let jersey_positions = [
        (Vec3::new(-8.8, 0.42, 6.5), 0.0),
        (Vec3::new(-8.8, 0.42, -6.5), 0.0),
        (Vec3::new(8.8, 0.42, 6.5), 0.0),
        (Vec3::new(8.8, 0.42, -6.5), 0.0),
        (Vec3::new(0.0, 0.42, 10.5), 1.57),
        (Vec3::new(0.0, 0.42, -10.5), 1.57),
        (Vec3::new(-19.0, 0.42, 12.0), 0.3),
        (Vec3::new(-19.0, 0.42, -12.0), -0.3),
        (Vec3::new(19.0, 0.42, 12.0), -0.3),
        (Vec3::new(19.0, 0.42, -12.0), 0.3),
    ];
    for (pos, yaw) in jersey_positions {
        spawn_jersey_barrier(&mut commands, &mut meshes, &concrete_mat, pos, yaw);
    }

    // 10. Hazard-Striped Barricades
    let barricade_positions = [
        Vec3::new(-3.5, 0.45, 14.5),
        Vec3::new(3.5, 0.45, -14.5),
        Vec3::new(-15.0, 0.45, 18.0),
        Vec3::new(15.0, 0.45, -18.0),
    ];
    for pos in barricade_positions {
        spawn_hazard_barricade(
            &mut commands,
            &mut meshes,
            &marking_yellow,
            &steel_mat,
            pos,
        );
    }

    // 11. Sandbag Walls & Defensive Stacks
    // Parapets on West (Blue) Platform
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-29.2, 2.38, 2.5),
        1.57,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-29.2, 2.38, -2.5),
        1.57,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-24.0, 2.38, 5.2),
        0.0,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-24.0, 2.38, -5.2),
        0.0,
    );

    // Parapets on East (Red) Platform
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(29.2, 2.38, 2.5),
        1.57,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(29.2, 2.38, -2.5),
        1.57,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(24.0, 2.38, 5.2),
        0.0,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(24.0, 2.38, -5.2),
        0.0,
    );

    // Ground Sandbag Bunker Positions
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(-3.0, 0.38, 0.0),
        1.57,
    );
    spawn_sandbag_wall(
        &mut commands,
        &mut meshes,
        &sandbag_mat,
        Vec3::new(3.0, 0.38, 0.0),
        1.57,
    );

    // 12. Green Hedge Bushes & Cover Blocks
    let hedge_positions = [
        Vec3::new(-34.0, 0.7, 10.0),
        Vec3::new(-34.0, 0.7, -10.0),
        Vec3::new(34.0, 0.7, 10.0),
        Vec3::new(34.0, 0.7, -10.0),
        Vec3::new(-18.0, 0.7, 26.0),
        Vec3::new(18.0, 0.7, 26.0),
        Vec3::new(-18.0, 0.7, -26.0),
        Vec3::new(18.0, 0.7, -26.0),
        Vec3::new(0.0, 0.7, 26.0),
        Vec3::new(0.0, 0.7, -26.0),
    ];
    for pos in hedge_positions {
        spawn_hedge(&mut commands, &mut meshes, &hedge_mat, pos);
    }

    // Wooden Crates / Supply Blocks
    let crate_positions = [
        Vec3::new(-14.0, 0.6, 13.5),
        Vec3::new(14.0, 0.6, -13.5),
        Vec3::new(-22.0, 0.6, -15.0),
        Vec3::new(22.0, 0.6, 15.0),
    ];
    for pos in crate_positions {
        spawn_crate(&mut commands, &mut meshes, &crate_wood, pos);
    }

    // 13. Industrial Barrels / Oil Drums (Green, Brown, Red clusters)
    spawn_barrel_cluster(
        &mut commands,
        &mut meshes,
        &drum_green,
        &drum_brown,
        Vec3::new(-14.5, 0.55, 9.5),
    );
    spawn_barrel_cluster(
        &mut commands,
        &mut meshes,
        &drum_brown,
        &drum_red,
        Vec3::new(14.5, 0.55, -9.5),
    );
    spawn_barrel_cluster(
        &mut commands,
        &mut meshes,
        &drum_red,
        &drum_green,
        Vec3::new(-22.0, 0.55, 16.5),
    );
    spawn_barrel_cluster(
        &mut commands,
        &mut meshes,
        &drum_green,
        &drum_brown,
        Vec3::new(22.0, 0.55, -16.5),
    );
    spawn_barrel_cluster(
        &mut commands,
        &mut meshes,
        &drum_red,
        &drum_brown,
        Vec3::new(0.0, 0.55, 12.5),
    );
}

fn spawn_curb(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    pos: Vec3,
    size: Vec3,
) {
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(pos),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_perimeter_fence(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    fence_mat: &Handle<StandardMaterial>,
    post_mat: &Handle<StandardMaterial>,
    center: Vec3,
    length: f32,
    is_x_aligned: bool,
) {
    let height = 3.6;
    let collider_thickness = 0.4;

    if is_x_aligned {
        // Wire mesh panel
        let mesh = meshes.add(Cuboid::new(length, height, 0.08));
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(fence_mat.clone()),
            Transform::from_translation(center),
            RigidBody::Static,
            Collider::cuboid(length, height + 0.5, collider_thickness),
        ));

        // Steel Posts spaced every 6m
        let num_posts = (length / 6.0) as i32;
        let half = length / 2.0;
        let post_mesh = meshes.add(Cylinder::new(0.08, height + 0.2));
        for i in 0..=num_posts {
            let px = -half + i as f32 * 6.0;
            commands.spawn((
                Mesh3d(post_mesh.clone()),
                MeshMaterial3d(post_mat.clone()),
                Transform::from_xyz(center.x + px, center.y, center.z),
            ));
        }
    } else {
        // Wire mesh panel along Z
        let mesh = meshes.add(Cuboid::new(0.08, height, length));
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(fence_mat.clone()),
            Transform::from_translation(center),
            RigidBody::Static,
            Collider::cuboid(collider_thickness, height + 0.5, length),
        ));

        let num_posts = (length / 6.0) as i32;
        let half = length / 2.0;
        let post_mesh = meshes.add(Cylinder::new(0.08, height + 0.2));
        for i in 0..=num_posts {
            let pz = -half + i as f32 * 6.0;
            commands.spawn((
                Mesh3d(post_mesh.clone()),
                MeshMaterial3d(post_mat.clone()),
                Transform::from_xyz(center.x, center.y, center.z + pz),
            ));
        }
    }
}

fn spawn_floodlight_tower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    base_pos: Vec3,
) {
    // Tall steel pole
    let pole_mesh = meshes.add(Cylinder::new(0.2, 9.0));
    commands.spawn((
        Mesh3d(pole_mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_xyz(base_pos.x, 4.5, base_pos.z),
        RigidBody::Static,
        Collider::cylinder(0.25, 9.0),
    ));

    // Top crossbar and light head fixtures
    let crossbar_mesh = meshes.add(Cuboid::new(1.8, 0.15, 0.4));
    commands.spawn((
        Mesh3d(crossbar_mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_xyz(base_pos.x, 9.0, base_pos.z),
    ));

    let lamp_mesh = meshes.add(Cuboid::new(0.4, 0.3, 0.4));
    for offset_x in [-0.6, 0.6] {
        commands.spawn((
            Mesh3d(lamp_mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(base_pos.x + offset_x, 8.8, base_pos.z),
        ));
    }
}

fn spawn_flag(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    pole_mat: &Handle<StandardMaterial>,
    flag_mat: &Handle<StandardMaterial>,
    pos: Vec3,
    direction: f32,
) {
    // Steel flagpole
    let pole_mesh = meshes.add(Cylinder::new(0.08, 4.5));
    commands.spawn((
        Mesh3d(pole_mesh),
        MeshMaterial3d(pole_mat.clone()),
        Transform::from_xyz(pos.x, pos.y + 2.25, pos.z),
        RigidBody::Static,
        Collider::cylinder(0.08, 4.5),
    ));

    // Team flag banner
    let banner_mesh = meshes.add(Cuboid::new(1.6, 0.9, 0.04));
    commands.spawn((
        Mesh3d(banner_mesh),
        MeshMaterial3d(flag_mat.clone()),
        Transform::from_xyz(pos.x + direction * 0.9, pos.y + 3.8, pos.z),
    ));
}

fn spawn_container(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
    yaw: f32,
) {
    let size = Vec3::new(6.2, 2.6, 2.5);
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    let rotation = Quat::from_axis_angle(Vec3::Y, yaw);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_jersey_barrier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
    yaw: f32,
) {
    let size = Vec3::new(2.4, 0.85, 0.65);
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    let rotation = Quat::from_axis_angle(Vec3::Y, yaw);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_hazard_barricade(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    board_mat: &Handle<StandardMaterial>,
    post_mat: &Handle<StandardMaterial>,
    position: Vec3,
) {
    // Crossbeam
    let board_mesh = meshes.add(Cuboid::new(2.2, 0.45, 0.1));
    commands.spawn((
        Mesh3d(board_mesh),
        MeshMaterial3d(board_mat.clone()),
        Transform::from_xyz(position.x, position.y + 0.25, position.z),
        RigidBody::Static,
        Collider::cuboid(2.2, 0.9, 0.4),
    ));

    // A-frame support legs
    let leg_mesh = meshes.add(Cuboid::new(0.08, 0.9, 0.4));
    commands.spawn((
        Mesh3d(leg_mesh.clone()),
        MeshMaterial3d(post_mat.clone()),
        Transform::from_xyz(position.x - 0.95, position.y, position.z),
    ));
    commands.spawn((
        Mesh3d(leg_mesh),
        MeshMaterial3d(post_mat.clone()),
        Transform::from_xyz(position.x + 0.95, position.y, position.z),
    ));
}

fn spawn_sandbag_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
    yaw: f32,
) {
    let size = Vec3::new(2.6, 0.75, 0.7);
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    let rotation = Quat::from_axis_angle(Vec3::Y, yaw);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_hedge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
) {
    let size = Vec3::new(2.8, 1.4, 1.3);
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_crate(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
) {
    let size = Vec3::new(1.2, 1.2, 1.2);
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn spawn_barrel_cluster(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mat1: &Handle<StandardMaterial>,
    mat2: &Handle<StandardMaterial>,
    center: Vec3,
) {
    let barrel_mesh = meshes.add(Cylinder::new(0.4, 1.05));

    commands.spawn((
        Mesh3d(barrel_mesh.clone()),
        MeshMaterial3d(mat1.clone()),
        Transform::from_xyz(center.x - 0.42, center.y, center.z - 0.25),
        RigidBody::Static,
        Collider::cylinder(0.4, 1.05),
    ));

    commands.spawn((
        Mesh3d(barrel_mesh.clone()),
        MeshMaterial3d(mat2.clone()),
        Transform::from_xyz(center.x + 0.42, center.y, center.z - 0.25),
        RigidBody::Static,
        Collider::cylinder(0.4, 1.05),
    ));

    commands.spawn((
        Mesh3d(barrel_mesh),
        MeshMaterial3d(mat1.clone()),
        Transform::from_xyz(center.x, center.y, center.z + 0.45),
        RigidBody::Static,
        Collider::cylinder(0.4, 1.05),
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
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    let rotation = Quat::from_axis_angle(axis, angle);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(position).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
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
        if transform.translation.y < -5.0
            || transform.translation.x.abs() > 42.0
            || transform.translation.z.abs() > 34.0
        {
            transform.translation = char_info.spawn_point;
            lin_vel.0 = Vec3::ZERO;
            ang_vel.0 = Vec3::ZERO;
            stagger.duration = 0.5;
        }
    }
}
