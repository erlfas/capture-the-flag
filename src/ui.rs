use avian3d::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::prelude::*;

use crate::koth::{FlagState, KothManager, TeamFlag};
use crate::player::Player;
use crate::types::{
    Character, Health, PauseState, StaggerState, Stamina, TackleAbility, Team,
};

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
    pub label_text: Entity,
    pub hp_bar: Entity,
    pub stamina_bar: Entity,
}

#[derive(Component)]
pub struct NametagHpFill;

#[derive(Component)]
pub struct NametagStaminaFill;

#[derive(Component)]
pub struct HudPauseButton;

#[derive(Component)]
pub struct PauseOverlayRoot;

#[derive(Component)]
pub struct PauseModalCard;

#[derive(Component)]
pub struct ExitModalCard;

#[derive(Component)]
pub struct PauseMatchSummaryText;

#[derive(Component)]
pub struct MusicToggleButtonText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuAction {
    Resume,
    RestartMatch,
    ToggleMusic,
    ExitGame,
    StartNewMatchFromExit,
    ReplayIntroFromExit,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (
                    handle_pause_input_and_menu,
                    update_pause_overlay_ui,
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
                                    Text::new("BLUE FLAG: 1"),
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
                                    Text::new("ZONE ALPHA"),
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
                                    Text::new("RED FLAG: 0"),
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

                // PROMINENT DYNAMIC WARNING ALERT (Matching «OUR FLAG IS STOLEN!»)
                top_section.spawn((
                    Text::new("«OUR FLAG IS STOLEN!»"),
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
                                    Text::new("Throwable Net (x2)"),
                                    TextFont {
                                        font: ui_font.clone(),
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                                    TackleCdText,
                                ));
                            });

                        // Speed Boost / Sprint Ability
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
                                    Text::new("Speed Boost"),
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

    // 3. Top-Right HUD Pause Button ("PAUSE [ESC]")
    commands
        .spawn((
            Button,
            HudPauseButton,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                right: Val::Px(22.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.5)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.08, 0.14, 0.90)),
            BorderColor(Color::srgba(0.0, 0.85, 1.0, 0.65)),
            BorderRadius::all(Val::Px(6.0)),
            ZIndex(20),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("❚❚ PAUSE [ESC]"),
                TextFont {
                    font: ui_font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.92, 1.0)),
            ));
        });

    // 4. Full-Screen Pause & Exit Menu Overlay (Hidden until Esc or Pause button pressed)
    spawn_pause_overlay(&mut commands, &ui_font);
}

fn spawn_pause_menu_button(
    parent: &mut ChildBuilder,
    ui_font: &Handle<Font>,
    label: &str,
    action: PauseMenuAction,
    is_danger: bool,
    is_music_btn: bool,
) {
    let (bg_color, border_color, text_color) = if is_danger {
        (
            Color::srgba(0.32, 0.06, 0.08, 0.92),
            Color::srgb(0.95, 0.25, 0.28),
            Color::srgb(1.0, 0.45, 0.45),
        )
    } else {
        (
            Color::srgba(0.06, 0.14, 0.24, 0.92),
            Color::srgb(0.0, 0.78, 1.0),
            Color::srgb(0.88, 0.96, 1.0),
        )
    };

    parent
        .spawn((
            Button,
            action,
            Node {
                width: Val::Px(320.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(13.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.8)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|btn| {
            let mut text_cmd = btn.spawn((
                Text::new(label),
                TextFont {
                    font: ui_font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(text_color),
            ));
            if is_music_btn {
                text_cmd.insert(MusicToggleButtonText);
            }
        });
}

fn spawn_pause_overlay(commands: &mut Commands, ui_font: &Handle<Font>) {
    commands
        .spawn((
            PauseOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.84)),
            ZIndex(100),
        ))
        .with_children(|overlay| {
            // --- A) PAUSE MENU MODAL CARD ---
            overlay
                .spawn((
                    PauseModalCard,
                    Node {
                        width: Val::Px(440.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::axes(Val::Px(36.0), Val::Px(32.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.04, 0.08, 0.14, 0.97)),
                    BorderColor(Color::srgb(0.0, 0.85, 1.0)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|card| {
                    // Header Title
                    card.spawn((
                        Text::new("GAME PAUSED"),
                        TextFont {
                            font: ui_font.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.0, 0.92, 1.0)),
                    ));

                    // Live Match Score & Timer Summary
                    card.spawn((
                        Text::new("BLUE FLAG: 1   •   07:42   •   RED FLAG: 0"),
                        TextFont {
                            font: ui_font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.72, 0.84, 0.94)),
                        PauseMatchSummaryText,
                    ));

                    // Divider line
                    card.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.85, 1.0, 0.30)),
                    ));

                    // Menu Buttons
                    spawn_pause_menu_button(
                        card,
                        ui_font,
                        "RESUME GAME  [ESC]",
                        PauseMenuAction::Resume,
                        false,
                        false,
                    );
                    spawn_pause_menu_button(
                        card,
                        ui_font,
                        "RESTART MATCH  [R]",
                        PauseMenuAction::RestartMatch,
                        false,
                        false,
                    );
                    spawn_pause_menu_button(
                        card,
                        ui_font,
                        "MUSIC: ON  [M]",
                        PauseMenuAction::ToggleMusic,
                        false,
                        true,
                    );
                    spawn_pause_menu_button(
                        card,
                        ui_font,
                        "EXIT GAME  [Q]",
                        PauseMenuAction::ExitGame,
                        true,
                        false,
                    );

                    // Controls Footer inside Pause Menu
                    card.spawn((
                        Node {
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        Text::new("WASD: Move  •  SHIFT: Sprint  •  SPACE: Jump  •  LMB/F: Tackle  •  TAB/V: View"),
                        TextFont {
                            font: ui_font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.65, 0.76)),
                    ));
                });

            // --- B) EXIT CONFIRMATION / MAIN MENU SCREEN CARD ---
            overlay
                .spawn((
                    ExitModalCard,
                    Node {
                        width: Val::Px(460.0),
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(16.0),
                        padding: UiRect::axes(Val::Px(36.0), Val::Px(34.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.04, 0.06, 0.11, 0.98)),
                    BorderColor(Color::srgb(0.95, 0.25, 0.28)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new("MATCH EXITED"),
                        TextFont {
                            font: ui_font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.35, 0.35)),
                    ));

                    card.spawn((
                        Text::new("You have left the Cyber-Athletic Coliseum arena.\nYou may close this tab or launch a new match below."),
                        TextFont {
                            font: ui_font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor(Color::srgb(0.75, 0.82, 0.90)),
                    ));

                    spawn_pause_menu_button(
                        card,
                        ui_font,
                        "START NEW MATCH",
                        PauseMenuAction::StartNewMatchFromExit,
                        false,
                        false,
                    );
                    spawn_pause_menu_button(
                        card,
                        ui_font,
                        "RETURN TO INTRO CINEMATIC",
                        PauseMenuAction::ReplayIntroFromExit,
                        false,
                        false,
                    );
                });
        });
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

        // Spawn label, HP fill, and Stamina fill entities
        let mut label_text_ent = Entity::PLACEHOLDER;
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
                // Nametag Text (e.g. "ANDERS [BLUE]", "SIRI [RED]")
                label_text_ent = parent
                    .spawn((
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
                    ))
                    .id();

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
            label_text: label_text_ent,
            hp_bar: hp_bar_ent,
            stamina_bar: stamina_bar_ent,
        });
    }
}

fn update_overhead_nametags(
    koth: Res<KothManager>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    char_query: Query<(
        Entity,
        &GlobalTransform,
        &Character,
        Option<&Health>,
        Option<&Stamina>,
    )>,
    mut tag_query: Query<(Entity, &mut Node, &CharacterNametag)>,
    mut label_texts: Query<
        &mut Text,
        (
            Without<BlueScoreText>,
            Without<RedScoreText>,
            Without<ZoneTimerText>,
            Without<ZoneStatusText>,
            Without<TackleCdText>,
            Without<ZoneNameText>,
            Without<HpLabelText>,
            Without<StaminaLabelText>,
        ),
    >,
    mut fill_nodes: Query<&mut Node, (Without<CharacterNametag>, Without<HpFillBar>, Without<StaminaFillBar>)>,
    mut commands: Commands,
) {
    let Ok((camera, cam_trans)) = camera_query.get_single() else {
        return;
    };

    for (tag_ent, mut tag_node, tag_data) in tag_query.iter_mut() {
        let Ok((_char_ent, char_trans, char_info, maybe_hp, maybe_stam)) =
            char_query.get(tag_data.target)
        else {
            // Despawn orphaned nametag
            commands.entity(tag_ent).despawn_recursive();
            continue;
        };

        // Highlight on overhead nametag when this specific character is currently speaking
        if let Ok(mut label) = label_texts.get_mut(tag_data.label_text) {
            let is_speaking = koth
                .active_speaker
                .as_ref()
                .map(|(spk, _)| spk.eq_ignore_ascii_case(&char_info.name))
                .unwrap_or(false)
                && koth.event_notice.is_some();

            if is_speaking {
                **label = format!(
                    "[SPEAKING] {} [{}]",
                    char_info.name.to_uppercase(),
                    char_info.team.name()
                );
            } else {
                **label = format!(
                    "{} [{}]",
                    char_info.name.to_uppercase(),
                    char_info.team.name()
                );
            }
        }

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
        **text = format!("BLUE FLAG: {:.0}", koth.blue_score);
    }
    if let Ok(mut text) = red_text.get_single_mut() {
        **text = format!("RED FLAG: {:.0}", koth.red_score);
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
            **text = "DELIVER RED FLAG TO BLUE BASE!".to_string();
        } else if !koth.blue_flag_at_base {
            **text = "TACKLE THE ENEMY TO SAVE BLUE FLAG!".to_string();
        } else {
            **text = "CAPTURE THE FLAG (FIRST TO 5)".to_string();
        }
    }

    // 3. Dynamic CTF Warning & Event Alert
    if let Ok((mut text, mut color)) = status_text.get_single_mut() {
        **text = koth.status_banner.clone();
        color.0 = koth.status_color;

        if let Some(winner) = koth.winner {
            match winner {
                Team::Blue => {
                    **text =
                        "ANDERS [BLUE]: «Victory! Blue Team won Capture the Flag!»".to_string();
                    color.0 = Color::srgb(0.2, 1.0, 0.6);
                }
                Team::Red => {
                    **text =
                        "SIRI [RED]: «Victory for Red Team! We own this arena!»".to_string();
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

        // Bottom-Right Throwable Net Action Indicator
        if let Ok((mut text, mut color)) = tackle_text.get_single_mut() {
            if tackle.cooldown > 0.0 {
                **text = format!("Throwable Net ({:.1}s)", tackle.cooldown);
                color.0 = Color::srgb(0.7, 0.7, 0.7);
            } else if stamina.current < 25.0 {
                **text = "Low Stamina for Net".to_string();
                color.0 = Color::srgb(1.0, 0.5, 0.3);
            } else {
                **text = "Throwable Net (x2)".to_string();
                color.0 = Color::srgb(0.9, 0.95, 1.0);
            }
        }
    }
}

fn reset_match_entities(
    koth: &mut KothManager,
    flags: &mut Query<&mut TeamFlag>,
    characters: &mut Query<(
        &mut Transform,
        Option<&mut Position>,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut Health,
        &mut Stamina,
        &mut TackleAbility,
        &mut StaggerState,
        &Character,
    )>,
) {
    koth.reset_for_new_match();

    for mut flag in flags.iter_mut() {
        flag.state = FlagState::AtBase;
    }

    for (
        mut transform,
        mut maybe_pos,
        mut lin_vel,
        mut ang_vel,
        mut health,
        mut stamina,
        mut tackle,
        mut stagger,
        char_info,
    ) in characters.iter_mut()
    {
        transform.translation = char_info.spawn_point;
        let yaw = match char_info.team {
            Team::Blue => std::f32::consts::FRAC_PI_2,
            Team::Red => -std::f32::consts::FRAC_PI_2,
        };
        transform.rotation = Quat::from_rotation_y(yaw);
        if let Some(ref mut pos) = maybe_pos {
            pos.0 = char_info.spawn_point;
        }
        lin_vel.0 = Vec3::ZERO;
        ang_vel.0 = Vec3::ZERO;
        health.current = health.max;
        stamina.current = stamina.max;
        tackle.cooldown = 0.0;
        tackle.is_tackling = false;
        tackle.tackle_timer = 0.0;
        tackle.punch_anim_timer = 0.0;
        stagger.duration = 0.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_pause_input_and_menu(
    keys: Res<ButtonInput<KeyCode>>,
    real_time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut physics_time: ResMut<Time<Physics>>,
    mut pause_state: ResMut<PauseState>,
    mut koth: ResMut<KothManager>,
    #[cfg(not(target_arch = "wasm32"))] mut app_exit_events: EventWriter<AppExit>,
    mut flags: Query<&mut TeamFlag>,
    mut characters: Query<(
        &mut Transform,
        Option<&mut Position>,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut Health,
        &mut Stamina,
        &mut TackleAbility,
        &mut StaggerState,
        &Character,
    )>,
    mut hud_pause_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<HudPauseButton>, Without<PauseMenuAction>),
    >,
    mut menu_btn_query: Query<
        (&Interaction, &PauseMenuAction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>, Without<HudPauseButton>),
    >,
) {
    if koth.match_started {
        pause_state.match_active_secs += real_time.delta_secs();
    } else {
        pause_state.match_active_secs = 0.0;
    }

    // 1. Keyboard Toggle: Esc or P (after a brief 0.2s grace window from match start so skipping intro with Esc doesn't pause)
    if koth.match_started
        && pause_state.match_active_secs > 0.20
        && (keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP))
    {
        if pause_state.exited {
            // Pressing Esc from the exited screen starts a fresh match
            reset_match_entities(&mut koth, &mut flags, &mut characters);
            pause_state.exited = false;
            pause_state.is_paused = false;
        } else {
            pause_state.is_paused = !pause_state.is_paused;
        }
    }

    // Keyboard shortcuts while inside the Pause Menu
    if pause_state.is_paused && !pause_state.exited {
        if keys.just_pressed(KeyCode::KeyR) {
            reset_match_entities(&mut koth, &mut flags, &mut characters);
            pause_state.is_paused = false;
        } else if keys.just_pressed(KeyCode::KeyQ) {
            pause_state.exited = true;
            #[cfg(not(target_arch = "wasm32"))]
            {
                app_exit_events.send(AppExit::Success);
            }
        }
    }

    // 2. Top-Right HUD Pause Button Click & Hover
    for (interaction, mut bg, mut border) in hud_pause_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if koth.match_started {
                    pause_state.is_paused = !pause_state.is_paused;
                    pause_state.exited = false;
                }
                *bg = BackgroundColor(Color::srgba(0.0, 0.55, 0.78, 0.96));
                *border = BorderColor(Color::WHITE);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.08, 0.22, 0.36, 0.95));
                *border = BorderColor(Color::srgb(0.2, 0.95, 1.0));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.04, 0.08, 0.14, 0.90));
                *border = BorderColor(Color::srgba(0.0, 0.85, 1.0, 0.65));
            }
        }
    }

    // 3. Pause & Exit Menu Action Buttons
    for (interaction, action, mut bg, mut border) in menu_btn_query.iter_mut() {
        let is_danger = *action == PauseMenuAction::ExitGame;
        match *interaction {
            Interaction::Pressed => {
                match *action {
                    PauseMenuAction::Resume => {
                        pause_state.is_paused = false;
                        pause_state.exited = false;
                    }
                    PauseMenuAction::RestartMatch | PauseMenuAction::StartNewMatchFromExit => {
                        reset_match_entities(&mut koth, &mut flags, &mut characters);
                        pause_state.is_paused = false;
                        pause_state.exited = false;
                    }
                    PauseMenuAction::ToggleMusic => {
                        pause_state.music_muted = !pause_state.music_muted;
                    }
                    PauseMenuAction::ExitGame => {
                        pause_state.is_paused = true;
                        pause_state.exited = true;
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            app_exit_events.send(AppExit::Success);
                        }
                    }
                    PauseMenuAction::ReplayIntroFromExit => {
                        reset_match_entities(&mut koth, &mut flags, &mut characters);
                        koth.match_started = false;
                        pause_state.is_paused = false;
                        pause_state.exited = false;
                        pause_state.match_active_secs = 0.0;
                        crate::types::trigger_return_to_intro();
                    }
                }
            }
            Interaction::Hovered => {
                if is_danger {
                    *bg = BackgroundColor(Color::srgba(0.58, 0.10, 0.14, 0.98));
                    *border = BorderColor(Color::srgb(1.0, 0.55, 0.55));
                } else {
                    *bg = BackgroundColor(Color::srgba(0.10, 0.28, 0.46, 0.98));
                    *border = BorderColor(Color::srgb(0.35, 0.96, 1.0));
                }
            }
            Interaction::None => {
                if is_danger {
                    *bg = BackgroundColor(Color::srgba(0.32, 0.06, 0.08, 0.92));
                    *border = BorderColor(Color::srgb(0.95, 0.25, 0.28));
                } else {
                    *bg = BackgroundColor(Color::srgba(0.06, 0.14, 0.24, 0.92));
                    *border = BorderColor(Color::srgb(0.0, 0.78, 1.0));
                }
            }
        }
    }

    // 4. Sync Virtual Time and Physics Time with PauseState
    let should_freeze = pause_state.is_paused || pause_state.exited;
    if should_freeze {
        if !virtual_time.is_paused() {
            virtual_time.pause();
        }
        if !physics_time.is_paused() {
            physics_time.pause();
        }
    } else {
        if virtual_time.is_paused() {
            virtual_time.unpause();
        }
        if physics_time.is_paused() {
            physics_time.unpause();
        }
    }
}

fn update_pause_overlay_ui(
    pause_state: Res<PauseState>,
    koth: Res<KothManager>,
    mut overlay_query: Query<
        &mut Node,
        (
            With<PauseOverlayRoot>,
            Without<PauseModalCard>,
            Without<ExitModalCard>,
        ),
    >,
    mut pause_card_query: Query<
        &mut Node,
        (
            With<PauseModalCard>,
            Without<PauseOverlayRoot>,
            Without<ExitModalCard>,
        ),
    >,
    mut exit_card_query: Query<
        &mut Node,
        (
            With<ExitModalCard>,
            Without<PauseOverlayRoot>,
            Without<PauseModalCard>,
        ),
    >,
    mut summary_text_query: Query<
        &mut Text,
        (With<PauseMatchSummaryText>, Without<MusicToggleButtonText>),
    >,
    mut music_btn_text_query: Query<
        &mut Text,
        (With<MusicToggleButtonText>, Without<PauseMatchSummaryText>),
    >,
) {
    let show_overlay = pause_state.is_paused || pause_state.exited;

    if let Ok(mut overlay_node) = overlay_query.get_single_mut() {
        overlay_node.display = if show_overlay {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut pause_card_node) = pause_card_query.get_single_mut() {
        pause_card_node.display = if pause_state.is_paused && !pause_state.exited {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut exit_card_node) = exit_card_query.get_single_mut() {
        exit_card_node.display = if pause_state.exited {
            Display::Flex
        } else {
            Display::None
        };
    }

    if show_overlay {
        let mins = (koth.match_timer / 60.0) as u32;
        let secs = (koth.match_timer % 60.0) as u32;
        if let Ok(mut summary_text) = summary_text_query.get_single_mut() {
            **summary_text = format!(
                "BLUE FLAG: {:.0}   •   {:02}:{:02}   •   RED FLAG: {:.0}",
                koth.blue_score, mins, secs, koth.red_score
            );
        }

        if let Ok(mut music_text) = music_btn_text_query.get_single_mut() {
            **music_text = if pause_state.music_muted {
                "MUSIC: OFF  [M]".to_string()
            } else {
                "MUSIC: ON  [M]".to_string()
            };
        }
    }
}
