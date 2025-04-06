// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[path = "../utils/mod.rs"]
mod utils;
use anyhow::anyhow;
use shared_crypto::intent::Intent;
use sui_config::{SUI_KEYSTORE_FILENAME, sui_config_dir};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{
    rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions},
    types::{
        Identifier,
        base_types::ObjectID,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        quorum_driver_types::ExecuteTransactionRequestType,
        transaction::{CallArg, Command, ObjectArg, Transaction, TransactionData},
    },
};
use sui_types::object::Owner;
use utils::wallet::setup_for_write;

// This example shows how to use programmable transactions to chain multiple
// commands into one transaction, and specifically how to call a function from a move package
// These are the following steps:
// 1) finds a coin from the active address that has Sui,
// 2) creates a PTB ,
// 3) adds a move call to the PTB,
// 4) signs the transaction,
// 5) executes it.
// For some of these actions it prints some output.
// Finally, at the end of the program it prints the number of coins for the
// Sui address that received the coin.
// If you run this program several times, you should see the number of coins
// for the recipient address increases.

const PKG_ID: &str = "0xad3225e7d4827f81dc0686177067e1b458e8468ceabcff3456888ce3d806eb8c";

const COUNTER_OBJ_ID: &str = "0x1feb03541d20064d1876c26cfa44514f2e029c8201a2fe12a60589842b9d391d";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 1) get the Sui client, the sender and recipient that we will use
    // for the transaction, and find the coin we use as gas
    let (sui, sender, _recipient) = setup_for_write().await?;

    // we need to find the coin we will use as gas
    let coins = sui
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await?;
    let coin = coins.data.into_iter().next().unwrap();

    // 2) create a programmable transaction builder to add commands and create a PTB
    let mut ptb = ProgrammableTransactionBuilder::new();

    let counter_object = ObjectID::from_hex_literal(COUNTER_OBJ_ID).map_err(|e| anyhow!(e))?;

    // Query the object to get its correct initial shared version
    let object_response = sui
        .read_api()
        .get_object_with_options(counter_object, SuiObjectDataOptions::full_content())
        .await?;

    // Extract the initial shared version from the object data
    let initial_shared_version = if let Some(data) = object_response.data {
        if let Some(owner) = &data.owner {
            match owner {
                Owner::Shared {
                    initial_shared_version,
                } => *initial_shared_version,
                _ => return Err(anyhow!("Object is not shared")),
            }
        } else {
            return Err(anyhow!("Object owner information not found"));
        }
    } else {
        return Err(anyhow!("Object data not found"));
    };

    println!("Using initial shared version: {}", initial_shared_version);

    let input_value = ObjectArg::SharedObject {
        id: counter_object,
        initial_shared_version,
        mutable: true,
    };
    let input_argument = CallArg::Object(input_value);
    let counter_object = ptb.input(input_argument)?;

    // 3) add a move call to the PTB
    // Replace the pkg_id with the package id you want to call
    let package = ObjectID::from_hex_literal(PKG_ID).map_err(|e| anyhow!(e))?;
    let module = Identifier::new("counter").map_err(|e| anyhow!(e))?;
    let function = Identifier::new("increment").map_err(|e| anyhow!(e))?;
    ptb.command(Command::move_call(
        package,
        module,
        function,
        vec![],
        vec![counter_object],
    ));

    // build the transaction block by calling finish on the ptb
    let builder = ptb.finish();

    let gas_budget = 10_000_000;
    let gas_price = sui.read_api().get_reference_gas_price().await?;
    // create the transaction data that will be sent to the network
    let tx_data = TransactionData::new_programmable(
        sender,
        vec![coin.object_ref()],
        builder,
        gas_budget,
        gas_price,
    );

    // 4) sign transaction
    let keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(&sender, &tx_data, Intent::sui_transaction())?;

    // 5) execute the transaction
    print!("Executing the transaction...");
    let transaction_response = sui
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![signature]),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;
    println!("{}", transaction_response);
    Ok(())
}
