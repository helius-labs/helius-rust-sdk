use solana_program::pubkey::Pubkey;
use mpl_token_metadata::ID;
use solana_sdk::signature::{Keypair, Signer};
use mpl_token_metadata::instructions::RevokeCollectionAuthority;
use solana_program::instruction::Instruction;

pub fn get_collection_authority_record(
    collection_mint: &Pubkey,
    collection_authority: &Pubkey
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            "metadata".as_bytes(),
            ID.as_ref(),
            &collection_mint.to_bytes(),
            "collection_authority".as_bytes(),
            &collection_authority.to_bytes()],
        &ID,
    )
        .0
}
pub fn get_collection_metadata_account(collection_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            "metadata".as_bytes(),
            ID.as_ref(),
            &collection_mint.to_bytes()
        ],
        &ID,
    )
        .0
}

pub fn revoke_collection_authority_instruction(
    collection_mint: Pubkey,
    collection_authority: Pubkey,
    revoke_authority_keypair: &Keypair
) -> Instruction {
    let collection_metadata = get_collection_metadata_account(&collection_mint);
    let collection_authority_record= get_collection_authority_record(&collection_mint, &collection_authority);
    let revoke_instruction = RevokeCollectionAuthority {
        collection_authority_record,
        delegate_authority: collection_authority,
        revoke_authority: revoke_authority_keypair.pubkey(),
        metadata: collection_metadata,
        mint: collection_mint,
    };
    revoke_instruction.instruction()
}

