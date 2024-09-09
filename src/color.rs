use std::collections::HashMap;

use valence::prelude::*;
use PlayerColor::*;

use crate::teams::Team;

#[derive(Component, Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum PlayerColor {
    White,
    Orange,
    Magenta,
    Cyan,
    Yellow,
    Lime,
    Pink,
    DarkGray,
    LightGray,
    Aqua,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Resource, Debug)]
pub struct ColorMap {
    players: HashMap<PlayerColor, Entity>,
}

impl ColorMap {
    pub(crate) fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub(crate) fn register_player(
        &mut self,
        entity: Entity,
        team: &Team,
    ) -> Result<PlayerColor, ()> {
        let Some(next_color) = PlayerColor::iter()
            .filter(|col| col.valid_for_team(team))
            .filter(|col| !self.players.contains_key(col))
            .next()
        else {
            return Err(());
        };

        self.players.insert(next_color, entity);
        Ok(next_color)
    }

    pub(crate) fn unregister_player(&mut self, entity: Entity) {
        match self.color_of_player(entity) {
            Some(ref color) => {
                self.players.remove(color);
            }
            None => {}
        }
    }

    pub(crate) fn color_of_player(&self, entity: Entity) -> Option<PlayerColor> {
        self.players
            .iter()
            .find(|(_, &e)| e == entity)
            .map(|(color, _)| color)
            .copied()
    }
}

impl PlayerColor {
    pub(crate) fn iter() -> impl Iterator<Item = PlayerColor> {
        [
            White, Orange, Magenta, Cyan, Yellow, Lime, Pink, DarkGray, LightGray, Aqua, Purple,
            Blue, Brown, Green, Red, Black,
        ]
        .into_iter()
    }

    pub(crate) fn team(&self) -> Team {
        match *self {
            White | DarkGray | LightGray | Black => Team::Golem,
            _ => Team::Sheep,
        }
    }

    pub(crate) fn valid_for_team(&self, team: &Team) -> bool {
        self.team() == *team
    }
}
