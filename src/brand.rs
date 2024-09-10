use std::{net::SocketAddr, sync::atomic::Ordering};

use valence::{
    app::{Plugin, Update}, client::Client, network::{async_trait, ConnectionMode, HandshakeData, NetworkCallbacks, NetworkSettings, ServerListPing, SharedNetworkState}, prelude::{Added, Query}, text::IntoText, MINECRAFT_VERSION, PROTOCOL_VERSION
};

pub struct SheeptagBrandPlugin;

impl Plugin for SheeptagBrandPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Online {
                prevent_proxy_connections: false,
            },
            callbacks: SheeptagCallbacks.into(),
            ..Default::default()
        })
        .add_systems(Update, set_brand);
    }
}

struct SheeptagCallbacks;

#[async_trait]
impl NetworkCallbacks for SheeptagCallbacks {
    async fn server_list_ping(
        &self,
        shared: &SharedNetworkState,
        _: SocketAddr,
        _: &HandshakeData,
    ) -> ServerListPing {
        ServerListPing::Respond {
            online_players: shared.player_count().load(Ordering::Relaxed) as i32,
            max_players: shared.max_players() as i32,
            player_sample: vec![],
            description: "Sheeptag!".into_text(),
            favicon_png: &[],
            version_name: MINECRAFT_VERSION.to_owned(),
            protocol: PROTOCOL_VERSION,
        }
       
    }
}

fn set_brand(
    mut clients: Query<&mut Client,
        Added<Client>,
    >,
) {
    use valence::brand::SetBrand;
    
    for mut client in &mut clients {
        client.set_brand("\u{00A7}6Sheeptag\u{00A7}f by \u{00A7}1Danny\u{00A7}r");
    }
}
