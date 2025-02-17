use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Tile {
    pub tile_type: TileType,
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Floor,
    Wall,
    Door,
    }

    #[derive(Component, Debug)]
    pub struct Npc {
        pub speaking: bool,
        pub dialog_text: String,
    }

    impl Default for Npc {
        fn default() -> Self {
            Self {
                speaking: false,
                dialog_text: "Hello!".to_string(),
            }
        }
    }

    #[derive(Component, Debug)]
    pub struct DialogBox {
        pub text: String,
        pub visible: bool,
    }

    impl Default for DialogBox {
        fn default() -> Self {
            Self {
                text: String::new(),
                visible: false,
            }
        }
    }
