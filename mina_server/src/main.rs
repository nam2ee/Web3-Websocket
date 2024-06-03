use warp::Filter;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::convert::Infallible;
use futures_util::{StreamExt, SinkExt};
use tokio::task;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BlockData {
    creator: String,
    state_hash: String,
    block_height: u64,
    previous_state_hash: String,
    coinbase: u64,
}

async fn fetch_block_data(client: &Client) -> Result<BlockData, reqwest::Error> {
    let query = r#"
        query {
          bestChain(maxLength: 1) {
            creator
            stateHash
            protocolState {
              consensusState {
                blockHeight
              }
              previousStateHash
            }
            transactions {
              coinbase
            }
          }
        }
    "#;

    let response = client.post("http://localhost:3085/graphql")
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("Response: {:?}", response); // 응답 데이터 출력

    let block_info = &response["data"]["bestChain"][0];
    let protocol_state = &block_info["protocolState"];
    let consensus_state = &protocol_state["consensusState"];
    let transactions = &block_info["transactions"];

    Ok(BlockData {
        creator: block_info["creator"].as_str().unwrap_or_default().to_string(),
        state_hash: block_info["stateHash"].as_str().unwrap_or_default().to_string(),
        block_height: consensus_state["blockHeight"].as_u64().unwrap_or_default(),
        previous_state_hash: protocol_state["previousStateHash"].as_str().unwrap_or_default().to_string(),
        coinbase: transactions["coinbase"].as_u64().unwrap_or_default(),
    })
}

async fn websocket_handler(ws: warp::ws::Ws, tx: broadcast::Sender<BlockData>) -> Result<impl warp::Reply, Infallible> {
    Ok(ws.on_upgrade(move |socket| {
        let (mut ws_tx, mut ws_rx) = socket.split();
        let mut rx = tx.subscribe();

        async move {
            task::spawn(async move {
                while let Some(result) = ws_rx.next().await {
                    if let Ok(msg) = result {
                        if msg.is_text() {
                            // Handle incoming messages if needed
                        }
                    }
                }
            });

            while let Ok(block_data) = rx.recv().await {
                let msg = warp::ws::Message::text(serde_json::to_string(&block_data).unwrap());
                if let Err(e) = ws_tx.send(msg).await {
                    eprintln!("websocket send error: {}", e);
                    break;
                }
            }
        }
    }))
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(32);
    let client = Client::new();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        loop {
            match fetch_block_data(&client).await {
                Ok(block_data) => {
                    if tx_clone.send(block_data).is_err() {
                        eprintln!("receiver dropped");
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("fetch error: {}", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || tx.clone()))
        .and_then(websocket_handler);

    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}
