use dan_world::blockdata::{
    Axis, BisectionHalf, DanBlockData, Direction, RailShape, Side, StairShape,
};
use dan_world::{DanDimension, DanWorld};
use valence::client::despawn_disconnected_clients;
use valence::prelude::*;

use valence::registry::RegistryIdx;
use valence::spawn::IsFlat;
use valence_sheeptag::brand::SheeptagBrandPlugin;
use valence_sheeptag::SheeptagPlugins;

#[derive(Resource)]
struct DanWorldFile(&'static str);

fn main() {
    App::new()
        .insert_resource(DanWorldFile("nether_test.dan"))
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
    let world = match DanWorld::load(world_file.0) {
        Ok(world) => world,
        Err(e) => {
            eprintln!("Failed to load DanWorld: {e:#?}");

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
            return;
        }
    };

    let dim = match world.dimension {
        DanDimension::Overworld => ident!("overworld"),
        DanDimension::Nether => ident!("the_nether"),
        DanDimension::End => ident!("the_end"),
    };

    let mut layer = LayerBundle::new(dim, &dimensions, &biomes, &server);
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
            &mut IsFlat,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, With<ChunkLayer>>,
) {
    for (mut layer_id, mut visible_chunk_layer, mut pos, mut gm, mut flat) in &mut clients {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;

        pos.set([0.5, 65.0, 0.5]);
        *gm = GameMode::Creative;
        flat.0 = true;
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
        let mut base_y = 1;

        for section in chunk.sections {
            for y in 0..16u16 {
                for x in 0..16u16 {
                    for z in 0..16u16 {
                        let data = section.data.get(&(x as usize, y as usize, z as usize));

                        let p_idx = section.blocks[((y * 256) + (x * 16) + z) as usize];
                        let block = BlockKind::from_str(&section.palette[p_idx as usize])
                            .unwrap_or(BlockKind::Podzol);

                        let mut block_state = block.to_state();
                        if data.is_some() {
                            block_state = set_props(block_state, data.unwrap());
                        }

                        let biome = section.biomes[((y * 256) + (x * 16) + z) as usize];

                        let actual_x = (chunk.x * 16 + x) as i32;
                        let actual_z = (chunk.z * 16 + z) as i32;
                        let actual_y = (base_y + y) as i32;

                        layer
                            .chunk
                            .set_block([actual_x, actual_y, actual_z], block_state);
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

fn set_props(mut state: BlockState, data: &[DanBlockData]) -> BlockState {
    for d in data {
        if let DanBlockData::MultipleFacing(mf) = d {
            for facing in mf {
                let prop = match facing {
                    &Direction::North => PropName::North,
                    &Direction::East => PropName::East,
                    &Direction::South => PropName::South,
                    &Direction::West => PropName::West,
                    &Direction::Up => PropName::Up,
                    &Direction::Down => PropName::Down,
                    _ => unreachable!(),
                };

                state = state.set(prop, PropValue::True);
            }

            continue;
        };

        let (prop, val) = to_prop(d);

        if prop == PropName::Half {
            //Handles minecraft using two different names for the property. Valence
            //felt the need to encode this oddity into their API, so while both doors (as an example)
            //and stairs both use the `Half` property, Minecraft opted to name the value either "top" or "upper"
            //and "bottom" or "lower", depending on the block. Rather than attempt to match against all possibilities (error-prone),
            //I'll just brute force it and set both values.
            //When set is called on BlockState, it checks the PropValue for validity, returning self unmodified if the value
            //is not applicable, resolving the issue.
            state = match val {
                PropValue::Top => state.set(prop, PropValue::Upper),
                PropValue::Bottom => state.set(prop, PropValue::Lower),
                //My code only ever attempts to use `Top` and `Bottom` for the `Half` prop,
                //so this branch will never be hit.
                _ => unreachable!(),
            };
        }
        state = state.set(prop, val);
    }

    state
}

fn to_prop(data: &DanBlockData) -> (PropName, PropValue) {
    match data {
        &DanBlockData::Orientation(ref axis) => (
            PropName::Axis,
            match axis {
                Axis::X => PropValue::X,
                Axis::Y => PropValue::Y,
                Axis::Z => PropValue::Z,
            },
        ),
        &DanBlockData::Age(ref age) => (PropName::Age, num_to_prop_val(age)),
        &DanBlockData::SnowLevel(ref level) => (PropName::Layers, num_to_prop_val(level)),
        &DanBlockData::LiquidLevel(ref level) => (PropName::Level, num_to_prop_val(level)),
        &DanBlockData::Bisected(ref half) => (
            PropName::Half,
            match half {
                &BisectionHalf::Top => PropValue::Top,
                &BisectionHalf::Bottom => PropValue::Bottom,
            },
        ),
        &DanBlockData::Direction(ref dir) => (PropName::Facing, dir_to_prop_val(dir)),
        &DanBlockData::Waterlogged(ref wl) => (PropName::Waterlogged, bool_to_prop_val(*wl)),
        &DanBlockData::Rotation(ref rot) => (PropName::Rotation, rot_to_prop_val(rot)),
        &DanBlockData::Open(ref o) => (PropName::Open, bool_to_prop_val(*o)),
        &DanBlockData::RailShape(ref r) => (PropName::Shape, rail_to_prop(r)),
        &DanBlockData::StairShape(ref s) => (PropName::Shape, stair_to_shape(s)),
        &DanBlockData::Attached(ref a) => (PropName::Attached, bool_to_prop_val(*a)),
        &DanBlockData::Hinge(ref side) => (
            PropName::Hinge,
            match side {
                Side::Left => PropValue::Left,
                Side::Right => PropValue::Right,
            },
        ),
        &DanBlockData::Farmland(ref m) => (PropName::Moisture, num_to_prop_val(m)),
        _ => unreachable!(),
    }
}

fn stair_to_shape(s: &StairShape) -> PropValue {
    match s {
        StairShape::InnerLeft => PropValue::InnerLeft,
        StairShape::InnerRight => PropValue::InnerRight,
        StairShape::OuterLeft => PropValue::OuterLeft,
        StairShape::OuterRight => PropValue::OuterRight,
        StairShape::Straight => PropValue::Straight,
    }
}

fn rail_to_prop(r: &RailShape) -> PropValue {
    match r {
        RailShape::AscEast => PropValue::AscendingEast,
        RailShape::AscNorth => PropValue::AscendingNorth,
        RailShape::AscSouth => PropValue::AscendingSouth,
        RailShape::AscWest => PropValue::AscendingWest,
        RailShape::EastWest => PropValue::EastWest,
        RailShape::NorthEast => PropValue::NorthEast,
        RailShape::NorthSouth => PropValue::NorthSouth,
        RailShape::NorthWest => PropValue::NorthWest,
        RailShape::SouthEast => PropValue::SouthEast,
        RailShape::SouthWest => PropValue::SouthWest,
    }
}

fn dir_to_prop_val(dir: &Direction) -> PropValue {
    match dir {
        &Direction::North => PropValue::North,
        &Direction::NorthEast => PropValue::NorthEast,
        &Direction::East => PropValue::East,
        &Direction::SouthEast => PropValue::SouthEast,
        &Direction::South => PropValue::South,
        &Direction::SouthWest => PropValue::SouthWest,
        &Direction::West => PropValue::West,
        &Direction::NorthWest => PropValue::NorthWest,
        &Direction::Up => PropValue::Up,
        &Direction::Down => PropValue::Down,
        _ => PropValue::South,
    }
}

fn rot_to_prop_val(dir: &Direction) -> PropValue {
    match dir {
        Direction::South => PropValue::_0,
        Direction::SouthSouthWest => PropValue::_1,
        Direction::SouthWest => PropValue::_2,
        Direction::WestSouthWest => PropValue::_3,
        Direction::West => PropValue::_4,
        Direction::WestNorthWest => PropValue::_5,
        Direction::NorthWest => PropValue::_6,
        Direction::NorthNorthWest => PropValue::_7,
        Direction::North => PropValue::_8,
        Direction::NorthNorthEast => PropValue::_9,
        Direction::NorthEast => PropValue::_10,
        Direction::EastNorthEast => PropValue::_11,
        Direction::East => PropValue::_12,
        Direction::EastSouthEast => PropValue::_13,
        Direction::SouthEast => PropValue::_14,
        Direction::SouthSouthEast => PropValue::_15,
        _ => unreachable!(),
    }
}

fn bool_to_prop_val(b: bool) -> PropValue {
    if b {
        PropValue::True
    } else {
        PropValue::False
    }
}

fn num_to_prop_val(num: &u8) -> PropValue {
    match *num {
        0 => PropValue::_0,
        1 => PropValue::_1,
        2 => PropValue::_2,
        3 => PropValue::_3,
        4 => PropValue::_4,
        5 => PropValue::_5,
        6 => PropValue::_6,
        7 => PropValue::_7,
        8 => PropValue::_8,
        9 => PropValue::_9,
        10 => PropValue::_10,
        11 => PropValue::_11,
        12 => PropValue::_12,
        13 => PropValue::_13,
        14 => PropValue::_14,
        15 => PropValue::_15,
        16 => PropValue::_16,
        17 => PropValue::_17,
        18 => PropValue::_18,
        19 => PropValue::_19,
        20 => PropValue::_20,
        21 => PropValue::_21,
        22 => PropValue::_22,
        23 => PropValue::_23,
        24 => PropValue::_24,
        25 => PropValue::_25,
        _ => PropValue::_0,
    }
}
