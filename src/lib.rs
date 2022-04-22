#![cfg_attr(not(feature = "std"), no_std)]
// todo need in future to remove
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(clippy::type_complexity)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! # IBC Module
//!
//! This module implements the standard [IBC protocol](https://github.com/cosmos/ics).
//!
//! ## Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to
//! interact with other chains in a trustees way via IBC protocol
//!
//! This project is currently in an early stage and will eventually be submitted to upstream.
//!
//! The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs), which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).
//!
//! The chain specific logic of the modules in ICS spec implemented::
//! * ics-002-client-semantics
//! * ics-003-connection-semantics
//! * ics-004-channel-and-packet-semantics
//! * ics-005-port-allocation
//! * ics-010-grandpa-client
//! * ics-018-relayer-algorithms
//! * ics-025-handler-interface
//! * ics-026-routing-module
//!
//! ### Terminology
//!
//! Please refer to [IBC Terminology](https://github.com/cosmos/ibc/blob/a983dd86815175969099d041906f6a14643e51ef/ibc/1_IBC_TERMINOLOGY.md).
//!
//! ### Goals
//!
//! This IBC module handles authentication, transport, and ordering of structured data packets
//! relayed between modules on separate machines.
//!
//! ## Interface
//!
//! ###  Public Functions
//!
//! * `deliver` - `ibc::ics26_routing::handler::deliver` Receives datagram transmitted from
//!   relayers/users, and pass to ICS26 router to look for the correct handler.
//!
//! ## Usage
//! Please refer to section "How to Interact with the Pallet" in the repository's README.md

extern crate alloc;

pub use pallet::*;

use crate::alloc::string::ToString;
use alloc::{format, string::String};
use core::str::FromStr;

use beefy_light_client::commitment;
use codec::{Codec, Decode, Encode};
use core::marker::PhantomData;
use frame_system::ensure_signed;
use ibc::{
	applications::ics20_fungible_token_transfer::msgs::transfer::MsgTransfer,
	core::{ics02_client::height, ics24_host::identifier},
	timestamp,
	tx_msg::Msg,
};

use ibc::{
	clients::ics10_grandpa::{
		client_state::ClientState,
		help,
		help::{BlockHeader, Commitment},
	},
	core::{
		ics02_client::client_state::AnyClientState, ics24_host::identifier::ChainId as ICS24ChainId,
	},
};

use ibc::core::ics26_routing::msgs::Ics26Envelope;

use sp_runtime::DispatchError;

pub use routing::ModuleCallbacks;
use scale_info::{prelude::vec, TypeInfo};
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug, TypeId};
use sp_std::prelude::*;
use tendermint_proto::Protobuf;

mod channel;
mod client;
mod connection;
pub mod event;
mod ics20_handler;
mod ics20_ibc_module_impl;
mod port;
mod routing;
pub mod transfer;

// pub use temp router
use crate::routing::MockRouter;

use frame_support::{
	sp_runtime::traits::{AtLeast32BitUnsigned, CheckedConversion},
	sp_std::fmt::Debug,
	traits::{tokens::fungibles, Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};

/// A struct corresponds to `Any` in crate "prost-types", used in ibc-rs.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Any {
	pub type_url: Vec<u8>,
	pub value: Vec<u8>,
}

impl From<ibc_proto::google::protobuf::Any> for Any {
	fn from(any: ibc_proto::google::protobuf::Any) -> Self {
		Self { type_url: any.type_url.as_bytes().to_vec(), value: any.value }
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::event::primitive::Sequence;
	use event::primitive::{
		ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height,
		Packet, PortId, Timestamp,
	};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::{
		applications::ics20_fungible_token_transfer::context::Ics20Context,
		core::{
			ics04_channel::{
				channel::{Counterparty, Order},
				events::WriteAcknowledgement,
				Version,
			},
			ics05_port::capabilities::Capability,
			ics24_host::identifier::{ChannelId as IbcChannelId, PortId as IbcPortId},
			ics26_routing::error::Error as Ics26Error,
		},
		events::IbcEvent,
		signer::Signer,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ModuleCallbacks: routing::ModuleCallbacks;
		type TimeProvider: UnixTime;

		type Currency: Currency<Self::AccountId>;

		type AssetId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Codec
			+ Copy
			+ Debug
			+ Default
			+ MaybeSerializeDeserialize;

		type AssetBalance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ From<u128>
			+ Into<u128>
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug;

		type Assets: fungibles::Mutate<
			<Self as frame_system::Config>::AccountId,
			AssetId = Self::AssetId,
			Balance = Self::AssetBalance,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// client_id vector
	pub type ClientStatesKeys<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientProcessedTimes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientProcessedHeights<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// client_id => Vector<(Height, ConsensusState)>
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// connection_id => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// connection_id vector
	pub type ConnectionsKeys<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// (port_identifier, channel_identifier) => ChannelEnd
	pub type Channels<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// vector of (port_id, channel_id)
	pub type ChannelsKeys<T: Config> = StorageValue<_, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// connection_id => Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// vector of (port_identifier, channel_identifier, sequence)
	pub type AcknowledgementsKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// client_id => client_type
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn client_counter)]
	/// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_counter)]
	/// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// client_id => connection_id
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => hash of (timestamp, heigh, packet)
	pub type PacketCommitment<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// vector of (port_id, channel_id, sequence)
	pub type PacketCommitmentKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// (height, port_id, channel_id, sequence) => sendpacket event
	pub type SendPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => writeack event
	pub type WriteAckPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// store latest height
	pub type LatestHeight<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// sha256(tracePath + "/" + baseDenom) => DenomTrace
	pub type Denomination<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// port, channel -> escrow address
	pub type EscrowAddresses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PortId,
		Blake2_128Concat,
		ChannelId,
		T::AccountId,
		ValueQuery,
	>;

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewBlock(Height),

		CreateClient(Height, ClientId, ClientType, Height),
		UpdateClient(Height, ClientId, ClientType, Height),
		UpdateClientState(Height, EventClientState),
		UpgradeClient(Height, ClientId, ClientType, Height),
		ClientMisbehaviour(Height, ClientId, ClientType, Height),
		OpenInitConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		OpenTryConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		OpenAckConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		OpenConfirmConnection(
			Height,
			Option<ConnectionId>,
			ClientId,
			Option<ConnectionId>,
			ClientId,
		),
		OpenInitChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		OpenTryChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		OpenAckChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		OpenConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		CloseInitChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		CloseConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		SendPacket(Height, Packet),
		ReceivePacket(Height, Packet),
		WriteAcknowledgement(Height, Packet, Vec<u8>),
		AcknowledgePacket(Height, Packet),
		TimeoutPacket(Height, Packet),
		TimeoutOnClosePacket(Height, Packet),
		Empty(Vec<u8>),
		ChainError(Vec<u8>),
	}

	/// Convert events of ibc-rs to the corresponding events in substrate-ibc
	impl<T: Config> From<ibc::events::IbcEvent> for Event<T> {
		fn from(value: ibc::events::IbcEvent) -> Self {
			match value {
				ibc::events::IbcEvent::NewBlock(value) => Event::NewBlock(value.height.into()),
				ibc::events::IbcEvent::CreateClient(value) => {
					let height = value.0.height;
					let client_id = value.0.client_id;
					let client_type = value.0.client_type;
					let consensus_height = value.0.consensus_height;
					Event::CreateClient(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				ibc::events::IbcEvent::UpdateClient(value) => {
					let height = value.common.height;
					let client_id = value.common.client_id;
					let client_type = value.common.client_type;
					let consensus_height = value.common.consensus_height;
					Event::UpdateClient(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				// Upgrade client events are not currently being used
				// UpgradeClient(
				// 	height: Height,
				// 	client_id: ClientId,
				// 	client_type: ClientType,
				// 	consensus_height: Height,
				// )
				ibc::events::IbcEvent::UpgradeClient(value) => {
					let height = value.0.height;
					let client_id = value.0.client_id;
					let client_type = value.0.client_type;
					let consensus_height = value.0.consensus_height;
					Event::UpgradeClient(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				ibc::events::IbcEvent::ClientMisbehaviour(value) => {
					let height = value.0.height;
					let client_id = value.0.client_id;
					let client_type = value.0.client_type;
					let consensus_height = value.0.consensus_height;
					Event::ClientMisbehaviour(
						height.into(),
						client_id.into(),
						client_type.into(),
						consensus_height.into(),
					)
				},
				ibc::events::IbcEvent::OpenInitConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenInitConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenTryConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenTryConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenAckConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenAckConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenConfirmConnection(value) => {
					let height = value.attributes().height;
					let connection_id: Option<ConnectionId> =
						value.attributes().connection_id.clone().map(|val| val.into());
					let client_id = value.attributes().client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						value.attributes().counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = value.attributes().counterparty_client_id.clone();
					Event::OpenConfirmConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				ibc::events::IbcEvent::OpenInitChannel(value) => {
					let height = value.height.clone();
					let port_id = value.port_id.clone();
					let channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					let connection_id = value.connection_id.clone();
					let counterparty_port_id = value.counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					Event::OpenInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::OpenTryChannel(value) => {
					let height = value.height.clone();
					let port_id = value.port_id.clone();
					let channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					let connection_id = value.connection_id.clone();
					let counterparty_port_id = value.counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					Event::OpenTryChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::OpenAckChannel(value) => {
					let height = value.height.clone();
					let port_id = value.port_id.clone();
					let channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					let connection_id = value.connection_id.clone();
					let counterparty_port_id = value.counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					Event::OpenAckChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::OpenConfirmChannel(value) => {
					let height = value.height.clone();
					let port_id = value.port_id.clone();
					let channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					let connection_id = value.connection_id.clone();
					let counterparty_port_id = value.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.channel_id.map(|val| val.into());
					Event::OpenConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::CloseInitChannel(value) => {
					let height = value.height.clone();
					let port_id = value.port_id.clone();
					let channel_id: Option<ChannelId> = Some(value.channel_id.clone().into());
					let connection_id = value.connection_id.clone();
					let counterparty_port_id = value.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.counterparty_channel_id.map(|val| val.into());
					Event::CloseInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::CloseConfirmChannel(value) => {
					let height = value.height.clone();
					let port_id = value.port_id.clone();
					let channel_id: Option<ChannelId> =
						value.channel_id.clone().map(|val| val.into());
					let connection_id = value.connection_id.clone();
					let counterparty_port_id = value.counterparty_port_id.clone();
					let counterparty_channel_id: Option<ChannelId> =
						value.channel_id.map(|val| val.into());
					Event::CloseConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				ibc::events::IbcEvent::SendPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::SendPacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::ReceivePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::ReceivePacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::WriteAcknowledgement(value) => {
					let height = value.height;
					let packet = value.packet;
					let ack = value.ack;
					Event::WriteAcknowledgement(height.into(), packet.into(), ack)
				},
				ibc::events::IbcEvent::AcknowledgePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::AcknowledgePacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::TimeoutPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::TimeoutPacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::TimeoutOnClosePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::TimeoutOnClosePacket(height.into(), packet.into())
				},
				ibc::events::IbcEvent::Empty(value) => Event::Empty(value.as_bytes().to_vec()),
				ibc::events::IbcEvent::ChainError(value) =>
					Event::ChainError(value.as_bytes().to_vec()),
				_ => unimplemented!(),
			}
		}
	}

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// update the beefy light client failure!
		UpdateBeefyLightClientFailure,

		/// receive mmr root block number less than client_state.latest_commitment.block_number
		ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber,

		/// client id not found
		ClientIdNotFound,

		/// Encode error
		InvalidEncode,

		/// Decode Error
		InvalidDecode,

		/// FromUtf8Error
		InvalidFromUtf8,

		/// ics26router error
		Ics26Error,

		/// invalid signer
		InvalidSigner,

		/// empty channel id
		EmptyChannelId,

		/// ics20 error
		Ics20Error,

		/// parse ibc packet error
		InvalidPacket,

		/// invalid signed_commitment
		InvalidSignedCommitment,

		/// invalid identifier
		InvalidIdentifier,

		/// invalid timestamp
		InvalidTimestamp,

		/// empty latest_commitment
		EmptyLatestCommitment,
	}

	// // mock client state
	// fn mock_client_state() -> ClientState {
	// 	// mock light client
	// 	let public_keys = vec![String::from(
	// 		"0x020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1",
	// 	) /* Alice */];
	// 	let lc = beefy_light_client::new(public_keys);
	// 	log::info!("mock beefy light client: {:?}", lc);

	// 	//mock client state
	// 	let epoch_number = 10;
	// 	// let chain_id = ICS24ChainId::new(String::from("chainA"), epoch_number);
	// 	let chain_id = ICS24ChainId::new(String::from("chainA"), epoch_number);
	// 	let client_state = ClientState {
	// 		chain_id,
	// 		block_number: u32::default(),
	// 		frozen_height: None,
	// 		block_header: BlockHeader::default(),
	// 		latest_commitment: Commitment::default(),
	// 		validator_set: lc.validator_set.into(),
	// 	};
	// 	log::info!("mock client_state : {:?}", client_state);

	// 	client_state
	// }

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for all of the IBC request(except MMR root update).
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		///
		/// Example of invoking this function via subxt
		///
		/// ```ignore
		///     let api = client.to_runtime_api::<ibc_node::RuntimeApi<ibc_node::DefaultConfig>>();
		///
		///     let result = api
		///         .tx()
		///         .ibc()
		///         .deliver(msg, 0)
		///         .sign_and_submit(&signer)
		///         .await?;
		/// ```
		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<Any>,
			_tmp: u8,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;
			let mut ctx = routing::Context::<T>::new();

			let messages: Vec<ibc_proto::google::protobuf::Any> = messages
				.into_iter()
				.map(|message| ibc_proto::google::protobuf::Any {
					type_url: String::from_utf8(message.type_url.clone()).unwrap(),
					value: message.value.clone(),
				})
				.collect();

			let mut results: Vec<IbcEvent> = vec![];
			for (index, message) in messages.clone().into_iter().enumerate() {
				let (mut result, _) =
					ibc::core::ics26_routing::handler::deliver(&mut ctx, message.clone())
						.map_err(|_| Error::<T>::Ics26Error)?;

				log::info!("result: {:?}", result);

				results.append(&mut result);
			}

			let ret = Self::handle_result(&ctx, messages, results)?;

			Ok(().into())
		}

		/// Update the MMR root stored in client_state
		/// Example of invoking this function via subxt
		///
		/// ```ignore
		///     let api = client.to_runtime_api::<ibc_node::RuntimeApi<ibc_node::DefaultConfig>>();
		///
		///     let result = api
		///         .tx()
		///         .ibc()
		///         .update_client_state(encode_client_id, encode_mmr_root)
		///         .sign_and_submit(&signer)
		///         .await?;
		/// ```
		#[pallet::weight(0)]
		pub fn update_client_state(
			origin: OriginFor<T>,
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			log::trace!("update_client_state: update_client_state request.");
			let _who = ensure_signed(origin)?;

			// check if the client id exist?
			let client_id_str =
				String::from_utf8(client_id.clone()).map_err(|_| Error::<T>::InvalidFromUtf8)?;
			log::trace!("update_client_state:  client id is {:?}", client_id_str);

			let decode_received_mmr_root =
				help::MmrRoot::decode(&mut &mmr_root[..]).map_err(|_| Error::<T>::InvalidDecode)?;
			log::trace!("update_client_state:  decode mmr root is {:?}", decode_received_mmr_root);

			let mut client_state = ClientState::default();

			if !<ClientStates<T>>::contains_key(client_id.clone()) {
				log::error!("in update_client_state: {:?} client_state not found !", client_id_str);

				return Err(Error::<T>::ClientIdNotFound.into())
			} else {
				// get client state from chain storage
				let data = <ClientStates<T>>::get(client_id.clone());
				let any_client_state =
					AnyClientState::decode_vec(&*data).map_err(|_| Error::<T>::InvalidDecode)?;
				client_state = match any_client_state {
					AnyClientState::Grandpa(value) => value,
					_ => unimplemented!(),
				};

				log::trace!(
					"in update_client_state: get client_state from chain storage: {:?}",
					client_state
				);
			}

			let signed_commitment =
				commitment::SignedCommitment::try_from(decode_received_mmr_root.signed_commitment)
					.map_err(|_| Error::<T>::InvalidSignedCommitment)?;
			let rev_block_number = signed_commitment.commitment.block_number;
			if rev_block_number <= client_state.latest_commitment.block_number {
				log::trace!("receive mmr root block number({}) less than client_state.latest_commitment.block_number({})",
				rev_block_number,client_state.latest_commitment.block_number);

				return Err(Error::<T>::ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber.into());
			}
			// build new beefy light client by client_state
			let mut light_client = beefy_light_client::LightClient {
				latest_commitment: Some(client_state.latest_commitment.clone().into()),
				validator_set: client_state.validator_set.clone().into(),
				in_process_state: None,
			};
			log::trace!(
				"build new beefy_light_client from client_state store in chain \n {:?}",
				light_client
			);

			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs = decode_received_mmr_root.validator_merkle_proofs;
			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs: Vec<beefy_light_client::ValidatorMerkleProof> = validator_proofs
				.into_iter()
				.map(|validator_proof| validator_proof.into())
				.collect();

			// encode signed_commitment
			let encoded_signed_commitment =
				commitment::SignedCommitment::encode(&signed_commitment);

			let mmr_leaf = decode_received_mmr_root.mmr_leaf;
			let mmr_leaf_proof = decode_received_mmr_root.mmr_leaf_proof;

			// verfiy mmr proof and update lc state
			let result = light_client.update_state(
				&encoded_signed_commitment,
				&validator_proofs,
				&mmr_leaf,
				&mmr_leaf_proof,
			);

			match result {
				Ok(_) => {
					log::trace!("update the beefy light client sucesse! and the beefy light client state is : {:?} \n",light_client);

					// update client_client block number and latest commitment
					let latest_commitment =
						light_client.latest_commitment.ok_or(Error::<T>::EmptyLatestCommitment)?;
					client_state.block_number = latest_commitment.block_number;
					client_state.latest_commitment = help::Commitment::from(latest_commitment);

					// update validator_set
					client_state.validator_set =
						help::ValidatorSet::from(light_client.validator_set.clone());

					// update block header
					client_state.block_header = decode_received_mmr_root.block_header;

					// save to chain
					let any_client_state = AnyClientState::Grandpa(client_state.clone());
					let data =
						any_client_state.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					// store client states key-value
					<ClientStates<T>>::insert(client_id.clone(), data);

					// store client states keys
					let _ = <ClientStatesKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
						if let Some(_value) = val.iter().find(|&x| x == &client_id.clone()) {
						} else {
							val.push(client_id.clone());
						}
						Ok(())
					});

					log::trace!("the updated client state is : {:?}", client_state);

					use ibc::{
						clients::ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
						core::ics02_client::client_consensus::AnyConsensusState,
					};

					let mut consensus_state =
						GPConsensusState::new(client_state.block_header.clone());
					consensus_state.digest = client_state.latest_commitment.payload.clone();
					let any_consensus_state = AnyConsensusState::Grandpa(consensus_state);

					let height = ibc::Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};

					log::trace!("in ibc-lib : [store_consensus_state] >> client_id: {:?}, height = {:?}, consensus_state = {:?}", client_id, height, any_consensus_state);

					let height = height.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					let data =
						any_consensus_state.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					if <ConsensusStates<T>>::contains_key(client_id.clone()) {
						// if consensus_state is no empty use push insert an exist ConsensusStates
						let _ = <ConsensusStates<T>>::try_mutate(
							client_id,
							|val| -> Result<(), &'static str> {
								val.push((height, data));
								Ok(())
							},
						);
					} else {
						// if consensus state is empty insert a new item.
						<ConsensusStates<T>>::insert(client_id, vec![(height, data)]);
					}

					// emit update state sucesse event
					let event_height = Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};
					let event_client_state = EventClientState::from(client_state);
					Self::deposit_event(Event::<T>::UpdateClientState(
						event_height,
						event_client_state,
					));
				},
				Err(e) => {
					log::error!("update the beefy light client failure! : {:?}", e);

					return Err(Error::<T>::UpdateBeefyLightClientFailure.into())
				},
			}

			Ok(().into())
		}

		/// Transfer interface for user test by explore
		///
		///
		/// Example of invoking this function via subxt
		///
		/// ```ignore
		///     let api = client.to_runtime_api::<ibc_node::RuntimeApi<ibc_node::DefaultConfig>>();
		///
		///     let result = api
		///         .tx()
		///         .ibc()
		///         .transfer(msg, 0)
		///         .sign_and_submit(&signer)
		///         .await?;
		/// ```
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			source_port: Vec<u8>,
			source_channel: Vec<u8>,
			token: Vec<u8>,
			amount: u32,
			receiver: Vec<u8>,
			timeout_height: u64,
			timeout_timestamp: u64,
		) -> DispatchResult {
			let _source_port = identifier::PortId::from_str(
				&String::from_utf8(source_port).map_err(|_| Error::<T>::InvalidFromUtf8)?,
			)
			.map_err(|_| Error::<T>::InvalidIdentifier)?;
			let _source_channel = identifier::ChannelId::from_str(
				&String::from_utf8(source_channel).map_err(|_| Error::<T>::InvalidFromUtf8)?,
			)
			.map_err(|_| Error::<T>::InvalidIdentifier)?;

			let _token = Some(ibc_proto::cosmos::base::v1beta1::Coin {
				denom: String::from_utf8(token).map_err(|_| Error::<T>::InvalidFromUtf8)?,
				amount: amount.to_string(),
			});

			let _sender = ensure_signed(origin)?;
			let _sender = Signer::new(format!("{:?}", _sender.clone()));
			let _receiver =
				Signer::new(String::from_utf8(receiver).map_err(|_| Error::<T>::InvalidFromUtf8)?);

			let _timeout_height =
				height::Height { revision_number: 0, revision_height: timeout_height };
			let _timeout_timestamp = timestamp::Timestamp::from_nanoseconds(timeout_timestamp)
				.map_err(|_| Error::<T>::InvalidTimestamp)?;

			let msg = MsgTransfer {
				source_port: _source_port,
				source_channel: _source_channel,
				token: _token,
				sender: _sender,
				receiver: _receiver,
				timeout_height: _timeout_height,
				timeout_timestamp: _timeout_timestamp,
			};

			// send to router
			let mut ctx = routing::Context::<T>::new();
			let (result, _) =
				ibc::core::ics26_routing::handler::deliver(&mut ctx, msg.clone().to_any())
					.map_err(|_| Error::<T>::Ics20Error)?;
			// handle the result
			log::info!("result: {:?}", result);
			// let events = result.events
			Self::handle_result(&mut ctx, vec![msg.to_any()], result)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// handle the event returned by ics26 route module
		fn handle_result<Ctx>(
			ctx: &Ctx,
			messages: Vec<ibc_proto::google::protobuf::Any>,
			result: Vec<IbcEvent>,
		) -> DispatchResult
		where
			Ctx: Ics20Context,
		{
			for (index, event) in result.into_iter().enumerate() {
				match event.clone() {
					IbcEvent::SendPacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/f5962c3324ee7e69eeaa9918b65eb1b089da6095/modules/apps/transfer/keeper/msg_server.go#L16

						let ret =
							ics20_handler::handle_transfer::<Ctx, T>(ctx, value.clone().packet)
								.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::ReceivePacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L364

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();
						let ack = ibc::core::ics26_routing::ibc_module::IBCModule::on_recv_packet(
							&ics20_modlue,
							ctx,
							value.clone().packet,
							relayer_signer,
						)
						.map_err(|_| Error::<T>::Ics20Error)?;

						let packet = value.packet;

						let write_ack_event =
							ibc::core::ics04_channel::handler::write_acknowledgement::process(
								ctx,
								packet.clone(),
								ack.clone(),
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						// store write acknowledgement event
						match write_ack_event.result {
							ibc::core::ics04_channel::packet::PacketResult::WriteAck(value) => {
								<WriteAckPacketEvent<T>>::insert(
									(
										value.port_id.as_bytes().to_vec(),
										value.channel_id.as_bytes().to_vec(),
										u64::from(value.seq),
									),
									ack.clone(),
								);
							},
							_ => unimplemented!(),
						}

						// Emit write acknowledgement event
						Self::deposit_event(Event::<T>::WriteAcknowledgement(
							Height::new(0, 0),
							packet.into(),
							ack,
						));

						// Emit recv event
						Self::deposit_event(event.clone().into());
					},
					IbcEvent::TimeoutPacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L442

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_module = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_timeout_packet(
								&ics20_module,
								ctx,
								value.clone().packet,
								relayer_signer,
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::AcknowledgePacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L581

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_module = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();

						let ret = ibc::core::ics26_routing::ibc_module::IBCModule::on_acknowledgement_packet(&ics20_module, ctx, value.clone().packet, vec![], relayer_signer).map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenInitChannel(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L163
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let height = value.clone().height;
						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;
						let connection_id = value.clone().connection_id;
						let counterparty_port_id = value.clone().counterparty_port_id;
						let counterparty_channel_id = value.clone().counterparty_channel_id;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_init(
								&ics20_modlue,
								ctx,
								Order::Unordered,
								vec![connection_id],
								port_id,
								channel_id,
								&Capability::default(), // todo
								Counterparty {
									port_id: counterparty_port_id,
									channel_id: counterparty_channel_id,
								},
								Version::ics20(),
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenTryChannel(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L203

						let height = value.clone().height;
						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;
						let connection_id = value.clone().connection_id;
						let counterparty_port_id = value.clone().counterparty_port_id;
						let counterparty_channel_id = value.clone().counterparty_channel_id;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_try(
								&ics20_modlue,
								ctx,
								Order::Unordered,
								vec![connection_id],
								port_id,
								channel_id,
								&Capability::default(), // todo
								Counterparty {
									port_id: counterparty_port_id,
									channel_id: counterparty_channel_id,
								},
								Version::ics20(),
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenAckChannel(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L241

						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();
						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_ack(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
								Version::ics20(),
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenConfirmChannel(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L277

						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_confirm(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},
					IbcEvent::CloseInitChannel(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L309

						let port_id = value.clone().port_id;
						let channel_id = value.clone().channel_id;
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_close_init(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::CloseConfirmChannel(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L336

						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = ics20_ibc_module_impl::Ics20IBCModule::<T>::new();
						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_close_confirm(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
							)
							.map_err(|_| Error::<T>::Ics20Error)?;

						Self::deposit_event(event.clone().into());
					},
					_ => {
						log::warn!("Unhandled event: {:?}", event);
						Self::deposit_event(event.clone().into());
					},
				}
			}
			Ok(().into())
		}

		/// update the latest height of a client
		fn store_latest_height(ibc_event: IbcEvent) -> DispatchResultWithPostInfo {
			match ibc_event {
				IbcEvent::Empty(_value) => {
					log::warn!("ibc event: {}", "Empty");
				},
				IbcEvent::NewBlock(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::SendPacket(value) => {
					// store height
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);

					// store send-packet
					let _value = value.clone();
					let packet = Packet {
						sequence: Sequence::from(_value.packet.sequence),
						source_channel: ChannelId::from(_value.packet.source_channel),
						source_port: PortId::from(_value.packet.source_port),
						destination_channel: ChannelId::from(_value.packet.destination_channel),
						destination_port: PortId::from(_value.packet.destination_port),
						data: _value.packet.data,
						timeout_timestamp: Timestamp::from(_value.packet.timeout_timestamp),
						timeout_height: Height::from(_value.packet.timeout_height),
					};
					let packet = packet
						.to_ibc_packet()
						.map_err(|_| Error::<T>::InvalidPacket)?
						.encode_vec()
						.map_err(|_| Error::<T>::InvalidEncode)?;

					let port_id = value.packet.source_port.as_bytes().to_vec();
					let channel_id = value.packet.source_channel.as_bytes().to_vec();

					<SendPacketEvent<T>>::insert(
						(port_id, channel_id, u64::from(value.packet.sequence)),
						packet,
					);
				},
				IbcEvent::WriteAcknowledgement(value) => {
					// store height
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::UpdateClient(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ReceivePacket(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CloseConfirmChannel(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CreateClient(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::UpgradeClient(value) => {
					let height = value
						.0
						.height
						.clone()
						.encode_vec()
						.map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ClientMisbehaviour(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenInitConnection(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenTryConnection(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenAckConnection(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenConfirmConnection(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenInitChannel(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenTryChannel(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenAckChannel(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenConfirmChannel(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CloseInitChannel(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::AcknowledgePacket(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::TimeoutPacket(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::TimeoutOnClosePacket(value) => {
					let height =
						value.height().encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ChainError(_value) => {
					log::warn!("Ibc event: {}", "chainError");
				},
			}
			Ok(().into())
		}
	}
}

/// FungibleTokenPacketData defines a struct for the packet payload
/// See FungibleTokenPacketData spec: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures
#[derive(Decode, Encode, Debug, PartialEq)]
pub struct FungibleTokenPacketData<T: Config> {
	// the token denomination to be transferred
	pub denomination: Vec<u8>,
	// the token amount to be transferred
	pub amount: u128,
	// pub amount: T::AssetBalance,
	// the sender address
	pub sender: T::AccountId,
	// the recipient address on the destination chain
	pub receiver: T::AccountId,
}

use ibc::applications::ics20_fungible_token_transfer::msgs::fungible_token_packet_data::FungibleTokenPacketData as IBCFungibleTokenPacketData;

impl<T: Config> From<IBCFungibleTokenPacketData> for FungibleTokenPacketData<T> {
	fn from(value: IBCFungibleTokenPacketData) -> Self {
		Self {
			denomination: value.denom.as_bytes().to_vec(),
			amount: value.amount.parse::<u128>().unwrap_or_default(),
			sender: IbcId(value.sender.as_str().as_bytes().to_vec()).into_account(),
			receiver: IbcId(value.receiver.as_str().as_bytes().to_vec()).into_account(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FungibleTokenPacketAcknowledgement {
	Success(FungibleTokenPacketSuccess),
	Err(FungibleTokenPacketError),
}

impl FungibleTokenPacketAcknowledgement {
	pub fn new() -> Self {
		Self::Success(FungibleTokenPacketSuccess::new())
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleTokenPacketSuccess {
	result: AQ,
}

impl FungibleTokenPacketSuccess {
	pub fn new() -> Self {
		let aq = AQ;
		Self { result: aq }
	}
	pub fn result(&self) -> &str {
		// this is binary 0x01 base64 encoded
		"AQ=="
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct AQ;

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleTokenPacketError {
	pub error: String,
}

/// A pallet identifier. These are per pallet and should be stored in a registry somewhere.
#[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub struct IbcId(pub Vec<u8>);

impl TypeId for IbcId {
	const TYPE_ID: [u8; 4] = *b"Ibcs";
}

fn get_signer<T: Config>(
	message: ibc_proto::google::protobuf::Any,
) -> Result<ibc::signer::Signer, DispatchError> {
	let decode_message = ibc::core::ics26_routing::handler::decode(message)
		.map_err(|_| Error::<T>::InvalidDecode)?;
	let signer = match decode_message {
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics2Msg(value) => match value {
			ibc::core::ics02_client::msgs::ClientMsg::CreateClient(val) => val.signer.clone(),
			ibc::core::ics02_client::msgs::ClientMsg::UpdateClient(val) => val.signer.clone(),
			ibc::core::ics02_client::msgs::ClientMsg::Misbehaviour(val) => val.signer.clone(),
			ibc::core::ics02_client::msgs::ClientMsg::UpgradeClient(val) => val.signer.clone(),
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics3Msg(value) => match value {
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenInit(val) =>
				val.signer.clone(),
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenTry(val) =>
				val.signer.clone(),
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenAck(val) =>
				val.signer.clone(),
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenConfirm(val) =>
				val.signer.clone(),
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics4ChannelMsg(value) => match value {
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenInit(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenTry(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenAck(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenConfirm(val) =>
				val.signer.clone(),
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelCloseInit(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelCloseConfirm(val) =>
				val.signer.clone(),
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics4PacketMsg(value) => match value {
			ibc::core::ics04_channel::msgs::PacketMsg::RecvPacket(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::PacketMsg::AckPacket(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::PacketMsg::ToPacket(val) => val.signer.clone(),
			ibc::core::ics04_channel::msgs::PacketMsg::ToClosePacket(val) => val.signer.clone(),
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics20Msg(value) => value.sender.clone(),
	};

	Ok(signer)
}
