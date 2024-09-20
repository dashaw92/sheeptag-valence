use anticheat::AnticheatPlugin;
use disguise::DisguisePlugin;
use perms::PermissionsPlugin;
use teams::TeamPlugin;
use valence::app::{PluginGroup, PluginGroupBuilder};

pub mod anticheat;
pub mod brand;
pub mod color;
pub mod disguise;
pub mod perms;
pub mod teams;

pub struct SheeptagPlugins;

impl PluginGroup for SheeptagPlugins {
    fn build(self) -> valence::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TeamPlugin)
            .add(DisguisePlugin)
            .add(AnticheatPlugin)
            .add(PermissionsPlugin)
    }
}
