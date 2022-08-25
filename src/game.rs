use crate::{cursor::MousePos, particles::ParticleExplosion, GameState, TextFont};
use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    math::Vec3Swizzles,
    prelude::*,
};
use rand::{rngs::SmallRng, Rng, SeedableRng};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Level {
            seed: rand::thread_rng().gen_range(u64::MIN..=u64::MAX),
            level_index: 0,
        });
        app.insert_resource(GameResult::default());
        app.add_system_set(SystemSet::on_enter(GameState::Game).with_system(new_game))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(update_inventory)
                    .with_system(handle_clicks)
                    .with_system(react_play_round)
                    .with_system(visibility_selection),
            )
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(despawn_game));
    }
}

pub struct Level {
    seed: u64,
    pub level_index: u64,
}

#[derive(Default)]
pub struct GameResult {
    pub last_number: f32,
    pub target_number: f32,
}

#[derive(Component, Debug, PartialEq, Clone)]
enum Operation {
    Plus,
    Minus,
    Multiply,
    Divide,
    // TODO: modulo, sqrt, power...
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operation::Plus => "+",
                Operation::Minus => "-",
                Operation::Multiply => "*",
                Operation::Divide => "/",
            }
        )
    }
}

impl Operation {
    fn apply(&self, n1: f32, n2: f32) -> Result<f32, ()> {
        match self {
            Operation::Plus => Ok(n1 + n2),
            Operation::Minus => Ok(n1 - n2),
            Operation::Multiply => Ok(n1 * n2),
            Operation::Divide => {
                if n2 == 0f32 {
                    Err(())
                } else {
                    Ok(n1 / n2)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct PlayingNumber {
    pub entity: Entity,
    pub inventory_index: usize,
}

#[derive(Debug, Clone)]
struct PlayRound {
    pub operation: Option<Operation>,
    pub number1: Option<PlayingNumber>,
    pub number2: Option<PlayingNumber>,
}

impl PlayRound {
    pub fn reset(&mut self) {
        self.operation = None;
        self.number1 = None;
        self.number2 = None;
    }
}

struct TargetNumber {
    pub target: f32,
}

struct Inventory {
    pub numbers: Vec<f32>,
}

#[derive(Component)]
struct InventorySlot {
    inventory_index: usize,
}

#[derive(Component)]
struct SelectionVisual(pub Entity);

#[derive(Component)]
struct GameEntity;

fn despawn_game(
    mut commands: Commands,
    q_to_despawn: Query<Entity, Or<(With<GameEntity>, With<InventorySlot>, With<Operation>)>>,
) {
    for e in q_to_despawn.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn lerp(start_value: f32, end_value: f32, ratio: f32) -> f32 {
    start_value + (end_value - start_value) * ratio
}

fn new_game(mut commands: Commands, level: Res<Level>, font: Res<TextFont>) {
    let mut rand = SmallRng::seed_from_u64(level.seed.wrapping_add(level.level_index));
    let mut numbers = vec![];

    let number_count = usize::clamp(
        lerp(2f32, 10f32, (level.level_index as f32 + 1f32) / 20f32) as usize,
        2,
        10,
    );
    let number_range = 1..=10;

    for _ in 0..number_count {
        numbers.push(rand.gen_range(number_range.clone()));
    }
    commands.insert_resource(Inventory {
        numbers: numbers.iter().map(|n| *n as f32).collect(),
    });

    let mut operations = vec![
        Operation::Plus,
        Operation::Minus,
        Operation::Multiply,
        Operation::Divide,
    ];
    let operation_count = usize::clamp(
        lerp(1f32, 4f32, (level.level_index as f32 + 2f32) / 8f32) as usize,
        1,
        4,
    );
    operations = operations.drain(..operation_count).collect();

    let mut numbers_to_simulate: Vec<f32> = numbers.iter().map(|v| *v as f32).collect();
    use rand::seq::SliceRandom;
    while numbers_to_simulate.len() > 1 {
        let chosen_indexes: Vec<usize> = (0..numbers_to_simulate.len())
            .collect::<Vec<usize>>()
            .choose_multiple(&mut rand, 2)
            .cloned()
            .collect();
        if let Ok(new_number) = operations.choose(&mut rand).unwrap().apply(
            numbers_to_simulate[chosen_indexes[0]],
            numbers_to_simulate[chosen_indexes[1]],
        ) {
            numbers_to_simulate[chosen_indexes[0]] = new_number;
            numbers_to_simulate.remove(chosen_indexes[1]);
        }
    }
    // TODO: simulate operations + end up on a doable target
    let target = numbers_to_simulate[0];

    commands.insert_resource(TargetNumber {
        target: target as f32,
    });
    commands.insert_resource(PlayRound {
        operation: None,
        number1: None,
        number2: None,
    });

    let text_alignment = TextAlignment::CENTER;
    let text_style = TextStyle {
        font: font.0.clone(),
        font_size: 60.0,
        color: Color::WHITE,
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(format!("Target: {target}"), text_style.clone())
                .with_alignment(text_alignment),
            transform: Transform::from_translation(Vec3::new(0f32, 300f32, 20f32)),
            ..default()
        })
        .insert(GameEntity);

    let spacing = 100f32;
    let offset = Vec2::new(-((operations.len() - 1) as f32 * spacing) / 2f32, -50f32);
    for (x, op) in operations.iter().enumerate() {
        let mut op_commands = commands.spawn_bundle(Text2dBundle {
            text: Text::from_section(format!("{op}"), text_style.clone())
                .with_alignment(text_alignment),
            transform: Transform::from_translation(
                (Vec2::new(x as f32 * spacing, 0f32) + offset).extend(20f32),
            ),
            ..default()
        });
        op_commands.insert(op.clone());

        let mut visual_entity = None;
        op_commands.with_children(|parent| {
            visual_entity = Some(
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::YELLOW_GREEN,
                            custom_size: Some(Vec2::splat(50f32)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec2::ZERO.extend(-1f32)),
                        visibility: Visibility { is_visible: false },
                        ..default()
                    })
                    .id(),
            );
        });

        let op_entity = op_commands.id();
        commands
            .entity(op_entity)
            .insert(SelectionVisual(visual_entity.unwrap()));
    }
}

fn update_inventory(
    mut commands: Commands,
    font: Res<TextFont>,
    inventory: Res<Inventory>,
    q_inventory_slots: Query<Entity, With<InventorySlot>>,
) {
    if inventory.is_changed() {
        for e in q_inventory_slots.iter() {
            commands.entity(e).despawn_recursive();
        }
        let text_alignment = TextAlignment::CENTER;
        let text_style = TextStyle {
            font: font.0.clone(),
            font_size: 60.0,
            color: Color::WHITE,
        };
        let column_count = 5;
        let spacing = 100f32;
        let offset = Vec2::new(-((column_count - 1) as f32 * spacing) / 2f32, 200f32);
        for (i, number) in inventory.numbers.iter().enumerate() {
            let x = i % column_count;
            let y = i / column_count;
            let mut slot = commands.spawn_bundle(Text2dBundle {
                text: Text::from_section(
                    format!("{number:.1}").trim_end_matches(".0"),
                    text_style.clone(),
                )
                .with_alignment(text_alignment),
                transform: Transform::from_translation(
                    (Vec2::new(x as f32 * spacing, -(y as f32) * spacing) + offset).extend(20f32),
                ),
                ..default()
            });

            slot.insert(InventorySlot { inventory_index: i });
            let slot_entity = slot.id();
            let mut visual_entity = None;
            slot.with_children(|parent| {
                visual_entity = Some(
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: Color::YELLOW_GREEN,
                                custom_size: Some(Vec2::splat(50f32)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec2::ZERO.extend(-1f32)),
                            visibility: Visibility { is_visible: false },
                            ..default()
                        })
                        .id(),
                );
            });
            commands
                .entity(slot_entity)
                .insert(SelectionVisual(visual_entity.unwrap()));
        }
    }
}

fn handle_clicks(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mouse_pos: Res<MousePos>,
    mut play_round: ResMut<PlayRound>,
    q_inventory_slots: Query<(Entity, &Transform, &InventorySlot)>,
    q_operations: Query<(Entity, &Transform, &Operation)>,
) {
    for event in mouse_button_input_events.iter() {
        if ButtonState::Pressed == event.state {
            let mut found_something = false;
            for (e, t, slot) in &q_inventory_slots {
                let distance = t.translation.xy().distance(mouse_pos.0);
                if distance < 50f32 {
                    if let Some(n1) = &play_round.number1 {
                        if n1.entity == e {
                            play_round.number1 = None;
                            found_something = true;
                            break;
                        }
                    }
                    if let Some(n2) = &play_round.number2 {
                        if n2.entity == e {
                            play_round.number2 = None;
                            found_something = true;
                            break;
                        }
                    }
                    if play_round.number1.is_none() {
                        play_round.number1 = Some(PlayingNumber {
                            entity: e,
                            inventory_index: slot.inventory_index,
                        });
                        found_something = true;
                        break;
                    }
                    if play_round.number2.is_none() {
                        play_round.number2 = Some(PlayingNumber {
                            entity: e,
                            inventory_index: slot.inventory_index,
                        });
                        found_something = true;
                        break;
                    }
                    // close to something, but we cannot select more.
                    // TODO: show feedback to encourage deselection
                    break;
                }
            }
            if found_something {
                continue;
            }
            for (e, t, operation) in &q_operations {
                let distance = t.translation.xy().distance(mouse_pos.0);
                if distance < 50f32 {
                    if let Some(op) = &play_round.operation {
                        if op == operation {
                            play_round.operation = None;
                            break;
                        }
                    }
                    if play_round.operation.is_none() {
                        play_round.operation = Some(operation.clone());
                        break;
                    }
                    // close to something, but we cannot select more.
                    // TODO: show feedback to encourage deselection
                    break;
                }
            }
        }
    }
}

fn visibility_selection(
    mut play_round: ResMut<PlayRound>,
    q_selectable: Query<(Entity, &SelectionVisual, Option<&Operation>)>,
    mut q_visibility: Query<(&mut Visibility, &mut Sprite)>,
) {
    if play_round.is_changed() {
        for (e, v, op) in q_selectable.iter() {
            if let Some(n1) = &play_round.number1 {
                if n1.entity == e {
                    let mut res = q_visibility.get_mut(v.0).unwrap();
                    res.0.is_visible = true;
                    res.1.color = Color::GREEN;
                    continue;
                }
            }
            if let Some(n2) = &play_round.number2 {
                if n2.entity == e {
                    let mut res = q_visibility.get_mut(v.0).unwrap();
                    res.0.is_visible = true;
                    res.1.color = Color::BLUE;
                    continue;
                }
            }
            if let Some(op) = op {
                if let Some(selected_op) = &play_round.operation {
                    if selected_op == op {
                        let mut res = q_visibility.get_mut(v.0).unwrap();
                        res.0.is_visible = true;
                        res.1.color = Color::YELLOW_GREEN;
                        continue;
                    }
                }
            }
            q_visibility.get_mut(v.0).unwrap().0.is_visible = false;
        }
    }
}

fn react_play_round(
    mut particles: EventWriter<ParticleExplosion>,
    mut inventory: ResMut<Inventory>,
    mut play_round: ResMut<PlayRound>,
    mut state: ResMut<State<GameState>>,
    // TODO: shoud be in the done state
    mut game_result: ResMut<GameResult>,
    target: Res<TargetNumber>,
) {
    if play_round.is_changed() {
        if let PlayRound {
            operation: Some(op),
            number1: Some(n1),
            number2: Some(n2),
        } = play_round.clone()
        {
            let result = op.apply(
                inventory.numbers[n1.inventory_index],
                inventory.numbers[n2.inventory_index],
            );
            if let Ok(result) = result {
                play_round.as_mut().reset();
                inventory.numbers[n1.inventory_index] = result;
                inventory.numbers.remove(n2.inventory_index);
                particles.send(ParticleExplosion {
                    location: Vec2::ZERO,
                    color: Color::ANTIQUE_WHITE,
                });
                if inventory.numbers.len() == 1 {
                    game_result.target_number = target.target;
                    game_result.last_number = inventory.numbers[0];

                    let _ = state.set(GameState::Done);
                }
            } else {
                play_round.as_mut().reset();
                particles.send(ParticleExplosion {
                    location: Vec2::ZERO,
                    color: Color::ORANGE_RED,
                })
            }
        }
    }
}
