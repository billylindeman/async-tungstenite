use async_tungstenite::{
    accept_async,
    gio_futures::{SocketConnection, SocketListener},
    tungstenite::{Error, Result},
};
use futures::prelude::*;
use log::*;

async fn accept_connection(peer: gio::SocketAddress, stream: SocketConnection) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: gio::SocketAddress, stream: SocketConnection) -> Result<()> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    info!("New WebSocket connection: {}", peer);

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await?;
        }
    }

    Ok(())
}

async fn run() {
    env_logger::init();

    let port = 9002;
    let listener = SocketListener::new();
    listener.add_inet_port(port).expect("error setting port");
    info!("Listening on: {}", port);

    let mut incoming = listener.incoming();
    while let Some(conn) = incoming.next().await {
        let conn = conn.unwrap();
        let peer = conn
            .get_remote_address()
            .expect("connected streams should have a peer address");

        info!("Peer address: {}", peer);

        glib::MainContext::default().spawn(accept_connection(peer, conn))
    }
}

fn main() {
    let ctx = glib::MainContext::default();
    ctx.push_thread_default();
    ctx.block_on(run());
}
