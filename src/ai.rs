use avian3d::prelude::*;
use bevy::prelude::*;

use crate::arena::PLATEAU_COORDS;
use crate::combat::{TACKLE_COOLDOWN, TACKLE_LUNGE_SPEED, TACKLE_STAMINA_COST};
use crate::koth::KothManager;
use crate::player::spawn_character_visuals;
use crate::types::{Character, Health, StaggerState, Stamina, TackleAbility, Team};

#[allow(dead_code)]
#[derive(Component)]
pub struct Bot {
    pub target_pos: Vec3,
    pub state_timer: f32,
    pub reaction_cooldown: f32,
}

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bots)
            .add_systems(Update, update_bot_behaviors);
    }
}

fn spawn_bots(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 2 Teammates on Team Blue on West Blue Platform (Anders is player, Morten and Jonas are teammates)
    let blue_bots = [
        ("Morten", Vec3::new(-24.5, 3.2, -3.2)),
        ("Jonas", Vec3::new(-24.5, 3.2, 3.2)),
    ];

    for (name, spawn_pos) in blue_bots.iter() {
        spawn_single_bot(
            &mut commands,
            &mut meshes,
            &mut materials,
            Team::Blue,
            name,
            *spawn_pos,
        );
    }

    // 3 Opponents on Team Red on East Red Platform (Siri, Lisa, Per)
    let red_bots = [
        ("Siri", Vec3::new(21.0, 3.2, 0.0)),
        ("Lisa", Vec3::new(24.5, 3.2, -3.2)),
        ("Per", Vec3::new(24.5, 3.2, 3.2)),
    ];

    for (name, spawn_pos) in red_bots.iter() {
        spawn_single_bot(
            &mut commands,
            &mut meshes,
            &mut materials,
            Team::Red,
            name,
            *spawn_pos,
        );
    }
}

fn spawn_single_bot(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    team: Team,
    name: &str,
    spawn_pos: Vec3,
) {
    let bot_entity = commands
        .spawn((
            Bot {
                target_pos: Vec3::ZERO,
                state_timer: 0.0,
                reaction_cooldown: 0.0,
            },
            Character {
                team,
                name: name.to_string(),
                is_player: false,
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

    spawn_character_visuals(commands, bot_entity, meshes, materials, team, name);
}

fn update_bot_behaviors(
    time: Res<Time>,
    koth: Res<KothManager>,
    mut queries: ParamSet<(
        Query<(Entity, &Transform, &Character)>,
        Query<(
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut TackleAbility,
            &mut Stamina,
            &StaggerState,
            &Character,
            &mut Bot,
        ), Without<crate::player::Player>>,
    )>,
) {
    let dt = time.delta_secs();
    let (_, _, active_hill_center, hill_radius) = PLATEAU_COORDS[koth.active_index];

    // Collect snapshots of character positions & teams first to avoid query aliasing
    let char_snapshots: Vec<(Entity, Vec3, Team)> = queries
        .p0()
        .iter()
        .map(|(ent, trans, ch)| (ent, trans.translation, ch.team))
        .collect();

    for (
        bot_ent,
        mut transform,
        mut lin_vel,
        mut tackle,
        mut stamina,
        stagger,
        bot_char,
        mut bot_data,
    ) in queries.p1().iter_mut()
    {
        if stagger.duration > 0.0 {
            continue;
        }

        bot_data.reaction_cooldown = (bot_data.reaction_cooldown - dt).max(0.0);
        let my_pos = transform.translation;

        // Find nearest opponent
        let mut nearest_enemy_dist = f32::MAX;
        let mut nearest_enemy_pos = Vec3::ZERO;

        for (other_ent, other_pos, other_team) in char_snapshots.iter() {
            if *other_ent == bot_ent || *other_team == bot_char.team {
                continue;
            }
            let d = my_pos.distance(*other_pos);
            if d < nearest_enemy_dist {
                nearest_enemy_dist = d;
                nearest_enemy_pos = *other_pos;
            }
        }

        // Tactical Decision Tree:
        // 1. If enemy is close (< 3.0m), face them and execute tackle!
        if nearest_enemy_dist < 3.2
            && tackle.cooldown <= 0.0
            && stamina.current >= TACKLE_STAMINA_COST
            && bot_data.reaction_cooldown <= 0.0
        {
            let to_enemy = (nearest_enemy_pos - my_pos).normalize_or_zero();
            let target_rot = Quat::from_rotation_y(to_enemy.x.atan2(to_enemy.z));
            transform.rotation = target_rot;

            tackle.is_tackling = true;
            tackle.tackle_timer = 0.28;
            tackle.cooldown = TACKLE_COOLDOWN + 0.3; // Slight variance
            stamina.current -= TACKLE_STAMINA_COST;
            bot_data.reaction_cooldown = 0.4;

            lin_vel.0 += to_enemy * TACKLE_LUNGE_SPEED + Vec3::Y * 2.0;
            continue;
        }

        // 2. Navigation Target:
        // If enemy is on the hill, contest them. Otherwise, move to the active hill center with slight offset
        let target = if nearest_enemy_dist < 10.0 && nearest_enemy_pos.distance(active_hill_center) < hill_radius + 2.0 {
            nearest_enemy_pos
        } else {
            // Offset slightly by entity id hash so bots don't all stack on top of each other
            let hash_offset = (bot_ent.index() as f32 * 1.3).sin() * 2.5;
            active_hill_center + Vec3::new(hash_offset, 0.0, hash_offset.cos() * 2.5)
        };

        let mut move_dir = target - my_pos;
        move_dir.y = 0.0;
        let dist_to_target = move_dir.length();

        if dist_to_target > 0.8 {
            let dir = move_dir.normalize();
            let bot_speed = 7.5;
            let desired_vel = dir * bot_speed;

            lin_vel.x = lin_vel.x.lerp(desired_vel.x, 10.0 * dt);
            lin_vel.z = lin_vel.z.lerp(desired_vel.z, 10.0 * dt);

            // Turn towards movement around Y axis
            let target_rot = Quat::from_rotation_y(desired_vel.x.atan2(desired_vel.z));
            transform.rotation = transform.rotation.slerp(target_rot, 12.0 * dt);

            // Auto-jump if hitting a ramp edge or stuck
            if my_pos.y < active_hill_center.y - 0.5 && lin_vel.y.abs() < 0.2 && dist_to_target < 15.0 {
                // If approaching ramp
                if lin_vel.length() < 2.0 {
                    lin_vel.y = 6.0;
                }
            }
        } else {
            // Patrol / stand ground
            lin_vel.x = lin_vel.x.lerp(0.0, 8.0 * dt);
            lin_vel.z = lin_vel.z.lerp(0.0, 8.0 * dt);
        }
    }
}
