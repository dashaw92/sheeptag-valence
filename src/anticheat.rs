use valence::{
    app::{Plugin, Update},
    client::Client,
    entity::{
        active_status_effects::{ActiveStatusEffect, ActiveStatusEffects},
        attributes::{EntityAttribute, EntityAttributes},
        living::Health,
        player::Food,
    },
    prelude::{Added, Query},
    status_effects::StatusEffect,
    uuid::Uuid,
};

//Not really an anticheat, I just couldn't think of a better name for what this does.
//The goal of this plugin is to disable jumping and set health to a lower amount.
//A side effect of adding jump boost level 128 to clients (how jump is disabled)
//is that clients can still spam jump while sprinting to achieve super fast speeds.
//This isn't really a problem, but it's kinda silly and it puts new players at a
//huge disadvantage. To rectify this, players also have their food set to 0, which
//disables their ability to sprint. Since walking is as slow as it is, their
//movement speed is then increased slightly just to make it less boring to move around.
pub struct AnticheatPlugin;

impl Plugin for AnticheatPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(Update, setup);
    }
}

fn setup(
    mut clients: Query<
        (
            &mut EntityAttributes,
            &mut Health,
            &mut Food,
            &mut ActiveStatusEffects,
        ),
        Added<Client>,
    >,
) {
    for (mut attributes, mut hp, mut food, mut statuses) in &mut clients {
        statuses.apply(
            ActiveStatusEffect::from_effect(StatusEffect::JumpBoost)
                .with_infinite()
                .with_ambient(true)
                .with_show_particles(false)
                .with_show_icon(false)
                .with_amplifier(128),
        );

        hp.0 = 6.0f32;
        food.0 = 0;

        attributes.set_base_value(EntityAttribute::GenericMaxHealth, 6.0);
        attributes.set_add_modifier(EntityAttribute::GenericMovementSpeed, Uuid::nil(), 0.03);
    }
}
