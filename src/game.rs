use bevy::prelude::*;

use crate::{enemy, game, player};

pub const LEVEL: &[&str] = &[
    "............",
    "....W.......",
    ".P..........",
    "....W...W.E.",
    ".P..W...W...",
    "........W.E.",
    "...WWWW.....",
    "............",
];

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Win), spawn_win_screen)
        .add_systems(OnEnter(GameState::Lose), spawn_lose_screen)
        .add_systems(PostStartup, start)
        .add_systems(PostUpdate, game::check_win);
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GameState {
    #[default]
    Begin,
    PlayerTurn,
    EnemyTurn,
    Win,
    Lose,
}

pub(crate) fn check_win(
    enemies: Query<&enemy::EnemyUnit>,
    players: Query<&player::PlayerUnit>,
    mut next_state: ResMut<NextState<game::GameState>>,
) {
    if players.count() == 0 {
        next_state.set(game::GameState::Lose);
    }
    if enemies.count() == 0 {
        next_state.set(game::GameState::Win);
    }
}

#[derive(Component)]
pub struct WinScreen;

pub fn spawn_win_screen(mut commands: Commands) {
    commands.spawn(overlay_screen("You win!"));
}

pub fn spawn_lose_screen(mut commands: Commands) {
    commands.spawn(overlay_screen("You lose."));
}

pub fn start(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::PlayerTurn);
}

fn overlay_screen(text: &str) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        BackgroundColor(Color::WHITE),
        WinScreen,
        children![(
            Text::new(text),
            TextColor::BLACK,
            // TODO: different font
            TextFont {
                font_size: 64.0,
                ..default()
            },
        )],
    )
}
