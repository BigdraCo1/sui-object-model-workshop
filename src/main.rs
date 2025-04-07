use sui_sdk::SuiClientBuilder;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Sui testnet -- https://fullnode.testnet.sui.io:443
    let sui_testnet = SuiClientBuilder::default().build_testnet().await?;
    println!("Sui testnet version: {}", sui_testnet.api_version());

     // Sui devnet -- https://fullnode.devnet.sui.io:443
    let sui_devnet = SuiClientBuilder::default().build_devnet().await?;
    println!("Sui devnet version: {}", sui_devnet.api_version());

    // Sui mainnet -- https://fullnode.mainnet.sui.io:443
    let sui_mainnet = SuiClientBuilder::default().build_mainnet().await?;
    println!("Sui mainnet version: {}", sui_mainnet.api_version());

    // Example usage of utils modules
    let mut wallet = utils::wallet::retrieve_wallet()?;
    let active_address = wallet.active_address()?;
    println!("Wallet active address: {:?}", active_address);

    // Get balance using the SUI client instead of wallet.get_balance()
    let coins = utils::transaction::fetch_coin(&sui_testnet, &active_address).await?;
    if let Some(coin) = coins {
        println!("Wallet balance before faucet: {}", mist_to_sui(coin.balance));
    }
    
    utils::faucet::request_tokens_from_faucet(active_address, &sui_testnet).await?;
    
    // Get updated balance after faucet
    let updated_coins = utils::transaction::fetch_coin(&sui_testnet, &active_address).await?;
    if let Some(coin) = updated_coins {
        println!("Wallet balance before faucet: {}", mist_to_sui(coin.balance));
    }

    Ok(())
}

pub fn mist_to_sui(mist: u64) -> f64 {
    let sui = mist as f64 / 1_000_000_000_f64;
    sui
}
