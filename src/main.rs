mod cursor;
mod done;
mod game;
mod menu;
mod particles;

use bevy::prelude::*;
use bevy_jornet::{JornetPlugin, Leaderboard};
use cursor::{CursorPlugin, MainCamera};
use done::DonePlugin;
use game::GamePlugin;
use menu::MenuPlugin;
use particles::ParticlesPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(JornetPlugin::with_leaderboard(
            option_env!("JORNET_LEADERBOARD_ID").unwrap_or("429cd002-f885-4d62-8d07-1f556e4e110e"),
            option_env!("JORNET_LEADERBOARD_SECRET")
                .unwrap_or("eb1ccf59-f519-4be0-a113-db4059813922"),
        ))
        .add_plugin(ParticlesPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(DonePlugin)
        .add_state(GameState::Menu)
        .add_startup_system(setup)
        .run();
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Menu,
    Game,
    Done,
}

struct TextFont(pub Handle<Font>);

fn setup(
    mut commands: Commands,
    mut leaderboard: ResMut<Leaderboard>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);
    leaderboard.create_player(None);
    commands.insert_resource(TextFont(asset_server.load("fonts/FiraSans-Bold.ttf")));
}
