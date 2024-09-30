use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast::{channel, Sender}, Mutex};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};
use std::sync::Arc;

// Structure to hold user data
struct User {
    username: String,
    addr: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
}

// Structure to hold server state
struct ServerState {
    users: Mutex<HashMap<SocketAddr, String>>, // map addr to username
    bcast_tx: Sender<String>, // broadcast channel for sending messages to all users
}

//Make sure to broadcast to all others except sender
impl ServerState {
    async fn broadcast_message(&self, addr: &SocketAddr, message: String) {
        let users = self.users.lock().await;
        let sender_name = users.get(addr).unwrap();
        let full_msg = format!("{}: {}", sender_name, message);

        for (user_addr, _) in users.iter() {
            if user_addr != addr {
                self.bcast_tx.send(full_msg.clone()).unwrap();
            }
        }
    }
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    state: Arc<ServerState>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Send a welcome message
    ws_stream.send(Message::text("Welcome to chat! Type '/join <username>' to join.".to_string())).await?;

    let mut bcast_rx = state.bcast_tx.subscribe();
    let mut username: Option<String> = None;

    loop {
        tokio::select! {
            // Handle incoming messages from the client
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            if text.starts_with("/join ") {
                                // Handle user joining
                                let new_username = text[6..].trim().to_string();
                                let mut users = state.users.lock().await;

                                if users.values().any(|name| name == &new_username) {
                                    ws_stream.send(Message::text("Username already taken.".to_string())).await?;
                                } else {
                                    users.insert(addr, new_username.clone());
                                    ws_stream.send(Message::text(format!("Joined as {}", new_username))).await?;
                                    state.bcast_tx.send(format!("{} has joined the chat.", new_username))?;
                                    username = Some(new_username);
                                }
                            } else if text == "/leave" {
                                // Handle user leaving
                                if let Some(name) = username.take() {
                                    let mut users = state.users.lock().await;
                                    users.remove(&addr);
                                    state.bcast_tx.send(format!("{} has left the chat.", name))?;
                                    return Ok(());
                                }
                            } else if let Some(_) = username {
                                // Broadcast regular messages
                                state.broadcast_message(&addr, text.into()).await;
                            } else {
                                ws_stream.send(Message::text("Please join with '/join <username>' first.".to_string())).await?;
                            }
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }

            // Handle messages from the broadcast channel
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let state = Arc::new(ServerState {
        users: Mutex::new(HashMap::new()),
        bcast_tx: bcast_tx.clone(),
    });

    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("Listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, state.clone()).await
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;
    use tokio_websockets::{Message, WebSocketStream, ServerBuilder};
    use futures_util::stream::StreamExt;
    use tokio::net::TcpListener;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_user_join() {
        // Create a broadcast channel for the test
        let (bcast_tx, _) = broadcast::channel(16);
        let state = Arc::new(ServerState {
            users: Mutex::new(HashMap::new()),
            bcast_tx: bcast_tx.clone(),
        });

        // Set up a TCP listener to simulate a server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap(); // Bind to an available port
        let addr = listener.local_addr().unwrap();


        // Spawn a task to accept a connection in the background
        let server_state = state.clone();
        tokio::spawn(async move {
            let (socket, _peer_addr) = listener.accept().await.unwrap();
            //let ws_stream = ServerBuilder::new().accept(socket).await.unwrap();
            let (mut ws_stream, _) = tokio_websockets::ServerBuilder::new().accept(mock_socket).await.unwrap();
            handle_connection(_peer_addr, ws_stream, server_state).await.unwrap();
        });

        // Simulate a client connecting to the server
        let client_socket = TcpStream::connect(addr).await.unwrap();
        let mut ws_stream = tokio_websockets::ClientBuilder::new()
            .connect(client_socket)
            .await
            .unwrap();

        // Simulate sending a join message from the client
        ws_stream.send(Message::text("/join username")).await.unwrap();

        // Simulate receiving the welcome message from the server
        if let Some(Ok(received)) = ws_stream.next().await {
            assert!(received.as_text().unwrap().contains("Joined as username"));
        }

        // Check that error is raised on duplicate username
        ws_stream.send(Message::text("/join username")).await.unwrap();
        if let Some(Ok(received)) = ws_stream.next().await {
            assert!(received.as_text().unwrap().contains("Username already taken"));
        }
    }
}