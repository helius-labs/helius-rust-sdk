use helius::types::EnhancedTransaction;

#[test]
fn dese_enhanced_tx_with_transaction_error_insufficient_founds() {
    let raw_json = include_str!("./enhanced_tx__transaction_error_insufficient_founds.json");
    let enhanced_tx: EnhancedTransaction = serde_json::from_str(raw_json).unwrap();

    // There was different error that can be handled because of backward compatibility
    assert_eq!(
        enhanced_tx.transaction_error.unwrap().instruciton_error,
        serde_json::Value::Null
    );
}

#[test]
fn dese_enhanced_tx_with_instruction_error() {
    let raw_json = include_str!("./enhanced_tx__transaction_error_instruction_error.json");
    let enhanced_tx: EnhancedTransaction = serde_json::from_str(raw_json).unwrap();

    assert_ne!(
        enhanced_tx.transaction_error.unwrap().instruciton_error,
        serde_json::Value::Null
    );
}
