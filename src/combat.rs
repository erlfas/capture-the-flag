use avian3d::prelude::*;
use bevy::prelude::*;

use crate::koth::KothManager;
use crate::types::{Character, Health, StaggerState, TackleAbility, Team, VoiceCallout};

pub const TACKLE_COOLDOWN: f32 = 1.15;
pub const TACKLE_STAMINA_COST: f32 = 20.0;
pub const TACKLE_LUNGE_SPEED: f32 = 16.5;
pub const KNOCKBACK_FORCE: f32 = 18.5;
pub const UPWARD_LIFT: f32 = 3.8;
pub const HIT_RANGE: f32 = 3.15;

#[derive(Component)]
pub struct ShoveShockwave {
    pub elapsed: f32,
    pub duration: f32,
    pub start_scale: f32,
    pub end_scale: f32,
    pub normal_rot: Quat,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_cooldowns_and_stagger,
                check_tackle_hits,
                handle_body_pushing,
                animate_shove_shockwaves,
                update_health_regen,
            ),
        );
    }
}

fn update_cooldowns_and_stagger(
    time: Res<Time>,
    mut query: Query<(&mut TackleAbility, &mut StaggerState)>,
) {
    let dt = time.delta_secs();

    for (mut ability, mut stagger) in query.iter_mut() {
        if ability.cooldown > 0.0 {
            ability.cooldown = (ability.cooldown - dt).max(0.0);
        }

        if ability.is_tackling {
            ability.tackle_timer -= dt;
            if ability.tackle_timer <= 0.0 {
                ability.is_tackling = false;
            }
        }

        if stagger.duration > 0.0 {
            stagger.duration = (stagger.duration - dt).max(0.0);
        }
    }
}

fn update_health_regen(time: Res<Time>, mut health_query: Query<&mut Health>) {
    let dt = time.delta_secs();
    for mut health in health_query.iter_mut() {
        if health.current < health.max {
            health.current = (health.current + 3.0 * dt).min(health.max);
        }
    }
}

fn check_tackle_hits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut koth: ResMut<KothManager>,
    mut characters: Query<(
        Entity,
        &Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut TackleAbility,
        &mut StaggerState,
        Option<&mut Health>,
        &Character,
    )>,
) {
    // 1. Collect active tacklers / shovers this frame
    let active_tacklers: Vec<(Entity, Vec3, Vec3, Vec3, Character)> = characters
        .iter()
        .filter_map(|(ent, trans, lin_vel, _, ability, _, _, ch)| {
            if ability.is_tackling {
                Some((
                    ent,
                    trans.translation,
                    trans.rotation * Vec3::Z,
                    lin_vel.0,
                    ch.clone(),
                ))
            } else {
                None
            }
        })
        .collect();

    if active_tacklers.is_empty() {
        return;
    }

    let mut hit_tacklers: Vec<(Entity, Vec3)> = Vec::new();

    for (tackler_ent, tackler_pos, forward, tackler_vel, tackler_char) in active_tacklers.iter() {
        let mut connected_dir: Option<Vec3> = None;

        for (
            target_ent,
            target_trans,
            mut target_vel,
            mut target_ang_vel,
            mut _target_ability,
            mut target_stagger,
            mut maybe_health,
            target_char,
        ) in characters.iter_mut()
        {
            if *tackler_ent == target_ent || tackler_char.team == target_char.team {
                continue;
            }

            let to_target = target_trans.translation - *tackler_pos;
            let horiz_to_target = Vec3::new(to_target.x, 0.0, to_target.z);
            let horiz_dist = horiz_to_target.length();
            let vert_dist = to_target.y.abs();

            if horiz_dist < HIT_RANGE && vert_dist < 2.2 {
                let dir_to_target = horiz_to_target.normalize_or(*forward);
                let dot = forward.dot(dir_to_target);

                // Generous ~165-degree front shove arc (plus instant close-range body check within 1.35m)
                if dot > 0.12 || horiz_dist < 1.35 {
                    let mut knockback_dir = (dir_to_target * 0.72 + *forward * 0.28).normalize_or(*forward);
                    knockback_dir.y = 0.0;
                    knockback_dir = knockback_dir.normalize_or(Vec3::Z);

                    // Combine base shove force with 45% of the shover's own forward momentum
                    let momentum_bonus = Vec3::new(tackler_vel.x, 0.0, tackler_vel.z)
                        .dot(knockback_dir)
                        .max(0.0)
                        * 0.45;
                    let total_push_speed = KNOCKBACK_FORCE + momentum_bonus;

                    target_vel.0 = knockback_dir * total_push_speed + Vec3::Y * UPWARD_LIFT;
                    target_ang_vel.0 = Vec3::ZERO;
                    target_stagger.duration = 1.05;

                    if let Some(ref mut health) = maybe_health {
                        health.current = (health.current - 22.0).max(10.0);
                    }

                    // Spawn a glowing 3D kinetic shockwave ring at the impact midpoint
                    let impact_pos =
                        (*tackler_pos + target_trans.translation) * 0.5 + Vec3::new(0.0, 0.15, 0.0);
                    let ring_rot = Quat::from_rotation_arc(Vec3::Y, knockback_dir);
                    let shock_mat = materials.add(StandardMaterial {
                        base_color: tackler_char.team.color().with_alpha(0.85),
                        emissive: tackler_char.team.emissive(),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    });
                    commands.spawn((
                        ShoveShockwave {
                            elapsed: 0.0,
                            duration: 0.28,
                            start_scale: 0.35,
                            end_scale: 2.35,
                            normal_rot: ring_rot,
                        },
                        Mesh3d(meshes.add(Torus {
                            minor_radius: 0.055,
                            major_radius: 0.65,
                        })),
                        MeshMaterial3d(shock_mat),
                        Transform::from_translation(impact_pos)
                            .with_rotation(ring_rot)
                            .with_scale(Vec3::splat(0.35)),
                    ));

                    // Trigger character-specific tackle voice callout if no higher-priority flag alert is active
                    if koth.event_notice.is_none() {
                        let spoken = match tackler_char.name.to_lowercase().as_str() {
                            "anders" => "Target neutralized! Stay down!",
                            "morten" => "Boom! Direct tackle hit!",
                            "jonas" => "Enemy stopped in their tracks!",
                            "siri" => "Too slow! Took you down!",
                            "lisa" => "Caught you in the net!",
                            "per" => "Sit down! Nobody gets past me!",
                            _ => "Direct tackle hit!",
                        };
                        koth.event_notice = Some(VoiceCallout::new(
                            tackler_char.name.clone(),
                            tackler_char.team,
                            "tackle_hit",
                            spoken,
                            tackler_char.team.color(),
                            2.2,
                        ));
                    }

                    connected_dir = Some(knockback_dir);
                    break;
                }
            }
        }

        if let Some(dir) = connected_dir {
            hit_tacklers.push((*tackler_ent, dir));
        }
    }

    // Apply follow-through momentum step to tacklers who landed a hit and end their lunge
    for (tackler_ent, hit_dir) in hit_tacklers {
        if let Ok((_, _, mut tackler_vel, mut tackler_ang, mut ability, _, _, _)) =
            characters.get_mut(tackler_ent)
        {
            ability.is_tackling = false;
            tackler_vel.x = hit_dir.x * 5.2;
            tackler_vel.z = hit_dir.z * 5.2;
            tackler_ang.0 = Vec3::ZERO;
        }
    }
}

/// Momentum-based continuous body-to-body pushing / shoulder-checking when characters collide
fn handle_body_pushing(
    time: Res<Time>,
    mut characters: Query<(
        Entity,
        &Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &StaggerState,
        &Character,
    )>,
) {
    let dt = time.delta_secs();
    let snapshots: Vec<(Entity, Vec3, Vec3, Team, bool)> = characters
        .iter()
        .map(|(e, t, v, _, s, c)| (e, t.translation, v.0, c.team, s.duration > 0.0))
        .collect();

    let contact_radius = 1.18_f32;

    for (ent_a, trans_a, mut vel_a, mut ang_a, _stag_a, char_a) in characters.iter_mut() {
        ang_a.0 = Vec3::ZERO;
        let pos_a = trans_a.translation;
        let horiz_vel_a = Vec3::new(vel_a.x, 0.0, vel_a.z);

        for (ent_b, pos_b, vel_b, team_b, _is_staggered_b) in snapshots.iter() {
            if ent_a == *ent_b {
                continue;
            }
            let delta = pos_a - *pos_b;
            let horiz_delta = Vec3::new(delta.x, 0.0, delta.z);
            let dist = horiz_delta.length();
            if dist < contact_radius && dist > 0.001 && delta.y.abs() < 1.65 {
                // Push direction from B toward A
                let push_away = horiz_delta / dist;
                let overlap = (contact_radius - dist) / contact_radius;

                // How fast B is ramming into A along push_away
                let horiz_vel_b = Vec3::new(vel_b.x, 0.0, vel_b.z);
                let ram_speed_from_b = (horiz_vel_b - horiz_vel_a).dot(push_away).max(0.0);

                let team_mult = if char_a.team != *team_b { 1.45 } else { 0.75 };
                let push_accel = (overlap * 24.0 + ram_speed_from_b * 1.85) * team_mult;

                vel_a.x += push_away.x * push_accel * dt;
                vel_a.z += push_away.z * push_accel * dt;
            }
        }
    }
}

fn animate_shove_shockwaves(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShoveShockwave, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (ent, mut wave, mut transform) in query.iter_mut() {
        wave.elapsed += dt;
        if wave.elapsed >= wave.duration {
            commands.entity(ent).despawn();
            continue;
        }
        let u = (wave.elapsed / wave.duration).clamp(0.0, 1.0);
        let scale = wave.start_scale + (wave.end_scale - wave.start_scale) * (1.0 - (1.0 - u) * (1.0 - u));
        let thickness = (1.0 - u * 0.75).max(0.15);
        transform.rotation = wave.normal_rot;
        transform.scale = Vec3::new(scale, thickness, scale);
    }
}
