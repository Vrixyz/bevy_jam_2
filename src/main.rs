use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Option::<Inventory>::None);
        app.insert_resource(Option::<PlayRound>::None);
        app.add_startup_system(setup);
        app.add_system(update_inventory);
    }
}

enum Operation {
    Plus,
    Minus,
    Multiply,
    Divide,
    // TODO: modulo, sqrt, power...
}

struct PlayRound {
    pub operation: Operation,
    pub number1: i32,
    pub number2: i32,
}

struct Inventory {
    pub numbers: Vec<f32>,
}

#[derive(Component)]
struct InventorySlot;

fn setup(mut commands: Commands, mut inventory: ResMut<Inventory>) {
    commands.spawn_bundle(Camera2dBundle::default());
    // TODO: use a cross platform rand
    let mut rand = rand::thread_rng();
    let mut numbers = vec![];
    for _ in 0..10 {
        numbers.push(rand.gen_range(0..=10));
    }
    *inventory = Inventory {
        numbers: numbers.iter().map(|n| *n as f32).collect(),
    };
}

fn update_inventory(
    mut commands: Commands,
    mut inventory: ResMut<Option<Inventory>>,
    q_inventory_slots: Query<Entity, With<InventorySlot>>,
) {
    if inventory.is_changed() {
        for e in q_inventory_slots.iter() {
            commands.entity(e).despawn_recursive();
        }
        if let Some(inventory) = &*inventory {
            for i in &inventory.numbers {
                // TODO: spawn all inventory numbers
            }
        }
    }
}
