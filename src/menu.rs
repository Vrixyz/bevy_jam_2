use bevy::prelude::*;
use bevy_jornet::Leaderboard;

use crate::{game::Level, GameState, TextFont};
pub struct MenuPlugin;

const BACKGROUND: &str = "339755";
const BUTTON: &str = "2A4747";
const TEXT: &str = "BeDaD6";

#[derive(Component)]
struct MenuUI;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Menu).with_system(display_menu))
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(button_system)
                    .with_system(display_scores),
            )
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(despawn_menu));
    }
}

fn display_menu(
    mut commands: Commands,
    font: Res<TextFont>,
    leaderboard: Res<Leaderboard>,
    level: Res<Level>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                border: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            color: Color::hex(BACKGROUND).unwrap().into(),
            ..default()
        })
        .insert(MenuUI)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Math it",
                TextStyle {
                    font: font.0.clone(),
                    font_size: 60.0,
                    color: Color::hex(TEXT).unwrap(),
                },
            ));
            parent.spawn_bundle(TextBundle::from_section(
                format!("Level {}", level.level_index + 1),
                TextStyle {
                    font: font.0.clone(),
                    font_size: 30.0,
                    color: Color::hex(TEXT).unwrap(),
                },
            ));
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(300.0), Val::Undefined),
                                flex_direction: FlexDirection::ColumnReverse,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(20.0)),
                                ..default()
                            },
                            color: Color::NONE.into(),
                            ..default()
                        })
                        .insert(LeaderboardMarker::Player);
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(150.0), Val::Undefined),
                                flex_direction: FlexDirection::ColumnReverse,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(20.0)),
                                ..default()
                            },
                            color: Color::NONE.into(),
                            ..default()
                        })
                        .insert(LeaderboardMarker::Score);
                });

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::hex(BUTTON).unwrap().into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 40.0,
                            color: Color::hex(TEXT).unwrap(),
                        },
                    ));
                });
        });
    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection {
                    value: "you are: ".to_string(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: 20.0,
                        color: Color::hex(TEXT).unwrap(),
                    },
                },
                TextSection {
                    value: leaderboard
                        .get_player()
                        .map(|p| p.name.clone())
                        .unwrap_or_default(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: 25.0,
                        color: Color::hex(TEXT).unwrap(),
                    },
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MenuUI)
        .insert(PlayerName);

    leaderboard.refresh_leaderboard();
}

#[derive(Component)]
struct PlayerName;

#[derive(Component)]
enum LeaderboardMarker {
    Score,
    Player,
}

fn display_scores(
    leaderboard: Res<Leaderboard>,
    mut commands: Commands,
    font: Res<TextFont>,
    root_ui: Query<(Entity, &LeaderboardMarker)>,
    mut player_name: Query<&mut Text, With<PlayerName>>,
) {
    if leaderboard.is_changed() {
        if let Some(player) = leaderboard.get_player() {
            player_name.single_mut().sections[1].value = player.name.clone();
        }
        let mut leaderboard = leaderboard.get_leaderboard();
        leaderboard.sort_unstable_by(|s1, s2| s2.score.partial_cmp(&s1.score).unwrap());
        let mut i = 0;
        while i < leaderboard.len() {
            if leaderboard[..i]
                .iter()
                .any(|s| s.player == leaderboard[i].player)
            {
                let val = leaderboard.remove(i);
            } else {
                i += 1;
            }
        }
        leaderboard.truncate(10);
        for (root_entity, marker) in &root_ui {
            commands.entity(root_entity).despawn_descendants();
            for score in &leaderboard {
                commands.entity(root_entity).with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        match marker {
                            LeaderboardMarker::Score => format!("{} ", score.score),
                            LeaderboardMarker::Player => score.player.clone(),
                        },
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 30.0,
                            color: Color::hex(TEXT).unwrap(),
                        },
                    ));
                });
            }
        }
    }
}

fn despawn_menu(
    mut commands: Commands,
    root_ui: Query<Entity, (With<Node>, With<MenuUI>, Without<Parent>)>,
) {
    for entity in &root_ui {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = (Color::hex(BUTTON).unwrap() + Color::GRAY).into();
                let _ = state.set(GameState::Game);
            }
            Interaction::Hovered => {
                *color = (Color::hex(BUTTON).unwrap() + Color::DARK_GRAY).into();
            }
            Interaction::None => {
                *color = Color::hex(BUTTON).unwrap().into();
            }
        }
    }
}
