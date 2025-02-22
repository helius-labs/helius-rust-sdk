use crate::error::Result;
use crate::types::{MintCompressedNftRequest, MintResponse};
use crate::utils::collection_authority::{
    delegate_collection_authority_instruction, revoke_collection_authority_instruction,
};
use crate::Helius;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

impl Helius {
    /// The easiest way to mint a compressed NFT (cNFT)
    ///
    /// # Arguments
    /// * `request` - A struct containing the various desired metadata of the cNFT to be minted, such as its name, symbol, URI, and attributes
    ///
    /// # Returns
    /// A `Result` containing a `MintResponse` detailing the transaction signature, asset ID, and whether the cNFT was minted successfully
    #[deprecated(
        since = "0.2.5",
        note = "Please refer to ZK Compression for all future compression-related work: https://docs.helius.dev/zk-compression-and-photon-api/what-is-zk-compression-on-solana"
    )]
    pub async fn mint_compressed_nft(&self, request: MintCompressedNftRequest) -> Result<MintResponse> {
        self.rpc_client.post_rpc_request("mintCompressedNft", request).await
    }

    /// Delegates collection authority to a new authority for a given collection mint.
    ///
    /// # Arguments
    /// * `collection_mint` - The public key of the collection mint.
    /// * `new_collection_authority` - The public key of the new authority to delegate to.
    /// * `update_authority_keypair` - The keypair of the current update authority who is delegating the authority.
    /// * `payer_keypair` - Optional keypair to pay for the transaction fees. If `None`, `update_authority_keypair` is used as the payer.
    ///
    /// # Returns
    /// A `Result` containing the transaction `Signature` if successful.
    pub async fn delegate_collection_authority(
        &self,
        collection_mint: Pubkey,
        new_collection_authority: Pubkey,
        update_authority_keypair: &Keypair,
        payer_keypair: Option<&Keypair>,
    ) -> Result<Signature> {
        let payer_keypair = payer_keypair.unwrap_or(update_authority_keypair);
        let delegate_instruction = delegate_collection_authority_instruction(
            collection_mint,
            new_collection_authority,
            update_authority_keypair,
            payer_keypair.pubkey(),
        );
        let recent_blockhash = self.async_connection()?.get_latest_blockhash().await?;
        let transaction = Transaction::new_signed_with_payer(
            &[delegate_instruction],
            Some(&payer_keypair.pubkey()),
            &[payer_keypair, update_authority_keypair],
            recent_blockhash,
        );
        self.async_connection()?
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|e| e.into())
    }

    /// Revokes a delegated collection authority for a given collection mint.
    ///
    /// # Arguments
    /// * `collection_mint` - The public key of the collection mint.
    /// * `delegated_collection_authority` - Optional public key of the delegated authority to revoke. If `None`, the default mint API authority is used.
    /// * `revoke_authority_keypair` - The keypair of the authority revoking the delegated authority.
    /// * `payer_keypair` - Optional keypair to pay for the transaction fees. If `None`, `revoke_authority_keypair` is used as the payer.
    ///
    /// # Returns
    /// A `Result` containing the transaction `Signature` if successful.
    pub async fn revoke_collection_authority(
        &self,
        collection_mint: Pubkey,
        delegated_collection_authority: Option<Pubkey>,
        revoke_authority_keypair: &Keypair,
        payer_keypair: Option<&Keypair>,
    ) -> Result<Signature> {
        let collection_authority = delegated_collection_authority.unwrap_or(self.config().mint_api_authority().into());
        let revoke_instruction =
            revoke_collection_authority_instruction(collection_mint, collection_authority, revoke_authority_keypair);
        let payer_keypair = payer_keypair.unwrap_or(revoke_authority_keypair);
        let recent_blockhash = self.async_connection()?.get_latest_blockhash().await?;
        self.async_connection()?
            .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
                &vec![revoke_instruction],
                Some(&payer_keypair.pubkey()),
                &vec![&payer_keypair, &revoke_authority_keypair],
                recent_blockhash,
            ))
            .await
            .map_err(|e| e.into())
    }
}
