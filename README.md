# Substrate IBC Pallet (work in progress)
[![crates.io](https://img.shields.io/crates/v/pallet-ibc.svg)](https://crates.io/crates/pallet-ibc)
[![Released API docs](https://docs.rs/pallet-ibc/badge.svg)](https://docs.rs/pallet-ibc)

This project is [funded by Interchain Foundation](https://interchain-io.medium.com/ibc-on-substrate-with-cdot-a7025e521028).

## Purpose

This pallet implements the standard [IBC protocol](https://github.com/cosmos/ics).

The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to interact with other chains in a trustless way via IBC protocol.

This project is currently in an early stage and will eventually be submitted to upstream.

The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs), which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).

The chain specific logic of the modules in ICS spec implemented:
* ics-002-client-semantics
* ics-003-connection-semantics
* ics-004-channel-and-packet-semantics
* ics-005-port-allocation
* ics-010-grandpa-client
* ics-018-relayer-algorithms
* ics-025-handler-interface
* ics-026-routing-module

Here is a [demo](~~https://github.com/cdot-network/ibc-demo~~) for showing how to utilize this pallet, which initializes a series of steps for cross-chain communication, from client creation to sending packet data.

## Design Overview
The ibc pallet is integrated with the [modules in ibc-rs](https://github.com/octopus-network/ibc-rs/tree/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules), which implements the [ibc spec](https://github.com/cosmos/ibc/tree/7046202b645c65b1a2b7f293312bca5d651a13a4/spec) and leave the chain specific logics, which are named `???Readers` and `???Keepers`, to the ibc pallet.

List of `???Readers` and `???Keepers`:
* [ClientReader](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics02_client/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L14) & [ClientKeeper](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics02_client/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L29)
* [ConnectionReader](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics03_connection/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L17) & [ConnectionKeeper](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics03_connection/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L67)
* [ChannelReader](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics04_channel/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L20) & [ChannelKeeper](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics04_channel/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L82)



## Installation
Thie section describe the modification of your substrate chain needed to integrate pallet ibc.

### `Cargo.toml`
Specify some versions of ibc relevant crate
```toml
[patch.crates-io]
# for tendermint
tendermint              = { git = "https://github.com/informalsystems/tendermint-rs", rev = "4f6ef3d6" }
tendermint-rpc          = { git = "https://github.com/informalsystems/tendermint-rs", rev = "4f6ef3d6" }
tendermint-proto        = { git = "https://github.com/informalsystems/tendermint-rs", rev = "4f6ef3d6" }
tendermint-light-client = { git = "https://github.com/informalsystems/tendermint-rs", rev = "4f6ef3d6" }
tendermint-testgen      = { git = "https://github.com/informalsystems/tendermint-rs", rev = "4f6ef3d6" }
#ics23                   = { git = "https://github.com/informalsystems/ics23.git", branch = "master" }
ics23                   = { git = "https://github.com/informalsystems/ics23.git", rev = "4461b673" }
safe-regex          = { git = "https://github.com/informalsystems/safe-regex.git", rev = "842d31f5" }
safe-regex-macro    = { git = "https://github.com/informalsystems/safe-regex.git", rev = "842d31f5" }
safe-regex-compiler = { git = "https://github.com/informalsystems/safe-regex.git", rev = "842d31f5" }
safe-quote          = { git = "https://github.com/informalsystems/safe-regex.git", rev = "842d31f5" }
safe-proc-macro2    = { git = "https://github.com/informalsystems/safe-regex.git", rev = "842d31f5" }
tonic = { package = "informalsystems-tonic", git = "https://github.com/informalsystems/tonic.git", rev = "99edfe23" }
```

#### `bin`'s `Cargo.toml`
And include the following to your `bin`'s `Cargo.toml` file:
```TOML
pallet-ibc-rpc = { git = "https://github.com/octopus-network/substrate-ibc", branch = "dv-ibc-dev-0.9.12-tag", default-features = false}
pallet-ibc-runtime-api = { git = "https://github.com/octopus-network/substrate-ibc", branch = "dv-ibc-dev-0.9.12-tag", default-features = false}
```

#### Runtime's `Cargo.toml`
To add this pallet to your runtime, include the following to your runtime's `Cargo.toml` file:

```TOML
pallet-ibc-rpc = { git = "https://github.com/octopus-network/substrate-ibc", branch = "dv-ibc-dev-0.9.12-tag", default-features = false}
pallet-ibc-runtime-api = { git = "https://github.com/octopus-network/substrate-ibc", branch = "dv-ibc-dev-0.9.12-tag", default-features = false}
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    "pallet-ibc/std",
    "pallet-ibc-runtime-api/std",
	]
```

### Runtime `lib.rs`
A custom structure that implements the pallet_ibc::ModuleCallbacks must be defined to dispatch messages to receiving module.
```rust
pub struct ModuleCallbacksImpl;

impl pallet_ibc::ModuleCallbacks for ModuleCallbacksImpl {}

impl pallet_ibc::Config for Runtime {
    type Event = Event;
    type ModuleCallbacks = ModuleCallbacksImpl;
    type TimeProvider = pallet_timestamp::Pallet<Runtime>;
}
```

You should implement it's trait like so:

```rust
    // Here we implement our custom runtime API.
impl  pallet_ibc_runtime_api::IbcApi<Block> for Runtime {
    // --snip--
}
```

and include it in your `construct_runtime!` macro:

```rust
Ibc: pallet_ibc::{Pallet, Call, Storage, Event<T>} = 110,
```

#### `bin`'s `service.rs`
Add below for the type interface of `RuntimeApi::RuntimeApi` in `async fn start_node_impl`, which starts a node with the given `Configuration`.

```rust
+ pallet_ibc_runtime_api::IbcApi<Block>,
```

#### `bin`'s `rpc.rs`
When instantiating all RPC extensions, add pallet ibc's
```rust
C::Api: pallet_ibc_runtime_api::IbcApi<Block>,
```

```rust
io.extend_with(pallet_ibc_rpc::IbcApi::to_delegate(
    pallet_ibc_rpc::IbcStorage::new(client.clone()),
));
```

### Genesis Configuration

This pallet does not have any genesis configuration.

## How to Interact with the Pallet
The Hermes (IBC Relayer CLI) offers commands to send reqeusts to pallet ibc to trigger the standard ibc communications defined in [ibc spce](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f/spec). 
[Hermes Command List](https://hermes.informal.systems/commands/raw/index.html).

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

or by visiting this site: https://docs.rs/pallet-ibc