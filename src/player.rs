use std::time::Duration;

use avian3d::prelude::*;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::camera::OrbitCamera;
use crate::combat::TACKLE_STAMINA_COST;
use crate::koth::KothManager;
use crate::types::{Character, Health, StaggerState, Stamina, TackleAbility, Team};

pub const PLAYER_SPEED: f32 = 8.8;
pub const SPRINT_MULTIPLIER: f32 = 1.55;
pub const JUMP_IMPULSE: f32 = 5.4;
pub const TACKLE_IMPULSE: f32 = 19.5;
pub const TACKLE_COOLDOWN: f32 = 1.35;

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct MainPlayerAnimations {
    pub graph: Handle<AnimationGraph>,
    pub idle_node: AnimationNodeIndex,
    pub run_node: AnimationNodeIndex,
}

#[derive(Component)]
pub struct MainPlayerAnimState {
    pub is_running: bool,
}

#[derive(Component)]
pub struct MainPlayerMeshConfigured;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                handle_player_movement,
                update_stamina,
                update_main_player_animation_and_materials,
            ),
        );
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
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    // Load the embedded Idle (Animation 0) and Walk/Run (Animation 1) clips from main_player.glb
    let idle_clip =
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/main_player.glb"));
    let run_clip =
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/main_player.glb"));
    let (graph, node_indices) = AnimationGraph::from_clips([idle_clip, run_clip]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(MainPlayerAnimations {
        graph: graph_handle,
        idle_node: node_indices[0],
        run_node: node_indices[1],
    });

    // Spawn Anders on the West Blue Platform deck (y = 2.53m) right beside the Blue Flag
    let spawn_pos = Vec3::new(-18.2, 3.6, 3.8);

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
            Stamina {
                regen: 28.0,
                ..default()
            },
            TackleAbility {
                max_cooldown: TACKLE_COOLDOWN,
                ..default()
            },
            StaggerState::default(),
            RigidBody::Dynamic,
            Collider::capsule(0.35, 1.2),
            LockedAxes::ROTATION_LOCKED,
            Friction::new(0.45),
            Restitution::new(0.0),
            LinearVelocity::default(),
            AngularVelocity::default(),
            Visibility::default(),
            Transform::from_translation(spawn_pos)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ))
        .id();

    // Team underfoot selection ring + Rigged 3D GLB Character Model (`models/main_player.glb`)
    // Local GLB bounds: y in [0.000, 1.000], facing +Z (forward along movement axis).
    // Scaling by 1.88 normalizes character height to 1.88m; y = -0.90m places feet on ground plane.
    let ring_mat = materials.add(StandardMaterial {
        base_color: Team::Blue.color(),
        emissive: Team::Blue.emissive(),
        unlit: true,
        ..default()
    });

    commands.entity(player_entity).with_children(|builder| {
        builder.spawn((
            Mesh3d(meshes.add(Torus {
                minor_radius: 0.035,
                major_radius: 0.72,
            })),
            MeshMaterial3d(ring_mat),
            Transform::from_xyz(0.0, -0.90, 0.0),
        ));

        builder.spawn((
            SceneRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/main_player.glb")),
            ),
            Transform::from_xyz(0.0, -0.90, 0.0).with_scale(Vec3::splat(1.88)),
        ));
    });

    // Link orbit camera to follow player
    if let Ok(mut cam) = camera_query.get_single_mut() {
        cam.target = Some(player_entity);
    }
}

fn handle_player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    koth: Res<KothManager>,
    spatial_query: SpatialQuery,
    camera_query: Query<&OrbitCamera>,
    mut jump_cooldown: Local<f32>,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut Stamina,
            &mut TackleAbility,
            &StaggerState,
        ),
        With<Player>,
    >,
) {
    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    let Ok((player_ent, mut transform, mut lin_vel, mut ang_vel, mut stamina, mut tackle, stagger)) =
        player_query.get_single_mut()
    else {
        return;
    };

    // Prevent any physics-induced self-spinning
    ang_vel.0 = Vec3::ZERO;

    let dt = time.delta_secs();
    *jump_cooldown = (*jump_cooldown - dt).max(0.0);

    // Downward raycast from capsule center (0.95m to soles of boots + 0.12m ground margin = 1.07m)
    // combined with exact 3D model terrain height check so jumping works on all slopes and stairs
    let filter = SpatialQueryFilter::from_excluded_entities([player_ent]);
    let terrain_y = crate::terrain_heights::sample_terrain_height(
        transform.translation.x,
        transform.translation.z,
    );
    let is_grounded = spatial_query
        .cast_ray(transform.translation, Dir3::NEG_Y, 1.07, true, &filter)
        .is_some()
        || transform.translation.y <= terrain_y + 1.02;

    // Apply snappy extra downward gravity during descent so jumps feel athletic and responsive
    if !is_grounded && lin_vel.y < 0.0 {
        lin_vel.y -= 6.0 * dt;
    }

    // Stagger blocks input
    if stagger.duration > 0.0 {
        return;
    }

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

    // Fast, responsive tactical running speeds across the 64m x 48m arena
    let is_carrying_flag = koth
        .blue_carrier
        .as_ref()
        .map(|(e, _)| *e == player_ent)
        .unwrap_or(false);
    let carry_mult = if is_carrying_flag { 0.90 } else { 1.0 };

    let is_sprinting = keys.pressed(KeyCode::ShiftLeft) && stamina.current > 5.0 && move_len > 0.1;
    let base_speed = if is_sprinting {
        PLAYER_SPEED * SPRINT_MULTIPLIER
    } else {
        PLAYER_SPEED
    };
    let speed = base_speed * carry_mult;

    if is_sprinting && is_grounded {
        stamina.current = (stamina.current - 14.0 * dt).max(0.0);
    }

    // Grounded traction vs mid-air momentum preservation
    let accel_rate = if is_grounded { 14.0 } else { 3.2 };
    let decel_rate = if is_grounded { 14.0 } else { 0.8 };

    if move_len > 0.05 {
        let move_dir = move_input.normalize();
        let desired_vel = move_dir * speed;
        lin_vel.x = lin_vel.x.lerp(desired_vel.x, (accel_rate * dt).min(1.0));
        lin_vel.z = lin_vel.z.lerp(desired_vel.z, (accel_rate * dt).min(1.0));

        // Smooth uphill slope/staircase climbing so capsules glide up all ramps and stairs without snagging
        if is_grounded {
            let ahead_y = crate::terrain_heights::sample_terrain_height(
                transform.translation.x + move_dir.x * 0.48,
                transform.translation.z + move_dir.z * 0.48,
            );
            let dh = ahead_y - terrain_y;
            if dh > 0.01 && dh < 1.15 {
                let climb_vy = (speed * (dh / 0.48)).clamp(0.0, 12.5);
                if lin_vel.y < climb_vy {
                    lin_vel.y = climb_vy;
                }
                let target_center_y = ahead_y + 0.92;
                if transform.translation.y < target_center_y {
                    transform.translation.y = transform
                        .translation
                        .y
                        .lerp(target_center_y, (18.0 * dt).min(1.0));
                }
            }
        }

        // Turn character towards movement direction around Y axis (visual front is +Z)
        let target_rot = Quat::from_rotation_y(desired_vel.x.atan2(desired_vel.z));
        transform.rotation = transform.rotation.slerp(target_rot, (16.0 * dt).min(1.0));
    } else {
        // Friction dampening on ground; preserve horizontal momentum in air
        lin_vel.x = lin_vel.x.lerp(0.0, (decel_rate * dt).min(1.0));
        lin_vel.z = lin_vel.z.lerp(0.0, (decel_rate * dt).min(1.0));
    }

    // Athletic Jump
    if keys.just_pressed(KeyCode::Space)
        && is_grounded
        && *jump_cooldown <= 0.0
        && stamina.current >= 8.0
    {
        lin_vel.y = JUMP_IMPULSE;
        stamina.current = (stamina.current - 8.0).max(0.0);
        *jump_cooldown = 0.35;
    }

    // Tackle / Kastbart Nett Lunge (LMB or F)
    let tackle_pressed =
        mouse_buttons.just_pressed(MouseButton::Left) || keys.just_pressed(KeyCode::KeyF);
    if tackle_pressed
        && is_grounded
        && tackle.cooldown <= 0.0
        && stamina.current >= TACKLE_STAMINA_COST
    {
        tackle.is_tackling = true;
        tackle.tackle_timer = 0.28;
        tackle.cooldown = TACKLE_COOLDOWN;
        stamina.current -= TACKLE_STAMINA_COST;

        let tackle_dir = transform.rotation * Vec3::Z;
        let lunge_vel = tackle_dir * TACKLE_IMPULSE;
        lin_vel.x = lunge_vel.x;
        lin_vel.z = lunge_vel.z;
        lin_vel.y = 1.25;
    }
}

fn update_stamina(time: Res<Time>, mut stamina_query: Query<&mut Stamina>) {
    let dt = time.delta_secs();
    for mut stamina in stamina_query.iter_mut() {
        if stamina.current < stamina.max {
            let regen_rate = stamina.regen.max(28.0);
            stamina.current = (stamina.current + regen_rate * dt).min(stamina.max);
        }
    }
}

fn update_main_player_animation_and_materials(
    mut commands: Commands,
    anims: Option<Res<MainPlayerAnimations>>,
    player_query: Query<(Entity, &LinearVelocity), With<Player>>,
    children_query: Query<&Children>,
    unconfigured_meshes: Query<Entity, (With<Mesh3d>, Without<MainPlayerMeshConfigured>)>,
    mut anim_players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&mut MainPlayerAnimState>,
    )>,
) {
    let Some(anims) = anims else {
        return;
    };
    let Ok((player_entity, lin_vel)) = player_query.get_single() else {
        return;
    };

    let horiz_speed = Vec2::new(lin_vel.x, lin_vel.z).length();
    let should_run = horiz_speed > 0.55;

    for descendant in children_query.iter_descendants(player_entity) {
        // 1. Shadow & Material Pass: ensure castShadow = true and receiveShadow = true across all child meshes
        // while retaining the embedded PBR material from main_player.glb.
        if unconfigured_meshes.get(descendant).is_ok() {
            commands
                .entity(descendant)
                .remove::<NotShadowCaster>()
                .remove::<NotShadowReceiver>()
                .insert(MainPlayerMeshConfigured);
        }

        // 2. Rigging & Animation Mixer State Machine:
        // Initialize AnimationGraph + AnimationTransitions on the GLB's AnimationPlayer,
        // and crossfade smoothly between Idle and Walk/Run based on WASD horizontal velocity.
        if let Ok((anim_entity, mut anim_player, transitions_opt, state_opt)) =
            anim_players.get_mut(descendant)
        {
            match (transitions_opt, state_opt) {
                (Some(mut transitions), Some(mut state)) => {
                    if should_run != state.is_running {
                        state.is_running = should_run;
                        let target_node = if should_run {
                            anims.run_node
                        } else {
                            anims.idle_node
                        };
                        transitions
                            .play(&mut anim_player, target_node, Duration::from_millis(180))
                            .repeat();
                    }

                    if should_run {
                        if let Some(active_anim) = anim_player.animation_mut(anims.run_node) {
                            let speed_scale = (horiz_speed / 8.5).clamp(0.75, 1.65);
                            active_anim.set_speed(speed_scale);
                        }
                    }
                }
                _ => {
                    let mut transitions = AnimationTransitions::new();
                    let initial_node = if should_run {
                        anims.run_node
                    } else {
                        anims.idle_node
                    };
                    transitions
                        .play(&mut anim_player, initial_node, Duration::ZERO)
                        .repeat();
                    commands.entity(anim_entity).insert((
                        AnimationGraphHandle(anims.graph.clone()),
                        transitions,
                        MainPlayerAnimState {
                            is_running: should_run,
                        },
                    ));
                }
            }
        }
    }
}
