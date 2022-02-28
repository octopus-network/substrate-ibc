#![cfg_attr(not(feature = "std"), no_std)]
// TODO to remove
#![allow(unreachable_patterns)]
#![allow(clippy::type_complexity)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

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

use alloc::{format, string::String};
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_system::ensure_signed;
pub use pallet::*;
pub use routing::ModuleCallbacks;
use scale_info::{prelude::vec, TypeInfo};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

mod channel;
mod client;
mod connection;
pub mod event;
mod port;
mod routing;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Any {
	pub type_url: Vec<u8>,
	pub value: Vec<u8>,
}

impl From<prost_types::Any> for Any {
	fn from(any: prost_types::Any) -> Self {
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
		ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId, Timestamp,
	};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::events::IbcEvent;
	use tendermint_proto::Protobuf;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ModuleCallbacks: routing::ModuleCallbacks;
		type TimeProvider: UnixTime;

		/// Prefix for events stored in the Off-chain DB via Indexing API.
		const INDEXING_PREFIX: &'static [u8];
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// vector client_id
	pub type ClientStatesKeys<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	// fix before : (client_id, height) => ConsensusState
	// fix after: client_id => (Height, ConsensusState)
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	// connection_id => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// vector connection_id
	pub type ConnectionsKeys<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => ChannelEnd
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
	// vector (port_identifier, channel_identifier)
	pub type ChannelsKeys<T: Config> = StorageValue<_, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	// store_connection_channels
	#[pallet::storage]
	// connection_identifier => Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => Sequence
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
	// (port_identifier, channel_identifier) => Sequence
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
	// (port_identifier, channel_identifier) = Sequence
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
	// (port_identifier, channel_identifier, sequence) => Hash
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// TODO
	// vector (port_identifier, channel_identifier, sequence)
	pub type AcknowledgementsKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	// clientId => ClientType
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn client_counter)]
	// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_counter)]
	// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	// client_id => Connection id
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// TODO
	// (port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// TODO
	// (port_id, channel_id, sequence) => hash
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// vector (port_identifier, channel_identifier, sequence)
	pub type PacketCommitmentKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	// TODO
	// (height, port_id, channel_id, sequence) => event
	pub type SendPacketEvent<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, u64), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	//TODO
	// (port_id, channel_id, sequence), ackHash)
	pub type WriteAckPacketEvent<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, u64), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// store latest height
	pub type LatestHeight<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// store latest height
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// NewBlock(height: Height)
		NewBlock(Height),

		// This event for Client
		//
		// CreateClient Event
		//
		// CreateClient(
		// 	height: Height,
		// 	client_id: ClientId,
		// 	client_type: ClientType,
		// 	consensus_height: Height,
		// )
		CreateClient(Height, ClientId, ClientType, Height),
		// UpdateClient Event
		//
		// UpdateClient(
		// 	height: Height,
		// 	client_id: ClientId,
		// 	client_type: ClientType,
		// 	consensus_height: Height,
		// )
		UpdateClient(Height, ClientId, ClientType, Height),
		// UpgradeClient Event
		//
		// UpgradeClient(
		// 	height: Height,
		// 	client_id: ClientId,
		// 	client_type: ClientType,
		// 	consensus_height: Height,
		// )
		UpgradeClient(Height, ClientId, ClientType, Height),
		// ClientMisbehaviour Event
		//
		// ClientMisbehaviour(
		// 	height: Height,
		// 	client_id: ClientId,
		// 	client_type: ClientType,
		// 	consensus_height: Height,
		// )
		ClientMisbehaviour(Height, ClientId, ClientType, Height),
		// This Event for Connection
		//
		// Open Init Connection
		//
		// OpenInitConnection(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenInitConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		// Open try Connection
		//
		// OpenTryConnection(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenTryConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		// Open ack Connection
		//
		// OpenAckConnection(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenAckConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		// Open ack Connection
		//
		// OpenConfirmConnection(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenConfirmConnection(
			Height,
			Option<ConnectionId>,
			ClientId,
			Option<ConnectionId>,
			ClientId,
		),
		//  This Event for Channel
		//
		// OpenInitChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenInitChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		// OpenTryChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenTryChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		// OpenAckChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenAckChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		// OpenAckChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		// CloseInitChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		CloseInitChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		// CloseConfirmChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		CloseConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		// SendPacket {
		// 	height: Height,
		// 	packet: Packet,
		// }
		SendPacket(Height, Packet),
		// ReceivePacket {
		// 	height: Height,
		// 	packet: Packet,
		// }
		ReceivePacket(Height, Packet),
		// WriteAcknowledgement {
		// 	height: Height,
		// 	packet: Packet,
		//  ack: Vec<u8>,
		// }
		WriteAcknowledgement(Height, Packet, Vec<u8>),
		// AcknowledgePacket {
		// 	height: Height,
		// 	packet: Packet,
		// }
		AcknowledgePacket(Height, Packet),
		// TimeoutPacket {
		// 	height: Height,
		// 	packet: Packet,
		// }
		TimeoutPacket(Height, Packet),
		// TimeoutOnClosePacket {
		// 	height: Height,
		// 	packet: Packet,
		// }
		TimeoutOnClosePacket(Height, Packet),
		// Empty(String) Special event, signifying empty response
		Empty(Vec<u8>),
		// ChainError(String) Special event, signifying an error an CheckTx or DeliverTx
		ChainError(Vec<u8>),
	}

	impl<T: Config> From<ibc::events::IbcEvent> for Event<T> {
		fn from(value: ibc::events::IbcEvent) -> Self {
			match value {
				// NewBlock(height: Height)
				ibc::events::IbcEvent::NewBlock(value) => Event::NewBlock(value.height.into()),
				// CreateClient(
				// 	height: Height,
				// 	client_id: ClientId,
				// 	client_type: ClientType,
				// 	consensus_height: Height,
				// )
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
				// UpdateClient(
				// 	height: Height,
				// 	client_id: ClientId,
				// 	client_type: ClientType,
				// 	consensus_height: Height,
				// )
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
				// TODO! Upgrade client events are not currently being used
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
				// OpenInitConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenInitConnection(value) => {
					let attributes = value.attributes();
					let height = attributes.height;
					let connection_id: Option<ConnectionId> =
						attributes.connection_id.clone().map(|val| val.into());
					let client_id = attributes.client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						attributes.counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = attributes.counterparty_client_id.clone();
					Event::OpenInitConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},

				// OpenTryConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenTryConnection(value) => {
					let attributes = value.attributes();
					let height = attributes.height;
					let connection_id: Option<ConnectionId> =
						attributes.connection_id.clone().map(|val| val.into());
					let client_id = attributes.client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						attributes.counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = attributes.counterparty_client_id.clone();
					Event::OpenTryConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				// OpenAckConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenAckConnection(value) => {
					let attributes = value.attributes();
					let height = attributes.height;
					let connection_id: Option<ConnectionId> =
						attributes.connection_id.clone().map(|val| val.into());
					let client_id = attributes.client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						attributes.counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = attributes.counterparty_client_id.clone();
					Event::OpenAckConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				// OpenConfirmConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenConfirmConnection(value) => {
					let attributes = value.attributes();
					let height = attributes.height;
					let connection_id: Option<ConnectionId> =
						attributes.connection_id.clone().map(|val| val.into());
					let client_id = attributes.client_id.clone();
					let counterparty_connection_id: Option<ConnectionId> =
						attributes.counterparty_connection_id.clone().map(|val| val.into());

					let counterparty_client_id = attributes.counterparty_client_id.clone();
					Event::OpenConfirmConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				},
				// OpenInitChannel(
				// 	height: Height,
				// 	port_id: PortId,
				// 	channel_id: Option<ChannelId>,
				// 	connection_id: ConnectionId,
				// 	counterparty_port_id: PortId,
				// 	counterparty_channel_id: Option<ChannelId>
				// );
				ibc::events::IbcEvent::OpenInitChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::OpenInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				// OpenTryChannel(
				// 	height: Height,
				// 	port_id: PortId,
				// 	channel_id: Option<ChannelId>,
				// 	connection_id: ConnectionId,
				// 	counterparty_port_id: PortId,
				// 	counterparty_channel_id: Option<ChannelId>
				// );
				ibc::events::IbcEvent::OpenTryChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::OpenTryChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				// OpenAckChannel(
				// 	height: Height,
				// 	port_id: PortId,
				// 	channel_id: Option<ChannelId>,
				// 	connection_id: ConnectionId,
				// 	counterparty_port_id: PortId,
				// 	counterparty_channel_id: Option<ChannelId>
				// );
				ibc::events::IbcEvent::OpenAckChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::OpenAckChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				// OpenConfirmChannel(
				// 	height: Height,
				// 	port_id: PortId,
				// 	channel_id: Option<ChannelId>,
				// 	connection_id: ConnectionId,
				// 	counterparty_port_id: PortId,
				// 	counterparty_channel_id: Option<ChannelId>
				// );
				ibc::events::IbcEvent::OpenConfirmChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::OpenConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				// CloseInitChannel(
				// 	height: Height,
				// 	port_id: PortId,
				// 	channel_id: Option<ChannelId>,
				// 	connection_id: ConnectionId,
				// 	counterparty_port_id: PortId,
				// 	counterparty_channel_id: Option<ChannelId>
				// );
				ibc::events::IbcEvent::CloseInitChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::CloseInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				// CloseConfirmChannel(
				// 	height: Height,
				// 	port_id: PortId,
				// 	channel_id: Option<ChannelId>,
				// 	connection_id: ConnectionId,
				// 	counterparty_port_id: PortId,
				// 	counterparty_channel_id: Option<ChannelId>
				// );
				ibc::events::IbcEvent::CloseConfirmChannel(value) => {
					let height = value.0.height;
					let port_id = value.0.port_id;
					let channel_id: Option<ChannelId> =
						value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> =
						value.0.channel_id.map(|val| val.into());
					Event::CloseConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				},
				// SendPacket {
				//     pub height: Height,
				//     pub packet: Packet,
				// }
				ibc::events::IbcEvent::SendPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::SendPacket(height.into(), packet.into())
				},
				// ReceivePacket {
				//     pub height: Height,
				//     pub packet: Packet,
				// }
				ibc::events::IbcEvent::ReceivePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::ReceivePacket(height.into(), packet.into())
				},
				// WriteAcknowledgement {
				//     pub height: Height,
				//     pub packet: Packet,
				//     pub ack: Vec<u8>,
				// }
				ibc::events::IbcEvent::WriteAcknowledgement(value) => {
					let height = value.height;
					let packet = value.packet;
					let ack = value.ack;
					Event::WriteAcknowledgement(height.into(), packet.into(), ack)
				},
				// AcknowledgePacket {
				//     pub height: Height,
				//     pub packet: Packet,
				// }
				ibc::events::IbcEvent::AcknowledgePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::AcknowledgePacket(height.into(), packet.into())
				},
				// TimeoutPacket {
				//     pub height: Height,
				//     pub packet: Packet,
				// }
				ibc::events::IbcEvent::TimeoutPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::TimeoutPacket(height.into(), packet.into())
				},
				// TimeoutOnClosePacket {
				//     pub height: Height,
				//     pub packet: Packet,
				// }
				ibc::events::IbcEvent::TimeoutOnClosePacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::TimeoutOnClosePacket(height.into(), packet.into())
				},
				// Empty(String)
				ibc::events::IbcEvent::Empty(value) => Event::Empty(value.as_bytes().to_vec()),
				// ChainError(String)
				ibc::events::IbcEvent::ChainError(value) =>
					Event::ChainError(value.as_bytes().to_vec()),
				_ => unimplemented!(),
			}
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	// Dispatch able functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsic", which are often compared to transactions.
	// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn deliver(origin: OriginFor<T>, messages: Vec<Any>, tmp: u8) -> DispatchResult {
			log::info!("in deliver");

			let _sender = ensure_signed(origin)?;
			let mut ctx = routing::Context { _pd: PhantomData::<T>, tmp };
			let messages = messages
				.iter()
				.map(|message| prost_types::Any {
					type_url: String::from_utf8(message.type_url.clone()).unwrap(),
					value: message.value.clone(),
				})
				.collect();
			let result = ibc::core::ics26_routing::handler::deliver(&mut ctx, messages).unwrap();

			log::info!("result: {:?}", result);

			for event in result {
				log::info!("Event: {:?}", event);
				Self::deposit_event(event.clone().into());
				Self::store_latest_height(event.clone());
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn store_latest_height(ibc_event: IbcEvent) {
			match ibc_event {
				IbcEvent::Empty(_value) => {
					log::info!("ibc event: {}", "Empty");
				},
				IbcEvent::NewBlock(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::SendPacket(value) => {
					// store height
					let height = value.height().encode_vec().unwrap();
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
					let packet = packet.encode();

					let port_id = value.packet.source_port.as_bytes().to_vec();
					let channel_id = value.packet.source_channel.as_bytes().to_vec();

					<SendPacketEvent<T>>::insert(
						(port_id, channel_id, u64::from(value.packet.sequence)),
						packet,
					);
				},
				IbcEvent::WriteAcknowledgement(value) => {
					// store height
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);

					// store ack
					let port_id = value.packet.source_port.as_bytes().to_vec();
					let channel_id = value.packet.source_channel.as_bytes().to_vec();
					let sequence = u64::from(value.packet.sequence);
					let ack = value.ack;
					// store.Set((portID, channelID, sequence), ackHash)
					<WriteAckPacketEvent<T>>::insert((port_id, channel_id, sequence), ack)
				},
				IbcEvent::UpdateClient(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ReceivePacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CloseConfirmChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CreateClient(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::UpgradeClient(value) => {
					let height = value.0.height.clone().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ClientMisbehaviour(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenInitConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenTryConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenAckConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenConfirmConnection(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenInitChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenTryChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenAckChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::OpenConfirmChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::CloseInitChannel(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::AcknowledgePacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::TimeoutPacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::TimeoutOnClosePacket(value) => {
					let height = value.height().encode_vec().unwrap();
					<LatestHeight<T>>::set(height);
				},
				IbcEvent::ChainError(_value) => {
					log::info!("Ibc event: {}", "chainError");
				},
			}
		}

		fn offchain_key() -> &'static str {
			""
		}
	}
}
