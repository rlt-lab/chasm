use bevy::prelude::*;

// Maximum number of messages to keep in history
const MAX_MESSAGES: usize = 50;

#[derive(Resource)]
pub struct MessageLog {
    messages: Vec<String>,
}

impl Default for MessageLog {
    fn default() -> Self {
        let mut log = MessageLog {
            messages: Vec::new(),
        };
        log.add_message("Welcome to Chasm!".to_string());
        log
    }
}

impl MessageLog {
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        if self.messages.len() > MAX_MESSAGES {
            self.messages.remove(0);
        }
    }
}

pub fn setup_ui(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 16.0,
        color: Color::WHITE,
        // Use a monospace font for consistent character widths
        font: Default::default(), // Will use system monospace font
        ..default()
    };

    // Root node
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            width: Val::Px(640.0),  // Match map width (20 tiles * 32px)
            height: Val::Px(100.0),
            left: Val::Px(80.0),    // Center align with map ((800 - 640) / 2)
            bottom: Val::Px(32.0),  // Position one tile (32px) below bottom wall
            padding: UiRect::all(Val::Px(4.0)),
            margin: UiRect::all(Val::Px(0.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
        z_index: ZIndex::Global(100),
        ..default()
    })
        .with_children(|parent| {
            // Message border - top
            // Top border
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "################################################################################",
                    text_style.clone(),
                ),
                style: Style {
                    width: Val::Px(640.0),
                    ..default()
                },
                ..default()
            });

            // Message line with borders
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(640.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                // Message text with side borders in a single line
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "#                                                                              #",
                        text_style.clone(),
                    ),
                    style: Style {
                        width: Val::Px(640.0),
                        ..default()
                    },
                    ..default()
                });

                // Message text centered
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "Welcome to Chasm!",
                        text_style.clone(),
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                });
            });

            // Message border - bottom
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "################################################################################",
                    text_style.clone(),
                ),
                style: Style {
                    width: Val::Px(640.0),
                    ..default()
                },
                ..default()
            });
        });
}

pub fn update_message_log(
    message_log: Res<MessageLog>,
    mut query: Query<&mut Text>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let messages = message_log.messages.join("\n");
        text.sections[0].value = messages;
    }
}

