use avian3d::prelude::*;
use bevy::prelude::*;

use crate::koth::KothManager;
use crate::types::{Character, Health, StaggerState, TackleAbility, VoiceCallout};

pub const TACKLE_COOLDOWN: f32 = 1.4;
pub const TACKLE_STAMINA_COST: f32 = 25.0;
pub const TACKLE_LUNGE_SPEED: f32 = 8.2;
pub const KNOCKBACK_FORCE: f32 = 8.0;
pub const UPWARD_LIFT: f32 = 2.4;
pub const HIT_RANGE: f32 = 2.4;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_cooldowns_and_stagger, check_tackle_hits, update_health_regen),
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
    mut koth: ResMut<KothManager>,
    mut tacklers: Query<(Entity, &Transform, &mut TackleAbility, &Character)>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut StaggerState,
        Option<&mut Health>,
        &Character,
    )>,
) {
    for (tackler_ent, tackler_trans, mut ability, tackler_char) in tacklers.iter_mut() {
        if !ability.is_tackling {
            continue;
        }

        let forward = tackler_trans.rotation * Vec3::Z;
        let tackler_pos = tackler_trans.translation;

        for (
            target_ent,
            target_trans,
            mut target_vel,
            mut target_ang_vel,
            mut target_stagger,
            mut maybe_health,
            target_char,
        ) in targets.iter_mut()
        {
            if tackler_ent == target_ent || tackler_char.team == target_char.team {
                continue;
            }

            let to_target = target_trans.translation - tackler_pos;
            let dist = to_target.length();

            if dist < HIT_RANGE {
                let dir_to_target = to_target.normalize_or_zero();
                let dot = forward.dot(dir_to_target);

                // Cone of ~120 degrees in front
                if dot > 0.35 {
                    // Knockback impulse vector
                    let mut knockback_dir = (to_target + forward * 0.5).normalize_or_zero();
                    knockback_dir.y = 0.0;
                    knockback_dir = knockback_dir.normalize_or(Vec3::Z);

                    target_vel.0 = knockback_dir * KNOCKBACK_FORCE + Vec3::Y * UPWARD_LIFT;
                    target_ang_vel.0 = Vec3::ZERO;
                    target_stagger.duration = 0.75;

                    if let Some(ref mut health) = maybe_health {
                        health.current = (health.current - 20.0).max(10.0);
                    }

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

                    // End the tackle lunge early on impact
                    ability.is_tackling = false;
                    break;
                }
            }
        }
    }
}
