use valence::client::despawn_disconnected_clients;
use valence::prelude::*;

use valence_sheeptag::brand::SheeptagBrandPlugin;
use valence_sheeptag::disguise::DisguisePlugin;
use valence_sheeptag::teams::TeamPlugin;

fn main() {
    App::new()
        .add_plugins(SheeptagBrandPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(TeamPlugin)
        .add_plugins(DisguisePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (despawn_disconnected_clients, init_clients))
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    for z in -25..25 {
        for x in -25..25 {
            layer.chunk.set_block([x, 64, z], BlockState::PODZOL);
        }
    }

    commands.spawn(layer);
}

fn init_clients(
    mut clients: Query<
        (
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            // &mut VisibleEntityLayers,
            &mut Position,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, With<ChunkLayer>>,
) {
    for (_, mut layer_id, mut visible_chunk_layer, mut pos) in &mut clients {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        // visible_entity_layers.0.insert(layer);

        pos.set([0.5, 65.0, 0.5]);
    }
}
