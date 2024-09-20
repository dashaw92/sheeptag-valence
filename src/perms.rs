use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use valence::{
    app::Plugin,
    log::{self},
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

        app.insert_resource(perms).add_systems(Update, monitor_ops);
    }
}

impl Permissions {
    pub fn is_owner(&self, player: &Uuid) -> bool {
        Some(player) == self.owner.as_ref()
    }

    pub fn is_op(&self, player: &Uuid) -> bool {
        self.is_owner(player) || self.ops.contains(player)
    }

    pub fn set_op(&mut self, player: &Uuid, op: bool) {
        //Cannot op/deop the owner. The owner status can
        //only be applied via manual edits to ops.txt.
        if self.is_owner(player) {
            return;
        }

        if !self.is_op(player) && op {
            self.ops.push_back(player.clone());
        } else if self.is_op(player) && !op {
            self.ops.retain(|uuid| uuid != player);
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
