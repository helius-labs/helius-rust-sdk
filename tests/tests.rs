mod utils {
    mod test_deserialize_str_to_number;
    mod test_is_valid_solana_address;
    mod test_make_keypairs;
}

mod types {
    mod test_deserialize_asset_list;
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
    mod test_get_rwa_asset;
    mod test_get_signatures_for_asset;
    mod test_get_token_accounts;
    mod test_search_assets;
}
mod webhook {
    mod test_create_webhook;
    mod test_edit_webhook;
    mod test_get_webhook_by_id;

    mod test_append_addresses_to_webhook;
    mod test_delete_webhook;
    mod test_get_all_webhooks;
    mod test_remove_addresses_from_webhook;
}
