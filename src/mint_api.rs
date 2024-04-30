use crate::error::Result;
use crate::types::{MintCompressedNftRequest, MintResponse};
use crate::Helius;

impl Helius {
    pub async fn mint_compressed_nft(&self, request: MintCompressedNftRequest) -> Result<MintResponse> {
        self.rpc_client.post_rpc_request("mintCompressedNft", request).await
    }
}
