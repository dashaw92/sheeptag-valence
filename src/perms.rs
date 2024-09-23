use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use valence::{
    app::Plugin,
    command::{
        handler::CommandResultEvent, parsers::GreedyString, scopes::CommandScopes, AddCommand,
        CommandScopeRegistry,
    },
    command_macros::Command,
    log::{self},
    message::SendMessage,
    op_level::OpLevel,
    prelude::*,
    uuid::Uuid,
};

const OPS_FILE_PATH: &'static str = "ops.txt";

pub struct PermissionsPlugin;

#[derive(Resource, Default)]
pub struct Permissions {
    owner: Option<Uuid>,
    ops: VecDeque<Uuid>,
}

//Being an `op` in the Permissions struct means a player is an admin.
//Having this component marks them as being in "operator" mode,
//which negates specific conditions and enables them to perform
//administrative actions. Having this on at all times would hinder their
//ability to participate in the server activities.
#[derive(Component, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct OperMode;

fn load_perms<P: AsRef<Path>>(path: P) -> std::io::Result<Permissions> {
    let mut ops: VecDeque<Uuid> = std::fs::read_to_string(path)?
        .lines()
        .filter_map(|maybe_uuid| Uuid::parse_str(maybe_uuid).ok())
        .collect();

    let owner = ops.pop_front();

    Ok(Permissions { owner, ops })
}

impl Plugin for PermissionsPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        let perms = match load_perms(OPS_FILE_PATH) {
            Ok(perms) => perms,
            Err(_) => {
                log::info!("ops.txt not found. Attempting to create an empty ops.txt for you...");
                match File::create(OPS_FILE_PATH) {
                    Ok(f) => {
                        log::info!("ops.txt created. Please enter the server owner (your) UUID as the top line in this file.");
                        log::info!("-> {f:?}");
                        log::info!("This must be done while the server is offline, as the server periodically overwrites the file with new information.");
                        log::info!("If you do not add your UUID to this file, you will not be able to /op anyone, including yourself.");
                    }
                    Err(e) => {
                        log::warn!("ops.txt could not be created. Please resolve this. Error:");
                        log::error!("{e:?}");
                    }
                }

                Default::default()
            }
        };

        app.add_command::<OpCommand>()
            .add_command::<GmCommand>()
            .add_command::<DeopCommand>()
            .insert_resource(perms)
            .add_systems(Startup, register_scopes)
            .add_systems(
                Update,
                (
                    monitor_ops,
                    add_perms_to_ops,
                    handle_op_command,
                    handle_deop_command,
                    handle_gm_command,
                ),
            )
            .observe(notify_gm_mode_add)
            .observe(notify_gm_mode_remove);
    }
}

#[derive(Command)]
#[paths("deop {player}")]
#[scopes("danny.owner")]
struct DeopCommand {
    #[allow(dead_code)]
    player: GreedyString,
}

#[derive(Command)]
#[paths("op {player}")]
#[scopes("danny.op")]
struct OpCommand {
    #[allow(dead_code)]
    player: GreedyString,
}

#[derive(Command)]
#[paths("gm", "admin")]
#[scopes("danny.op")]
struct GmCommand;

impl Permissions {
    pub fn is_owner(&self, player: &Uuid) -> bool {
        Some(player) == self.owner.as_ref()
    }

    pub fn is_op(&self, player: &Uuid) -> bool {
        self.is_owner(player) || self.ops.contains(player)
    }

    pub fn set_op(&mut self, player: &Uuid, op: bool) -> bool {
        //Cannot op/deop the owner. The owner status can
        //only be applied via manual edits to ops.txt.
        if self.is_owner(player) {
            return false;
        }

        if !self.is_op(player) && op {
            self.ops.push_back(player.clone());
            return true;
        } else if self.is_op(player) && !op {
            self.ops.retain(|uuid| uuid != player);
            return true;
        }

        return false;
    }
}

fn register_scopes(mut scopes: ResMut<CommandScopeRegistry>) {
    scopes.add_scope("danny.owner");
    scopes.add_scope("danny.op");
}

fn add_perms_to_ops(
    mut clients: Query<(&UniqueId, &mut CommandScopes), Added<Client>>,
    ops: Res<Permissions>,
) {
    for (client, mut perms) in &mut clients {
        if ops.is_owner(client) {
            perms.add("danny.owner")
        }

        if ops.is_op(client) {
            perms.add("danny.op");
        }
    }
}

fn monitor_ops(perms: Res<Permissions>) {
    if !perms.is_changed() || perms.owner.is_none() {
        return;
    }

    let Ok(f) = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(OPS_FILE_PATH)
    else {
        log::error!("Failed to save updated ops.txt!");
        return;
    };

    let mut bw = BufWriter::new(f);

    _ = writeln!(
        bw,
        "{}",
        perms.owner.expect("Already checked if this is_some()")
    );
    for op in &perms.ops {
        _ = writeln!(bw, "{op}");
    }
}

fn handle_op_command(
    mut events: EventReader<CommandResultEvent<OpCommand>>,
    mut clients: Query<(&UniqueId, &Username, &mut Client, &mut CommandScopes)>,
    mut perms: ResMut<Permissions>,
) {
    for event in events.read() {
        let target_name = event.result.player.as_str();
        let Some((target_id, target_ign, mut target, mut scopes)) = clients
            .iter_mut()
            .find(|&(_, ign, _, _)| target_name == &ign.0)
        else {
            continue;
        };

        if perms.is_op(target_id) {
            continue;
        }

        if perms.set_op(target_id, true) {
            scopes.add("danny.op");
            target.send_chat_message("You are now op.");
            log::info!("{target_ign} is now op.");
        }
    }
}

fn handle_deop_command(
    mut events: EventReader<CommandResultEvent<DeopCommand>>,
    mut clients: Query<(
        &UniqueId,
        &Username,
        &mut Client,
        &mut CommandScopes,
        Entity,
    )>,
    mut perms: ResMut<Permissions>,
    mut commands: Commands,
) {
    for event in events.read() {
        let target_name = event.result.player.as_str();
        let Some((target_id, target_ign, mut target, mut scopes, target_ent)) = clients
            .iter_mut()
            .find(|&(_, ign, _, _, _)| target_name == &ign.0)
        else {
            continue;
        };

        if !perms.is_op(target_id) {
            continue;
        }

        if perms.set_op(target_id, false) {
            scopes.remove("danny.op");
            target.send_chat_message("You are no longer op.");
            commands.entity(target_ent).remove::<OperMode>();
            log::info!("{target_ign} is no longer op.");
        }
    }
}

fn handle_gm_command(
    mut events: EventReader<CommandResultEvent<GmCommand>>,
    clients: Query<(&Username, Has<OperMode>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let Ok((ign, is_gm)) = clients.get(event.executor) else {
            return;
        };

        let mut ent = commands.entity(event.executor);
        if is_gm {
            ent.remove::<OperMode>();
        } else {
            ent.insert(OperMode);
        }

        log::info!("{ign} toggled op mode.");
    }
}

fn notify_gm_mode_add(
    trigger: Trigger<OnInsert, OperMode>,
    mut clients: Query<(&mut Client, &mut OpLevel)>,
) {
    let ent = trigger.entity();
    if let Ok((mut client, mut oplevel)) = clients.get_mut(ent) {
        client.send_chat_message("You are now in op mode.");
        oplevel.set(3);
    }
}

fn notify_gm_mode_remove(
    trigger: Trigger<OnRemove, OperMode>,
    mut clients: Query<(&mut Client, &mut OpLevel)>,
) {
    let ent = trigger.entity();
    if let Ok((mut client, mut oplevel)) = clients.get_mut(ent) {
        client.send_chat_message("You are no longer in op mode.");
        oplevel.set(0);
    }
}
