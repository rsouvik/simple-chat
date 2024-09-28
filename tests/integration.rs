use tokio_websockets::{ClientBuilder, Message}; // Import Message from the crate
use http::Uri;
use futures_util::{stream::StreamExt, SinkExt};
//use futures_util::sink::SinkExt;

#[tokio::test]
async fn test_chat_interaction() {
    let server_handle = tokio::spawn(async {
        // Start the server
        bin::server::main().await.unwrap();
    });

    let client1_handle = tokio::spawn(async {
        let mut ws_stream = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:2000"))
            .connect()
            .await
            .unwrap();

        let (mut ws_stream, _) = ws_stream;

        ws_stream.send(Message::text("/join user1")).await.unwrap();
        ws_stream.send(Message::text("send Hello, this is user1")).await.unwrap();

        let received = ws_stream.next().await.unwrap();
        assert!(received.as_text().unwrap().contains("INFO: user1 has joined"));
        assert!(received.as_text().unwrap().contains("Hello, this is user1"));
    });

    let client2_handle = tokio::spawn(async {
        let mut ws_stream = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:2000"))
            .connect()
            .await
            .unwrap();

        let (mut ws_stream, _) = ws_stream;
        ws_stream.send(Message::text("/join user2")).await.unwrap();
        let received = ws_stream.next().await.unwrap();
        assert!(received.as_text().unwrap().contains("INFO: user1 has joined"));

        ws_stream.send(Message::text("send Hello from user2")).await.unwrap();
        let received = ws_stream.next().await.unwrap();
        assert!(received.as_text().unwrap().contains("Hello from user2"));
    });

    let _ = tokio::join!(server_handle, client1_handle, client2_handle);
}
