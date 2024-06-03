use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee::core::client::SubscriptionClientT;
use futures::StreamExt;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // WebSocket 클라이언트 생성
    let client = WsClientBuilder::default()
        .build("wss://api.avax-test.network:443/ext/bc/C/ws")
        .await?;

    // 새로운 블록을 구독
    let mut subscription = client
        .subscribe::<Value>("eth_subscribe", Some(vec!["newHeads".into()].into()), "eth_unsubscribe")
        .await?;

    // 새로운 블록이 생성될 때마다 알림 수신
    while let Some(block) = subscription.next().await {
        match block {
            Ok(block) => println!("New block: {:?}\n", block),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
