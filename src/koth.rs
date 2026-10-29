use bevy::audio::Volume;
use bevy::prelude::*;

use crate::arena::PLATEAU_COORDS;
use crate::types::{Character, StaggerState, Team, VoiceCallout};

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
    pub match_started: bool,
    pub match_timer: f32,
    pub blue_score: f32,
    pub red_score: f32,
    pub contest_state: ZoneContestState,
    pub winner: Option<Team>,
    pub status_banner: String,
    pub status_color: Color,
    pub active_speaker: Option<(String, Team)>,
    pub event_notice: Option<VoiceCallout>,
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
            match_started: false,
            match_timer: INITIAL_MATCH_TIME,
            blue_score: 1.0, // Matches initial "Blue Flag: 1" from art
            red_score: 0.0,  // Matches initial "Red Flag: 0" from art
            contest_state: ZoneContestState::Neutral,
            winner: None,
            status_banner:
                "DELTA SECTOR COMPOUND — STAND BY FOR MATCH COMMENCEMENT...".to_string(),
            status_color: Color::srgb(0.3, 0.9, 1.0),
            active_speaker: None,
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
                    check_match_start,
                    update_ctf_flags_and_scoring,
                    update_flag_visual_transforms,
                    update_hologram_visuals,
                    play_voice_callout_audio,
                )
                    .chain(),
            );
    }
}

fn check_match_start(mut koth: ResMut<KothManager>) {
    if !koth.match_started && crate::types::is_match_started() {
        koth.match_started = true;
        koth.status_banner =
            "ANDERS [BLUE]: «Capture the Red Flag on the East Platform and bring it to Blue Base!»"
                .to_string();
        koth.status_color = Color::srgb(0.3, 0.9, 1.0);
        koth.active_speaker = Some(("Anders".to_string(), Team::Blue));
        koth.event_notice = Some(VoiceCallout::new(
            "Anders",
            Team::Blue,
            "match_start",
            "Capture the Red Flag on the East Platform and bring it to Blue Base!",
            Color::srgb(0.3, 0.9, 1.0),
            5.0,
        ));
    }
}

fn setup_hologram_beacon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (_idx, _, center, _radius) = PLATEAU_COORDS[0];

    // Subtle low-profile glowing ground indicator ring around the active objective
    let ring_mesh = meshes.add(Torus {
        minor_radius: 0.04,
        major_radius: 1.35,
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
        base_color: Color::srgb(0.45, 0.48, 0.52),
        metallic: 0.85,
        perceptual_roughness: 0.35,
        ..default()
    });

    let blue_beacon_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.85, 1.0),
        emissive: LinearRgba::new(1.5, 6.0, 12.0, 1.0),
        unlit: true,
        ..default()
    });

    let red_beacon_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.35, 0.25),
        emissive: LinearRgba::new(12.0, 1.5, 0.8, 1.0),
        unlit: true,
        ..default()
    });

    let blue_flag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.42, 0.78),
        emissive: LinearRgba::new(0.08, 0.45, 1.2, 1.0),
        perceptual_roughness: 0.65,
        cull_mode: None,
        ..default()
    });

    let red_flag_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.16, 0.14),
        emissive: LinearRgba::new(1.2, 0.12, 0.08, 1.0),
        perceptual_roughness: 0.65,
        cull_mode: None,
        ..default()
    });

    let blue_home = PLATEAU_COORDS[1].2;
    let red_home = PLATEAU_COORDS[2].2;

    spawn_single_ctf_flag(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &blue_beacon_mat,
        &blue_flag_mat,
        Team::Blue,
        blue_home,
        1.0,
    );

    spawn_single_ctf_flag(
        &mut commands,
        &mut meshes,
        &steel_mat,
        &red_beacon_mat,
        &red_flag_mat,
        Team::Red,
        red_home,
        1.0,
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
                Mesh3d(meshes.add(Cylinder::new(0.065, 4.6))),
                MeshMaterial3d(pole_mat.clone()),
                Transform::from_xyz(0.0, 2.3, 0.0),
            ));
            // Glowing beacon bulb at the top of the flagpole (matching art/scene/post_apocalyptic_scene.jpg)
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.22))),
                MeshMaterial3d(finial_mat.clone()),
                Transform::from_xyz(0.0, 4.65, 0.0),
            ));
            // Tattered post-apocalyptic cloth banner (main body + ragged tail strips)
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.6, 1.15, 0.05))),
                MeshMaterial3d(banner_mat.clone()),
                Transform::from_xyz(dir_sign * 0.85, 3.85, 0.0)
                    .with_rotation(Quat::from_rotation_z(-0.08)),
            ));
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.75, 0.42, 0.04))),
                MeshMaterial3d(banner_mat.clone()),
                Transform::from_xyz(dir_sign * 1.85, 4.10, 0.04)
                    .with_rotation(Quat::from_rotation_z(-0.14)),
            ));
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.65, 0.36, 0.04))),
                MeshMaterial3d(banner_mat.clone()),
                Transform::from_xyz(dir_sign * 1.78, 3.52, -0.03)
                    .with_rotation(Quat::from_rotation_z(-0.18)),
            ));
        });
}

fn update_ctf_flags_and_scoring(
    time: Res<Time>,
    mut koth: ResMut<KothManager>,
    mut flags: Query<&mut TeamFlag>,
    characters: Query<(Entity, &Transform, &Character, &StaggerState)>,
) {
    if !koth.match_started {
        return;
    }

    let dt = time.delta_secs();

    // 1. Match countdown timer
    if koth.match_timer > 0.0 && koth.winner.is_none() {
        koth.match_timer = (koth.match_timer - dt).max(0.0);
        if koth.match_timer <= 0.0 {
            if koth.blue_score >= koth.red_score {
                koth.winner = Some(Team::Blue);
                koth.event_notice = Some(VoiceCallout::new(
                    "Anders",
                    Team::Blue,
                    "victory",
                    "Victory! Blue Team won Capture the Flag!",
                    Color::srgb(0.2, 0.95, 0.55),
                    8.0,
                ));
            } else {
                koth.winner = Some(Team::Red);
                koth.event_notice = Some(VoiceCallout::new(
                    "Siri",
                    Team::Red,
                    "victory",
                    "Victory for Red Team! We own this arena!",
                    Color::srgb(1.0, 0.25, 0.25),
                    8.0,
                ));
            }
        }
    }

    // Tick event notice banner timer
    if let Some(ref mut callout) = koth.event_notice {
        callout.timer -= dt;
        if callout.timer <= 0.0 {
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
                            let spoken = match carrier_name.to_lowercase().as_str() {
                                "anders" => "I dropped the Red Flag! Someone grab it!",
                                "morten" => "I lost the Red Flag! Pick it up!",
                                "jonas" => "Dropped the Red Flag! Recover it fast!",
                                "siri" => "Ugh, I dropped the Blue Flag! Recover it!",
                                "lisa" => "I lost the Blue Flag! Don't let them touch it!",
                                "per" => "Dropped the Blue Flag! Hold them back!",
                                _ => match flag.team {
                                    Team::Blue => "I dropped the Blue Flag! Recover it!",
                                    Team::Red => "I dropped the Red Flag! Pick it up!",
                                },
                            };
                            koth.event_notice = Some(VoiceCallout::new(
                                carrier_name.clone(),
                                *carrier_team,
                                "dropped_flag",
                                spoken,
                                Color::srgb(1.0, 0.85, 0.2),
                                3.2,
                            ));
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
                                let spoken = match carrier_name.to_lowercase().as_str() {
                                    "anders" => "Flag captured! Plus one point for Blue Team!",
                                    "morten" => "Touchdown! I captured the Red Flag for Blue Team!",
                                    "jonas" => "Red Flag delivered! Point for Blue Team!",
                                    _ => "Flag captured! Plus one Blue point!",
                                };
                                koth.event_notice = Some(VoiceCallout::new(
                                    carrier_name.clone(),
                                    Team::Blue,
                                    "captured_flag",
                                    spoken,
                                    Color::srgb(0.2, 1.0, 0.5),
                                    4.5,
                                ));
                                if koth.blue_score >= WIN_SCORE {
                                    koth.winner = Some(Team::Blue);
                                    koth.event_notice = Some(VoiceCallout::new(
                                        "Anders",
                                        Team::Blue,
                                        "victory",
                                        "Victory! Blue Team won Capture the Flag!",
                                        Color::srgb(0.2, 0.95, 0.55),
                                        8.0,
                                    ));
                                }
                            }
                            Team::Red => {
                                koth.red_score += 1.0;
                                let spoken = match carrier_name.to_lowercase().as_str() {
                                    "siri" => "I captured the Blue Flag! Point for Red Team!",
                                    "lisa" => "Score! Another flag capture for Red Team!",
                                    "per" => "Blue Flag delivered to Red Base! Point scored!",
                                    _ => "We captured the Blue Flag! Plus one Red point!",
                                };
                                koth.event_notice = Some(VoiceCallout::new(
                                    carrier_name.clone(),
                                    Team::Red,
                                    "captured_flag",
                                    spoken,
                                    Color::srgb(1.0, 0.25, 0.25),
                                    4.5,
                                ));
                                if koth.red_score >= WIN_SCORE {
                                    koth.winner = Some(Team::Red);
                                    koth.event_notice = Some(VoiceCallout::new(
                                        "Siri",
                                        Team::Red,
                                        "victory",
                                        "Victory for Red Team! We own this arena!",
                                        Color::srgb(1.0, 0.25, 0.25),
                                        8.0,
                                    ));
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
                    let (speaker, speaker_team, spoken) = match flag.team {
                        Team::Blue => (
                            "Jonas",
                            Team::Blue,
                            "Blue Flag automatically returned to base!",
                        ),
                        Team::Red => (
                            "Per",
                            Team::Red,
                            "Red Flag automatically returned to base!",
                        ),
                    };
                    koth.event_notice = Some(VoiceCallout::new(
                        speaker,
                        speaker_team,
                        "flag_auto_returned",
                        spoken,
                        Color::srgb(0.4, 0.85, 1.0),
                        3.0,
                    ));
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
                            let spoken = match c_name.to_lowercase().as_str() {
                                "anders" => "I returned our Blue Flag to base!",
                                "morten" => "Blue Flag returned to base! We're safe!",
                                "jonas" => "I recovered our Blue Flag! Base is secure!",
                                "siri" => "I returned the Red Flag to our base!",
                                "lisa" => "Red Flag is back at base!",
                                "per" => "I secured the Red Flag! Back at base!",
                                _ => "Flag returned to base!",
                            };
                            koth.event_notice = Some(VoiceCallout::new(
                                c_name.clone(),
                                *c_team,
                                "returned_flag",
                                spoken,
                                Color::srgb(0.3, 0.95, 0.6),
                                3.5,
                            ));
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
                                let spoken = match c_name.to_lowercase().as_str() {
                                    "siri" => "I stole the Blue Flag! Catch me if you can!",
                                    "lisa" => "Blue Flag is mine! Sprinting to Red Base!",
                                    "per" => "I have the Blue Flag! Clear the way to Red Base!",
                                    _ => "I took the Blue Flag! Run to Red Base!",
                                };
                                koth.event_notice = Some(VoiceCallout::new(
                                    c_name.clone(),
                                    Team::Red,
                                    "took_flag",
                                    spoken,
                                    Color::srgb(1.0, 0.22, 0.22),
                                    4.0,
                                ));
                            }
                            Team::Red => {
                                blue_carrier_info = Some((*ent, c_name.clone()));
                                let spoken = match c_name.to_lowercase().as_str() {
                                    "anders" => "I've got the Red Flag! Cover me back to Blue Base!",
                                    "morten" => "I took the Red Flag! Run to Blue Base!",
                                    "jonas" => "Red Flag secured! Heading back to Blue Base!",
                                    _ => "I took the Red Flag! Run to Blue Base!",
                                };
                                koth.event_notice = Some(VoiceCallout::new(
                                    c_name.clone(),
                                    Team::Blue,
                                    "took_flag",
                                    spoken,
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

    // Determine primary status headline and active speaker when no short event notice is overriding
    if let Some((banner, color, spk)) = koth.event_notice.as_ref().map(|c| {
        (
            c.formatted_banner(),
            c.color,
            (c.speaker.clone(), c.speaker_team),
        )
    }) {
        koth.status_banner = banner;
        koth.status_color = color;
        koth.active_speaker = Some(spk);
    } else if let (Some((_, b_name)), Some((_, r_name))) = (&blue_carrier_info, &red_carrier_info)
    {
        koth.contest_state = ZoneContestState::Contested;
        koth.active_speaker = Some(("Jonas".to_string(), Team::Blue));
        koth.status_banner = format!(
            "JONAS [BLUE]: «Both flags stolen! Protect {} and tackle {}!»",
            b_name, r_name
        );
        koth.status_color = Color::srgb(1.0, 0.85, 0.15);
    } else if let Some((_, r_name)) = &red_carrier_info {
        koth.contest_state = ZoneContestState::RedControlling;
        koth.active_speaker = Some(("Anders".to_string(), Team::Blue));
        koth.status_banner = format!(
            "ANDERS [BLUE]: «Our flag was stolen by {}! Tackle them!»",
            r_name
        );
        koth.status_color = Color::srgb(1.0, 0.22, 0.22);
    } else if let Some((_, b_name)) = &blue_carrier_info {
        koth.contest_state = ZoneContestState::BlueControlling;
        let speaker = if b_name.eq_ignore_ascii_case("anders") {
            "Morten"
        } else {
            "Anders"
        };
        koth.active_speaker = Some((speaker.to_string(), Team::Blue));
        koth.status_banner = format!(
            "{} [BLUE]: «{} has the Red Flag! Clear the way to Blue Base!»",
            speaker.to_uppercase(),
            b_name
        );
        koth.status_color = Color::srgb(0.2, 0.95, 0.45);
    } else {
        koth.contest_state = ZoneContestState::Neutral;
        koth.active_speaker = Some(("Anders".to_string(), Team::Blue));
        koth.status_banner =
            "ANDERS [BLUE]: «Capture the Red Flag on the East Platform and bring it to Blue Base!»"
                .to_string();
        koth.status_color = Color::srgb(0.75, 0.88, 1.0);
    }
}

fn update_flag_visual_transforms(
    mut flags: Query<(&TeamFlag, &mut Transform, &mut Visibility)>,
    characters: Query<&Transform, (With<Character>, Without<TeamFlag>)>,
) {
    for (flag, mut flag_transform, mut vis) in flags.iter_mut() {
        match flag.state {
            FlagState::AtBase => {
                // The 3D battlefield model (.glb) renders the sculpted flagpole at base
                *vis = Visibility::Hidden;
                flag_transform.translation = flag.home_pos;
                flag_transform.rotation = Quat::IDENTITY;
                flag_transform.scale = Vec3::ONE;
            }
            FlagState::Dropped { pos, .. } => {
                *vis = Visibility::Inherited;
                flag_transform.translation = pos;
                // Slightly tilted flagpole stuck into the ground when dropped
                flag_transform.rotation = Quat::from_rotation_z(0.22);
                flag_transform.scale = Vec3::splat(0.85);
            }
            FlagState::CarriedBy(carrier_ent) => {
                *vis = Visibility::Inherited;
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

/// Plays the corresponding audio file for active character voice callouts
fn play_voice_callout_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut koth: ResMut<KothManager>,
) {
    if let Some(ref mut callout) = koth.event_notice {
        if !callout.played_audio {
            callout.played_audio = true;
            let sound_path = callout.audio_asset_path();
            commands.spawn((
                AudioPlayer::new(asset_server.load(sound_path)),
                PlaybackSettings::DESPAWN.with_volume(Volume::new(1.0)),
            ));
        }
    }
}
