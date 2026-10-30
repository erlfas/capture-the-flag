use std::time::Duration;

use avian3d::prelude::*;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::camera::OrbitCamera;
use crate::combat::TACKLE_STAMINA_COST;
use crate::koth::KothManager;
use crate::types::{Character, Health, PauseState, StaggerState, Stamina, TackleAbility, Team};

pub const PLAYER_SPEED: f32 = 8.8;
pub const SPRINT_MULTIPLIER: f32 = 1.55;
#[allow(dead_code)]
pub const JUMP_IMPULSE: f32 = 5.4;
pub const TACKLE_IMPULSE: f32 = 19.5;
pub const TACKLE_COOLDOWN: f32 = 1.15;

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct CharacterAnimClips {
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
        app.add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    handle_player_movement,
                    update_stamina,
                    update_character_animation_and_materials,
                ),
            )
            .add_systems(
                PostUpdate,
                apply_character_punch_pose
                    .after(bevy::animation::animate_targets)
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

pub fn character_model_info(character_name: &str) -> (&'static str, f32) {
    if character_name.eq_ignore_ascii_case("morten") {
        ("models/morten-blue.glb", 1.92)
    } else if character_name.eq_ignore_ascii_case("jonas") {
        ("models/jonas-blue.glb", 1.92)
    } else if character_name.eq_ignore_ascii_case("siri") {
        ("models/siri-red.glb", 1.88)
    } else if character_name.eq_ignore_ascii_case("lisa") {
        ("models/lisa-red.glb", 1.88)
    } else if character_name.eq_ignore_ascii_case("per") {
        ("models/per-red.glb", 2.08)
    } else {
        ("models/main_player.glb", 1.88)
    }
}

pub fn spawn_character_visuals(
    commands: &mut Commands,
    parent: Entity,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graphs: &mut ResMut<Assets<AnimationGraph>>,
    team: Team,
    character_name: &str,
) {
    let (model_path, model_scale) = character_model_info(character_name);

    // Load the embedded Idle (Animation 0) and Walk/Run (Animation 1) clips from the character's GLB
    let idle_clip = asset_server.load(GltfAssetLabel::Animation(0).from_asset(model_path));
    let run_clip = asset_server.load(GltfAssetLabel::Animation(1).from_asset(model_path));
    let (graph, node_indices) = AnimationGraph::from_clips([idle_clip, run_clip]);
    let graph_handle = graphs.add(graph);

    commands
        .entity(parent)
        .insert((
            Visibility::default(),
            CharacterAnimClips {
                graph: graph_handle,
                idle_node: node_indices[0],
                run_node: node_indices[1],
            },
        ));

    // Team underfoot selection ring + Rigged 3D GLB Character Model
    let ring_mat = materials.add(StandardMaterial {
        base_color: team.color(),
        emissive: team.emissive(),
        unlit: true,
        ..default()
    });

    commands.entity(parent).with_children(|builder| {
        builder.spawn((
            Mesh3d(meshes.add(Torus {
                minor_radius: 0.035,
                major_radius: 0.72,
            })),
            MeshMaterial3d(ring_mat),
            Transform::from_xyz(0.0, -0.90, 0.0),
        ));

        builder.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path))),
            Transform::from_xyz(0.0, -0.90, 0.0).with_scale(Vec3::splat(model_scale)),
        ));
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
        .insert(SweptCcd::default())
        .id();

    spawn_character_visuals(
        &mut commands,
        player_entity,
        &asset_server,
        &mut meshes,
        &mut materials,
        &mut graphs,
        Team::Blue,
        "Anders",
    );

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
    pause_state: Res<PauseState>,
    spatial_query: SpatialQuery,
    camera_query: Query<&OrbitCamera>,
    enemies_query: Query<(&Transform, &Character), Without<Player>>,
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

    if pause_state.is_paused || pause_state.exited {
        return;
    }

    if !koth.match_started {
        lin_vel.x = 0.0;
        lin_vel.z = 0.0;
        return;
    }

    let dt = time.delta_secs();

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

    // Shove / Tackle / Punch Lunge Trigger (Space, LMB, or F) with magnetic target assist
    let space_punch = keys.just_pressed(KeyCode::Space) && pause_state.match_active_secs > 0.15;
    let tackle_pressed = mouse_buttons.just_pressed(MouseButton::Left)
        || keys.just_pressed(KeyCode::KeyF)
        || space_punch;
    if tackle_pressed && tackle.cooldown <= 0.0 && stamina.current >= TACKLE_STAMINA_COST {
        tackle.is_tackling = true;
        tackle.tackle_timer = 0.30;
        tackle.punch_anim_timer = 0.42;
        tackle.cooldown = TACKLE_COOLDOWN;
        stamina.current -= TACKLE_STAMINA_COST;

        // Base shove direction: WASD input direction if moving, otherwise character facing direction
        let base_aim = if move_len > 0.05 {
            move_input.normalize()
        } else {
            (transform.rotation * Vec3::Z).normalize_or(forward_dir)
        };

        // Target magnetism: snap toward the closest Red opponent within 7.0m in front cone
        let mut best_dir = base_aim;
        let mut best_score = f32::MAX;
        for (enemy_trans, enemy_char) in enemies_query.iter() {
            if enemy_char.team == Team::Blue {
                continue;
            }
            let delta = enemy_trans.translation - transform.translation;
            let horiz = Vec3::new(delta.x, 0.0, delta.z);
            let dist = horiz.length();
            if dist > 0.1 && dist < 7.0 && delta.y.abs() < 2.4 {
                let dir_to_enemy = horiz / dist;
                let align = base_aim.dot(dir_to_enemy).max(forward_dir.dot(dir_to_enemy));
                if align > 0.15 {
                    let score = dist - align * 2.5;
                    if score < best_score {
                        best_score = score;
                        best_dir = dir_to_enemy;
                    }
                }
            }
        }

        transform.rotation = Quat::from_rotation_y(best_dir.x.atan2(best_dir.z));
        let lunge_vel = best_dir * TACKLE_IMPULSE;
        lin_vel.x = lunge_vel.x;
        lin_vel.z = lunge_vel.z;
        if is_grounded && lin_vel.y < 1.0 {
            lin_vel.y = 1.0;
        }
    }

    // Sustain forward drive during an active shove/tackle lunge
    if tackle.is_tackling {
        let lunge_dir = (transform.rotation * Vec3::Z).normalize_or(forward_dir);
        let sustain_speed = TACKLE_IMPULSE * 0.88;
        lin_vel.x = lunge_dir.x * sustain_speed;
        lin_vel.z = lunge_dir.z * sustain_speed;

        if is_grounded {
            let ahead_y = crate::terrain_heights::sample_terrain_height(
                transform.translation.x + lunge_dir.x * 0.50,
                transform.translation.z + lunge_dir.z * 0.50,
            );
            let dh = ahead_y - terrain_y;
            if dh > 0.01 && dh < 0.85 {
                lin_vel.y = lin_vel.y.max((sustain_speed * (dh / 0.50)).clamp(0.0, 3.8));
            }
        }
        return;
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
            if dh > 0.01 && dh < 0.85 {
                let climb_vy = (speed * (dh / 0.48)).clamp(0.0, 4.2);
                if lin_vel.y < climb_vy {
                    lin_vel.y = climb_vy;
                }
                let target_center_y = ahead_y + 0.91;
                if transform.translation.y < target_center_y {
                    transform.translation.y = transform
                        .translation
                        .y
                        .lerp(target_center_y, (14.0 * dt).min(1.0));
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
}

fn update_stamina(
    time: Res<Time>,
    pause_state: Res<PauseState>,
    mut stamina_query: Query<&mut Stamina>,
) {
    if pause_state.is_paused || pause_state.exited {
        return;
    }
    let dt = time.delta_secs();
    for mut stamina in stamina_query.iter_mut() {
        if stamina.current < stamina.max {
            let regen_rate = stamina.regen.max(28.0);
            stamina.current = (stamina.current + regen_rate * dt).min(stamina.max);
        }
    }
}

fn update_character_animation_and_materials(
    mut commands: Commands,
    character_query: Query<(Entity, &LinearVelocity, &CharacterAnimClips), With<Character>>,
    children_query: Query<&Children>,
    unconfigured_meshes: Query<Entity, (With<Mesh3d>, Without<MainPlayerMeshConfigured>)>,
    mut anim_players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&mut MainPlayerAnimState>,
    )>,
) {
    for (char_entity, lin_vel, anims) in character_query.iter() {
        let horiz_speed = Vec2::new(lin_vel.x, lin_vel.z).length();
        let should_run = horiz_speed > 0.55;

        for descendant in children_query.iter_descendants(char_entity) {
            // 1. Shadow & Material Pass: ensure castShadow = true and receiveShadow = true across all child meshes
            // while retaining the embedded PBR material from the character's GLB.
            if unconfigured_meshes.get(descendant).is_ok() {
                commands
                    .entity(descendant)
                    .remove::<NotShadowCaster>()
                    .remove::<NotShadowReceiver>()
                    .insert(MainPlayerMeshConfigured);
            }

            // 2. Rigging & Animation Mixer State Machine:
            // Initialize AnimationGraph + AnimationTransitions on the GLB's AnimationPlayer,
            // and crossfade smoothly between Idle and Walk/Run based on horizontal velocity.
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
}

fn punch_envelope(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    if u < 0.18 {
        let t = u / 0.18;
        t * t * (3.0 - 2.0 * t)
    } else if u < 0.52 {
        1.0
    } else {
        let t = (u - 0.52) / 0.48;
        1.0 - t * t * (3.0 - 2.0 * t)
    }
}

fn model_space_parent_rot(
    bone_ent: Entity,
    char_ent: Entity,
    parent_query: &Query<&Parent>,
    transforms: &Query<&mut Transform>,
) -> Quat {
    let mut chain = Vec::with_capacity(8);
    for anc in parent_query.iter_ancestors(bone_ent) {
        if anc == char_ent {
            break;
        }
        chain.push(anc);
    }
    let mut rot = Quat::IDENTITY;
    for &anc in chain.iter().rev() {
        if let Ok(t) = transforms.get(anc) {
            rot = rot * t.rotation;
        }
    }
    rot
}

fn orient_bone_in_model_space(
    bone_ent: Entity,
    char_ent: Entity,
    target_dir: Vec3,
    weight: f32,
    parent_query: &Query<&Parent>,
    transforms: &mut Query<&mut Transform>,
) {
    if weight <= 0.001 {
        return;
    }
    let parent_model_rot = model_space_parent_rot(bone_ent, char_ent, parent_query, transforms);
    if let Ok(mut t) = transforms.get_mut(bone_ent) {
        let model_rot = parent_model_rot * t.rotation;
        let curr_dir = (model_rot * Vec3::Y).normalize_or(Vec3::Z);
        let delta = Quat::from_rotation_arc(curr_dir, target_dir);
        let new_model_rot = Quat::IDENTITY.slerp(delta, weight) * model_rot;
        t.rotation = (parent_model_rot.inverse() * new_model_rot).normalize();
    }
}

fn apply_character_punch_pose(
    character_query: Query<(Entity, &TackleAbility), With<Character>>,
    children_query: Query<&Children>,
    parent_query: Query<&Parent>,
    name_query: Query<&Name>,
    mut transforms: Query<&mut Transform>,
) {
    for (char_ent, tackle) in character_query.iter() {
        if tackle.punch_anim_timer <= 0.0 {
            continue;
        }

        let u = (1.0 - (tackle.punch_anim_timer / 0.42)).clamp(0.0, 1.0);
        let right_w = punch_envelope(u);
        let left_w = punch_envelope((u * 1.06 - 0.06).clamp(0.0, 1.0));

        let mut right_arm = None;
        let mut left_arm = None;
        let mut right_forearm = None;
        let mut left_forearm = None;

        for desc in children_query.iter_descendants(char_ent) {
            if let Ok(name) = name_query.get(desc) {
                match name.as_str() {
                    "mixamorig:RightArm" => right_arm = Some(desc),
                    "mixamorig:LeftArm" => left_arm = Some(desc),
                    "mixamorig:RightForeArm" => right_forearm = Some(desc),
                    "mixamorig:LeftForeArm" => left_forearm = Some(desc),
                    _ => {}
                }
            }
        }

        // 1. Drive upper arms forward (+Z in model space)
        if let Some(ent) = right_arm {
            orient_bone_in_model_space(
                ent,
                char_ent,
                Vec3::new(-0.08, 0.02, 0.99).normalize(),
                right_w,
                &parent_query,
                &mut transforms,
            );
        }
        if let Some(ent) = left_arm {
            orient_bone_in_model_space(
                ent,
                char_ent,
                Vec3::new(0.08, 0.0, 0.99).normalize(),
                left_w,
                &parent_query,
                &mut transforms,
            );
        }

        // 2. Extend forearms forward in a punching thrust after upper arms are updated
        if let Some(ent) = right_forearm {
            orient_bone_in_model_space(
                ent,
                char_ent,
                Vec3::new(0.06, 0.08, 0.99).normalize(),
                right_w,
                &parent_query,
                &mut transforms,
            );
        }
        if let Some(ent) = left_forearm {
            orient_bone_in_model_space(
                ent,
                char_ent,
                Vec3::new(-0.06, 0.08, 0.99).normalize(),
                left_w,
                &parent_query,
                &mut transforms,
            );
        }
    }
}
