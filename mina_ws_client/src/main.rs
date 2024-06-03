use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::stream::StreamExt;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // WebSocket 클라이언트 생성
    let url = url::Url::parse("ws://localhost:3030/ws")?;
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // 구독된 데이터 수신
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let block_data: Value = serde_json::from_str(&text)?;
                println!("New block data: {:?}", block_data);
            }
            Ok(Message::Binary(_)) => {
                eprintln!("Received binary data, ignoring...");
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
