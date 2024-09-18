use dan_world::DanWorld;
use valence::client::despawn_disconnected_clients;
use valence::prelude::*;

use valence::registry::RegistryIdx;
use valence_sheeptag::brand::SheeptagBrandPlugin;
use valence_sheeptag::SheeptagPlugins;

#[derive(Resource)]
struct DanWorldFile(&'static str);

fn main() {
    App::new()
        .insert_resource(DanWorldFile("demo_world.dan"))
        .add_plugins(SheeptagBrandPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(SheeptagPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (despawn_disconnected_clients, init_clients))
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    world_file: Res<DanWorldFile>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    let world = match DanWorld::load(world_file.0) {
        Ok(world) => world,
        Err(e) => {
            eprintln!("Failed to load DanWorld: {e:#?}");

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
            return;
        }
    };

    place_world(world, &mut layer);
    commands.spawn(layer);
}

fn init_clients(
    mut clients: Query<
        (
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut Position,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, With<ChunkLayer>>,
) {
    for (mut layer_id, mut visible_chunk_layer, mut pos, mut gm) in &mut clients {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;

        pos.set([0.5, 65.0, 0.5]);
        *gm = GameMode::Spectator;
    }
}

fn place_world(world: DanWorld, layer: &mut LayerBundle) {
    let width_and_padding = (world.width as i32) + 10;
    let depth_and_padding = (world.depth as i32) + 10;

    for chunk_x in -width_and_padding..width_and_padding {
        for chunk_z in -depth_and_padding..depth_and_padding {
            layer
                .chunk
                .insert_chunk([chunk_x, chunk_z], UnloadedChunk::new());
        }
    }

    for chunk in world.chunks {
        let mut base_y = 64;

        for section in chunk.sections {
            for y in 0..16u16 {
                for x in 0..16u16 {
                    for z in 0..16u16 {
                        let p_idx = section.blocks[((y * 256) + (x * 16) + z) as usize];
                        let block = BlockKind::from_str(&section.palette[p_idx as usize])
                            .unwrap_or(BlockKind::Podzol);

                        let biome = section.biomes[((y * 256) + (x * 16) + z) as usize];

                        let actual_x = (chunk.x * 16 + x) as i32;
                        let actual_z = (chunk.z * 16 + z) as i32;
                        let actual_y = (base_y + y) as i32;

                        layer
                            .chunk
                            .set_block([actual_x, actual_y, actual_z], block.to_state());
                        layer.chunk.set_biome(
                            DVec3 {
                                x: actual_x as f64,
                                y: actual_y as f64,
                                z: actual_z as f64,
                            },
                            BiomeId::from_index(biome as usize),
                        );
                    }
                }
            }

            base_y += 16;
        }
    }
}
