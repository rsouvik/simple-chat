
#[tokio::main]
async fn main() -> std::io::Result<(),()> {
    // 1
    let listener = tokio::TcpListener::bind(ADDR).await?;
    // 2
    match litener.accept().await {
        // 3
        Ok((mut socket, addr)) => {
            // do something
        }

        Err(err) => println!("Error connecting. {}", err);
    }
}