use bevy::prelude::*;

use crate::arena::PLATEAU_COORDS;
use crate::types::{Character, StaggerState, Team};

pub const WIN_SCORE: f32 = 5.0; // First to 5 captured flags wins
pub const INITIAL_MATCH_TIME: f32 = 462.0; // 07:42 in seconds, matching art reference
pub const FLAG_AUTO_RETURN_SECS: f32 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneContestState {
    Neutral,
    Contested,
    BlueControlling,
    RedControlling,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagState {
    AtBase,
    CarriedBy(Entity),
    Dropped {
        pos: Vec3,
        return_timer: f32,
        pickup_cooldown: f32,
    },
}

#[derive(Component, Debug, Clone)]
pub struct TeamFlag {
    pub team: Team,
    pub home_pos: Vec3,
    pub state: FlagState,
}

#[derive(Resource)]
pub struct KothManager {
    pub match_timer: f32,
    pub blue_score: f32,
    pub red_score: f32,
    pub contest_state: ZoneContestState,
    pub winner: Option<Team>,
    pub status_banner: String,
    pub status_color: Color,
    pub event_notice: Option<(String, Color, f32)>,
    pub blue_carrier: Option<(Entity, String)>,
    pub red_carrier: Option<(Entity, String)>,
    pub blue_flag_world_pos: Vec3,
    pub red_flag_world_pos: Vec3,
    pub blue_flag_at_base: bool,
    pub red_flag_at_base: bool,
}

impl Default for KothManager {
    fn default() -> Self {
        Self {
            match_timer: INITIAL_MATCH_TIME,
            blue_score: 1.0, // Matches initial "Blå Flagg: 1" from art
            red_score: 0.0,  // Matches initial "Rød Flagg: 0" from art
            contest_state: ZoneContestState::Neutral,
            winner: None,
            status_banner: "«STJEL DET RØDE FLAGGET PÅ ØSTPLATTFORMEN!»".to_string(),
            status_color: Color::srgb(0.3, 0.9, 1.0),
            event_notice: None,
            blue_carrier: None,
            red_carrier: None,
            blue_flag_world_pos: PLATEAU_COORDS[1].2,
            red_flag_world_pos: PLATEAU_COORDS[2].2,
            blue_flag_at_base: true,
            red_flag_at_base: true,
        }
    }
}

#[derive(Component)]
pub struct HologramZone;

pub struct KothPlugin;

impl Plugin for KothPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KothManager>()
            .add_systems(Startup, (setup_hologram_beacon, setup_ctf_flags))
            .add_systems(
                Update,
                (
                    update_ctf_flags_and_scoring,
                    update_flag_visual_transforms,
                    update_hologram_visuals,
                )
                    .chain(),
            );
    }
}

fn setup_hologram_beacon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (_idx, _, center, _radius) = PLATEAU_COORDS[0];

    // Low-profile glowing ground indicator ring
    let ring_mesh = meshes.add(Torus {
        minor_radius: 0.08,
        major_radius: 2.2,
    });
    let holo_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.9, 1.0, 0.45),
        emissive: LinearRgba::new(0.0, 3.0, 4.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(ring_mesh),
        MeshMaterial3d(holo_mat),
        Transform::from_translation(center + Vec3::new(0.0, 0.12, 0.0)),
        HologramZone,
    ));
}

fn setup_ctf_flags(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let steel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.68, 0.72),
        metallic: 0.85,
        perceptual_roughness: 0.3,
        ..default()
    });

    let gold_finial_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.80, 0.25),
        emissive: LinearRgba::new(1.5, 1.1, 0.2, 1.0),
        metallic: 0.7,
        perceptual_roughness: 0.25,
        ..default()
    });

    let blue_flag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.50, 0.96),
        emissive: LinearRgba::new(0.1, 0.8, 2.5, 1.0),
        perceptual_roughness: 0.5,
        cull_mode: None,
        ..default()
    });

    let red_flag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.18, 0.15),
        emissive: LinearRgba::new(2.5, 0.2, 0.1, 1.0),
        perceptual_roughness: 0.5,
        cull_mode: None,
        ..default()
    });

    let blue_home = PLATEAU_COORDS[1].2;
    let red_home = PLATEAU_COORDS[2].2;

    spawn_single_ctf_flag(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &gold_finial_mat,
        &blue_flag_mat,
        Team::Blue,
        blue_home,
        1.0,
    );

    spawn_single_ctf_flag(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &gold_finial_mat,
        &red_flag_mat,
        Team::Red,
        red_home,
        -1.0,
    );
}

fn spawn_single_ctf_flag(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    pole_mat: &Handle<StandardMaterial>,
    finial_mat: &Handle<StandardMaterial>,
    banner_mat: &Handle<StandardMaterial>,
    team: Team,
    home_pos: Vec3,
    dir_sign: f32,
) {
    commands
        .spawn((
            Transform::from_translation(home_pos),
            Visibility::default(),
            TeamFlag {
                team,
                home_pos,
                state: FlagState::AtBase,
            },
        ))
        .with_children(|parent| {
            // Tall steel flagpole
            parent.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.065, 4.4))),
                MeshMaterial3d(pole_mat.clone()),
                Transform::from_xyz(0.0, 2.2, 0.0),
            ));
            // Golden top finial sphere
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.16))),
                MeshMaterial3d(finial_mat.clone()),
                Transform::from_xyz(0.0, 4.45, 0.0),
            ));
            // Waving cloth banner
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.0, 1.15, 0.06))),
                MeshMaterial3d(banner_mat.clone()),
                Transform::from_xyz(dir_sign * 1.0, 3.75, 0.0),
            ));
        });
}

fn update_ctf_flags_and_scoring(
    time: Res<Time>,
    mut koth: ResMut<KothManager>,
    mut flags: Query<&mut TeamFlag>,
    characters: Query<(Entity, &Transform, &Character, &StaggerState)>,
) {
    let dt = time.delta_secs();

    // 1. Match countdown timer
    if koth.match_timer > 0.0 && koth.winner.is_none() {
        koth.match_timer = (koth.match_timer - dt).max(0.0);
        if koth.match_timer <= 0.0 {
            if koth.blue_score >= koth.red_score {
                koth.winner = Some(Team::Blue);
            } else {
                koth.winner = Some(Team::Red);
            }
        }
    }

    // Tick event notice banner timer
    if let Some((_, _, ref mut timer)) = koth.event_notice {
        *timer -= dt;
        if *timer <= 0.0 {
            koth.event_notice = None;
        }
    }

    if koth.winner.is_some() {
        return;
    }

    // Collect character snapshots
    let char_list: Vec<(Entity, Vec3, Team, String, bool)> = characters
        .iter()
        .map(|(ent, trans, ch, stag)| {
            (
                ent,
                trans.translation,
                ch.team,
                ch.name.clone(),
                stag.duration > 0.0,
            )
        })
        .collect();

    let blue_base_pos = PLATEAU_COORDS[1].2;
    let red_base_pos = PLATEAU_COORDS[2].2;
    let base_capture_radius = 4.2_f32;
    let flag_touch_radius = 2.4_f32;

    // Track which entities are already carrying a flag
    let mut current_carriers: Vec<Entity> = flags
        .iter()
        .filter_map(|f| match f.state {
            FlagState::CarriedBy(e) => Some(e),
            _ => None,
        })
        .collect();

    let mut blue_carrier_info: Option<(Entity, String)> = None;
    let mut red_carrier_info: Option<(Entity, String)> = None;

    for mut flag in flags.iter_mut() {
        match flag.state {
            FlagState::CarriedBy(carrier_ent) => {
                // Find carrier in character snapshots
                if let Some((_, carrier_pos, carrier_team, carrier_name, is_staggered)) =
                    char_list.iter().find(|(e, ..)| *e == carrier_ent)
                {
                    // A) Did the carrier get tackled (staggered) or fall off the arena?
                    if *is_staggered || carrier_pos.y < -2.0 {
                        let drop_pos = if carrier_pos.y < -2.0 {
                            flag.home_pos
                        } else {
                            Vec3::new(carrier_pos.x, (carrier_pos.y - 0.95).max(0.0), carrier_pos.z)
                        };

                        if carrier_pos.y < -2.0 {
                            flag.state = FlagState::AtBase;
                        } else {
                            flag.state = FlagState::Dropped {
                                pos: drop_pos,
                                return_timer: FLAG_AUTO_RETURN_SECS,
                                pickup_cooldown: 1.1,
                            };
                            let msg = match flag.team {
                                Team::Blue => format!(
                                    "«{} MISTET DET BLÅ FLAGGET! TA DET TILBAKE!»",
                                    carrier_name.to_uppercase()
                                ),
                                Team::Red => format!(
                                    "«{} MISTET DET RØDE FLAGGET!»",
                                    carrier_name.to_uppercase()
                                ),
                            };
                            koth.event_notice =
                                Some((msg, Color::srgb(1.0, 0.85, 0.2), 3.2));
                        }
                        continue;
                    }

                    // B) Did the carrier reach their own team's Home Base capture circle to score?
                    let target_base = match carrier_team {
                        Team::Blue => blue_base_pos,
                        Team::Red => red_base_pos,
                    };
                    let horiz_dist = Vec2::new(
                        carrier_pos.x - target_base.x,
                        carrier_pos.z - target_base.z,
                    )
                    .length();
                    let vert_dist = (carrier_pos.y - target_base.y).abs();

                    if horiz_dist <= base_capture_radius && vert_dist <= 2.4 {
                        // SCORE A FLAG CAPTURE!
                        flag.state = FlagState::AtBase;
                        match carrier_team {
                            Team::Blue => {
                                koth.blue_score += 1.0;
                                koth.event_notice = Some((
                                    format!(
                                        "«FLAGG EROBRET AV {}! +1 BLÅTT POENG!»",
                                        carrier_name.to_uppercase()
                                    ),
                                    Color::srgb(0.2, 1.0, 0.5),
                                    4.5,
                                ));
                                if koth.blue_score >= WIN_SCORE {
                                    koth.winner = Some(Team::Blue);
                                }
                            }
                            Team::Red => {
                                koth.red_score += 1.0;
                                koth.event_notice = Some((
                                    format!(
                                        "«{} EROBRET VÅRT FLAGG! +1 RØDT POENG!»",
                                        carrier_name.to_uppercase()
                                    ),
                                    Color::srgb(1.0, 0.25, 0.25),
                                    4.5,
                                ));
                                if koth.red_score >= WIN_SCORE {
                                    koth.winner = Some(Team::Red);
                                }
                            }
                        }
                        continue;
                    }

                    // Still carrying
                    match flag.team {
                        Team::Blue => {
                            // Blue Flag is carried by a Red player
                            red_carrier_info = Some((carrier_ent, carrier_name.clone()));
                        }
                        Team::Red => {
                            // Red Flag is carried by a Blue player
                            blue_carrier_info = Some((carrier_ent, carrier_name.clone()));
                        }
                    }
                } else {
                    // Carrier entity no longer exists
                    flag.state = FlagState::AtBase;
                }
            }
            FlagState::Dropped {
                pos,
                mut return_timer,
                mut pickup_cooldown,
            } => {
                return_timer -= dt;
                pickup_cooldown = (pickup_cooldown - dt).max(0.0);

                if return_timer <= 0.0 {
                    flag.state = FlagState::AtBase;
                    let msg = match flag.team {
                        Team::Blue => "«BLÅTT FLAGG RETURNERT TIL BASEN!»".to_string(),
                        Team::Red => "«RØDT FLAGG RETURNERT TIL BASEN!»".to_string(),
                    };
                    koth.event_notice = Some((msg, Color::srgb(0.4, 0.85, 1.0), 3.0));
                    continue;
                }

                let mut new_state = FlagState::Dropped {
                    pos,
                    return_timer,
                    pickup_cooldown,
                };

                // Check if any character touches the dropped flag
                for (ent, c_pos, c_team, c_name, is_staggered) in char_list.iter() {
                    if *is_staggered {
                        continue;
                    }
                    let h_dist = Vec2::new(c_pos.x - pos.x, c_pos.z - pos.z).length();
                    let v_dist = (c_pos.y - pos.y).abs();
                    if h_dist <= flag_touch_radius && v_dist <= 2.2 {
                        if *c_team == flag.team {
                            // Teammate touched their own dropped flag -> Instant Return to Base!
                            new_state = FlagState::AtBase;
                            let msg = match flag.team {
                                Team::Blue => format!(
                                    "«{} RETURNERTE VÅRT BLÅ FLAGG TIL BASEN!»",
                                    c_name.to_uppercase()
                                ),
                                Team::Red => format!(
                                    "«{} RETURNERTE DET RØDE FLAGGET!»",
                                    c_name.to_uppercase()
                                ),
                            };
                            koth.event_notice =
                                Some((msg, Color::srgb(0.3, 0.95, 0.6), 3.5));
                            break;
                        } else if pickup_cooldown <= 0.0 && !current_carriers.contains(ent) {
                            // Enemy picked up the dropped flag!
                            new_state = FlagState::CarriedBy(*ent);
                            current_carriers.push(*ent);
                            break;
                        }
                    }
                }

                flag.state = new_state;
            }
            FlagState::AtBase => {
                let pos = flag.home_pos;
                for (ent, c_pos, c_team, c_name, is_staggered) in char_list.iter() {
                    if *is_staggered || *c_team == flag.team || current_carriers.contains(ent) {
                        continue;
                    }
                    let h_dist = Vec2::new(c_pos.x - pos.x, c_pos.z - pos.z).length();
                    let v_dist = (c_pos.y - pos.y).abs();
                    if h_dist <= flag_touch_radius && v_dist <= 2.2 {
                        // Enemy stole the flag from its base!
                        flag.state = FlagState::CarriedBy(*ent);
                        current_carriers.push(*ent);
                        match flag.team {
                            Team::Blue => {
                                red_carrier_info = Some((*ent, c_name.clone()));
                                koth.event_notice = Some((
                                    format!(
                                        "«VÅRT FLAGG ER STJÅLET AV {}! TAKLE DEM!»",
                                        c_name.to_uppercase()
                                    ),
                                    Color::srgb(1.0, 0.22, 0.22),
                                    4.0,
                                ));
                            }
                            Team::Red => {
                                blue_carrier_info = Some((*ent, c_name.clone()));
                                koth.event_notice = Some((
                                    format!(
                                        "«{} TOK DET RØDE FLAGGET! LØP TIL BLÅ BASE!»",
                                        c_name.to_uppercase()
                                    ),
                                    Color::srgb(0.2, 0.95, 0.55),
                                    4.0,
                                ));
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // Update KothManager summary state for UI and AI
    koth.blue_carrier = blue_carrier_info.clone();
    koth.red_carrier = red_carrier_info.clone();

    for flag in flags.iter() {
        let is_home = matches!(flag.state, FlagState::AtBase);
        let world_pos = match flag.state {
            FlagState::AtBase => flag.home_pos,
            FlagState::Dropped { pos, .. } => pos,
            FlagState::CarriedBy(ent) => char_list
                .iter()
                .find(|(e, ..)| *e == ent)
                .map(|(_, p, ..)| *p)
                .unwrap_or(flag.home_pos),
        };
        match flag.team {
            Team::Blue => {
                koth.blue_flag_at_base = is_home;
                koth.blue_flag_world_pos = world_pos;
            }
            Team::Red => {
                koth.red_flag_at_base = is_home;
                koth.red_flag_world_pos = world_pos;
            }
        }
    }

    // Determine primary status headline when no short event notice is overriding
    if let Some((ref msg, color, _)) = koth.event_notice {
        koth.status_banner = msg.clone();
        koth.status_color = color;
    } else if let (Some((_, b_name)), Some((_, r_name))) = (&blue_carrier_info, &red_carrier_info)
    {
        koth.contest_state = ZoneContestState::Contested;
        koth.status_banner = format!(
            "«BEGGE FLAGG STJÅLET! {} [BLÅ] vs {} [RØD]»",
            b_name.to_uppercase(),
            r_name.to_uppercase()
        );
        koth.status_color = Color::srgb(1.0, 0.85, 0.15);
    } else if let Some((_, r_name)) = &red_carrier_info {
        koth.contest_state = ZoneContestState::RedControlling;
        koth.status_banner = format!(
            "«VÅRT FLAGG ER STJÅLET AV {}! TAKLE DEM!»",
            r_name.to_uppercase()
        );
        koth.status_color = Color::srgb(1.0, 0.22, 0.22);
    } else if let Some((_, b_name)) = &blue_carrier_info {
        koth.contest_state = ZoneContestState::BlueControlling;
        koth.status_banner = format!(
            "«{} HAR RØDT FLAGG! FRIGJØR VEIEN TIL BLÅ BASE!»",
            b_name.to_uppercase()
        );
        koth.status_color = Color::srgb(0.2, 0.95, 0.45);
    } else {
        koth.contest_state = ZoneContestState::Neutral;
        koth.status_banner =
            "«EROBRE RØDT FLAGG PÅ ØSTPLATTFORMEN OG BRING DET TIL BLÅ BASE!»".to_string();
        koth.status_color = Color::srgb(0.75, 0.88, 1.0);
    }
}

fn update_flag_visual_transforms(
    mut flags: Query<(&TeamFlag, &mut Transform)>,
    characters: Query<&Transform, (With<Character>, Without<TeamFlag>)>,
) {
    for (flag, mut flag_transform) in flags.iter_mut() {
        match flag.state {
            FlagState::AtBase => {
                flag_transform.translation = flag.home_pos;
                flag_transform.rotation = Quat::IDENTITY;
                flag_transform.scale = Vec3::ONE;
            }
            FlagState::Dropped { pos, .. } => {
                flag_transform.translation = pos;
                // Slightly tilted flagpole stuck into the ground when dropped
                flag_transform.rotation = Quat::from_rotation_z(0.22);
                flag_transform.scale = Vec3::splat(0.85);
            }
            FlagState::CarriedBy(carrier_ent) => {
                if let Ok(char_trans) = characters.get(carrier_ent) {
                    // Attach flagpole to the carrier's back so it waves visibly above them
                    let back_offset = char_trans.rotation * Vec3::new(0.0, -0.1, -0.26);
                    flag_transform.translation = char_trans.translation + back_offset;
                    flag_transform.rotation =
                        char_trans.rotation * Quat::from_rotation_x(-0.22);
                    flag_transform.scale = Vec3::splat(0.72);
                }
            }
        }
    }
}

fn update_hologram_visuals(
    koth: Res<KothManager>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<HologramZone>>,
) {
    // Highlight the player's current objective with the glowing ground ring:
    // If Blue team is carrying the Red Flag, highlight the Blue Home Base capture circle!
    // Otherwise, highlight the Red Flag's current position!
    let (target_pos, is_delivering) = if koth.blue_carrier.is_some() {
        (PLATEAU_COORDS[1].2, true)
    } else {
        (koth.red_flag_world_pos, false)
    };

    for (mut transform, mat_handle) in query.iter_mut() {
        transform.translation = target_pos + Vec3::new(0.0, 0.10, 0.0);

        if let Some(mat) = materials.get_mut(mat_handle) {
            if is_delivering {
                mat.base_color = Color::srgba(0.1, 1.0, 0.4, 0.45);
                mat.emissive = LinearRgba::new(0.2, 4.5, 1.5, 1.0);
            } else {
                mat.base_color = Color::srgba(1.0, 0.3, 0.2, 0.45);
                mat.emissive = LinearRgba::new(4.5, 0.8, 0.4, 1.0);
            }
        }
    }
}
