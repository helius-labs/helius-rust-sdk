use helius::error::Result;
use helius::types::Cluster;
use helius::utils::collection_authority::{get_collection_authority_record, get_collection_metadata_account};
use helius::Helius;
use mpl_token_metadata::instructions::CreateMetadataAccountV3;
use mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs;
use mpl_token_metadata::types::DataV2;
use solana_program::system_instruction::create_account;
use solana_program::system_program;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::solana_program::program_pack::Pack;
use spl_token::{instruction::initialize_mint, state::Mint};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = ""; // Replace with your Helius API key
    let payer = Keypair::from_base58_string("");
    let cluster = Cluster::MainnetBeta;
    let helius = Helius::new_with_async_solana(api_key, cluster)?;
    // Get the async Solana RPC client from Helius
    let rpc_client = helius.async_connection()?;

    // Create a new SPL Token mint (collection mint)
    let collection_mint_keypair = Keypair::new();
    // Calculate rent-exempt amount for the mint account
    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await
        .expect("Failed to get rent exemption amount");
    // Create mint account instruction
    let create_mint_account_ix = create_account(
        &payer.pubkey(),
        &collection_mint_keypair.pubkey(),
        rent,
        Mint::LEN as u64,
        &spl_token::id(),
    );
    let collection_authority_keypair = Keypair::new();
    // Initialize the mint instruction
    let initialize_mint_ix = initialize_mint(
        &spl_token::id(),
        &collection_mint_keypair.pubkey(),
        &collection_authority_keypair.pubkey(),
        None,
        9,
    )
    .expect("Failed to create initialize mint instruction");
    let recent_blockhash = rpc_client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &[create_mint_account_ix, initialize_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &collection_mint_keypair],
        recent_blockhash,
    );
    rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("Failed to create and initialize mint");
    println!(
        "Collection mint created and initialized: {}",
        collection_mint_keypair.pubkey()
    );

    // Create Metadata account for the collection mint
    let metadata_pubkey = get_collection_metadata_account(&collection_mint_keypair.pubkey());
    let create_metadata_accounts_ix = CreateMetadataAccountV3 {
        metadata: metadata_pubkey,
        mint: collection_mint_keypair.pubkey(),
        mint_authority: collection_authority_keypair.pubkey(),
        payer: payer.pubkey(),
        update_authority: (collection_authority_keypair.pubkey(), true),
        system_program: system_program::ID,
        rent: None,
    }
    .instruction(CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name: "".to_string(),
            symbol: "".to_string(),
            uri: "".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    });
    let recent_blockhash = rpc_client.get_latest_blockhash().await?;
    // Send the transaction to create the metadata account
    let transaction = Transaction::new_signed_with_payer(
        &[create_metadata_accounts_ix],
        Some(&payer.pubkey()),
        &[&payer, &collection_authority_keypair],
        recent_blockhash,
    );
    rpc_client.send_and_confirm_transaction(&transaction).await?;
    println!("Metadata account created: {}", metadata_pubkey.to_string());

    let delegated_authority_keypair = Keypair::new();
    let result = helius
        .delegate_collection_authority(
            collection_mint_keypair.pubkey(),
            delegated_authority_keypair.pubkey(),
            &collection_authority_keypair,
            Some(&payer),
        )
        .await;
    assert!(
        result.is_ok(),
        "Failed to delegate collection authority to {}",
        delegated_authority_keypair.pubkey()
    );
    println!(
        "Delegate collection authority to {} transaction signature: {}",
        delegated_authority_keypair.pubkey(),
        result?
    );

    // The record for delegated collection authority should exist in blockchain
    let collection_authority_record =
        get_collection_authority_record(&collection_mint_keypair.pubkey(), &delegated_authority_keypair.pubkey());
    let account = rpc_client.get_account(&collection_authority_record).await;
    assert!(account.is_ok(), "Collection authority record account should exist");

    // Revoke the collection authority using your method
    let result = helius
        .revoke_collection_authority(
            collection_mint_keypair.pubkey(),
            Some(delegated_authority_keypair.pubkey()),
            &collection_authority_keypair,
            Some(&payer),
        )
        .await;
    assert!(result.is_ok(), "Failed to revoke collection authority");
    println!(
        "Revoke collection authority from {} transaction signature: {}",
        delegated_authority_keypair.pubkey(),
        result?
    );

    // Fetch the collection authority record account
    let account = rpc_client.get_account(&collection_authority_record).await;
    // The account should not exist if the authority was revoked
    assert!(account.is_err(), "Collection authority record account should be closed");

    println!("Delegated collection authority successfully revoked");
    Ok(())
}
