use std::str::FromStr;

use valence::{
    command::{
        handler::CommandResultEvent,
        parsers::{CommandArg, CommandArgParseError, ParseInput},
        scopes::CommandScopes,
        AddCommand, CommandScopeRegistry,
    },
    command_macros::Command,
    prelude::*,
    protocol::packets::play::command_tree_s2c::{Parser, StringArg},
};

pub struct TeamPlugin;

impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        app.add_command::<JoinTeamCommand>()
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_join_command, init_clients));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Sheep,
    Golem,
}

impl FromStr for Team {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "sheep" => Team::Sheep,
            "golem" => Team::Golem,
            _ => return Err(format!("Invalid team '{s}'.")),
        })
    }
}

impl CommandArg for Team {
    fn parse_arg(input: &mut ParseInput) -> Result<Self, CommandArgParseError> {
        input.skip_whitespace();
        input
            .pop_word()
            .parse()
            .map_err(|msg| CommandArgParseError::InvalidArgument {
                expected: "team".to_owned(),
                got: msg,
            })
    }

    fn display() -> Parser {
        Parser::String(StringArg::SingleWord)
    }
}

#[derive(Command, Debug, Clone)]
#[paths("join {team?}")]
#[scopes("danny.sheeptag.join")]
pub struct JoinTeamCommand {
    team: Option<Team>,
}

fn handle_join_command(
    mut events: EventReader<CommandResultEvent<JoinTeamCommand>>,
    mut clients: Query<&mut Client, Without<Team>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let Ok(mut client) = clients.get_mut(event.executor) else {
            return;
        };

        match event.result.team {
            Some(team) => {
                commands.entity(event.executor).insert(team);
                client.send_chat_message(format!("Joined team {team:?}."));
            }
            None => {
                client.send_chat_message("Usage: /join <golem|sheep>");
            }
        }
    }
}

fn setup(mut cmd_scopes: ResMut<CommandScopeRegistry>) {
    cmd_scopes.add_scope("danny.sheeptag");
}

fn init_clients(mut clients: Query<&mut CommandScopes, Added<Client>>) {
    for mut perms in &mut clients {
        perms.add("danny.sheeptag");
    }
}
