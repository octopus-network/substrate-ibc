# Substrate IBC Pallet (work in progress)

[![crates.io](https://img.shields.io/crates/v/pallet-ibc.svg)](https://crates.io/crates/pallet-ibc)
[![Released API docs](https://docs.rs/pallet-ibc/badge.svg)](https://docs.rs/pallet-ibc)

This project is [funded by Interchain Foundation](https://interchain-io.medium.com/ibc-on-substrate-with-cdot-a7025e521028).

## Purpose

This pallet implements the standard [IBC protocol](https://github.com/cosmos/ics).

The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to interact with other chains in a trustless way via IBC protocol.

This project is currently in an early stage and will eventually be submitted to upstream.

The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/51f0c9e8d8ebcbe6f7f023a8b80f65a8fab705e3/spec), and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs), which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/51f0c9e8d8ebcbe6f7f023a8b80f65a8fab705e3/spec).