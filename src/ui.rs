use bevy::prelude::*;

use crate::units::player;

#[derive(Message, PartialEq)]
pub enum PlayerAction {
    Move,
    Wait,
}

#[derive(Component)]
pub enum ButtonAction {
    Move,
    Wait,
}

#[derive(Component)]
struct MoveButton;

impl From<&ButtonAction> for PlayerAction {
    fn from(value: &ButtonAction) -> Self {
        match value {
            ButtonAction::Move => Self::Move,
            ButtonAction::Wait => Self::Wait,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_message::<PlayerAction>()
        .add_systems(
            OnEnter(player::TurnState::SelectedPosition),
            |move_buttons: Query<&mut Visibility, With<MoveButton>>| {
                set_move_button_visibility(move_buttons, true);
            },
        )
        .add_systems(
            OnExit(player::TurnState::SelectedPosition),
            |move_buttons: Query<&mut Visibility, With<MoveButton>>| {
                set_move_button_visibility(move_buttons, false);
            },
        )
        .add_systems(OnEnter(player::TurnState::SelectedUnit), spawn_action_bar)
        .add_systems(OnEnter(player::TurnState::None), despawn_action_bar)
        .add_systems(
            Update,
            handle_move_button.run_if(not(in_state(player::TurnState::None))),
        );
}

fn spawn_action_bar(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            ..default()
        },
        children![
            (
                action_button("Move", ButtonAction::Move),
                MoveButton,
                Visibility::Hidden
            ),
            action_button("Wait", ButtonAction::Wait)
        ],
    ));
}

fn action_button(text: &str, button_action: ButtonAction) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(120.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        button_action,
        BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
        children![(
            Text::new(text),
            TextFont {
                font_size: 20.0,
                ..default()
            },
        )],
    )
}

fn despawn_action_bar(mut commands: Commands, query: Query<Entity, With<ButtonAction>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn set_move_button_visibility(
    move_buttons: Query<&mut Visibility, With<MoveButton>>,
    visible: bool,
) {
    for mut visibility in move_buttons {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    }
}

fn handle_move_button(
    query: Query<(&Interaction, &ButtonAction), (Changed<Interaction>, With<Button>)>,
    mut ev_player_action: MessageWriter<PlayerAction>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        ev_player_action.write(action.into());
    }
}
