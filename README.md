# Substrate IBC Pallet (work in progress)
[![crates.io](https://img.shields.io/crates/v/pallet-ibc.svg)](https://crates.io/crates/pallet-ibc)
[![Released API docs](https://docs.rs/pallet-ibc/badge.svg)](https://docs.rs/pallet-ibc)

This project is [funded by Interchain Foundation](https://interchain-io.medium.com/ibc-on-substrate-with-cdot-a7025e521028).

## Purpose

This pallet implements the standard [IBC protocol](https://github.com/cosmos/ics).

The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to interact with other chains in a trustless way via IBC protocol.

This project is currently in an early stage and will eventually be submitted to upstream.

The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/51f0c9e8d8ebcbe6f7f023a8b80f65a8fab705e3/spec),  and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs), which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/51f0c9e8d8ebcbe6f7f023a8b80f65a8fab705e3/spec).

The chain specific logic of the modules in ICS spec implemented:
* ics-002-client-semantics
* ics-003-connection-semantics
* ics-004-channel-and-packet-semantics
* ics-005-port-allocation
* ics-010-grandpa-client
* ics-018-relayer-algorithms
* ics-023-vector-commitments
* ics-024-host-requirements
* ics-025-handler-interface
* ics-026-routing-module

Here is a [demo](~~https://github.com/cdot-network/ibc-demo~~) for showing how to utilize this pallet, which initializes a series of steps for cross-chain communication, from client creation to sending packet data.

## Design Overview
The ibc pallet is integrated with the [modules in ibc-rs](https://github.com/octopus-network/ibc-rs/tree/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules), which implements the [ibc spec](https://github.com/cosmos/ibc/tree/7046202b645c65b1a2b7f293312bca5d651a13a4/spec) and leave the chain specific logics, which are named `???Readers` and `???Keepers`, to the ibc pallet.

List of `Readers` and `Keepers` of on-chain storage:
* [ClientReader](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics02_client/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L14) & [ClientKeeper](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics02_client/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L29)
* [ConnectionReader](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics03_connection/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L17) & [ConnectionKeeper](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics03_connection/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L67)
* [ChannelReader](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics04_channel/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L20) & [ChannelKeeper](https://github.com/octopus-network/ibc-rs/blob/b98094a57620d0b3d9f8d2caced09abfc14ab00f/modules/src/ics04_channel/context.rs?_pjax=%23js-repo-pjax-container%2C%20div%5Bitemtype%3D%22http%3A%2F%2Fschema.org%2FSoftwareSourceCode%22%5D%20main%2C%20%5Bdata-pjax-container%5D#L82)

## Installation
Thie section describe the modification of your substrate chain needed to integrate pallet ibc.

### `Cargo.toml`
Specify some versions of ibc relevant crate
```toml
[patch.crates-io]
tendermint              = { git = "https://github.com/informalsystems/tendermint-rs", branch = "v0.23.x" }
tendermint-rpc          = { git = "https://github.com/informalsystems/tendermint-rs", branch = "v0.23.x" }
tendermint-proto        = { git = "https://github.com/informalsystems/tendermint-rs", branch = "v0.23.x" }
tendermint-light-client = { git = "https://github.com/informalsystems/tendermint-rs", branch = "v0.23.x" }
tendermint-light-client-verifier = { git = "https://github.com/informalsystems/tendermint-rs", branch = "v0.23.x" }
tendermint-testgen      = { git = "https://github.com/informalsystems/tendermint-rs", branch = "v0.23.x" }
```

#### Runtime's `Cargo.toml`
To add this pallet to your runtime, include the following to your runtime's `Cargo.toml` file:

```TOML
pallet-ibc = { git = "https://github.com/octopus-network/substrate-ibc", branch = "master", default-features = false}
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    "pallet-ibc/std",
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

You should include it in your `construct_runtime!` macro:

```rust
Ibc: pallet_ibc::{Pallet, Call, Storage, Event<T>},
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