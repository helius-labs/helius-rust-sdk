use helius::error::HeliusError;
use helius::types::*;
use helius::Helius;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let request: MintCompressedNftRequest = MintCompressedNftRequest {
        name: "Exodia the Forbidden One".to_string(),
        symbol: "ETFO".to_string(),
        owner: "DCQnfUH6mHA333mzkU22b4hMvyqcejUBociodq8bB5HF".to_string(),
        description: "Exodia the Forbidden One is a powerful, legendary creature composed of five parts: the Right Leg, Left Leg, Right Arm, Left Arm, and the Head. When all five parts are assembled, Exodia becomes an unstoppable force".to_string(),
        attributes: vec![Attribute {
                trait_type: "Type".to_string(),
                value: Value::String("Legendary".to_string()),
            }, Attribute {
                trait_type: "Power".to_string(),
                value: Value::String("Infinite".to_string()),
            }, Attribute {
                trait_type: "Element".to_string(),
                value: Value::String("Dark".to_string()),
            }, Attribute {
                trait_type: "Rarity".to_string(),
                value: Value::String("Mythical".to_string()),
            },
        ],
        image_url: Some("https://cdna.artstation.com/p/assets/images/images/052/118/830/large/julie-almoneda-03.jpg?1658992401".to_string()),
        external_url: Some("https://www.yugioh-card.com/en/".to_string()),
        seller_fee_basis_points: Some(6900),
        delegate: None,
        collection: None,
        uri: None,
        creators: None,
        confirm_transaction: Some(true),
    };

    let response: Result<MintResponse, HeliusError> = helius.mint_compressed_nft(request).await;
    println!("Assets: {:?}", response);

    Ok(())
}
