use helius::error::Result;
use helius::types::Cluster;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let project_id: &str = "your_project_id";

    let helius: Helius = Helius::new(api_key, Cluster::MainnetBeta).unwrap();
    let response = helius.get_project_usage(project_id).await;

    match response {
        Ok(usage) => {
            println!("Project Usage:");
            println!("  Credits Used: {}", usage.credits_used);
            println!("  Credits Remaining: {}", usage.credits_remaining);
            println!("  Prepaid Credits Used: {}", usage.prepaid_credits_used);
            println!("  Prepaid Credits Remaining: {}", usage.prepaid_credits_remaining);
            println!("  Plan: {}", usage.subscription_details.plan);
            println!(
                "  Billing Cycle: {} -> {}",
                usage.subscription_details.billing_cycle.start, usage.subscription_details.billing_cycle.end
            );
            println!("  RPC Usage: {}", usage.usage.rpc);
            println!("  DAS Usage: {}", usage.usage.das);
            println!("  Webhook Usage: {}", usage.usage.webhook);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
