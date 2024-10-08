use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use crate::error::Result;
use crate::types::{MintCompressedNftRequest, MintResponse, SmartTransactionConfig};
use crate::Helius;
use crate::utils::collection_authority::revoke_collection_authority_instruction;

impl Helius {
    /// The easiest way to mint a compressed NFT (cNFT)
    ///
    /// # Arguments
    /// * `request` - A struct containing the various desired metadata of the cNFT to be minted, such as its name, symbol, URI, and attributes
    ///
    /// # Returns
    /// A `Result` containing a `MintResponse` detailing the transaction signature, asset ID, and whether the cNFT was minted successfully
    pub async fn mint_compressed_nft(&self, request: MintCompressedNftRequest) -> Result<MintResponse> {
        self.rpc_client.post_rpc_request("mintCompressedNft", request).await
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
        let collection_authority = delegated_collection_authority
            .unwrap_or(self.config().mint_api_authority().into());
        let revoke_instruction = revoke_collection_authority_instruction(collection_mint, collection_authority, revoke_authority_keypair);
        let payer_keypair = payer_keypair.unwrap_or(revoke_authority_keypair);
        let transaction_config = SmartTransactionConfig::new(vec![revoke_instruction], vec![payer_keypair]);
        self.send_smart_transaction(transaction_config).await
    }
}
