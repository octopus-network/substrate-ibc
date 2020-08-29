# Substrate IBC Pallet

## Purpose

This pallet implements the standard [IBC protocol](https://github.com/cosmos/ics).

The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to interact with other chains in a trustless way via IBC protocol, no matter what consensus the counterparty chains use.

This project is currently in an early stage and will eventually be submitted to upstream.

Here is a [demo](https://github.com/cdot-network/ibc-demo) for showing how to utilize this pallet.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies.ibc]
default_features = false
package = 'pallet-ibc'
git = 'https://github.com/cdot-network/substrate-ibc.git'
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'ibc/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
/// Used for test_module
impl ibc::Trait for Runtime {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
Ibc: ibc::{Module, Call, Storage, Event<T>},
```

### Genesis Configuration

This pallet does not have any genesis configuration.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

or by visiting this site: <Add Your Link>
