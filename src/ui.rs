use bevy::prelude::*;

#[derive(Component)]
pub struct MoveButton;

#[derive(Message)]
pub struct MoveButtonClicked;

pub fn spawn_action_bar(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Percent(50.0),
                ..default()
            },
            MoveButton,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Move"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                    ));
                });
        });
}

pub fn despawn_action_bar(mut commands: Commands, query: Query<Entity, With<MoveButton>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_move_button(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut ev_move_clicked: MessageWriter<MoveButtonClicked>,
) {
    for _interaction in &query {
        for interaction in &query {
            if *interaction == Interaction::Pressed {
                ev_move_clicked.write(MoveButtonClicked);
            }
        }
    }
}
