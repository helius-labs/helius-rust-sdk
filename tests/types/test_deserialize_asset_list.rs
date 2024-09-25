use helius::types::AssetList;

#[test]
fn des_asset_list_with_native_balance() {
    let raw_json = include_str!("./asset_list_with_native_balance.json");
    let deserialized: AssetList = serde_json::from_str(raw_json).unwrap();

    let native_balance = deserialized.native_balance.unwrap();
    assert_eq!(native_balance.lamports, 47770720);
    assert_eq!(native_balance.price_per_sol, Some(150.3314666748047_f64));
    assert_eq!(native_balance.total_price, Some(7.1814424017114264_f64));
}
