# Sui Object Model Workshop with Rust SDK

When learning Sui Move, developers are encouraged to use best practices to utilize the Sui object model and ensure on-chain object composability. Developers learn to write composable move code in a timely manner, but struggle to verify their code by deploying and executing the functionality on chain. The key to mastering the Sui object model is to pair your Sui move development sessions with interacting with on-chain objects via PTBs (Programmable Transaction Blocks). This workshop will guide you through the process of writing Sui Move code, deploying it to the Sui blockchain, and interacting with on-chain objects via PTBs using the Rust SDK.

## Table of Contents
- [Environment Setup](#environment-setup)
- [Lessons](#lessons)
  - [Handling Returned Objects](#handling-returned-objects)
    - [Exercise](#exercise)
  - [Objects as Input](#objects-as-input)
    - [Exercise](#exercise-1)

## Environment Setup

Before we start, we need to set up our environment.

Ensure you have Rust installed. If not, install it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install the Sui CLI:

```bash
cargo install --locked --git https://github.com/MystenLabs/sui.git --branch main sui
```

Initialize a Sui wallet:

```bash
sui client new-address ed25519
```

Fund your address using the Sui Faucet: [https://faucet.sui.io/](https://faucet.sui.io/) OR [Discord faucet](https://discord.gg/cKx75xrRMq)

## Lessons

### Handling Returned Objects

One of the best practices when writing Sui Move packages is to avoid self-transfers. In other words, avoid transferring objects to the sender of the transaction, and instead return the object from the current function. This allows a caller or programmable transaction block to use the object however they see fit. 

For example, avoid this: 

```move
public struct NewObject has key, store {
  id: UID
}

public fun new(ctx: &mut TxContext) {
  let new_object = NewObject{
    id: object::new(ctx),
  };

  transfer::transfer(new_object, ctx.sender());
}
```

Instead, do this:

```move
public struct NewObject has key, store {
  id: UID
}

public fun new(ctx: &mut TxContext): NewObject {
  let new_object = NewObject{
    id: object::new(ctx),
  };

  new_object
}
```

This is easy enough to do, but in most cases (when the object doesn't have the [`drop` ability](https://move-book.com/reference/abilities.html?highlight=drop#drop)), if the returned object is not handled properly, the transaction will fail.

In this lesson, you learn how to handle returned objects properly using the Rust SDK.

#### Exercise

View the contents [`banana.move`](./lessons/returning_objects/banana_without_display/sources/banana_without_display.move). There is a deployed instance of this package on the Sui blockchain. The address of the package is [`0xadfb946c8c887446d284dae80ac8501c02ec9b9157babb96ca884773bfbb7771`](https://suiscan.xyz/testnet/object/0xadfb946c8c887446d284dae80ac8501c02ec9b9157babb96ca884773bfbb7771/txs). Navigate to [`src/exercises/return_objects.rs`](./src/exercises/return_objects.rs) and complete the exercise.

### Objects as Input

There are a lot of situations where one will want to interact with objects on Sui. Referencing and using objects in Sui Move is simple but nuanced. To reference an object in Sui Move, make the object a function parameter. For example, 

```move
public struct SimpleObject has key, store {
  id: UID, 
  value: u64 
}

public fun update_value(obj: &mut SimpleObject, new_value: u64) {
  obj.value = new_value;
}

public fun delete(obj: SimpleObject) {
  let SimpleObject {
    id, 
    value: _,
  } = obj;

  id.delete();
}
```

The `update_value` function receives the mutable reference of the `SimpleObject` and updates one of its attributes. Note that it receives only the mutable reference, therefore, the object doesn't need to be returned at the end of the function. 

The `delete` function receives the actual instance of the `SimpleObject` and deletes it by destructuring it. An object can only be destructured in the module that originally defined the object type. Since the object is destructured, it does not need to be returned at the end of the function. 

This usage is straightforward, but tends to leave developers wondering what this looks out in a wider context. In this lesson, you learn how to use objects as inputs in PTBs with the Rust SDK. 

#### Exercise

View the contents [`counter.move`](./lessons/input_objects/counter/sources/counter.move). There is a deployed instance of this package on the Sui blockchain. The address of the package is [`0xad3225e7d4827f81dc0686177067e1b458e8468ceabcff3456888ce3d806eb8c`](https://suiscan.xyz/testnet/object/0xad3225e7d4827f81dc0686177067e1b458e8468ceabcff3456888ce3d806eb8c/txs). Navigate to [`src/exercises/input_objects.rs`](./src/exercises/input_objects.rs) and complete the exercise.
