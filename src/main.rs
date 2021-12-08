use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use serde::{Deserialize, Serialize};
/**
 * how does this work?
 * firstly, we have some source of data
 * secondly, we have some websockets connection
 * we forward that data along
*/
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task,
};
use tokio_tungstenite::tungstenite::protocol::Message;

use anyhow::{anyhow, Result};

type Tx = UnboundedSender<Message>;
type PubSubState = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

async fn data_intermediary<T: Serialize>(rx: Arc<Receiver<T>>, subs: PubSubState) {
    loop {
        if let Ok(x) = rx.try_recv() {
            if let Ok(s) = serde_json::to_string(&x) {
                let msg = Message::Text(s);
                for (addr, sock) in subs.lock().unwrap().iter() {
                    sock.unbounded_send(msg.clone()).unwrap();
                }
            }
        }
    }
}

async fn listener<K: ToSocketAddrs>(bind_addr: K, state: PubSubState) -> Result<()> {
    let tcp_socket = TcpListener::bind(&bind_addr).await?;
    println!("[SUCCESS]: LISTENER BOUND");
    while let Ok((stream, addr)) = tcp_socket.accept().await {
        tokio::spawn(json_server_conn_handler(state.clone(), stream, addr));
    }

    Ok(())
}

async fn json_server_conn_handler(state: PubSubState, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();
    state.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let handle_pubsub = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        future::ok(())
    });

    let receiver = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_pubsub, receiver);
    future::select(handle_pubsub, receiver).await;

    println!("{} disconnected", &addr);
    state.lock().unwrap().remove(&addr);
}

async fn spinup<T: Serialize, U: ToSocketAddrs>(source: Receiver<T>, addr: U) {
    let state = Arc::new(Mutex::new(HashMap::new()));
    let arc_rx = Arc::new(source);
    //task::spawn(listener(addr.clone(), state.clone()));
    task::spawn(data_intermediary(arc_rx.clone(), state.clone()));

    //tokio::spawn(listener(addr, state.clone()));
    //tokio::spawn(data_intermediary(source, state.clone()));
}

fn main() {}
