use bevy::prelude::*;

use crate::koth::KothManager;
use crate::player::Player;
use crate::types::{Character, Health, Stamina, TackleAbility, Team};

#[derive(Component)]
pub struct BlueScoreText;

#[derive(Component)]
pub struct RedScoreText;

#[derive(Component)]
pub struct ZoneStatusText;

#[derive(Component)]
pub struct ZoneTimerText;

#[derive(Component)]
pub struct ZoneNameText;

#[derive(Component)]
pub struct StaminaFillBar;

#[derive(Component)]
pub struct StaminaLabelText;

#[derive(Component)]
pub struct HpFillBar;

#[derive(Component)]
pub struct HpLabelText;

#[derive(Component)]
pub struct TackleCdText;

#[derive(Component)]
pub struct OverheadRoot;

#[derive(Component)]
pub struct CharacterNametag {
    pub target: Entity,
    pub hp_bar: Entity,
    pub stamina_bar: Entity,
}

#[derive(Component)]
pub struct NametagHpFill;

#[derive(Component)]
pub struct NametagStaminaFill;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (
                    sync_overhead_nametags,
                    update_overhead_nametags,
                    update_hud,
                ),
            );
    }
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ui_font: Handle<Font> = asset_server.load("fonts/Ubuntu-B.ttf");

    // 1. Root full-screen container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|root| {
            // --- TOP MATCH BANNER (Matching reference art: Blue Flag | 07:42 | Red Flag) ---
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|top_section| {
                // Tactical Angled Segmented Banner
                top_section
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    })
                    .insert(BorderColor(Color::srgba(0.0, 0.7, 1.0, 0.4)))
                    .insert(BorderRadius::all(Val::Px(6.0)))
                    .with_children(|banner_row| {
                        // Blue Team Banner (Left)
                        banner_row
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.08, 0.24, 0.52, 0.95)),
                            ))
                            .with_children(|blue_box| {
                                blue_box.spawn((
                                    Text::new("BLÅ FLAGG: 1"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 22.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.2, 0.85, 1.0)),
                                    BlueScoreText,
                                ));
                            });

                        // Match Timer & Zone Center Box (Center)
                        banner_row
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(24.0), Val::Px(8.0)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.98)),
                            ))
                            .with_children(|center_box| {
                                center_box.spawn((
                                    Text::new("07:42"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 26.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    ZoneTimerText,
                                ));
                                center_box.spawn((
                                    Text::new("SONE ALPHA"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.65, 0.75, 0.85)),
                                    ZoneNameText,
                                ));
                            });

                        // Red Team Banner (Right)
                        banner_row
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.55, 0.10, 0.10, 0.95)),
                            ))
                            .with_children(|red_box| {
                                red_box.spawn((
                                    Text::new("RØD FLAGG: 0"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 22.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(1.0, 0.35, 0.35)),
                                    RedScoreText,
                                ));
                            });
                    });

                // PROMINENT DYNAMIC WARNING ALERT (Matching «VÅRT FLAGG ER STJÅLET!»)
                top_section.spawn((
                    Text::new("«VÅRT FLAGG ER STJÅLET!»"),
                    TextFont {
                        font: ui_font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.22, 0.22)),
                    ZoneStatusText,
                ));
            });

            // Center Aim Reticle
            root.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.9, 1.0, 0.4)),
                BorderColor(Color::srgb(0.0, 0.9, 1.0)),
            ));

            // --- BOTTOM ROW: Operator Card (Left) & Tactical Abilities (Right) ---
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            })
            .with_children(|bottom_row| {
                // BOTTOM-LEFT OPERATOR CARD (Anders portrait, HP 100 green, STAMINA 75 yellow)
                bottom_row
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.04, 0.07, 0.11, 0.9)),
                        BorderColor(Color::srgb(0.0, 0.75, 1.0)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|card| {
                        // Anders Portrait Frame
                        card.spawn((
                            Node {
                                width: Val::Px(78.0),
                                height: Val::Px(94.0),
                                border: UiRect::all(Val::Px(1.5)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BorderColor(Color::srgb(0.0, 0.85, 1.0)),
                            BorderRadius::all(Val::Px(6.0)),
                            ImageNode::new(asset_server.load("ui/portrait_anders.png")),
                        ));

                        // Anders Stats Column
                        card.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
                            min_width: Val::Px(150.0),
                            ..default()
                        })
                        .with_children(|stats_col| {
                            stats_col.spawn((
                                Text::new("ANDERS"),
                                TextFont {
                                    font: ui_font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // HP Bar Row
                            stats_col.spawn((
                                Text::new("HP: 100"),
                                TextFont {
                                    font: ui_font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.2, 0.95, 0.4)),
                                HpLabelText,
                            ));
                            stats_col
                                .spawn((
                                    Node {
                                        width: Val::Px(140.0),
                                        height: Val::Px(11.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.04, 0.08, 0.05)),
                                    BorderColor(Color::srgb(0.15, 0.85, 0.35)),
                                    BorderRadius::all(Val::Px(2.0)),
                                ))
                                .with_children(|bar_bg| {
                                    bar_bg.spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.12, 0.88, 0.35)),
                                        HpFillBar,
                                    ));
                                });

                            // STAMINA Bar Row
                            stats_col.spawn((
                                Text::new("STAMINA: 75"),
                                TextFont {
                                    font: ui_font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                                StaminaLabelText,
                            ));
                            stats_col
                                .spawn((
                                    Node {
                                        width: Val::Px(140.0),
                                        height: Val::Px(11.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.08, 0.07, 0.03)),
                                    BorderColor(Color::srgb(0.9, 0.75, 0.15)),
                                    BorderRadius::all(Val::Px(2.0)),
                                ))
                                .with_children(|bar_bg| {
                                    bar_bg.spawn((
                                        Node {
                                            width: Val::Percent(75.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.96, 0.76, 0.12)),
                                        StaminaFillBar,
                                    ));
                                });
                        });
                    });

                // BOTTOM-RIGHT TACTICAL ACTION INDICATORS ('Hastighetsboost' & 'Kastbart nett (x2)')
                bottom_row
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|actions| {
                        // Kastbart nett (x2) / Tackle Ability
                        actions
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(4.0),
                                    padding: UiRect::all(Val::Px(6.0)),
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.04, 0.07, 0.11, 0.88)),
                                BorderColor(Color::srgb(0.0, 0.75, 1.0)),
                                BorderRadius::all(Val::Px(6.0)),
                            ))
                            .with_children(|box_node| {
                                box_node.spawn((
                                    Node {
                                        width: Val::Px(56.0),
                                        height: Val::Px(56.0),
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    BorderRadius::all(Val::Px(4.0)),
                                    ImageNode::new(asset_server.load("ui/icon_net.png")),
                                ));
                                box_node.spawn((
                                    Text::new("Kastbart nett (x2)"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                                    TackleCdText,
                                ));
                            });

                        // Hastighetsboost / Sprint Ability
                        actions
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(4.0),
                                    padding: UiRect::all(Val::Px(6.0)),
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.04, 0.07, 0.11, 0.88)),
                                BorderColor(Color::srgb(0.85, 0.2, 0.2)),
                                BorderRadius::all(Val::Px(6.0)),
                            ))
                            .with_children(|box_node| {
                                box_node.spawn((
                                    Node {
                                        width: Val::Px(56.0),
                                        height: Val::Px(56.0),
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    BorderRadius::all(Val::Px(4.0)),
                                    ImageNode::new(asset_server.load("ui/icon_boost.png")),
                                ));
                                box_node.spawn((
                                    Text::new("Hastighetsboost"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                                ));
                            });
                    });
            });
        });

    // 2. Full-screen root container for Floating Overhead Nametags
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        PickingBehavior::IGNORE,
        OverheadRoot,
    ));
}

fn sync_overhead_nametags(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    overhead_root_query: Query<Entity, With<OverheadRoot>>,
    new_chars: Query<(Entity, &Character), Added<Character>>,
) {
    let Ok(root_ent) = overhead_root_query.get_single() else {
        return;
    };
    let ui_font: Handle<Font> = asset_server.load("fonts/Ubuntu-B.ttf");

    for (char_ent, char_info) in new_chars.iter() {
        let is_blue = char_info.team == Team::Blue;
        let border_c = if is_blue {
            Color::srgb(0.1, 0.65, 1.0)
        } else {
            Color::srgb(1.0, 0.25, 0.25)
        };
        let text_c = if is_blue {
            Color::srgb(0.3, 0.85, 1.0)
        } else {
            Color::srgb(1.0, 0.4, 0.4)
        };

        // Spawn HP fill and Stamina fill entities
        let mut hp_bar_ent = Entity::PLACEHOLDER;
        let mut stamina_bar_ent = Entity::PLACEHOLDER;

        let tag_entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.88)),
                BorderColor(border_c),
                BorderRadius::all(Val::Px(4.0)),
                PickingBehavior::IGNORE,
            ))
            .with_children(|parent| {
                // Nametag Text (e.g. "ANDERS [BLÅ]", "SIRI [RØD]")
                parent.spawn((
                    Text::new(format!(
                        "{} [{}]",
                        char_info.name.to_uppercase(),
                        char_info.team.name()
                    )),
                    TextFont {
                        font: ui_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(text_c),
                ));

                // Mini Health & Stamina Row
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(3.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|bars_row| {
                        // Mini HP Bar Container
                        bars_row
                            .spawn((
                                Node {
                                    width: Val::Px(56.0),
                                    height: Val::Px(5.0),
                                    padding: UiRect::all(Val::Px(0.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                                BorderRadius::all(Val::Px(1.0)),
                            ))
                            .with_children(|hp_box| {
                                hp_bar_ent = hp_box
                                    .spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.15, 0.9, 0.35)),
                                        NametagHpFill,
                                    ))
                                    .id();
                            });

                        // Mini Stamina Bar Container
                        bars_row
                            .spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(5.0),
                                    padding: UiRect::all(Val::Px(0.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                                BorderRadius::all(Val::Px(1.0)),
                            ))
                            .with_children(|stam_box| {
                                stamina_bar_ent = stam_box
                                    .spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.95, 0.8, 0.15)),
                                        NametagStaminaFill,
                                    ))
                                    .id();
                            });
                    });
            })
            .id();

        // Attach tag to OverheadRoot and register CharacterNametag component
        commands.entity(root_ent).add_child(tag_entity);
        commands.entity(tag_entity).insert(CharacterNametag {
            target: char_ent,
            hp_bar: hp_bar_ent,
            stamina_bar: stamina_bar_ent,
        });
    }
}

fn update_overhead_nametags(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    char_query: Query<(
        Entity,
        &GlobalTransform,
        &Character,
        Option<&Health>,
        Option<&Stamina>,
    )>,
    mut tag_query: Query<(Entity, &mut Node, &CharacterNametag)>,
    mut fill_nodes: Query<&mut Node, (Without<CharacterNametag>, Without<HpFillBar>, Without<StaminaFillBar>)>,
    mut commands: Commands,
) {
    let Ok((camera, cam_trans)) = camera_query.get_single() else {
        return;
    };

    for (tag_ent, mut tag_node, tag_data) in tag_query.iter_mut() {
        let Ok((_char_ent, char_trans, _char_info, maybe_hp, maybe_stam)) =
            char_query.get(tag_data.target)
        else {
            // Despawn orphaned nametag
            commands.entity(tag_ent).despawn_recursive();
            continue;
        };

        // Position tag ~1.45m above character center
        let head_world = char_trans.translation() + Vec3::new(0.0, 1.45, 0.0);

        // Check if behind camera
        if let Some(ndc) = camera.world_to_ndc(cam_trans, head_world) {
            if ndc.z < 0.0 || ndc.z > 1.0 {
                tag_node.display = Display::None;
                continue;
            }
        }

        // Project to screen viewport
        if let Ok(viewport_pos) = camera.world_to_viewport(cam_trans, head_world) {
            tag_node.display = Display::Flex;
            // Center horizontally (half estimated tag width 48px) and position slightly above head
            tag_node.left = Val::Px(viewport_pos.x - 48.0);
            tag_node.top = Val::Px(viewport_pos.y - 32.0);

            // Update mini health fill
            if let Some(health) = maybe_hp {
                if let Ok(mut hp_fill) = fill_nodes.get_mut(tag_data.hp_bar) {
                    let pct = (health.current / health.max * 100.0).clamp(0.0, 100.0);
                    hp_fill.width = Val::Percent(pct);
                }
            }

            // Update mini stamina fill
            if let Some(stamina) = maybe_stam {
                if let Ok(mut stam_fill) = fill_nodes.get_mut(tag_data.stamina_bar) {
                    let pct = (stamina.current / stamina.max * 100.0).clamp(0.0, 100.0);
                    stam_fill.width = Val::Percent(pct);
                }
            }
        } else {
            tag_node.display = Display::None;
        }
    }
}

fn update_hud(
    koth: Res<KothManager>,
    player_query: Query<(&Health, &Stamina, &TackleAbility), With<Player>>,
    mut blue_text: Query<
        &mut Text,
        (
            With<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<TackleCdText>,
            Without<ZoneNameText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut red_text: Query<
        &mut Text,
        (
            With<RedScoreText>,
            Without<BlueScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<TackleCdText>,
            Without<ZoneNameText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut timer_text: Query<
        &mut Text,
        (
            With<ZoneTimerText>,
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneStatusText>,
            Without<TackleCdText>,
            Without<ZoneNameText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut name_text: Query<
        &mut Text,
        (
            With<ZoneNameText>,
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<TackleCdText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut status_text: Query<
        (&mut Text, &mut TextColor),
        (
            With<ZoneStatusText>,
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<TackleCdText>,
            Without<ZoneNameText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut tackle_text: Query<
        (&mut Text, &mut TextColor),
        (
            With<TackleCdText>,
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<ZoneNameText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut hp_label: Query<
        &mut Text,
        (
            With<HpLabelText>,
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<ZoneNameText>,
            Without<TackleCdText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut stam_label: Query<
        &mut Text,
        (
            With<StaminaLabelText>,
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<ZoneNameText>,
            Without<TackleCdText>,
            Without<HpLabelText>,
        ),
    >,
    mut stamina_bar: Query<&mut Node, (With<StaminaFillBar>, Without<HpFillBar>)>,
    mut hp_bar: Query<&mut Node, (With<HpFillBar>, Without<StaminaFillBar>)>,
) {
    // 1. Top Match Banner Scores
    if let Ok(mut text) = blue_text.get_single_mut() {
        **text = format!("BLÅ FLAGG: {:.0}", koth.blue_score);
    }
    if let Ok(mut text) = red_text.get_single_mut() {
        **text = format!("RØD FLAGG: {:.0}", koth.red_score);
    }

    // 2. Top Match Timer (Formatted MM:SS matching 07:42)
    let mins = (koth.match_timer / 60.0) as u32;
    let secs = (koth.match_timer % 60.0) as u32;
    if let Ok(mut text) = timer_text.get_single_mut() {
        **text = format!("{:02}:{:02}", mins, secs);
    }

    // Active CTF Objective Subtitle
    if let Ok(mut text) = name_text.get_single_mut() {
        if koth.blue_carrier.is_some() {
            **text = "LEVER RØDT FLAGG TIL BLÅ BASE!".to_string();
        } else if !koth.blue_flag_at_base {
            **text = "TAKLE FIENDEN FOR Å REDDE BLÅTT FLAGG!".to_string();
        } else {
            **text = "CAPTURE THE FLAG (FØRSTE TIL 5)".to_string();
        }
    }

    // 3. Dynamic CTF Warning & Event Alert
    if let Ok((mut text, mut color)) = status_text.get_single_mut() {
        **text = koth.status_banner.clone();
        color.0 = koth.status_color;

        if let Some(winner) = koth.winner {
            match winner {
                Team::Blue => {
                    **text = "«SEIER! BLÅTT LAG VANT CAPTURE THE FLAG!»".to_string();
                    color.0 = Color::srgb(0.2, 1.0, 0.6);
                }
                Team::Red => {
                    **text = "«NEDERLAG! RØDT LAG VANT CAPTURE THE FLAG!»".to_string();
                    color.0 = Color::srgb(1.0, 0.2, 0.2);
                }
            }
        }
    }

    // 4. Player Operator Card & Tactical Actions
    if let Ok((health, stamina, tackle)) = player_query.get_single() {
        // Anders HP Bar & Text
        if let Ok(mut label) = hp_label.get_single_mut() {
            **label = format!("HP: {:.0}", health.current);
        }
        if let Ok(mut bar_node) = hp_bar.get_single_mut() {
            let pct = (health.current / health.max * 100.0).clamp(0.0, 100.0);
            bar_node.width = Val::Percent(pct);
        }

        // Anders Stamina Bar & Text
        if let Ok(mut label) = stam_label.get_single_mut() {
            **label = format!("STAMINA: {:.0}", stamina.current);
        }
        if let Ok(mut bar_node) = stamina_bar.get_single_mut() {
            let pct = (stamina.current / stamina.max * 100.0).clamp(0.0, 100.0);
            bar_node.width = Val::Percent(pct);
        }

        // Bottom-Right Kastbart Nett Action Indicator
        if let Ok((mut text, mut color)) = tackle_text.get_single_mut() {
            if tackle.cooldown > 0.0 {
                **text = format!("Kastbart nett ({:.1}s)", tackle.cooldown);
                color.0 = Color::srgb(0.7, 0.7, 0.7);
            } else if stamina.current < 25.0 {
                **text = "Lav stamina for nett".to_string();
                color.0 = Color::srgb(1.0, 0.5, 0.3);
            } else {
                **text = "Kastbart nett (x2)".to_string();
                color.0 = Color::srgb(0.9, 0.95, 1.0);
            }
        }
    }
}
