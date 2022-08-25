use bevy::prelude::*;
use bevy_jornet::Leaderboard;

use crate::{
    game::{GameResult, Level},
    GameState, TextFont,
};

const BACKGROUND: &str = "43C775";
const BACKGROUND_CLOSE: &str = "A3A225";
const BACKGROUND_FAILED: &str = "F31215";
const BUTTON: &str = "2A4747";
const TEXT: &str = "BeDaD6";

pub struct DonePlugin;
impl Plugin for DonePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Done).with_system(display_menu))
            .add_system_set(
                SystemSet::on_update(GameState::Done)
                    .with_system(button_system_retry)
                    .with_system(button_system_next),
            )
            .add_system_set(SystemSet::on_exit(GameState::Done).with_system(despawn_menu));
    }
}

#[derive(Component)]
struct DoneUI;
#[derive(Component)]
struct ButtonNext;
#[derive(Component)]
struct ButtonRetry;

fn display_menu(
    mut commands: Commands,
    font: Res<TextFont>,
    game_result: Res<GameResult>,
    level: Res<Level>,
    leaderboard: Res<Leaderboard>,
) {
    let difference = (game_result.target_number - game_result.last_number).abs();
    let is_exact_win = difference == 0f32;
    let is_close_win = difference <= 0.5f32;
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                border: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            color: Color::hex(if is_exact_win {
                BACKGROUND
            } else if is_close_win {
                BACKGROUND_CLOSE
            } else {
                BACKGROUND_FAILED
            })
            .unwrap()
            .into(),
            ..default()
        })
        .insert(DoneUI)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                if is_exact_win {
                    "PERFECT WIN!".to_string()
                } else if is_close_win {
                    format!(
                        "Close enough! {:.3} == {}",
                        game_result.last_number, game_result.target_number
                    )
                } else {
                    format!(
                        "Target was {} but you had {}",
                        game_result.target_number,
                        format!("{:.1}", game_result.last_number).trim_end_matches(".0")
                    )
                },
                TextStyle {
                    font: font.0.clone(),
                    font_size: 50.0,
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
            if is_exact_win || is_close_win {
                leaderboard.send_score((level.level_index + 1) as f32);
                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(350.0), Val::Px(65.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        color: Color::hex(BUTTON).unwrap().into(),
                        ..default()
                    })
                    .insert(ButtonNext)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "NEXT LEVEL",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 40.0,
                                color: Color::hex(TEXT).unwrap(),
                            },
                        ));
                    });
            }
            if !is_exact_win {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(350.0), Val::Px(65.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        color: Color::hex(BUTTON).unwrap().into(),
                        ..default()
                    })
                    .insert(ButtonRetry)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "RETRY",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 40.0,
                                color: Color::hex(TEXT).unwrap(),
                            },
                        ));
                    });
            }
        });
}

fn despawn_menu(
    mut commands: Commands,
    root_ui: Query<Entity, (With<Node>, With<DoneUI>, Without<Parent>)>,
) {
    for entity in &root_ui {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_system_retry(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, (With<Button>, With<ButtonRetry>)),
    >,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = (Color::hex(BUTTON).unwrap() + Color::GRAY).into();
                let _ = state.set(GameState::Menu);
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
fn button_system_next(
    mut level: ResMut<Level>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, (With<Button>, With<ButtonNext>)),
    >,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                level.level_index += 1;
                *color = (Color::hex(BUTTON).unwrap() + Color::GRAY).into();
                let _ = state.set(GameState::Menu);
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
