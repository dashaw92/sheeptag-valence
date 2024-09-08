use valence::app::Plugin;
use valence::prelude::*;

pub struct DisguisePlugin;

impl Plugin for DisguisePlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        todo!()
    }
}

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Disguise {
    BabySheep,
    Sheep,
    Golem,
    Wolf,
}
