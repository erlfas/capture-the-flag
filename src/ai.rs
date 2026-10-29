use avian3d::prelude::*;
use bevy::prelude::*;

use crate::arena::PLATEAU_COORDS;
use crate::combat::{TACKLE_COOLDOWN, TACKLE_LUNGE_SPEED, TACKLE_STAMINA_COST};
use crate::koth::KothManager;
use crate::player::spawn_character_visuals;
use crate::types::{Character, Health, StaggerState, Stamina, TackleAbility, Team};

pub const BOT_BASE_SPEED: f32 = 8.2;
pub const BOT_SPRINT_MULTIPLIER: f32 = 1.48;

#[allow(dead_code)]
#[derive(Component)]
pub struct Bot {
    pub role_attacker: bool,
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
    // 2 Teammates on Team Blue on West Blue Platform (deck at y = 2.53m)
    let blue_bots = [
        ("Morten", Vec3::new(-21.5, 3.6, -2.5), true),
        ("Jonas", Vec3::new(-21.5, 3.6, 5.8), false),
    ];

    for (name, spawn_pos, role_attacker) in blue_bots.iter() {
        spawn_single_bot(
            &mut commands,
            &mut meshes,
            &mut materials,
            Team::Blue,
            name,
            *spawn_pos,
            *role_attacker,
        );
    }

    // 3 Opponents on Team Red on East Red Platform (deck at y = 2.53m)
    let red_bots = [
        ("Siri", Vec3::new(17.5, 3.6, 3.8), true),
        ("Lisa", Vec3::new(21.0, 3.6, -2.5), true),
        ("Per", Vec3::new(21.0, 3.6, 5.8), false),
    ];

    for (name, spawn_pos, role_attacker) in red_bots.iter() {
        spawn_single_bot(
            &mut commands,
            &mut meshes,
            &mut materials,
            Team::Red,
            name,
            *spawn_pos,
            *role_attacker,
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
    role_attacker: bool,
) {
    let bot_entity = commands
        .spawn((
            Bot {
                role_attacker,
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
            SweptCcd::default(),
            Transform::from_translation(spawn_pos),
        ))
        .id();

    spawn_character_visuals(commands, bot_entity, meshes, materials, team, name);
}

/// Guides a bot onto an elevated platform via its 3D model ramp/stairs if the bot is on the ground floor
fn route_via_platform_ramp(my_pos: Vec3, raw_target: Vec3) -> Vec3 {
    let blue_base = PLATEAU_COORDS[1].2; // (-20.2, 2.55, 1.8)
    let red_base = PLATEAU_COORDS[2].2; // (19.5, 2.55, 1.8)

    // If bot is on the courtyard floor (y < 1.8) and wants to reach the West Blue Platform
    if my_pos.y < 1.8 && raw_target.distance(blue_base) < 7.0 {
        let upper_ramp_entry = Vec3::new(-9.5, 0.2, -0.5);
        let lower_ramp_entry = Vec3::new(-9.5, 0.2, 6.5);
        let ramp_entry = if my_pos.distance(upper_ramp_entry) < my_pos.distance(lower_ramp_entry) {
            upper_ramp_entry
        } else {
            lower_ramp_entry
        };
        if my_pos.x > -14.5 {
            return ramp_entry;
        }
    }

    // If bot is on the courtyard floor (y < 1.8) and wants to reach the East Red Platform
    if my_pos.y < 1.8 && raw_target.distance(red_base) < 7.0 {
        let west_ramp_entry = Vec3::new(9.5, 0.2, 2.5);
        let south_stairs_entry = Vec3::new(20.0, -0.3, 15.2);
        if my_pos.z > 10.5 && (my_pos.x - 20.0).abs() < 2.2 {
            // Already aligned with the South Staircase -> walk straight up onto the Red Platform!
            return red_base;
        }
        let ramp_entry = if my_pos.distance(west_ramp_entry) < my_pos.distance(south_stairs_entry)
        {
            west_ramp_entry
        } else {
            south_stairs_entry
        };
        if my_pos.x < 14.2 || my_pos.z > 10.5 {
            return ramp_entry;
        }
    }

    raw_target
}

fn update_bot_behaviors(
    time: Res<Time>,
    koth: Res<KothManager>,
    mut queries: ParamSet<(
        Query<(Entity, &Transform, &Character)>,
        Query<
            (
                Entity,
                &mut Transform,
                &mut LinearVelocity,
                &mut AngularVelocity,
                &mut TackleAbility,
                &mut Stamina,
                &StaggerState,
                &Character,
                &mut Bot,
            ),
            Without<crate::player::Player>,
        >,
    )>,
) {
    let dt = time.delta_secs();
    let blue_base = PLATEAU_COORDS[1].2;
    let red_base = PLATEAU_COORDS[2].2;

    let char_snapshots: Vec<(Entity, Vec3, Team)> = queries
        .p0()
        .iter()
        .map(|(ent, trans, ch)| (ent, trans.translation, ch.team))
        .collect();

    for (
        bot_ent,
        mut transform,
        mut lin_vel,
        mut ang_vel,
        mut tackle,
        mut stamina,
        stagger,
        bot_char,
        mut bot_data,
    ) in queries.p1().iter_mut()
    {
        // Prevent any physics-induced self-spinning
        ang_vel.0 = Vec3::ZERO;

        // Apply snappy extra downward gravity when falling
        if lin_vel.y < -0.2 {
            lin_vel.y -= 6.0 * dt;
        }

        if stagger.duration > 0.0 || koth.winner.is_some() {
            continue;
        }

        bot_data.reaction_cooldown = (bot_data.reaction_cooldown - dt).max(0.0);
        let my_pos = transform.translation;

        // Check if this bot is currently carrying the enemy flag
        let am_carrying = match bot_char.team {
            Team::Blue => koth
                .blue_carrier
                .as_ref()
                .map(|(e, _)| *e == bot_ent)
                .unwrap_or(false),
            Team::Red => koth
                .red_carrier
                .as_ref()
                .map(|(e, _)| *e == bot_ent)
                .unwrap_or(false),
        };

        // Check if an enemy is carrying our team's flag
        let enemy_flag_carrier = match bot_char.team {
            Team::Blue => koth.red_carrier.as_ref().map(|(e, _)| *e),
            Team::Red => koth.blue_carrier.as_ref().map(|(e, _)| *e),
        };

        // Find nearest opponent (prioritizing the enemy flag carrier!)
        let mut nearest_enemy_dist = f32::MAX;
        let mut nearest_enemy_pos = Vec3::ZERO;

        for (other_ent, other_pos, other_team) in char_snapshots.iter() {
            if *other_ent == bot_ent || *other_team == bot_char.team {
                continue;
            }
            let d = my_pos.distance(*other_pos);
            let effective_d = if Some(*other_ent) == enemy_flag_carrier {
                d * 0.6 // Strongly prioritize tackling the flag thief
            } else {
                d
            };
            if effective_d < nearest_enemy_dist {
                nearest_enemy_dist = d;
                nearest_enemy_pos = *other_pos;
            }
        }

        // 1. If enemy is in tackle range (< 2.6m) and we are not carrying the flag, tackle them!
        if !am_carrying
            && nearest_enemy_dist < 2.6
            && tackle.cooldown <= 0.0
            && stamina.current >= TACKLE_STAMINA_COST
            && bot_data.reaction_cooldown <= 0.0
            && lin_vel.y.abs() < 0.8
        {
            let mut to_enemy = nearest_enemy_pos - my_pos;
            to_enemy.y = 0.0;
            let to_enemy = to_enemy.normalize_or_zero();
            transform.rotation = Quat::from_rotation_y(to_enemy.x.atan2(to_enemy.z));

            tackle.is_tackling = true;
            tackle.tackle_timer = 0.28;
            tackle.cooldown = TACKLE_COOLDOWN;
            stamina.current -= TACKLE_STAMINA_COST;
            bot_data.reaction_cooldown = 0.25;

            let lunge_speed = TACKLE_LUNGE_SPEED.max(16.5);
            lin_vel.0 = to_enemy * lunge_speed + Vec3::Y * 1.2;
            continue;
        }

        // 2. Capture the Flag Tactical Target Selection:
        let raw_target = if am_carrying {
            // Run straight back to our Home Base capture circle to score!
            match bot_char.team {
                Team::Blue => blue_base,
                Team::Red => red_base,
            }
        } else {
            let (own_flag_at_base, own_flag_pos, enemy_flag_pos) = match bot_char.team {
                Team::Blue => (
                    koth.blue_flag_at_base,
                    koth.blue_flag_world_pos,
                    koth.red_flag_world_pos,
                ),
                Team::Red => (
                    koth.red_flag_at_base,
                    koth.red_flag_world_pos,
                    koth.blue_flag_world_pos,
                ),
            };

            if !own_flag_at_base && (!bot_data.role_attacker || my_pos.distance(own_flag_pos) < 18.0)
            {
                // Chase down our stolen/dropped flag to tackle the thief or return it!
                own_flag_pos
            } else if bot_data.role_attacker {
                // Raid the enemy base to steal their flag!
                enemy_flag_pos
            } else {
                // Defender patrols near own flag or intercepts nearby enemies
                if nearest_enemy_dist < 14.0 {
                    nearest_enemy_pos
                } else {
                    let hash_offset = (bot_ent.index() as f32 * 1.7).sin() * 2.8;
                    own_flag_pos + Vec3::new(hash_offset, 0.0, hash_offset.cos() * 2.8)
                }
            }
        };

        let target = route_via_platform_ramp(my_pos, raw_target);

        let mut move_dir = target - my_pos;
        move_dir.y = 0.0;
        let dist_to_target = move_dir.length();

        if dist_to_target > 0.65 {
            let dir = move_dir.normalize();
            let wants_sprint =
                stamina.current > 18.0 && (dist_to_target > 7.0 || nearest_enemy_dist < 9.0);
            let carry_mult = if am_carrying { 0.90 } else { 1.0 };
            let bot_speed = if wants_sprint {
                stamina.current = (stamina.current - 12.0 * dt).max(0.0);
                BOT_BASE_SPEED * BOT_SPRINT_MULTIPLIER * carry_mult
            } else {
                BOT_BASE_SPEED * carry_mult
            };
            let desired_vel = dir * bot_speed;

            lin_vel.x = lin_vel.x.lerp(desired_vel.x, (11.5 * dt).min(1.0));
            lin_vel.z = lin_vel.z.lerp(desired_vel.z, (11.5 * dt).min(1.0));

            // Smooth uphill slope/staircase climbing for bots
            let curr_y = crate::terrain_heights::sample_terrain_height(
                transform.translation.x,
                transform.translation.z,
            );
            let ahead_y = crate::terrain_heights::sample_terrain_height(
                transform.translation.x + dir.x * 0.48,
                transform.translation.z + dir.z * 0.48,
            );
            let dh = ahead_y - curr_y;
            if dh > 0.01 && dh < 1.15 && transform.translation.y <= curr_y + 1.15 {
                let climb_vy = (bot_speed * (dh / 0.48)).clamp(0.0, 12.0);
                if lin_vel.y < climb_vy {
                    lin_vel.y = climb_vy;
                }
                let target_center_y = ahead_y + 0.92;
                if transform.translation.y < target_center_y {
                    transform.translation.y = transform
                        .translation
                        .y
                        .lerp(target_center_y, (16.0 * dt).min(1.0));
                }
            }

            let target_rot = Quat::from_rotation_y(desired_vel.x.atan2(desired_vel.z));
            transform.rotation = transform.rotation.slerp(target_rot, (14.0 * dt).min(1.0));

            // Athletic hop if blocked by a low obstacle
            if lin_vel.y.abs() < 0.15
                && Vec2::new(lin_vel.x, lin_vel.z).length() < 1.5
                && dist_to_target > 2.5
                && bot_data.reaction_cooldown <= 0.0
            {
                lin_vel.y = 5.4;
                bot_data.reaction_cooldown = 0.45;
            }
        } else {
            lin_vel.x = lin_vel.x.lerp(0.0, (12.0 * dt).min(1.0));
            lin_vel.z = lin_vel.z.lerp(0.0, (12.0 * dt).min(1.0));
        }
    }
}
