use sui_sdk::{
    SuiClient,
    types::{
        base_types::SuiAddress,
        digests::TransactionDigest,
        transaction::{Transaction, TransactionData, Argument, Command},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        quorum_driver_types::ExecuteTransactionRequestType,
    },
};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_json_rpc_types::{SuiTransactionBlockResponseOptions, Coin};
use shared_crypto::intent::Intent;
use anyhow::Result;
use futures::{future, stream::StreamExt};
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use super::faucet::request_tokens_from_faucet;

/// Return the coin owned by the address that has at least 5_000_000 MIST, otherwise returns None
pub async fn fetch_coin(
    sui: &SuiClient,
    sender: &SuiAddress,
) -> Result<Option<Coin>, anyhow::Error> {
    let coin_type = "0x2::sui::SUI".to_string();
    let coins_stream = sui
        .coin_read_api()
        .get_coins_stream(*sender, Some(coin_type));

    let mut coins = coins_stream
        .skip_while(|c| future::ready(c.balance < 5_000_000))
        .boxed();
    let coin = coins.next().await;
    Ok(coin)
}

pub async fn split_coin_digest(
    sui: &SuiClient,
    sender: &SuiAddress,
) -> Result<TransactionDigest, anyhow::Error> {
    let coin = match fetch_coin(sui, sender).await? {
        None => {
            request_tokens_from_faucet(*sender, sui).await?;
            fetch_coin(sui, sender)
                .await?
                .expect("Supposed to get a coin with SUI, but didn't. Aborting")
        }
        Some(c) => c,
    };

    println!(
        "Address: {sender}. The selected coin for split is {} and has a balance of {}\n",
        coin.coin_object_id, coin.balance
    );

    // set the maximum gas budget
    let max_gas_budget = 5_000_000;

    // get the reference gas price from the network
    let gas_price = sui.read_api().get_reference_gas_price().await?;

    // now we programmatically build the transaction through several commands
    let mut ptb = ProgrammableTransactionBuilder::new();
    // first, we want to split the coin, and we specify how much SUI (in MIST) we want
    // for the new coin
    let split_coin_amount = ptb.pure(1000u64)?; // note that we need to specify the u64 type here
    ptb.command(Command::SplitCoins(
        Argument::GasCoin,
        vec![split_coin_amount],
    ));
    // now we want to merge the coins (so that we don't have many coins with very small values)
    // observe here that we pass Argument::Result(0), which instructs the PTB to get
    // the result from the previous command
    ptb.command(Command::MergeCoins(
        Argument::GasCoin,
        vec![Argument::Result(0)],
    ));

    // we finished constructing our PTB and we need to call finish
    let builder = ptb.finish();

    // using the PTB that we just constructed, create the transaction data
    // that we will submit to the network
    let tx_data = TransactionData::new_programmable(
        *sender,
        vec![coin.object_ref()],
        builder,
        max_gas_budget,
        gas_price,
    );

    // sign & execute the transaction
    let keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(sender, &tx_data, Intent::sui_transaction())?;

    let transaction_response = sui
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![signature]),
            SuiTransactionBlockResponseOptions::new(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;
    Ok(transaction_response.digest)
}

