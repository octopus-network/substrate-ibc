// #![cfg_attr(not(feature = "std"), no_std)]

//! # IBC Module
//!
//! This module implements the standard [IBC protocol](https://github.com/cosmos/ics).
//!
//! ## Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to interact with other chains in a trustless way via IBC protocol, no matter what consensus the counterparty chains use.
//!
//! This project is currently in an early stage and will eventually be submitted to upstream.
//!
//! Some components in [IBC spec](https://github.com/cosmos/ics/tree/master/spec) are implemented to support a working demo (https://github.com/cdot-network/ibc-demo), but not fully implemented as the spec yet:
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
//! Please refer to [IBC Terminology](https://github.com/cosmos/ics/blob/master/ibc/1_IBC_TERMINOLOGY.md#1-ibc-terminology).
//!
//! ### Goals
//!
//! This IBC module handles authentication, transport, and ordering of structured data packets relayed between modules on separate machines.
//!
//! Example applications include cross-chain asset transfer, atomic swaps, multi-chain smart contracts (with or without mutually comprehensible VMs), and data & code sharding of various kinds.
//!
//! ## Interface
//!
//! ###  Public Functions
//!
//! * `handle_datagram` - Receives datagram transmitted from relayers, and implements the following:
//!     + Synchronizing block headers from other chains.
//!     + Process connection opening handshakes after its initialization - ICS-003.
//!     + Process channel opening handshakes after its initialization - ICS-004.
//!     + Handling packet flow after its initialization - ICS-004.
//!
//! ### Dispatchable Functions
//!
//! * `conn_open_init` - Connection opening handshake initialization.
//! * `chan_open_init` - Channel opening handshake initialization.
//! * `send_packet` - Packet flow initialization.
//!
//! ## Usage
//! Please refer to section "How to Interact with the Pallet" in the repository's README.md

extern crate alloc;

pub use pallet::*;

use alloc::format;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_system::ensure_signed;
use ibc;
pub use routing::ModuleCallbacks;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use std::str::FromStr;

mod channel;
mod client;
mod connection;
pub mod event;
mod port;
mod routing;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Any {
    pub type_url: String,
    pub value: Vec<u8>,
}

impl From<prost_types::Any> for Any {
	fn from(any: prost_types::Any) -> Self {
        Self {
			type_url: any.type_url,
			value: any.value,
		}
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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use event::primitive::{ClientType, ClientId, Height, ConnectionId, PortId, ChannelId, Packet};
	use ibc::events::IbcEvent;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ModuleCallbacks: routing::ModuleCallbacks;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// fix before : (client_id, height) => ConsensusState
	// fix after: client_id => (Height, ConsensusState)
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	// connection_id => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => ChannelEnd
	pub type Channels<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	// store_connection_channels
	#[pallet::storage]
	// connection_identifier => Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>,Vec<u8>)>, ValueQuery>;


	#[pallet::storage]
	// (port_identifier, channel_identifier) => Sequence
	pub type NextSequenceSend<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => Sequence
	pub type NextSequenceRecv<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) = Sequence
	pub type NextSequenceAck<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier, sequence) => Hash
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// clientId => ClientType
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultClientCounter() -> u64 {
		0u64
	}

	#[pallet::storage]
	#[pallet::getter(fn client_counter)]
	// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery, DefaultClientCounter>;

	#[pallet::type_value]
	pub fn DefaultConnectionCounter() -> u64 { 0u64 }

	#[pallet::storage]
	#[pallet::getter(fn connection_counter)]
	// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery, DefaultConnectionCounter>;

	#[pallet::type_value]
	pub fn DefaultChannelCounter() -> u64 { 0u64 }


	#[pallet::storage]
	// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery, DefaultChannelCounter>;

	#[pallet::storage]
	// client_id => Connection id
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (portid, channelid, sequence) => receipt
	pub type PacketReceipt<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (portid, channelid, sequence) => hash
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultOldHeight() -> u64 { 0u64 }

	#[pallet::storage]
	// store oldest height
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery, DefaultOldHeight>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
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
        OpenInitConnection(
            Height,
            Option<ConnectionId>,
            ClientId,
            Option<ConnectionId>,
            ClientId,
        ),
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
		OpenTryConnection(
			Height,
			Option<ConnectionId>,
			ClientId,
			Option<ConnectionId>,
			ClientId,
		),
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
		OpenAckConnection(
			Height,
			Option<ConnectionId>,
			ClientId,
			Option<ConnectionId>,
			ClientId,
		),
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
		OpenInitChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		// OpenTryChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenTryChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		// OpenAckChannel(
		// 	height: Height,
		// 	port_id: PortId,
		// 	channel_id: Option<ChannelId>,
		// 	connection_id: ConnectionId,
		// 	counterparty_port_id: PortId,
		// 	counterparty_channel_id: Option<ChannelId>
		// )
		OpenAckChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
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
		// SendPacket {
		// 	height: Height,
		// 	packet: Packet,
		// }
		SendPacket(
			Height,
			Packet,
		)
    }

    impl<T: Config> From<ibc::events::IbcEvent> for Event<T> {
        fn from(value: ibc::events::IbcEvent) -> Self {
            match value {
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
				}
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
						consensus_height.into()
					)
				}
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
				}
				ibc::events::IbcEvent::ClientMisbehaviour(value ) => {
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
				}
				// OpenInitConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenInitConnection(value) => {
					let height = value.0.height;
					let connection_id : Option<ConnectionId> = value.0.connection_id.map(|val| val.into());
					let client_id = value.0.client_id;
					let counterparty_connection_id: Option<ConnectionId> = value.0.counterparty_connection_id.map(|val| val.into());

					let counterparty_client_id = value.0.counterparty_client_id;
					Event::OpenInitConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				}

				// OpenTryConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenTryConnection(value) => {
					let height = value.0.height;
					let connection_id : Option<ConnectionId> = value.0.connection_id.map(|val| val.into());
					let client_id = value.0.client_id;
					let counterparty_connection_id: Option<ConnectionId> = value.0.counterparty_connection_id.map(|val| val.into());

					let counterparty_client_id = value.0.counterparty_client_id;
					Event::OpenTryConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				}
				// OpenAckConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenAckConnection(value) => {
					let height = value.0.height;
					let connection_id : Option<ConnectionId> = value.0.connection_id.map(|val| val.into());
					let client_id = value.0.client_id;
					let counterparty_connection_id: Option<ConnectionId> = value.0.counterparty_connection_id.map(|val| val.into());

					let counterparty_client_id = value.0.counterparty_client_id;
					Event::OpenAckConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				}
				// OpenConfirmConnection(
				// 	height: Height,
				// 	connection_id: Option<ConnectionId>,
				// 	client_id: ClientId,
				// 	counterparty_connection_id: Option<ConnectionId>,
				// 	counterparty_client_id: ClientId,
				// }
				ibc::events::IbcEvent::OpenConfirmConnection(value) => {
					let height = value.0.height;
					let connection_id : Option<ConnectionId> = value.0.connection_id.map(|val| val.into());
					let client_id = value.0.client_id;
					let counterparty_connection_id: Option<ConnectionId> = value.0.counterparty_connection_id.map(|val| val.into());

					let counterparty_client_id = value.0.counterparty_client_id;
					Event::OpenConfirmConnection(
						height.into(),
						connection_id,
						client_id.into(),
						counterparty_connection_id,
						counterparty_client_id.into(),
					)
				}
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
					let channel_id : Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					Event::OpenInitChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				}
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
					let channel_id : Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					Event::OpenTryChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				}
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
					let channel_id : Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					Event::OpenAckChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				}
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
					let channel_id : Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					let connection_id = value.0.connection_id;
					let counterparty_port_id = value.0.counterparty_port_id;
					let counterparty_channel_id: Option<ChannelId> = value.0.channel_id.clone().map(|val| val.into());
					Event::OpenConfirmChannel(
						height.into(),
						port_id.into(),
						channel_id,
						connection_id.into(),
						counterparty_port_id.into(),
						counterparty_channel_id,
					)
				}
				// SendPacket {
				//     pub height: Height,
				//     pub packet: Packet,
				// }
				ibc::events::IbcEvent::SendPacket(value) => {
					let height = value.height;
					let packet = value.packet;
					Event::SendPacket(
						height.into(),
						packet.into(),
					)
				}
                _ => unimplemented!(),
            }
        }
    }

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// /// The IBC client identifier already exists.
		// ClientIdExist,
		// /// The IBC client identifier doesn't exist.
		// ClientIdNotExist,
		// /// The IBC port identifier is already binded.
		// PortIdBinded,
		// /// The IBC connection identifier already exists.
		// ConnectionIdExist,
		// /// The IBC connection identifier doesn't exist.
		// ConnectionIdNotExist,
		// /// The IBC channel identifier already exists.
		// ChannelIdExist,
		// /// The IBC port identifier doesn't match.
		// PortIdNotMatch,
		// /// The IBC connection is closed.
		// ConnectionClosed,
		// /// Only allow 1 hop for v1 of the IBC protocol.
		// OnlyOneHopAllowedV1,
		// /// The sequence sending packet not match
		// PackedSequenceNotMatch,
		// /// The destination channel identifier doesn't match
		// DestChannelIdNotMatch,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
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
					type_url: message.type_url.clone(),
					value: message.value.clone(),
				})
				.collect();
			let result = ibc::ics26_routing::handler::deliver(&mut ctx, messages).unwrap();

			log::info!("result: {:?}", result);

			use ibc::events::IbcEvent;

			for event in result {
				log::info!("Event: {:?}", event);
				Self::deposit_event(event.into());
			}

			Ok(())
		}
	}


	impl <T: Config> Pallet<T> {
		/// get key-value pair (client_identifier, client_state)
		pub fn get_identified_any_client_state() -> Vec<(Vec<u8>, Vec<u8>)> {
			let mut result = vec![];

			<ClientStates<T>>::iter().for_each(|val| {
				result.push((val.0, val.1));
			});

			result
		}
	}
}


