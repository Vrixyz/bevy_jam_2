mod cursor;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    math::Vec3Swizzles,
    prelude::*,
};
use cursor::{CursorPlugin, MainCamera, MousePos};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CursorPlugin)
        .add_plugin(GamePlugin)
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_startup_system_to_stage(StartupStage::PostStartup, new_game);
        app.add_system(update_inventory);
        app.add_system(handle_clicks);
        app.add_system(react_play_round);
        app.add_system(visibility_selection);
    }
}

struct TextFont(pub Handle<Font>);

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);
    commands.insert_resource(TextFont(asset_server.load("fonts/FiraSans-Bold.ttf")));
}

fn new_game(mut commands: Commands, font: Res<TextFont>) {
    // TODO: use a cross platform rand
    let mut rand = rand::thread_rng();
    let mut numbers = vec![];
    for _ in 0..10 {
        numbers.push(rand.gen_range(0..=10));
    }
    commands.insert_resource(Inventory {
        numbers: numbers.iter().map(|n| *n as f32).collect(),
    });

    // TODO: simulate operations + end up on a doable target
    let target = rand.gen_range(-1000..=1000);
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
    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section(format!("Target: {target}"), text_style.clone())
            .with_alignment(text_alignment),
        transform: Transform::from_translation(Vec3::new(0f32, 300f32, 20f32)),
        ..default()
    });

    let operations = vec![
        Operation::Plus,
        Operation::Minus,
        Operation::Multiply,
        Operation::Divide,
    ];
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
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut inventory: ResMut<Inventory>,
    mut play_round: ResMut<PlayRound>,
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
            }
        }
    }
}
