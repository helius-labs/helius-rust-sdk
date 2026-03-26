mod utils {
    mod test_deserialize_str_to_number;
    mod test_is_valid_solana_address;
    mod test_make_keypairs;
}

mod rpc {
    mod test_get_asset;
    mod test_get_asset_batch;
    mod test_get_asset_proof;
    mod test_get_asset_proof_batch;
    mod test_get_assets_by_authority;
    mod test_get_assets_by_creator;
    mod test_get_assets_by_group;
    mod test_get_assets_by_owner;
    mod test_get_nft_editions;
    mod test_get_priority_fee_estimate;
    mod test_get_signatures_for_asset;
    mod test_get_token_accounts;
    mod test_get_transactions_for_address;
    mod test_search_assets;
}
mod wallet {
    mod test_get_batch_wallet_identity;
    mod test_get_wallet_balances;
    mod test_get_wallet_funding_source;
    mod test_get_wallet_history;
    mod test_get_wallet_identity;
    mod test_get_wallet_transfers;
}
mod webhook {
    mod test_create_webhook;
    mod test_edit_webhook;
    mod test_get_webhook_by_id;

    mod test_append_addresses_to_webhook;
    mod test_delete_webhook;
    mod test_get_all_webhooks;
    mod test_remove_addresses_from_webhook;
    mod test_toggle_webhook;
}
mod websocket {
    mod test_get_url;
    mod test_websocket_lifecycle;
}
mod staking {
    mod helpers;
    mod test_create_stake_transaction;
    mod test_create_unstake_transaction;
    mod test_create_withdraw_transaction;
    mod test_get_unstake_instruction;
    mod test_get_withdraw_instruction;
}
mod zk_compression {
    mod helpers;
    mod test_get_compressed_account;
    mod test_get_compressed_account_proof;
    mod test_get_compressed_accounts_by_owner;
    mod test_get_compressed_balance;
    mod test_get_compressed_balance_by_owner;
    mod test_get_compressed_mint_token_holders;
    mod test_get_compressed_token_account_balance;
    mod test_get_compressed_token_accounts_by_delegate;
    mod test_get_compressed_token_accounts_by_owner;
    mod test_get_compressed_token_balances_by_owner;
    mod test_get_compressed_token_balances_by_owner_v2;
    mod test_get_compression_signatures_for_account;
    mod test_get_compression_signatures_for_address;
    mod test_get_compression_signatures_for_owner;
    mod test_get_compression_signatures_for_token_owner;
    mod test_get_indexer_health;
    mod test_get_indexer_slot;
    mod test_get_latest_compression_signatures;
    mod test_get_latest_non_voting_signatures;
    mod test_get_multiple_compressed_account_proofs;
    mod test_get_multiple_compressed_accounts;
    mod test_get_multiple_new_address_proofs;
    mod test_get_multiple_new_address_proofs_v2;
    mod test_get_transaction_with_compression_info;
    mod test_get_validity_proof;
}
