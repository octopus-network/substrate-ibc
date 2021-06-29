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

pub use pallet::*;

pub use client::ClientType;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use finality_grandpa::voter_set::VoterSet;
use frame_support::ensure;
use frame_system::ensure_signed;
use grandpa::justification::GrandpaJustification;
use grandpa::state_machine::read_proof_check;
use ibc;
pub use routing::ModuleCallbacks;
use sp_core::H256;
use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, Hash},
	OpaqueExtrinsic as UncheckedExtrinsic, RuntimeDebug,
};
use sp_std::prelude::*;
use sp_trie::StorageProof;

mod client;
pub mod grandpa;
mod handler;
mod header;
pub mod informalsystems;
mod routing;
mod state;

type BlockNumber = u32;
type Block = generic::Block<generic::Header<BlockNumber, BlakeTwo256>, UncheckedExtrinsic>;

// Todo: Find a crate specific for semantic version
const VERSIONS: [u8; 3] = [1, 3, 5];

// Todo: Find a proper value for MAX_HISTORY_SIZE
const MAX_HISTORY_SIZE: u32 = 3;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Packet {
	pub sequence: u64,
	/// If the latest block height of the destination chain is greater than ```timeout_height```, the packet will not be processed.
	pub timeout_height: u32,
	pub source_port: Vec<u8>,
	pub source_channel: H256,
	pub dest_port: Vec<u8>,
	pub dest_channel: H256,
	pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum Datagram {
	ClientUpdate {
		client_id: H256,
		header: grandpa::header::Header,
	},
	ClientMisbehaviour {
		identifier: H256,
		evidence: Vec<u8>,
	},
	ConnOpenTry {
		connection_id: H256,
		counterparty_connection_id: H256,
		counterparty_client_id: H256,
		client_id: H256,
		version: Vec<u8>, // Todo: remove this field
		counterparty_version: Vec<u8>,
		proof_init: StorageProof,
		proof_consensus: StorageProof,
		proof_height: u32,
		consensus_height: u32,
	},
	ConnOpenAck {
		connection_id: H256,
		counterparty_connection_id: H256,
		version: u8,
		proof_try: StorageProof,
		proof_consensus: StorageProof,
		proof_height: u32,
		consensus_height: u32,
	},
	ConnOpenConfirm {
		connection_id: H256,
		proof_ack: StorageProof,
		proof_height: u32,
	},
	ChanOpenTry {
		order: ChannelOrder,
		connection_hops: Vec<H256>,
		port_id: Vec<u8>,
		channel_id: H256,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: H256,
		channel_version: Vec<u8>,
		counterparty_version: Vec<u8>,
		proof_init: StorageProof,
		proof_height: u32,
	},
	ChanOpenAck {
		port_id: Vec<u8>,
		channel_id: H256,
		version: Vec<u8>,
		proof_try: StorageProof, // Todo: In ibc-rs, proofs contains `object_proof`, `client_proof`, `consensus_proof` and `height`
		proof_height: u32,
	},
	ChanOpenConfirm {
		port_id: Vec<u8>,
		channel_id: H256,
		proof_ack: StorageProof,
		proof_height: u32,
	},
	PacketRecv {
		packet: Packet,
		proof: StorageProof,
		proof_height: u32,
	},
	PacketAcknowledgement {
		packet: Packet,
		acknowledgement: Vec<u8>,
		proof: StorageProof,
		proof_height: u32,
	},
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ConnectionState {
	None,
	Init,
	TryOpen,
	Open,
	Closed,
}

impl Default for ConnectionState {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct ConnectionEnd {
	pub state: ConnectionState,
	pub counterparty_connection_id: H256,
	/// The prefix used for state verification on the counterparty chain associated with this connection.
	/// If not specified, a default counterpartyPrefix of "ibc" should be used.
	counterparty_prefix: Vec<u8>,
	pub client_id: H256,
	counterparty_client_id: H256,
	pub version: Vec<u8>, // TODO: A ConnectionEnd should only store one version.
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChannelState {
	None,
	Init,
	TryOpen,
	Open,
	Closed,
}

impl Default for ChannelState {
	fn default() -> Self {
		Self::None
	}
}

// Todo: In ibc-rs, `ChannelOrder` is type i32
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum ChannelOrder {
	Ordered,
	Unordered,
}

impl Default for ChannelOrder {
	fn default() -> Self {
		Self::Ordered
	}
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct ChannelEnd {
	pub state: ChannelState,
	pub ordering: ChannelOrder,
	pub counterparty_port_id: Vec<u8>,
	pub counterparty_channel_id: H256,
	pub connection_hops: Vec<H256>,
	pub version: Vec<u8>,
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
	pub type ClientStatesV2<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// (client_id, height) => ConsensusState
	pub type ConsensusStatesV2<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// connection_identifier => ConnectionEnd
	pub type ConnectionsV2<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;


	#[pallet::storage]
	// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, H256, grandpa::client_state::ClientState, ValueQuery>;

	#[pallet::storage]
	// (client_id, height) => ConsensusState
	pub type ConsensusStates<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(H256, u32),
		grandpa::consensus_state::ConsensusState,
		ValueQuery,
	>;

	#[pallet::storage]
	// connection_identifier => ConnectionEnd
	pub type Connections<T: Config> =
		StorageMap<_, Blake2_128Concat, H256, ConnectionEnd, ValueQuery>;

	#[pallet::storage]
	// port_identifier => module_index
	pub type Ports<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u8, ValueQuery>;

	/// Channel structures are stored under a store path prefix unique to a combination of a port identifier and channel identifier.
	#[pallet::storage]
	// (port_identifier, channel_identifier) => ChannelEnd
	pub type Channels<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, H256), ChannelEnd, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => Sequence
	pub type NextSequenceSend<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, H256), u64, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => Sequence
	pub type NextSequenceRecv<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, H256), u64, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier) => Sequence
	pub type NextSequenceAck<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, H256), u64, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier, sequence) => Hash
	pub type Packets<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, H256, u64), H256, ValueQuery>;

	#[pallet::storage]
	// (port_identifier, channel_identifier, sequence) => Hash
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, H256, u64), H256, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClientCreated,
		ClientUpdated,
		ClientMisbehaviourReceived,
		ConnOpenInit,
		ConnOpenTry,
		ConnOpenAck,
		ConnOpenConfirm,
		PortBound(u8),
		PortReleased,
		ChanOpenInit,
		ChanOpenTry,
		ChanOpenAck,
		ChanOpenConfirm,
		SendPacket(u64, Vec<u8>, u32, Vec<u8>, H256, Vec<u8>, H256),
		RecvPacket(u64, Vec<u8>, u32, Vec<u8>, H256, Vec<u8>, H256, Vec<u8>),
		PacketRecvReceived,
		AcknowledgePacket,
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The IBC client identifier already exists.
		ClientIdExist,
		/// The IBC client identifier doesn't exist.
		ClientIdNotExist,
		/// The IBC port identifier is already binded.
		PortIdBinded,
		/// The IBC connection identifier already exists.
		ConnectionIdExist,
		/// The IBC connection identifier doesn't exist.
		ConnectionIdNotExist,
		/// The IBC channel identifier already exists.
		ChannelIdExist,
		/// The IBC port identifier doesn't match.
		PortIdNotMatch,
		/// The IBC connection is closed.
		ConnectionClosed,
		/// Only allow 1 hop for v1 of the IBC protocol.
		OnlyOneHopAllowedV1,
		/// The sequence sending packet not match
		PackedSequenceNotMatch,
		/// The destination channel identifier doesn't match
		DestChannelIdNotMatch,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn submit_datagram(origin: OriginFor<T>, datagram: Datagram) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			Self::handle_datagram(datagram)
		}

		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<informalsystems::Any>,
			tmp: u8,
		) -> DispatchResult {

			log::info!("in deliver");

			let _sender = ensure_signed(origin)?;
			let mut ctx = informalsystems::Context { _pd: PhantomData::<T>, tmp };
			let messages = messages
				.iter()
				.map(|message| prost_types::Any {
					type_url: message.type_url.clone(),
					value: message.value.clone(),
				})
				.collect();
			let result = ibc::ics26_routing::handler::deliver(&mut ctx, messages);

			log::info!("result: {:?}", result);

			Ok(())
		}
	}

	// The main implementation block for the module.
	impl<T: Config> Pallet<T> {
		/// Create an IBC client, by the 2 major steps:
		/// * Insert concensus state into storage "ConsensusStates"
		/// * Insert client state into storage "ClientStates"
		///
		/// Both storage's keys contains client id
		///
		/// # Example
		///
		/// ```ignore
		/// let identifier1 = Blake2Hasher::hash("appia".as_bytes());
		/// let identifier2 = Blake2Hasher::hash("flaminia".as_bytes());
		/// let height = 0;
		/// let consensus_state = ConsensusState {
		/// 	root: Blake2Hasher::hash("root".as_bytes()),
		///                height: 0,
		/// 	set_id: 0,
		/// 	authorities: vec![],
		/// };
		///
		/// assert_ok!(IbcModule::create_client(identifier1, ClientType::GRANDPA, height.clone(), consensus_state.clone()));
		/// ```
		pub fn create_client(
			client_id: H256,
			client_type: client::ClientType,
			height: u32,
			consensus_state: grandpa::consensus_state::ConsensusState,
		) -> DispatchResult {
			ensure!(!<ClientStates<T>>::contains_key(&client_id), Error::<T>::ClientIdExist);

			let client_state = match client_type {
				ClientType::GRANDPA => {
					grandpa::client_state::ClientState::new(client_id.clone(), height)
				}
				_ => grandpa::client_state::ClientState::new(client_id.clone(), height),
			};

			<ConsensusStates<T>>::insert((client_id, height), consensus_state);
			<ClientStates<T>>::insert(&client_id, client_state);
			// Todo: Persiste ClientType to substrate storage per ibc-spec

			Self::deposit_event(Event::ClientCreated);
			Ok(())
		}

		/// Initialize an IBC connection opening handshake.
		/// - Create a conneciton whose state is ```ConnectionState::Init```.
		/// - Insert the conneciton to storage ```Connections```.
		/// - Manipulate storage ```ClientStates``` by adding the connection id, e.g. Add "appia-connection", to the client id's connection list.
		///
		/// # Example
		///
		/// ```ignore
		///     let identifier = Blake2Hasher::hash("appia-connection".as_bytes());
		///     let desired_counterparty_connection_identifier =
		///         Blake2Hasher::hash("flaminia-connection".as_bytes());
		///     let client_id =
		///         hex::decode("53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10").unwrap();
		///     let client_id = H256::from_slice(&client_id);
		///     let counterparty_client_id =
		///         hex::decode("779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03").unwrap();
		///     let counterparty_client_id = H256::from_slice(&counterparty_client_id);
		///     conn_open_init(
		///         identifier,
		///         desired_counterparty_connection_identifier,
		///         client_id,
		///         counterparty_client_id
		///     );
		/// ```
		pub fn conn_open_init(
			connection_id: H256,
			counterparty_connection_id: H256,
			client_id: H256,
			counterparty_client_id: H256,
		) -> DispatchResult {
			// abortTransactionUnless(validateConnectionIdentifier(connection_id))
			ensure!(<ClientStates<T>>::contains_key(&client_id), Error::<T>::ClientIdNotExist);
			// TODO: ensure!(!client.connections.exists(&connection_id)))
			ensure!(!<Connections<T>>::contains_key(&connection_id), Error::<T>::ConnectionIdExist);
			let connection_end = ConnectionEnd {
				state: ConnectionState::Init,
				counterparty_connection_id,
				counterparty_prefix: vec![],
				client_id,
				counterparty_client_id,
				version: Self::get_compatible_versions(),
			};


			log::info!("connection inserted: {:?}", connection_id);

			<Connections<T>>::insert(&connection_id, connection_end);
			// addConnectionToClient(clientIdentifier, connection_id)
			<ClientStates<T>>::mutate(&client_id, |client_state| {
				(*client_state).connections.push(connection_id);
			});
			Self::deposit_event(Event::ConnOpenInit);
			Ok(())
		}

		/// Allocate a port, which modules can bind to uniquely named ports allocated by the IBC handler.
		///
		/// As the IBC spec "ics-005-port-allocation": Once a module has bound to a port, no other modules can use that port until the module releases it.
		///
		/// The restriction is implemented by binding a port to a module's index.
		///
		/// # Example
		/// ```ignore
		/// 	let identifier = "bank".as_bytes().to_vec();
		/// 	let module_index = 45 as u8;
		///     bind_port(identifier.clone(), module_index);
		/// ```
		pub fn bind_port(identifier: Vec<u8>, module_index: u8) -> DispatchResult {
			// abortTransactionUnless(validatePortIdentifier(id))
			ensure!(!<Ports<T>>::contains_key(&identifier), Error::<T>::PortIdBinded);
			<Ports<T>>::insert(&identifier, module_index);
			Self::deposit_event(Event::PortBound(module_index));
			Ok(())
		}

		pub fn release_port(identifier: Vec<u8>, module_index: u8) -> DispatchResult {
			#![warn(missing_docs)]
			ensure!(<Ports<T>>::get(&identifier) == module_index, "Port identifier not found");
			<Ports<T>>::remove(&identifier);
			Self::deposit_event(Event::PortReleased);
			Ok(())
		}

		/// Initialize an IBC channel opening handshake by:
		/// - Save a channel whose state is `ChannelState::Init` to the storage.
		/// - Guarantee the order of the packets by setting `NextSequenceSend`, `NextSequenceRecv`, and `NextSequenceAck` in the storage
		/// - Manipulate storage ```ClientStates``` by adding the (channel id, port id), e.g. Add "(CHANNEL_ID, PORT_ID)" to the client id's channel list.
		///
		/// # Example
		///
		/// ```ignore
		///     let module_index = 45 as u8;
		///     let order = ChannelOrder::Unordered;
		///     let connection_identifier =
		///         hex::decode("d93fc49e1b2087234a1e2fc204b500da5d16874e631e761bdab932b37907bd11").unwrap();
		///     let connection_identifier = H256::from_slice(&connection_identifier);
		///     let connection_hops = vec![connection_identifier];
		///     let port_identifier = "bank".as_bytes().to_vec();
		///     let channel_identifier = Blake2Hasher::hash(b"appia-channel");
		///     let counterparty_port_identifier = "bank".as_bytes().to_vec();
		///     let counterparty_channel_identifier = Blake2Hasher::hash(b"flaminia-channel");
		///     chan_open_init(
		///              module_index,
		///              order.clone(),
		///              connection_hops.clone(),
		///              port_identifier.clone(),
		///              channel_identifier,
		///              counterparty_port_identifier.clone(),
		///              counterparty_channel_identifier,
		///              vec![]
		///     );
		/// ```
		pub fn chan_open_init(
			module_index: u8,
			order: ChannelOrder,
			connection_hops: Vec<H256>,
			port_id: Vec<u8>,
			channel_id: H256,
			counterparty_port_id: Vec<u8>,
			counterparty_channel_id: H256,
			_version: Vec<u8>,
		) -> DispatchResult {
			// abortTransactionUnless(validateChannelIdentifier(portIdentifier, channelIdentifier))
			ensure!(connection_hops.len() == 1, Error::<T>::OnlyOneHopAllowedV1);

			ensure!(
				!<Channels<T>>::contains_key((port_id.clone(), channel_id)),
				Error::<T>::ChannelIdExist
			);
			ensure!(
				<Connections<T>>::contains_key(&connection_hops[0]),
				Error::<T>::ConnectionIdNotExist
			);

			// optimistic channel handshakes are allowed
			let connection = <Connections<T>>::get(&connection_hops[0]);
			ensure!(connection.state != ConnectionState::Closed, Error::<T>::ConnectionClosed);
			// abortTransactionUnless(authenticate(privateStore.get(portPath(portIdentifier))))
			ensure!(<Ports<T>>::get(&port_id) == module_index, Error::<T>::PortIdNotMatch);
			let channel_end = ChannelEnd {
				state: ChannelState::Init,
				ordering: order,
				counterparty_port_id,
				counterparty_channel_id,
				connection_hops,
				version: vec![],
			};
			<Channels<T>>::insert((port_id.clone(), channel_id), channel_end);
			// key = generate()
			// provableStore.set(channelCapabilityPath(portIdentifier, channelIdentifier), key)
			<NextSequenceSend<T>>::insert((port_id.clone(), channel_id), 1);
			<NextSequenceRecv<T>>::insert((port_id.clone(), channel_id), 1);
			<NextSequenceAck<T>>::insert((port_id.clone(), channel_id), 1);
			// return key
			<ClientStates<T>>::mutate(&connection.client_id, |client_state| {
				(*client_state).channels.push((port_id.clone(), channel_id));
			});
			Self::deposit_event(Event::ChanOpenInit);
			Ok(())
		}

		/// Initialize sending packet flow by:
		/// - Modify packet sequence by writting to storage `NextSequenceSend`.
		/// - Deposit a packet sending event.
		///
		/// # Example
		///
		/// ```ignore
		///     let sequence = 1;
		///     let timeout_height = 1000;
		///     let source_port = "bank".as_bytes().to_vec();
		///     let source_channel =
		///         hex::decode("00e2e14470ed9a017f586dfe6b76bb0871a8c91c3151778de110db3dfcc286ac").unwrap();
		///     let source_channel = H256::from_slice(&source_channel);
		///     let dest_port = "bank".as_bytes().to_vec();
		///     let dest_channel =
		///         hex::decode("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
		///     let dest_channel = H256::from_slice(&dest_channel);
		///     let data: Vec<u8> = hex::decode("01020304").unwrap();
		///
		///     let mut packet = Packet {
		///         sequence,
		///         timeout_height,
		///         source_port,
		///         source_channel,
		///         dest_port,
		///         dest_channel,
		///         data,
		///     };
		///     send_packet(packet.clone());
		/// ```
		///
		pub fn send_packet(packet: Packet) -> DispatchResult {
			let channel = <Channels<T>>::get((packet.source_port.clone(), packet.source_channel));
			// optimistic sends are permitted once the handshake has started
			ensure!(channel.state != ChannelState::Closed, "channel has been closed");

			// abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(packet.sourcePort, packet.sourceChannel))))
			ensure!(packet.dest_port == channel.counterparty_port_id, Error::<T>::PortIdNotMatch);
			ensure!(
				packet.dest_channel == channel.counterparty_channel_id,
				Error::<T>::DestChannelIdNotMatch
			);
			let connection = <Connections<T>>::get(&channel.connection_hops[0]);
			ensure!(connection.state != ConnectionState::Closed, "connection has been closed");

			// consensusState = provableStore.get(consensusStatePath(connection.clientIdentifier))
			// abortTransactionUnless(consensusState.getHeight() < packet.timeoutHeight)

			let mut next_sequence_send =
				<NextSequenceSend<T>>::get((packet.source_port.clone(), packet.source_channel));
			ensure!(packet.sequence == next_sequence_send, Error::<T>::PackedSequenceNotMatch);

			// all assertions passed, we can alter state
			next_sequence_send = next_sequence_send + 1;
			<NextSequenceSend<T>>::insert(
				(packet.source_port.clone(), packet.source_channel),
				next_sequence_send,
			);
			let timeout_height = packet.timeout_height.encode();
			let hash = BlakeTwo256::hash_of(&[&packet.data[..], &timeout_height[..]].concat());

			<Packets<T>>::insert(
				(packet.source_port.clone(), packet.source_channel, packet.sequence),
				hash,
			);
			// provableStore.set(packetCommitmentPath(packet.sourcePort, packet.sourceChannel, packet.sequence), hash(packet.data, packet.timeout))

			// log that a packet has been sent
			Self::deposit_event(Event::SendPacket(
				packet.sequence,
				packet.data,
				packet.timeout_height,
				packet.source_port,
				packet.source_channel,
				packet.dest_port,
				packet.dest_channel,
			));
			Ok(())
		}

		/// This function handles datagram, transmitted from relayers, for the kinds task below:
		///     + Synchronizing block headers from other chains.
		///     + After connection opening handshakes are initiated, processing the subsequent handshakes - ICS-003.
		///     + After channel opening handshakes are initiated, processing the subsequent handshakes - ICS-004.
		///     + After packet flows are initiated, processing the subsequent packet flows - ICS-004.
		pub fn handle_datagram(datagram: Datagram) -> DispatchResult {
			#![warn(missing_doc_code_examples)]
			match datagram {
				// Receiving the message containing a block header of other chains from relayers,  IBC module tryies to synchronize the block header.
				Datagram::ClientUpdate { client_id, header } => {
					ensure!(<ClientStates<T>>::contains_key(&client_id), "Client not found");
					let client_state = <ClientStates<T>>::get(&client_id);
					ensure!(client_state.latest_height < header.height, "Client already updated");
					ensure!(
						<ConsensusStates<T>>::contains_key((client_id, client_state.latest_height)),
						"ConsensusState not found"
					);
					let consensus_state =
						<ConsensusStates<T>>::get((client_id, client_state.latest_height));

					// TODO: verify header using validity_predicate
					// let justification =
					// 	GrandpaJustification::<Block>::decode(&mut &*header.justification);
					// TODO: julian
					let justification: Result<GrandpaJustification<Block>, u32> = Err(0);


					log::info!(
							"consensus_state: {:?}, header: {:?}",
							consensus_state,
							header,
						);

					let authorities = VoterSet::new(consensus_state.authorities.iter().cloned());
					ensure!(authorities.is_some(), "Invalid authorities set");
					let authorities = authorities.unwrap();
					if let Ok(justification) = justification {
						let result = justification.verify(consensus_state.set_id, &authorities);

						log::info!("verify result: {:?}", result);

						if result.is_ok() {

							log::info!("block_hash: {:?}", header.block_hash);

							assert_eq!(header.block_hash, justification.commit.target_hash);
							<ClientStates<T>>::mutate(&client_id, |client_state| {
								(*client_state).latest_height = header.height;
							});
							// TODO
							let new_consensus_state = grandpa::consensus_state::ConsensusState {
								root: header.commitment_root,
								height: header.height,
								set_id: consensus_state.set_id,
								authorities: consensus_state.authorities.clone(),
							};

							log::info!(
									"consensus_state inserted: {:?}, {}",
									client_id,
									header.height
								);

							<ConsensusStates<T>>::insert(
								(client_id, header.height),
								new_consensus_state,
							);

							let result = read_proof_check::<BlakeTwo256>(
								header.commitment_root,
								header.authorities_proof,
								&GRANDPA_AUTHORITIES_KEY.to_vec(),
							);
							// TODO
							let result = result.unwrap().unwrap();
							let new_authorities: AuthorityList =
								VersionedAuthorityList::decode(&mut &*result).unwrap().into();

							log::info!("new_authorities: {:?}", new_authorities);

							if new_authorities != consensus_state.authorities {
								<ConsensusStates<T>>::mutate(
									(client_id, header.height),
									|consensus_state| {
										(*consensus_state).set_id += 1;
										(*consensus_state).authorities = new_authorities;
									},
								);
							}
							Self::deposit_event(Event::ClientUpdated);
						}
					}
				}
				Datagram::ClientMisbehaviour { identifier : _, evidence: _ } => {
					Self::deposit_event(Event::ClientMisbehaviourReceived);
				}
				Datagram::ConnOpenTry {
					connection_id,
					counterparty_connection_id,
					counterparty_client_id,
					client_id,
					version: _,
					counterparty_version,
					proof_init,
					proof_consensus: _,
					proof_height,
					consensus_height: _,
				} => {
					let mut new_connection_end;
					if <Connections<T>>::contains_key(&connection_id) {
						let old_conn_end = <Connections<T>>::get(&connection_id);
						let state_is_consistent = old_conn_end.state.eq(&ConnectionState::Init)
							&& old_conn_end
								.counterparty_connection_id
								.eq(&counterparty_connection_id)
							&& old_conn_end.counterparty_client_id.eq(&counterparty_client_id);

						ensure!(state_is_consistent, "Local connection corrupted!");

						new_connection_end = old_conn_end.clone();
					} else {
						new_connection_end = ConnectionEnd {
							state: ConnectionState::Init,
							counterparty_connection_id,
							counterparty_prefix: vec![],
							client_id,
							counterparty_client_id,
							version: vec![],
						};
					}

					// abortTransactionUnless(validateConnectionIdentifier(desiredIdentifier))
					// abortTransactionUnless(consensusHeight <= getCurrentHeight())
					// expectedConsensusState = getConsensusState(consensusHeight)
					// expected = ConnectionEnd{INIT, desiredIdentifier, getCommitmentPrefix(), counterpartyClientIdentifier,
					//                          clientIdentifier, counterpartyVersions}
					// version = pickVersion(counterpartyVersions)
					log::info!(
							"query consensus_state: {:?}, {}",
							client_id,
							proof_height
						);

					ensure!(
						<ConsensusStates<T>>::contains_key((client_id, proof_height)),
						"ConsensusState not found"
					);
					let value = Self::verify_connection_state(
						client_id,
						proof_height,
						counterparty_connection_id,
						proof_init,
					);
					ensure!(value.is_some(), "verify connection state failed");
					// abortTransactionUnless(connection.verifyConnectionState(proofHeight, proofInit, counterpartyConnectionIdentifier, expected))
					// abortTransactionUnless(connection.verifyClientConsensusState(proofHeight, proofConsensus, counterpartyClientIdentifier, expectedConsensusState))
					// previous = provableStore.get(connectionPath(desiredIdentifier))
					// abortTransactionUnless(
					//   (previous === null) ||
					//   (previous.state === INIT &&
					//     previous.counterpartyConnectionIdentifier === counterpartyConnectionIdentifier &&
					//     previous.counterpartyPrefix === counterpartyPrefix &&
					//     previous.clientIdentifier === clientIdentifier &&
					//     previous.counterpartyClientIdentifier === counterpartyClientIdentifier &&
					//     previous.version === version))

					new_connection_end.state = ConnectionState::TryOpen;

					// Pick the version.
					let local_versions = Self::get_compatible_versions();
					let intersection: Vec<u8> = counterparty_version
						.iter()
						.filter(|cv| local_versions.contains(cv))
						.cloned()
						.collect();
					new_connection_end.version = vec![Self::pick_version(intersection)]; // Todo: change the field `version` in `new_connection_end` to `u8`

					let identifier = connection_id;
					<Connections<T>>::insert(&identifier, new_connection_end);
					// addConnectionToClient(clientIdentifier, identifier)
					<ClientStates<T>>::mutate(&client_id, |client_state| {
						(*client_state).connections.push(identifier);
					});
					Self::deposit_event(Event::ConnOpenTry);
				}
				Datagram::ConnOpenAck {
					connection_id,
					counterparty_connection_id: _,
					version,
					proof_try,
					proof_consensus: _,
					proof_height,
					consensus_height,
				} => {
					use sp_runtime::traits::SaturatedConversion;
					let current_block_number_self =
						<frame_system::Pallet<T>>::block_number().saturated_into::<u32>();
					Self::check_client_consensus_height(
						current_block_number_self,
						consensus_height,
					)?;

					ensure!(
						<Connections<T>>::contains_key(&connection_id),
						"Connection uninitialized"
					);

					// let mut new_connection_end;
					// {
					//     let old_conn_end = Connections::get(&connection_id);
					//     let state_is_consistent = old_conn_end.state.eq(&ConnectionState::Init)
					//             && old_conn_end.version.contains(&version)
					//         || old_conn_end.state.eq(&ConnectionState::TryOpen)
					//             && (old_conn_end.version.get(0) == Some(&version));

					//     // Check that if the msg's counterparty connection id is not empty then it matches
					//     // the old connection's counterparty.
					//     // Todo: Ensure connecion id is not empty?
					//     let counterparty_matches= old_conn_end.counterparty_connection_id == counterparty_connection_id;

					//     ensure!(state_is_consistent && counterparty_matches, "Connection mismatch!");

					//     new_connection_end = old_conn_end.clone();
					// }
					let mut new_connection_end = <Connections<T>>::get(&connection_id);

					// expectedConsensusState = getConsensusState(consensusHeight)
					// expected = ConnectionEnd{TRYOPEN, identifier, getCommitmentPrefix(),
					//                          connection.counterpartyClientIdentifier, connection.clientIdentifier,
					//                          version}
					ensure!(
						<ConsensusStates<T>>::contains_key((
							new_connection_end.client_id,
							proof_height
						)),
						"ConsensusState not found"
					);
					let value = Self::verify_connection_state(
						new_connection_end.client_id,
						proof_height,
						new_connection_end.counterparty_connection_id,
						proof_try,
					);
					ensure!(value.is_some(), "verify connection state failed");
					// abortTransactionUnless(connection.verifyConnectionState(proofHeight, proofTry, connection.counterpartyConnectionIdentifier, expected))
					// abortTransactionUnless(connection.verifyClientConsensusState(proofHeight, proofConsensus, connection.counterpartyClientIdentifier, expectedConsensusState))
					new_connection_end.version = vec![version];
					<Connections<T>>::mutate(&connection_id, |connection| {
						(*connection).state = ConnectionState::Open;
					});
					// abortTransactionUnless(getCompatibleVersions().indexOf(version) !== -1)
					// connection.version = version
					// provableStore.set(connectionPath(identifier), connection)
					Self::deposit_event(Event::ConnOpenAck);
				}
				Datagram::ConnOpenConfirm { connection_id, proof_ack, proof_height } => {
					ensure!(
						<Connections<T>>::contains_key(&connection_id),
						"Connection uninitialized"
					);

					let new_connection_end;
					{
						let old_conn_end = <Connections<T>>::get(&connection_id);
						ensure!(
							old_conn_end.state.eq(&ConnectionState::TryOpen),
							"Connection mismatch!"
						);
						new_connection_end = old_conn_end.clone();
					}

					ensure!(
						<ConsensusStates<T>>::contains_key((
							new_connection_end.client_id,
							proof_height
						)),
						"ConsensusState not found"
					);
					let value = Self::verify_connection_state(
						new_connection_end.client_id,
						proof_height,
						new_connection_end.counterparty_connection_id,
						proof_ack,
					);
					ensure!(value.is_some(), "verify connection state failed");
					// expected = ConnectionEnd{OPEN, identifier, getCommitmentPrefix(), connection.counterpartyClientIdentifier,
					//                          connection.clientIdentifier, connection.version}
					// abortTransactionUnless(connection.verifyConnectionState(proofHeight, proofAck, connection.counterpartyConnectionIdentifier, expected))
					<Connections<T>>::mutate(&connection_id, |connection| {
						(*connection).state = ConnectionState::Open;
					});
					// provableStore.set(connectionPath(identifier), connection)
					Self::deposit_event(Event::ConnOpenConfirm);
				}
				Datagram::ChanOpenTry {
					order,
					connection_hops,
					port_id,
					channel_id,
					counterparty_port_id,
					counterparty_channel_id,
					channel_version: version,
					counterparty_version,
					proof_init,
					proof_height,
				} => {
					// abortTransactionUnless(validateChannelIdentifier(portIdentifier, channelIdentifier))
					ensure!(
						connection_hops.len() == 1,
						"only allow 1 hop for v1 of the IBC protocol"
					);
					// ???
					// previous = provableStore.get(channelPath(portIdentifier, channelIdentifier))
					// abortTransactionUnless(
					//   (previous === null) ||
					//   (previous.state === INIT &&
					//    previous.order === order &&
					//    previous.counterpartyPortIdentifier === counterpartyPortIdentifier &&
					//    previous.counterpartyChannelIdentifier === counterpartyChannelIdentifier &&
					//    previous.connectionHops === connectionHops &&
					//    previous.version === version)
					//   )
					ensure!(
						!<Channels<T>>::contains_key((port_id.clone(), channel_id)),
						"channel identifier already exists"
					);
					// abortTransactionUnless(authenticate(privateStore.get(portPath(portIdentifier))))
					ensure!(
						<Connections<T>>::contains_key(&connection_hops[0]),
						"connection identifier not exists"
					);
					let connection = <Connections<T>>::get(&connection_hops[0]);
					ensure!(
						connection.state == ConnectionState::Open,
						"connection has been closed"
					);

					ensure!(
						<ConsensusStates<T>>::contains_key((connection.client_id, proof_height)),
						"ConsensusState not found"
					);
					let value = Self::verify_channel_state(
						connection.client_id,
						proof_height,
						counterparty_port_id.clone(),
						counterparty_channel_id,
						proof_init,
					);
					ensure!(value.is_some(), "verify channel state failed");
					// expected = ChannelEnd{INIT, order, portIdentifier,
					//                       channelIdentifier, connectionHops.reverse(), counterpartyVersion}
					// abortTransactionUnless(connection.verifyChannelState(
					//   proofHeight,
					//   proofInit,
					//   counterpartyPortIdentifier,
					//   counterpartyChannelIdentifier,
					//   expected
					// ))
					let dest_module_index = <Ports<T>>::get(port_id.clone());
					T::ModuleCallbacks::on_chan_open_try(
						dest_module_index.into(),
						order.clone(),
						connection_hops.clone(),
						port_id.clone(),
						channel_id,
						counterparty_port_id.clone(),
						counterparty_channel_id,
						version.clone(),
						counterparty_version,
					);

					let channel_end = ChannelEnd {
						state: ChannelState::TryOpen,
						ordering: order,
						counterparty_port_id,
						counterparty_channel_id,
						connection_hops,
						version,
					};
					<Channels<T>>::insert((port_id.clone(), channel_id), channel_end);
					// key = generate()
					// provableStore.set(channelCapabilityPath(portIdentifier, channelIdentifier), key)
					<NextSequenceSend<T>>::insert((port_id.clone(), channel_id), 1);
					<NextSequenceRecv<T>>::insert((port_id.clone(), channel_id), 1);
					// return key
					<ClientStates<T>>::mutate(&connection.client_id, |client_state| {
						(*client_state).channels.push((port_id.clone(), channel_id));
					});
					Self::deposit_event(Event::ChanOpenTry);
				}
				Datagram::ChanOpenAck {
					port_id,
					channel_id,
					version,
					proof_try,
					proof_height,
				} => {
					ensure!(
						<Channels<T>>::contains_key((port_id.clone(), channel_id)),
						"channel identifier not exists"
					);
					let channel = <Channels<T>>::get((port_id.clone(), channel_id));
					ensure!(
						channel.state == ChannelState::Init
							|| channel.state == ChannelState::TryOpen,
						"channel is not ready"
					);
					// abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(portIdentifier, channelIdentifier))))
					ensure!(
						<Connections<T>>::contains_key(&channel.connection_hops[0]),
						"connection identifier not exists"
					);
					let connection = <Connections<T>>::get(&channel.connection_hops[0]);
					ensure!(
						connection.state == ConnectionState::Open,
						"connection has been closed"
					);
					ensure!(
						<ConsensusStates<T>>::contains_key((connection.client_id, proof_height)),
						"ConsensusState not found"
					);
					let value = Self::verify_channel_state(
						connection.client_id,
						proof_height,
						channel.counterparty_port_id,
						channel.counterparty_channel_id,
						proof_try,
					);
					ensure!(value.is_some(), "verify channel state failed");
					// expected = ChannelEnd{TRYOPEN, channel.order, portIdentifier,
					//                       channelIdentifier, channel.connectionHops.reverse(), counterpartyVersion}
					// abortTransactionUnless(connection.verifyChannelState(
					//   proofHeight,
					//   proofTry,
					//   channel.counterpartyPortIdentifier,
					//   channel.counterpartyChannelIdentifier,
					//   expected
					// ))
					// channel.version = counterpartyVersion
					let dest_module_index = <Ports<T>>::get(port_id.clone());
					T::ModuleCallbacks::on_chan_open_ack(
						dest_module_index.into(),
						port_id.clone(),
						channel_id,
						version.clone(),
					);
					<Channels<T>>::mutate((port_id, channel_id), |channel| {
						(*channel).state = ChannelState::Open;
					});
					Self::deposit_event(Event::ChanOpenAck);
				}
				Datagram::ChanOpenConfirm {
					port_id,
					channel_id,
					proof_ack,
					proof_height,
				} => {
					ensure!(
						<Channels<T>>::contains_key((port_id.clone(), channel_id)),
						"channel identifier not exists"
					);
					let channel = <Channels<T>>::get((port_id.clone(), channel_id));
					ensure!(channel.state == ChannelState::TryOpen, "channel is not ready");
					// abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(portIdentifier, channelIdentifier))))
					ensure!(
						<Connections<T>>::contains_key(&channel.connection_hops[0]),
						"connection identifier not exists"
					);
					let connection = <Connections<T>>::get(&channel.connection_hops[0]);
					ensure!(
						connection.state == ConnectionState::Open,
						"connection has been closed"
					);
					ensure!(
						<ConsensusStates<T>>::contains_key((connection.client_id, proof_height)),
						"ConsensusState not found"
					);
					let value = Self::verify_channel_state(
						connection.client_id,
						proof_height,
						channel.counterparty_port_id,
						channel.counterparty_channel_id,
						proof_ack,
					);
					ensure!(value.is_some(), "verify channel state failed");
					// expected = ChannelEnd{OPEN, channel.order, portIdentifier,
					//                       channelIdentifier, channel.connectionHops.reverse(), channel.version}
					// abortTransactionUnless(connection.verifyChannelState(
					//   proofHeight,
					//   proofAck,
					//   channel.counterpartyPortIdentifier,
					//   channel.counterpartyChannelIdentifier,
					//   expected
					// ))
					let dest_module_index = <Ports<T>>::get(port_id.clone());
					T::ModuleCallbacks::on_chan_open_confirm(
						dest_module_index.into(),
						port_id.clone(),
						channel_id,
					);
					<Channels<T>>::mutate((port_id, channel_id), |channel| {
						(*channel).state = ChannelState::Open;
					});
					Self::deposit_event(Event::ChanOpenConfirm);
				}
				Datagram::PacketRecv { packet, proof, proof_height } => {
					ensure!(
						<Channels<T>>::contains_key((
							packet.dest_port.clone(),
							packet.dest_channel
						)),
						"channel identifier not exists"
					);
					let channel =
						<Channels<T>>::get((packet.dest_port.clone(), packet.dest_channel));
					ensure!(channel.state == ChannelState::Open, "channel is not ready");
					// abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(packet.destPort, packet.destChannel))))
					ensure!(
						packet.source_port == channel.counterparty_port_id,
						"source port not match"
					);
					ensure!(
						packet.source_channel == channel.counterparty_channel_id,
						"source channel not match"
					);

					ensure!(
						<Connections<T>>::contains_key(&channel.connection_hops[0]),
						"connection identifier not exists"
					);
					let connection = <Connections<T>>::get(&channel.connection_hops[0]);
					ensure!(
						connection.state == ConnectionState::Open,
						"connection has been closed"
					);

					// abortTransactionUnless(getConsensusHeight() < packet.timeoutHeight)

					ensure!(
						<ConsensusStates<T>>::contains_key((connection.client_id, proof_height)),
						"ConsensusState not found"
					);
					let value = Self::verify_packet_data(
						connection.client_id,
						proof_height,
						proof,
						packet.source_port.clone(),
						packet.source_channel,
						packet.sequence,
					);
					ensure!(value.is_some(), "verify packet data failed");
					let timeout_height = packet.timeout_height.encode();
					let hash =
						BlakeTwo256::hash_of(&[&packet.data[..], &timeout_height[..]].concat());
					ensure!(value.unwrap() == hash, "packet hash not match");
					// abortTransactionUnless(connection.verifyPacketCommitment(
					//   proofHeight,
					//   proof,
					//   packet.sourcePort,
					//   packet.sourceChannel,
					//   packet.sequence,
					//   hash(packet.data, packet.timeout)
					// ))

					// all assertions passed (except sequence check), we can alter state

					// for testing
					let dest_module_index = <Ports<T>>::get(packet.dest_port.clone());

					log::info!("dest_module_index: {}", dest_module_index);

					T::ModuleCallbacks::on_recv_packet(dest_module_index.into(), packet.clone());
					let acknowledgement: Vec<u8> = vec![1, 3, 3, 7];

					if acknowledgement.len() > 0 || channel.ordering == ChannelOrder::Unordered {
						let hash = BlakeTwo256::hash_of(&acknowledgement);

						<Acknowledgements<T>>::insert(
							(packet.dest_port.clone(), packet.dest_channel, packet.sequence),
							hash,
						);
					}

					if channel.ordering == ChannelOrder::Ordered {
						let mut next_sequence_recv = <NextSequenceRecv<T>>::get((
							packet.dest_port.clone(),
							packet.dest_channel,
						));
						ensure!(packet.sequence == next_sequence_recv, "recv sequence not match");
						next_sequence_recv = next_sequence_recv + 1;
						<NextSequenceRecv<T>>::insert(
							(packet.dest_port.clone(), packet.dest_channel),
							next_sequence_recv,
						);
					}

					// log that a packet has been received & acknowledged
					// emitLogEntry("recvPacket", {sequence: packet.sequence, timeout: packet.timeout, data: packet.data, acknowledgement})
					Self::deposit_event(Event::RecvPacket(
						packet.sequence,
						packet.data,
						packet.timeout_height,
						packet.source_port,
						packet.source_channel,
						packet.dest_port,
						packet.dest_channel,
						acknowledgement,
					));

					// return transparent packet
					// return packet
				}
				Datagram::PacketAcknowledgement {
					packet,
					acknowledgement,
					proof,
					proof_height,
				} => {
					// abort transaction unless that channel is open, calling module owns the associated port, and the packet fields match
					ensure!(
						<Channels<T>>::contains_key((
							packet.source_port.clone(),
							packet.source_channel
						)),
						"channel identifier not exists"
					);
					let channel =
						<Channels<T>>::get((packet.source_port.clone(), packet.source_channel));
					ensure!(channel.state == ChannelState::Open, "channel is not ready");
					// abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(packet.sourcePort, packet.sourceChannel))))
					ensure!(
						packet.dest_channel == channel.counterparty_channel_id,
						"dest channel not match"
					);

					ensure!(
						<Connections<T>>::contains_key(&channel.connection_hops[0]),
						"connection identifier not exists"
					);
					let connection = <Connections<T>>::get(&channel.connection_hops[0]);
					ensure!(
						connection.state == ConnectionState::Open,
						"connection has been closed"
					);
					ensure!(
						packet.dest_port == channel.counterparty_port_id,
						"dest port not match"
					);

					// verify we sent the packet and haven't cleared it out yet
					// abortTransactionUnless(provableStore.get(packetCommitmentPath(packet.sourcePort, packet.sourceChannel, packet.sequence))
					//        === hash(packet.data, packet.timeout))
					let timeout_height = packet.timeout_height.encode();
					let expect_hash =
						BlakeTwo256::hash_of(&[&packet.data[..], &timeout_height[..]].concat());

					let hash = <Packets<T>>::get((
						packet.source_port.clone(),
						packet.source_channel,
						packet.sequence,
					));
					ensure!(expect_hash == hash, "packet hash not match");

					// abort transaction unless correct acknowledgement on counterparty chain
					// abortTransactionUnless(connection.verifyPacketAcknowledgement(
					//   proofHeight,
					//   proof,
					//   packet.destPort,
					//   packet.destChannel,
					//   packet.sequence,
					//   hash(acknowledgement)
					// ))
					let value = Self::verify_packet_acknowledgement(
						connection.client_id,
						proof_height,
						proof,
						packet.dest_port.clone(),
						packet.dest_channel,
						packet.sequence,
					);
					ensure!(value.is_some(), "verify packet acknowledgement failed");
					let hash = BlakeTwo256::hash_of(&acknowledgement);
					ensure!(value.unwrap() == hash, "packet acknowledgement hash not match");

					// abort transaction unless acknowledgement is processed in order
					if channel.ordering == ChannelOrder::Ordered {
						let mut next_sequence_ack = <NextSequenceAck<T>>::get((
							packet.dest_port.clone(),
							packet.dest_channel,
						));
						ensure!(packet.sequence == next_sequence_ack, "recv sequence not match");
						next_sequence_ack = next_sequence_ack + 1;
						<NextSequenceAck<T>>::insert(
							(packet.dest_port.clone(), packet.dest_channel),
							next_sequence_ack,
						);
					}

					// all assertions passed, we can alter state

					// delete our commitment so we can't "acknowledge" again
					<Acknowledgements<T>>::remove((
						packet.dest_port.clone(),
						packet.dest_channel,
						packet.sequence,
					));

					// return transparent packet
					// return packet
				}
			}
			Ok(())
		}

		// TODO
		fn verify_client_consensus_state() {
			unimplemented!()
		}

		fn verify_connection_state(
			client_id: H256,
			proof_height: u32,
			connection_identifier: H256,
			proof: StorageProof,
		) -> Option<ConnectionEnd> {
			let consensus_state = <ConsensusStates<T>>::get((client_id, proof_height));
			let key = <Connections<T>>::hashed_key_for(connection_identifier);
			let value = read_proof_check::<BlakeTwo256>(consensus_state.root, proof, &key);
			match value {
				Ok(value) => match value {
					Some(value) => {
						let connection_end = ConnectionEnd::decode(&mut &*value);
						match connection_end {
							Ok(connection_end) => {
								return Some(connection_end);
							}
							Err(error) => {

								log::info!("trie value decode error: {:?}", error);

							}
						}
					}
					None => {

						log::info!("read_proof_check error: value not exists");

					}
				},
				Err(error) => {

					log::info!("read_proof_check error: {:?}", error);

				}
			}

			None
		}

		fn verify_channel_state(
			client_id: H256,
			proof_height: u32,
			port_identifier: Vec<u8>,
			channel_identifier: H256,
			proof: StorageProof,
		) -> Option<ChannelEnd> {
			let consensus_state = <ConsensusStates<T>>::get((client_id, proof_height));
			let key = <Channels<T>>::hashed_key_for((port_identifier, channel_identifier));
			let value = read_proof_check::<BlakeTwo256>(consensus_state.root, proof, &key);
			match value {
				Ok(value) => match value {
					Some(value) => {
						let channel_end = ChannelEnd::decode(&mut &*value);
						match channel_end {
							Ok(channel_end) => {
								return Some(channel_end);
							}
							Err(error) => {
								log::info!("trie value decode error: {:?}", error);
							}
						}
					}
					None => {

						log::info!("read_proof_check error: value not exists");

					}
				},
				Err(error) => {

					log::info!("read_proof_check error: {:?}", error);

				}
			}

			None
		}

		fn verify_packet_data(
			client_id: H256,
			proof_height: u32,
			proof: StorageProof,
			port_identifier: Vec<u8>,
			channel_identifier: H256,
			sequence: u64,
		) -> Option<H256> {
			let consensus_state = <ConsensusStates<T>>::get((client_id, proof_height));
			let key = <Packets<T>>::hashed_key_for((port_identifier, channel_identifier, sequence));
			let value = read_proof_check::<BlakeTwo256>(consensus_state.root, proof, &key);
			match value {
				Ok(value) => match value {
					Some(value) => {
						let hash = H256::decode(&mut &*value);
						match hash {
							Ok(hash) => {
								return Some(hash);
							}
							Err(error) => {

								log::info!("trie value decode error: {:?}", error);

							}
						}
					}
					None => {
						log::info!("read_proof_check error: value not exists");

					}
				},
				Err(error) => {

					log::info!("read_proof_check error: {:?}", error);

				}
			}

			None
		}

		fn verify_packet_acknowledgement(
			client_id: H256,
			proof_height: u32,
			proof: StorageProof,
			port_identifier: Vec<u8>,
			channel_identifier: H256,
			sequence: u64,
		) -> Option<H256> {
			let consensus_state = <ConsensusStates<T>>::get((client_id, proof_height));
			let key = <Acknowledgements<T>>::hashed_key_for((
				port_identifier,
				channel_identifier,
				sequence,
			));
			let value = read_proof_check::<BlakeTwo256>(consensus_state.root, proof, &key);
			match value {
				Ok(value) => match value {
					Some(value) => {
						let hash = H256::decode(&mut &*value);
						match hash {
							Ok(hash) => {
								return Some(hash);
							}
							Err(error) => {
								log::info!("trie value decode error: {:?}", error);
							}
						}
					}
					None => {
						log::info!("read_proof_check error: value not exists");
					}
				},
				Err(error) => {

					log::info!("read_proof_check error: {:?}", error);

				}
			}

			None
		}

		fn get_compatible_versions() -> Vec<u8> {
			VERSIONS.to_vec().clone()
		}

		fn pick_version(candidate_versions: Vec<u8>) -> u8 {
			*candidate_versions.get(0).unwrap()
		}

		fn check_client_consensus_height(
			host_current_height: u32,
			claimed_height: u32,
		) -> DispatchResult {
			// Todo: Use Height struct as ibc-rs/modules/src/ics02_client/height.rs?

			ensure!(claimed_height > host_current_height, "Consensus height is too advanced!");

			ensure!(
				claimed_height < host_current_height - MAX_HISTORY_SIZE,
				"Consensus height is too old!"
			);

			Ok(())
		}
	}
}
