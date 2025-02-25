use bevy::prelude::*;
use crate::input::InputState;
use crate::components::{Position, Player, Npc, DialogBox};

pub fn check_dialog_distance(
    mut npc_query: Query<(&Position, &mut Npc)>,
    player_query: Query<&Position, With<Player>>,
) {
    let player_pos = if let Ok(pos) = player_query.get_single() {
        pos
    } else {
        return; 
    };

    for (npc_pos, mut npc) in npc_query.iter_mut() {
        let dx = (npc_pos.x - player_pos.x).abs();
        let dy = (npc_pos.y - player_pos.y).abs();

        // If NPC is speaking and player moves too far away, stop dialog
        if npc.speaking && (dx > 1 || dy > 1) {
            npc.speaking = false;
        }
    }
}

pub fn handle_npc_interaction(
    input: Res<InputState>,
    mut npc_query: Query<(Entity, &Position, &mut Npc)>,
    player_query: Query<&Position, With<Player>>,
    mut dialog_query: Query<(Entity, &mut DialogBox)>
) {
    // Get player position
    let player_pos = if let Ok(pos) = player_query.get_single() {
        pos
    } else {
        return;
    };

    for (_npc_entity, npc_pos, mut npc) in npc_query.iter_mut() {
        let dx = (npc_pos.x - player_pos.x).abs();
        let dy = (npc_pos.y - player_pos.y).abs();

        // Check if player is adjacent to NPC (within 1 tile)
        if dx <= 1 && dy <= 1 {
            // If E is pressed, toggle speaking state
            if input.interact {
                npc.speaking = !npc.speaking;
                
                // Update dialog box text and visibility
                for (_, mut dialog) in dialog_query.iter_mut() {
                    dialog.visible = npc.speaking;
                    if npc.speaking {
                        dialog.text = "Hello!".to_string();
                    }
                }
            }
        }
    }
}
