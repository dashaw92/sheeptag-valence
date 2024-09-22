use valence::{
    entity::entity::Flags, interact_block::InteractBlockEvent, inventory::HeldItem, prelude::*,
};

use crate::perms::OperMode;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (block_place, block_break));
    }
}

fn try_open(layer: &mut ChunkLayer, event: &InteractBlockEvent, flags: &Flags) -> bool {
    //Sneaking always overrides opening things with building.
    if flags.sneaking() {
        return false;
    }

    let Some(block) = layer.block(event.position) else {
        return false;
    };

    let state = block.state;
    if !state.to_kind().props().contains(&PropName::Open) {
        return false;
    }

    layer.set_block(
        event.position,
        match state.get(PropName::Open) {
            Some(PropValue::True) => state.set(PropName::Open, PropValue::False),
            Some(PropValue::False) => state.set(PropName::Open, PropValue::True),
            _ => return false,
        },
    );

    true
}

fn block_place(
    mut clients: Query<(&HeldItem, &Inventory, &Flags), (With<Client>, With<OperMode>)>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
) {
    let mut layer = layers.single_mut();

    for event in events.read() {
        let Ok((held, inv, flags)) = clients.get_mut(event.client) else {
            continue;
        };

        //Try to open the block that was interacted with. If this
        //returns true, the block was openable
        if try_open(&mut layer, &event, flags) {
            continue;
        }

        if event.hand != Hand::Main {
            continue;
        }

        let stack = inv.slot(held.slot());
        if stack.is_empty() {
            continue;
        }

        let Some(block) = BlockKind::from_item_kind(stack.item) else {
            continue;
        };

        let place_pos = event.position.get_in_direction(event.face);
        let state = block.to_state().set(
            PropName::Axis,
            match event.face {
                Direction::Down | Direction::Up => PropValue::Y,
                Direction::North | Direction::South => PropValue::Z,
                Direction::West | Direction::East => PropValue::X,
            },
        );

        layer.set_block(place_pos, state);
    }
}

fn block_break(
    clients: Query<&GameMode, With<OperMode>>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<DiggingEvent>,
) {
    let mut layer = layers.single_mut();

    for event in events.read() {
        let Ok(gm) = clients.get(event.client) else {
            continue;
        };

        if *gm == GameMode::Creative && event.state == DiggingState::Start {
            layer.set_block(event.position, BlockState::AIR);
        }
    }
}
