use valence::{
    app::{Plugin, Update},
    client::Client,
    entity::{
        active_status_effects::{ActiveStatusEffect, ActiveStatusEffects},
        attributes::{EntityAttribute, EntityAttributes},
        living::Health,
        player::Food,
    },
    prelude::{Added, OnInsert, OnRemove, Query, Trigger},
    status_effects::StatusEffect,
    uuid::Uuid,
    GameMode,
};

use crate::perms::OperMode;

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
        app.add_systems(Update, setup)
            .observe(gm_mode_enable)
            .observe(gm_mode_disable);
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
        disable_jump(&mut statuses);

        hp.0 = 6.0f32;
        food.0 = 0;

        attributes.set_base_value(EntityAttribute::GenericMaxHealth, 6.0);
        attributes.set_add_modifier(EntityAttribute::GenericMovementSpeed, Uuid::nil(), 0.03);
    }
}

fn disable_jump(statuses: &mut ActiveStatusEffects) {
    statuses.apply(
        ActiveStatusEffect::from_effect(StatusEffect::JumpBoost)
            .with_infinite()
            .with_ambient(true)
            .with_show_particles(false)
            .with_show_icon(false)
            .with_amplifier(128),
    );
}

fn enable_jump(statuses: &mut ActiveStatusEffects) {
    statuses.remove(StatusEffect::JumpBoost);
}

fn gm_mode_enable(
    trigger: Trigger<OnInsert, OperMode>,
    mut clients: Query<(&mut ActiveStatusEffects, &mut GameMode)>,
) {
    let ent = trigger.entity();
    if let Ok((mut statuses, mut gm)) = clients.get_mut(ent) {
        enable_jump(&mut statuses);
        *gm = GameMode::Creative;
    }
}

fn gm_mode_disable(
    trigger: Trigger<OnRemove, OperMode>,
    mut clients: Query<(&mut ActiveStatusEffects, &mut GameMode)>,
) {
    let ent = trigger.entity();
    if let Ok((mut statuses, mut gm)) = clients.get_mut(ent) {
        disable_jump(&mut statuses);
        *gm = GameMode::Survival;
    }
}
