use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use structopt::StructOpt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long, default_value = "2000")]
    port: String,

    #[structopt(short, long)]
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let args = Cli::from_args();
    let ws_url = format!("ws://{}:{}/", args.host, args.port);

    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_maybe_shared(ws_url).unwrap())
        .connect()
        .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    // Send the join message immediately
    ws_stream
        .send(Message::text(format!("/join {}", args.username)))
        .await?;

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From server: {}", text);
                        }
                    },
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }

            res = stdin.next_line() => {
                match res {
                    Ok(None) => return Ok(()),
                    Ok(Some(line)) => {
                        if line.starts_with("send ") {
                            let msg = line[5..].to_string();
                            ws_stream.send(Message::text(msg)).await?;
                        } else if line == "leave" {
                            ws_stream.send(Message::text("/leave".to_string())).await?;
                            return Ok(());
                        } else {
                            println!("Unknown command.");
                        }
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        }
    }
}