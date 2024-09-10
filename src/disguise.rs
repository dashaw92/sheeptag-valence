use std::borrow::Cow;

use bevy_ecs::query::QueryData;
use valence::app::Plugin;
use valence::entity::entity::{CustomName, NameVisible};
use valence::entity::iron_golem::IronGolemEntityBundle;
use valence::entity::sheep::{Color, SheepEntityBundle};
use valence::entity::{EntityAnimations, EntityId, EntityStatuses, OnGround, Velocity};
use valence::prelude::*;
use valence::protocol::packets::play::team_s2c::TeamFlags;
use valence::protocol::packets::play::{team_s2c, EntitiesDestroyS2c, TeamS2c};
use valence::protocol::VarInt;
use valence::protocol::WritePacket;
use valence::scoreboard::{Objective, ObjectiveBundle, ObjectiveDisplay, ObjectiveScores};
use valence::text::color::{NamedColor, RgbColor};

use crate::color::{ColorMap, PlayerColor};
use crate::teams::{JoinTeamEvent, Team};

/*
TODO: I'm currently thinking that I can make a public API for this via events?
Something like:
```rust
event_writer.send(RequestDisguise {
    entity: Entity,
    disguise: Disguise::Sheep,
});
```

And in this module, I'll have a system that listens for those events and
retrieves the fields needed to actually execute that request.

RequestDisguise
Undisguise
*/

pub struct DisguisePlugin;

impl Plugin for DisguisePlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (init_clients, spawn_clones, update_clones, update_scoreboard),
        );
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
    id: &'static EntityId,
    position: &'static mut Position,
    head_yaw: &'static mut HeadYaw,
    velocity: &'static mut Velocity,
    look: &'static mut Look,
    animations: &'static mut EntityAnimations,
    on_ground: &'static mut OnGround,
    statuses: &'static mut EntityStatuses,
}

fn update_clones(
    mut ents: Query<(CloneQueryReadOnly, &mut Client), Without<ClonedEntity>>,
    mut clones: Query<(CloneQuery, &ClonedEntity, Entity)>,
    mut commands: Commands,
) {
    for clone in &mut clones {
        let (mut clone, cloned_from, ent) = clone;
        let Ok((src, mut client)) = ents.get_mut(cloned_from.0) else {
            commands.entity(ent).insert(Despawned);
            return;
        };

        //Hide clones from owners. Even though I could make
        //the player invisible, golems obscure vision due to their
        //size.
        client.write_packet(&EntitiesDestroyS2c {
            entity_ids: Cow::Borrowed(&[VarInt(clone.id.get())]),
        });

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
    mut query: Query<(
        &Position,
        // &EntityLayerId,
        &Username,
        &mut VisibleEntityLayers,
    )>,
    mut events: EventReader<JoinTeamEvent>,
    mut commands: Commands,
    plug_res: Res<DisguiseResource>,
) {
    for event in events.read() {
        let JoinTeamEvent {
            entity,
            team,
            color,
        } = event;

        let Ok((pos, ign, mut layers)) = query.get_mut(*entity) else {
            continue;
        };

        let ign = format_ign(ign, *color);

        match *team {
            Team::Sheep => commands.spawn((
                SheepEntityBundle {
                    sheep_color: to_sheep_color(color),
                    layer: EntityLayerId(plug_res.player_team_layer),
                    position: *pos,
                    entity_name_visible: NameVisible(true),
                    entity_custom_name: CustomName(Some(ign)),
                    ..Default::default()
                },
                ClonedEntity(*entity),
            )),
            Team::Golem => commands.spawn((
                IronGolemEntityBundle {
                    layer: EntityLayerId(plug_res.player_team_layer),
                    position: *pos,
                    entity_name_visible: NameVisible(true),
                    entity_custom_name: CustomName(Some(ign)),
                    ..Default::default()
                },
                ClonedEntity(*entity),
            )),
        };

        commands.entity(*entity).insert(match *team {
            Team::Sheep => Disguise::Sheep,
            Team::Golem => Disguise::Golem,
        });

        // layers.0.remove(&layer.0);
        layers.0.insert(plug_res.player_team_layer);
        layers.0.insert(plug_res.scoreboard_layer);
    }
}

#[derive(Debug, Resource)]
struct DisguiseResource {
    player_team_layer: Entity,
    scoreboard_layer: Entity,
}

fn setup(mut commands: Commands, server: Res<Server>) {
    let objective_layer = commands.spawn(EntityLayer::new(&server)).id();
    let objective = ObjectiveBundle {
        name: Objective::new("sheeptag"),
        display: ObjectiveDisplay("Sheeptag".color(NamedColor::Gold)),
        layer: EntityLayerId(objective_layer),
        // position: valence::protocol::packets::play::scoreboard_display_s2c::ScoreboardPosition::SidebarTeam(Black),
        ..Default::default()
    };
    let player_team_layer = commands.spawn(EntityLayer::new(&server)).id();
    commands.spawn(objective);

    commands.insert_resource(DisguiseResource {
        scoreboard_layer: objective_layer,
        player_team_layer,
    });
}

fn init_clients(
    mut clients: Query<(&mut Client, &mut VisibleEntityLayers, &Username), Added<Client>>,
    plug_res: Res<DisguiseResource>,
) {
    for (mut client, mut layers, ign) in &mut clients {
        client.write_packet(&TeamS2c {
            team_name: "no_collide",
            mode: team_s2c::Mode::CreateTeam {
                team_display_name: "NoCollide".into_cow_text(),
                friendly_flags: TeamFlags::default(),
                name_tag_visibility: team_s2c::NameTagVisibility::Always,
                collision_rule: team_s2c::CollisionRule::Never,
                team_color: team_s2c::TeamColor::White,
                team_prefix: "".into_cow_text(),
                team_suffix: "".into_cow_text(),
                entities: vec![&ign.0],
            },
        });

        layers.0.insert(plug_res.player_team_layer);
    }
}

fn update_scoreboard(
    players: Query<(&Username, &Team, &PlayerColor)>,
    mut objectives: Query<&mut ObjectiveScores, With<Objective>>,
    colors: Res<ColorMap>,
) {
    if !colors.is_changed() {
        return;
    }

    //TODO: This might be a bad way of getting the objective?
    let mut obj = objectives.single_mut();

    let mut i = 0;
    for (ign, _team, _color) in &players {
        obj.insert(format!("{ign}"), i);
        i += 1;
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

fn format_ign(ign: &Username, color: PlayerColor) -> Text {
    let text_color = match color {
        PlayerColor::White => RgbColor::new(249, 255, 254),
        PlayerColor::Orange => RgbColor::new(249, 128, 29),
        PlayerColor::Magenta => RgbColor::new(199, 78, 189),
        PlayerColor::Cyan => RgbColor::new(58, 179, 218),
        PlayerColor::Yellow => RgbColor::new(254, 216, 61),
        PlayerColor::Lime => RgbColor::new(128, 199, 31),
        PlayerColor::Pink => RgbColor::new(243, 139, 170),
        PlayerColor::DarkGray => RgbColor::new(71, 79, 82),
        PlayerColor::LightGray => RgbColor::new(157, 157, 151),
        PlayerColor::Aqua => RgbColor::new(22, 156, 156),
        PlayerColor::Purple => RgbColor::new(137, 50, 184),
        PlayerColor::Blue => RgbColor::new(60, 68, 170),
        PlayerColor::Brown => RgbColor::new(131, 84, 50),
        PlayerColor::Green => RgbColor::new(94, 124, 22),
        PlayerColor::Red => RgbColor::new(176, 46, 38),
        PlayerColor::Black => RgbColor::new(29, 29, 33),
    };

    ign.0.clone().color(text_color)
}
