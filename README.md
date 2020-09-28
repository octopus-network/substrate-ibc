# Substrate IBC Pallet (work in progress)

## Purpose

This pallet implements the standard [IBC protocol](https://github.com/cosmos/ics).

The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to interact with other chains in a trustless way via IBC protocol, no matter what consensus the counterparty chains use.

This project is currently in an early stage and will eventually be submitted to upstream.

Some components in [ICS spec](https://github.com/cosmos/ics/tree/master/spec) are implemented to support a working demo (https://github.com/cdot-network/ibc-demo), but not fully implemented as the spec:  
* ics-002-client-semantics
* ics-003-connection-semantics
* ics-004-channel-and-packet-semantics
* ics-005-port-allocation
* ics-010-grandpa-client
* ics-018-relayer-algorithms
* ics-025-handler-interface
* ics-026-routing-module

Here is a [demo](https://github.com/cdot-network/ibc-demo) for showing how to utilize this pallet, which initializes a series of steps for cross-chain communication, from client creation to sending packet data.

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

## How to Interact with the Pallet

In the ibc-demo repo, substrate-subxt invokes the pallet's callable functions by the macro ```substrate_subxt_proc_macro::Call```.

Let's take the function ```test_create_client``` as an example. [Client](https://docs.rs/substrate-subxt/0.12.0/substrate_subxt/struct.Client.html) extends the function 
```rust
// in https://github.com/cdot-network/ibc-demo/blob/master/pallets/template/src/lib.rs
pub fn test_create_client(
    origin,
    identifier: H256,
    height: u32,
    set_id: SetId,
    authorities: AuthorityList,
    root: H256
) -> dispatch::DispatchResult {
...
}
``` 
by 
```rust
// https://github.com/cdot-network/ibc-demo/blob/master/calls/src/template.rs
#[derive(Encode, Call)]
pub struct TestCreateClientCall<T: TemplateModule> {
    pub _runtime: PhantomData<T>,
    pub identifier: H256,
    pub height: u32,
    pub set_id: SetId,
    pub authority_list: AuthorityList,
    pub root: H256,
}
```

Therefore, 
```rust
//  https://github.com/cdot-network/ibc-demo/blob/master/cli/src/main.rs
client
.test_create_client(...)
```
can invoke the ```test_create_client``` function. 

Please refer to document [substrate_subxt_proc_macro::Call](https://docs.rs/substrate-subxt-proc-macro/0.12.0/substrate_subxt_proc_macro/derive.Call.html) for details.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```

or by visiting this site: <Add Your Link>

