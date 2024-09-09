use bevy_ecs::query::QueryData;
use valence::app::Plugin;
use valence::entity::iron_golem::IronGolemEntityBundle;
use valence::entity::sheep::{Color, SheepEntityBundle};
use valence::entity::{EntityAnimations, EntityStatuses, OnGround, Velocity};
use valence::prelude::*;

use crate::color::PlayerColor;
use crate::teams::{JoinTeamEvent, Team};

pub struct DisguisePlugin;

impl Plugin for DisguisePlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(Update, (spawn_clones, update_clones));
    }
}

/// The current disguise taken by a player. This is the type of entity currently shadowing the player.
#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum Disguise {
    // BabySheep,
    Sheep,
    Golem,
    // Wolf,
}

// Note: This is largely copied and adjusted from the ctf.rs example on the valence-rs repo on GitHub.
// The license on that repo is MIT, so this is all fine

//Marker to find player clones
#[derive(Debug, Component)]
struct ClonedEntity(Entity);

//Fields that need to be mirrored by the clones to look realistic
#[derive(Debug, QueryData)]
#[query_data(mutable)]
struct CloneQuery {
    position: &'static mut Position,
    head_yaw: &'static mut HeadYaw,
    velocity: &'static mut Velocity,
    look: &'static mut Look,
    animations: &'static mut EntityAnimations,
    on_ground: &'static mut OnGround,
    statuses: &'static mut EntityStatuses,
}

fn update_clones(
    ents: Query<CloneQueryReadOnly, Without<ClonedEntity>>,
    mut clones: Query<(CloneQuery, &ClonedEntity, Entity)>,
    mut commands: Commands,
) {
    for clone in &mut clones {
        let (mut clone, cloned_from, ent) = clone;
        let Ok(src) = ents.get(cloned_from.0) else {
            commands.entity(ent).insert(Despawned);
            return;
        };

        *clone.position = *src.position;
        *clone.head_yaw = *src.head_yaw;
        *clone.look = *src.look;
        *clone.animations = *src.animations;
        *clone.on_ground = *src.on_ground;
        *clone.statuses = *src.statuses;
    }
}

//Spawn in a clone for a player when they join a team.
fn spawn_clones(
    query: Query<(&Position, &EntityLayerId)>,
    mut events: EventReader<JoinTeamEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        let JoinTeamEvent {
            entity,
            team,
            color,
        } = event;

        let Ok((pos, layer)) = query.get(*entity) else {
            continue;
        };

        match *team {
            Team::Sheep => commands.spawn((
                SheepEntityBundle {
                    sheep_color: to_sheep_color(color),
                    layer: *layer,
                    position: *pos,
                    ..Default::default()
                },
                ClonedEntity(*entity),
            )),
            Team::Golem => commands.spawn((
                IronGolemEntityBundle {
                    layer: *layer,
                    position: *pos,
                    ..Default::default()
                },
                ClonedEntity(*entity),
            )),
        };

        commands.entity(*entity).insert(match *team {
            Team::Sheep => Disguise::Sheep,
            Team::Golem => Disguise::Golem,
        });
    }
}

fn to_sheep_color(color: &PlayerColor) -> Color {
    //NOTE: Even though I don't intend for sheep to be able to have all 16 colors,
    //I will keep it functional in case I change my mind

    //Color codes found on https://minecraft.wiki/w/Sheep#Entity_data (Sheep color foldable)
    Color(match color {
        PlayerColor::White => 0,
        PlayerColor::Orange => 1,
        PlayerColor::Magenta => 2,
        PlayerColor::Cyan => 3,
        PlayerColor::Yellow => 4,
        PlayerColor::Lime => 5,
        PlayerColor::Pink => 6,
        PlayerColor::DarkGray => 7,
        PlayerColor::LightGray => 8,
        PlayerColor::Aqua => 9,
        PlayerColor::Purple => 10,
        PlayerColor::Blue => 11,
        PlayerColor::Brown => 12,
        PlayerColor::Green => 13,
        PlayerColor::Red => 14,
        PlayerColor::Black => 15,
    })
}
