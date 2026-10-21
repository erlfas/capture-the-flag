use avian3d::prelude::*;
use bevy::prelude::*;

use crate::camera::OrbitCamera;
use crate::combat::{TACKLE_COOLDOWN, TACKLE_LUNGE_SPEED, TACKLE_STAMINA_COST};
use crate::types::{Character, Health, StaggerState, Stamina, TackleAbility, Team};

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (handle_player_movement, update_stamina));
    }
}

pub fn spawn_character_visuals(
    commands: &mut Commands,
    parent: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    team: Team,
    character_name: &str,
) {
    // 1. Team-specific and Tactical Materials
    let jumpsuit_mat = materials.add(StandardMaterial {
        base_color: match team {
            Team::Blue => Color::srgb(0.13, 0.28, 0.58), // Tactical Navy/Blue
            Team::Red => Color::srgb(0.75, 0.14, 0.14),  // Tactical Crimson Red
        },
        metallic: 0.1,
        perceptual_roughness: 0.7,
        ..default()
    });

    let plate_carrier_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.14, 0.18), // Dark tactical plate carrier
        metallic: 0.2,
        perceptual_roughness: 0.6,
        ..default()
    });

    let gear_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.11), // Pouches, webbing, belts
        metallic: 0.05,
        perceptual_roughness: 0.8,
        ..default()
    });

    let boots_and_pads_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.07, 0.08), // Reinforced combat boots and knee pads
        metallic: 0.3,
        perceptual_roughness: 0.45,
        ..default()
    });

    let skin_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.70, 0.58), // Natural human skin tone
        metallic: 0.0,
        perceptual_roughness: 0.75,
        ..default()
    });

    let brown_hair_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.15, 0.09), // Dark brown hair
        metallic: 0.05,
        perceptual_roughness: 0.85,
        ..default()
    });

    let blonde_hair_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.76, 0.38), // Lisa blonde hair
        metallic: 0.05,
        perceptual_roughness: 0.8,
        ..default()
    });

    let black_frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.03, 0.03, 0.03), // Glasses & headset
        metallic: 0.4,
        perceptual_roughness: 0.3,
        ..default()
    });

    let ring_mat = materials.add(StandardMaterial {
        base_color: team.color(),
        emissive: team.emissive(),
        unlit: true,
        ..default()
    });

    commands.entity(parent).with_children(|builder| {
        // --- Underfoot Ground Selection Ring (Matching art reference!) ---
        builder.spawn((
            Mesh3d(meshes.add(Torus {
                minor_radius: 0.035,
                major_radius: 0.72,
            })),
            MeshMaterial3d(ring_mat.clone()),
            Transform::from_xyz(0.0, -0.92, 0.0),
        ));

        // --- Combat Boots (Black reinforced tactical footwear) ---
        // Left foot
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.16, 0.20, 0.32))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(-0.16, -0.80, 0.05),
        ));
        // Right foot
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.16, 0.20, 0.32))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(0.16, -0.80, 0.05),
        ));

        // Boot ankles
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.10, 0.16))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(-0.16, -0.68, 0.0),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.10, 0.16))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(0.16, -0.68, 0.0),
        ));

        // --- Lower Legs (Tactical Jumpsuit Pants) ---
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.10, 0.32))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(-0.16, -0.52, 0.0),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.10, 0.32))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(0.16, -0.52, 0.0),
        ));

        // --- Tactical Knee Pads ---
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 0.08))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(-0.16, -0.42, 0.11),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 0.08))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(0.16, -0.42, 0.11),
        ));

        // --- Upper Legs / Thighs ---
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.12, 0.38))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(-0.16, -0.22, 0.0),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.12, 0.38))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(0.16, -0.22, 0.0),
        ));

        // --- Pelvis & Tactical Utility Belt ---
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.48, 0.20, 0.32))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(0.0, -0.02, 0.0),
        ));
        // Tactical utility belt
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.50, 0.08, 0.34))),
            MeshMaterial3d(gear_mat.clone()),
            Transform::from_xyz(0.0, 0.08, 0.0),
        ));
        // Side utility pouch
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.10, 0.09, 0.06))),
            MeshMaterial3d(gear_mat.clone()),
            Transform::from_xyz(0.24, 0.08, 0.0),
        ));

        // --- Torso Jumpsuit Base ---
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.22, 0.42))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(0.0, 0.34, 0.0),
        ));

        // --- Tactical Plate Carrier Vest (Dark tactical plate vest) ---
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.52, 0.44, 0.34))),
            MeshMaterial3d(plate_carrier_mat.clone()),
            Transform::from_xyz(0.0, 0.38, 0.0),
        ));

        // Front Chest Ammo / Mag Pouches (3 pouches)
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.11, 0.15, 0.06))),
            MeshMaterial3d(gear_mat.clone()),
            Transform::from_xyz(-0.13, 0.32, 0.18),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.11, 0.15, 0.06))),
            MeshMaterial3d(gear_mat.clone()),
            Transform::from_xyz(0.0, 0.32, 0.18),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.11, 0.15, 0.06))),
            MeshMaterial3d(gear_mat.clone()),
            Transform::from_xyz(0.13, 0.32, 0.18),
        ));

        // Chest tactical team indicator stripe
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.24, 0.04, 0.03))),
            MeshMaterial3d(ring_mat.clone()),
            Transform::from_xyz(0.0, 0.48, 0.18),
        ));

        // Vest shoulder straps
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.10, 0.14, 0.30))),
            MeshMaterial3d(plate_carrier_mat.clone()),
            Transform::from_xyz(-0.20, 0.58, 0.0),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.10, 0.14, 0.30))),
            MeshMaterial3d(plate_carrier_mat.clone()),
            Transform::from_xyz(0.20, 0.58, 0.0),
        ));

        // --- Arms: Sleeves, Forearms, and Hands ---
        // Upper sleeves (jumpsuit)
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.09, 0.26))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(-0.35, 0.38, 0.0),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.09, 0.26))),
            MeshMaterial3d(jumpsuit_mat.clone()),
            Transform::from_xyz(0.35, 0.38, 0.0),
        ));

        // Forearms (exposed athletic skin)
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.08, 0.26))),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(-0.36, 0.12, 0.04),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.08, 0.26))),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(0.36, 0.12, 0.04),
        ));

        // Left tactical wristwatch
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.085, 0.04))),
            MeshMaterial3d(black_frame_mat.clone()),
            Transform::from_xyz(-0.36, 0.04, 0.04),
        ));

        // Hands / Tactical Gloves
        builder.spawn((
            Mesh3d(meshes.add(Sphere::new(0.08))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(-0.36, -0.06, 0.05),
        ));
        builder.spawn((
            Mesh3d(meshes.add(Sphere::new(0.08))),
            MeshMaterial3d(boots_and_pads_mat.clone()),
            Transform::from_xyz(0.36, -0.06, 0.05),
        ));

        // --- Neck & Head (Human facial base) ---
        // Neck
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.09, 0.14))),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(0.0, 0.65, 0.0),
        ));

        // Head
        builder.spawn((
            Mesh3d(meshes.add(Sphere::new(0.20))),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(0.0, 0.82, 0.0),
        ));

        // Stylized Nose
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.04, 0.06, 0.05))),
            MeshMaterial3d(skin_mat.clone()),
            Transform::from_xyz(0.0, 0.82, 0.20),
        ));

        // --- Tactical Radio Headset (Present on tactical operators) ---
        // Headset headband over skull
        builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.42, 0.03, 0.04))),
            MeshMaterial3d(black_frame_mat.clone()),
            Transform::from_xyz(0.0, 0.98, 0.0),
        ));
        // Left earcup
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.07, 0.05))),
            MeshMaterial3d(black_frame_mat.clone()),
            Transform::from_xyz(-0.20, 0.82, 0.0)
                .with_rotation(Quat::from_axis_angle(Vec3::Z, 1.57)),
        ));
        // Right earcup
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.07, 0.05))),
            MeshMaterial3d(black_frame_mat.clone()),
            Transform::from_xyz(0.20, 0.82, 0.0)
                .with_rotation(Quat::from_axis_angle(Vec3::Z, 1.57)),
        ));
        // Boom microphone extending from left ear towards mouth
        builder.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.012, 0.16))),
            MeshMaterial3d(black_frame_mat.clone()),
            Transform::from_xyz(-0.15, 0.77, 0.12)
                .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.5, 0.3, 0.0)),
        ));
        // Mic tip
        builder.spawn((
            Mesh3d(meshes.add(Sphere::new(0.024))),
            MeshMaterial3d(black_frame_mat.clone()),
            Transform::from_xyz(-0.06, 0.76, 0.19),
        ));

        // --- Character-Specific Human Features (Hair, Glasses, Beards, Ponytails) ---
        let hair_mat = if character_name.eq_ignore_ascii_case("lisa") {
            blonde_hair_mat
        } else {
            brown_hair_mat.clone()
        };

        if character_name.eq_ignore_ascii_case("anders") {
            // Anders: Brown hair silhouette + Black frame glasses + stubble
            // Hair top & sides
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.38, 0.12, 0.38))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.94, -0.02),
            ));
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.39, 0.10, 0.22))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.87, -0.06),
            ));

            // Glasses: Distinct black rectangular frames around eyes
            // Frame front
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.32, 0.08, 0.03))),
                MeshMaterial3d(black_frame_mat.clone()),
                Transform::from_xyz(0.0, 0.84, 0.205),
            ));
            // Bridge
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.06, 0.03, 0.03))),
                MeshMaterial3d(black_frame_mat.clone()),
                Transform::from_xyz(0.0, 0.84, 0.21),
            ));
            // Glasses side arms
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.02, 0.025, 0.18))),
                MeshMaterial3d(black_frame_mat.clone()),
                Transform::from_xyz(-0.16, 0.84, 0.10),
            ));
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.02, 0.025, 0.18))),
                MeshMaterial3d(black_frame_mat.clone()),
                Transform::from_xyz(0.16, 0.84, 0.10),
            ));

            // Stubble / light beard along jaw
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.24, 0.08, 0.16))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.35, 0.22, 0.16),
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.73, 0.06),
            ));
        } else if character_name.eq_ignore_ascii_case("morten") {
            // Morten: Brown hair + Full facial beard
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.38, 0.12, 0.36))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.93, -0.02),
            ));
            // Prominent Full Beard around jaw & chin
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.28, 0.14, 0.22))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.73, 0.06),
            ));
            // Mustache
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.16, 0.04, 0.05))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.78, 0.18),
            ));
        } else if character_name.eq_ignore_ascii_case("siri") {
            // Siri: Dark brown hair with dynamic ponytail
            builder.spawn((
                Mesh3d(meshes.add(Sphere::new(0.21))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.86, -0.02),
            ));
            // Hair tie
            builder.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.06, 0.03))),
                MeshMaterial3d(gear_mat.clone()),
                Transform::from_xyz(0.0, 0.88, -0.20)
                    .with_rotation(Quat::from_axis_angle(Vec3::X, 0.8)),
            ));
            // Ponytail flowing back
            builder.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.06, 0.28))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.78, -0.28)
                    .with_rotation(Quat::from_axis_angle(Vec3::X, 0.7)),
            ));
        } else if character_name.eq_ignore_ascii_case("lisa") {
            // Lisa: Blonde hair with ponytail
            builder.spawn((
                Mesh3d(meshes.add(Sphere::new(0.21))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.86, -0.02),
            ));
            // Ponytail flowing back
            builder.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.06, 0.28))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.78, -0.28)
                    .with_rotation(Quat::from_axis_angle(Vec3::X, 0.7)),
            ));
        } else {
            // Default operator short cropped hair
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.38, 0.12, 0.36))),
                MeshMaterial3d(hair_mat.clone()),
                Transform::from_xyz(0.0, 0.93, -0.02),
            ));
            // Subtle tactical jawline / stubble
            builder.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.24, 0.08, 0.18))),
                MeshMaterial3d(hair_mat),
                Transform::from_xyz(0.0, 0.73, 0.06),
            ));
        }
    });
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    let spawn_pos = Vec3::new(0.0, 2.0, 28.0);

    let player_entity = commands
        .spawn((
            Player,
            Character {
                team: Team::Blue,
                name: "Anders".to_string(),
                is_player: true,
                spawn_point: spawn_pos,
            },
            Health::default(),
            Stamina::default(),
            TackleAbility {
                max_cooldown: TACKLE_COOLDOWN,
                ..default()
            },
            StaggerState::default(),
            RigidBody::Dynamic,
            Collider::capsule(0.35, 1.2),
            LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            Friction::new(0.4),
            Restitution::new(0.05),
            LinearVelocity::default(),
            AngularVelocity::default(),
            Transform::from_translation(spawn_pos),
        ))
        .id();

    // Attach humanoid visuals matching Anders reference art
    spawn_character_visuals(
        &mut commands,
        player_entity,
        &mut meshes,
        &mut materials,
        Team::Blue,
        "Anders",
    );

    // Link orbit camera to follow player
    if let Ok(mut cam) = camera_query.get_single_mut() {
        cam.target = Some(player_entity);
        cam.yaw = 3.14159; // Face towards arena center
    }
}

fn handle_player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    camera_query: Query<&OrbitCamera>,
    mut player_query: Query<(
        &mut Transform,
        &mut LinearVelocity,
        &mut Stamina,
        &mut TackleAbility,
        &StaggerState,
    ), With<Player>>,
) {
    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    let Ok((mut transform, mut lin_vel, mut stamina, mut tackle, stagger)) =
        player_query.get_single_mut()
    else {
        return;
    };

    // Stagger blocks input
    if stagger.duration > 0.0 {
        return;
    }

    let dt = time.delta_secs();

    // Direction vectors relative to camera yaw
    let forward_dir = Quat::from_axis_angle(Vec3::Y, camera.yaw) * Vec3::NEG_Z;
    let right_dir = Quat::from_axis_angle(Vec3::Y, camera.yaw) * Vec3::X;

    let mut move_input = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        move_input += forward_dir;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_input -= forward_dir;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_input += right_dir;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_input -= right_dir;
    }

    move_input.y = 0.0;
    let move_len = move_input.length();

    // Sprinting logic
    let is_sprinting = keys.pressed(KeyCode::ShiftLeft) && stamina.current > 5.0 && move_len > 0.1;
    let speed = if is_sprinting { 11.5 } else { 7.0 };

    if is_sprinting {
        stamina.current = (stamina.current - 22.0 * dt).max(0.0);
    }

    // Apply horizontal velocity smoothing
    if move_len > 0.05 {
        let desired_vel = move_input.normalize() * speed;
        lin_vel.x = lin_vel.x.lerp(desired_vel.x, 14.0 * dt);
        lin_vel.z = lin_vel.z.lerp(desired_vel.z, 14.0 * dt);

        // Turn character towards movement direction
        let target_rot =
            Quat::from_rotation_arc(Vec3::Z, Vec3::new(desired_vel.x, 0.0, desired_vel.z).normalize());
        transform.rotation = transform.rotation.slerp(target_rot, 16.0 * dt);
    } else {
        // Friction dampening
        lin_vel.x = lin_vel.x.lerp(0.0, 10.0 * dt);
        lin_vel.z = lin_vel.z.lerp(0.0, 10.0 * dt);
    }

    // Jump (ground check via low vertical velocity)
    if keys.just_pressed(KeyCode::Space) && lin_vel.y.abs() < 0.35 {
        lin_vel.y = 7.5;
    }

    // Tackle Attack (LMB or F)
    let tackle_pressed =
        mouse_buttons.just_pressed(MouseButton::Left) || keys.just_pressed(KeyCode::KeyF);
    if tackle_pressed && tackle.cooldown <= 0.0 && stamina.current >= TACKLE_STAMINA_COST {
        tackle.is_tackling = true;
        tackle.tackle_timer = 0.28;
        tackle.cooldown = TACKLE_COOLDOWN;
        stamina.current -= TACKLE_STAMINA_COST;

        let tackle_dir = transform.forward().as_vec3();
        lin_vel.0 += tackle_dir * TACKLE_LUNGE_SPEED + Vec3::Y * 2.0;
    }
}

fn update_stamina(time: Res<Time>, mut stamina_query: Query<&mut Stamina>) {
    let dt = time.delta_secs();
    for mut stamina in stamina_query.iter_mut() {
        if stamina.current < stamina.max {
            stamina.current = (stamina.current + stamina.regen * dt).min(stamina.max);
        }
    }
}
