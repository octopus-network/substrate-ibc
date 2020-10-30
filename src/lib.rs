#![cfg_attr(not(feature = "std"), no_std)]

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

use codec::{Decode, Encode};
use finality_grandpa::voter_set::VoterSet;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, traits::Get};
use frame_system::ensure_signed;
use sp_core::H256;
use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, Hash},
    OpaqueExtrinsic as UncheckedExtrinsic, RuntimeDebug,
};
use sp_std::{if_std, prelude::*};
use sp_trie::StorageProof;
use grandpa::state_machine::read_proof_check;
use grandpa::justification::GrandpaJustification;

pub use client::ClientType;
pub use routing::ModuleCallbacks;

mod client;
pub mod grandpa;
mod handler;
mod header;
mod routing;
mod state;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
        identifier: H256,
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
        version: Vec<u8>,  // Todo: remove this field
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
        port_identifier: Vec<u8>,
        channel_identifier: H256,
        counterparty_port_identifier: Vec<u8>,
        counterparty_channel_identifier: H256,
        version: Vec<u8>,
        counterparty_version: Vec<u8>,
        proof_init: StorageProof,
        proof_height: u32,
    },
    ChanOpenAck {
        port_identifier: Vec<u8>,
        channel_identifier: H256,
        version: Vec<u8>,
        proof_try: StorageProof,
        proof_height: u32,
    },
    ChanOpenConfirm {
        port_identifier: Vec<u8>,
        channel_identifier: H256,
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
    client_id: H256,
    counterparty_client_id: H256,
    version: Vec<u8>,      // TODO: A ConnectionEnd should only store one version.
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
    pub counterparty_port_identifier: Vec<u8>,
    pub counterparty_channel_identifier: H256,
    pub connection_hops: Vec<H256>,
    pub version: Vec<u8>,
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type ModuleCallbacks: routing::ModuleCallbacks;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	trait Store for Module<T: Trait> as Ibc {
		ClientStates: map hasher(blake2_128_concat) H256 => grandpa::client_state::ClientState; // client_id => ClientState
		ConsensusStates: map hasher(blake2_128_concat) (H256, u32) => grandpa::consensus_state::ConsensusState; // (client_id, height) => ConsensusState
		Connections: map hasher(blake2_128_concat) H256 => ConnectionEnd; // connection_identifier => ConnectionEnd
		Ports: map hasher(blake2_128_concat) Vec<u8> => u8; // port_identifier => module_index
		/// Channel structures are stored under a store path prefix unique to a combination of a port identifier and channel identifier.
		Channels: map hasher(blake2_128_concat) (Vec<u8>, H256) => ChannelEnd; // (port_identifier, channel_identifier) => ChannelEnd
		NextSequenceSend: map hasher(blake2_128_concat) (Vec<u8>, H256) => u64; // (port_identifier, channel_identifier) => Sequence
		NextSequenceRecv: map hasher(blake2_128_concat) (Vec<u8>, H256) => u64; // (port_identifier, channel_identifier) => Sequence
		NextSequenceAck: map hasher(blake2_128_concat) (Vec<u8>, H256) => u64; // (port_identifier, channel_identifier) => Sequence
		Packets: map hasher(blake2_128_concat) (Vec<u8>, H256, u64) => H256; // (port_identifier, channel_identifier, sequence) => Hash
		Acknowledgements: map hasher(blake2_128_concat) (Vec<u8>, H256, u64) => H256; // (port_identifier, channel_identifier, sequence) => Hash
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
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
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
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
		DestChannelIdNotMatch
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    /// The struct defines the major functions for the module.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 0]
		fn submit_datagram(origin, datagram: Datagram) -> dispatch::DispatchResult {
			let _sender = ensure_signed(origin)?;
			Self::handle_datagram(datagram)
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			Ok(())
		}
	}
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
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
        identifier: H256,
        client_type: client::ClientType,
        height: u32,
        consensus_state: grandpa::consensus_state::ConsensusState,
    ) -> dispatch::DispatchResult {
        ensure!(
            !ClientStates::contains_key(&identifier),
            Error::<T>::ClientIdExist
        );

        let client_state = match client_type {
            ClientType::GRANDPA => {
                grandpa::client_state::ClientState::new(
                    identifier.clone(),
                    height,
                )
            }
            _ => {
                grandpa::client_state::ClientState::new(
                    identifier.clone(),
                    height,
                )
            }
        };
        ConsensusStates::insert((identifier, height), consensus_state);
        ClientStates::insert(&identifier, client_state);

        Self::deposit_event(RawEvent::ClientCreated);
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
        counterparty_client_id: H256
    ) -> dispatch::DispatchResult {
        // abortTransactionUnless(validateConnectionIdentifier(connection_id))
        ensure!(
            ClientStates::contains_key(&client_id),
            Error::<T>::ClientIdNotExist
        );
        // TODO: ensure!(!client.connections.exists(&connection_id)))
        ensure!(
            !Connections::contains_key(&connection_id),
            Error::<T>::ConnectionIdExist
        );
        let connection_end = ConnectionEnd {
            state: ConnectionState::Init,
            counterparty_connection_id,
            counterparty_prefix: vec![],
            client_id,
            counterparty_client_id,
            version: Self::get_compatible_versions()
        };

        if_std! {
            println!("connection inserted: {:?}", connection_id);
        }
        Connections::insert(&connection_id, connection_end);
        // addConnectionToClient(clientIdentifier, connection_id)
        ClientStates::mutate(&client_id, |client_state| {
            (*client_state).connections.push(connection_id);
        });
        Self::deposit_event(RawEvent::ConnOpenInit);
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
    pub fn bind_port(identifier: Vec<u8>, module_index: u8) -> dispatch::DispatchResult {
        // abortTransactionUnless(validatePortIdentifier(id))
        ensure!(
            !Ports::contains_key(&identifier),
            Error::<T>::PortIdBinded
        );
        Ports::insert(&identifier, module_index);
        Self::deposit_event(RawEvent::PortBound(module_index));
        Ok(())
    }

    pub fn release_port(identifier: Vec<u8>, module_index: u8) -> dispatch::DispatchResult {
        #![warn(missing_docs)]
        ensure!(
            Ports::get(&identifier) == module_index,
            "Port identifier not found"
        );
        Ports::remove(&identifier);
        Self::deposit_event(RawEvent::PortReleased);
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
        port_identifier: Vec<u8>,
        channel_identifier: H256,
        counterparty_port_identifier: Vec<u8>,
        counterparty_channel_identifier: H256,
        version: Vec<u8>,
    ) -> dispatch::DispatchResult {
        // abortTransactionUnless(validateChannelIdentifier(portIdentifier, channelIdentifier))
        ensure!(
            connection_hops.len() == 1,
            Error::<T>::OnlyOneHopAllowedV1
        );

        ensure!(
            !Channels::contains_key((port_identifier.clone(), channel_identifier)),
            Error::<T>::ChannelIdExist
        );
        ensure!(
            Connections::contains_key(&connection_hops[0]),
            Error::<T>::ConnectionIdNotExist
        );

        // optimistic channel handshakes are allowed
        let connection = Connections::get(&connection_hops[0]);
        ensure!(
            connection.state != ConnectionState::Closed,
            Error::<T>::ConnectionClosed
        );
        // abortTransactionUnless(authenticate(privateStore.get(portPath(portIdentifier))))
        ensure!(
            Ports::get(&port_identifier) == module_index,
            Error::<T>::PortIdNotMatch
        );
        let channel_end = ChannelEnd {
            state: ChannelState::Init,
            ordering: order,
            counterparty_port_identifier,
            counterparty_channel_identifier,
            connection_hops,
            version: vec![],
        };
        Channels::insert((port_identifier.clone(), channel_identifier), channel_end);
        // key = generate()
        // provableStore.set(channelCapabilityPath(portIdentifier, channelIdentifier), key)
        NextSequenceSend::insert((port_identifier.clone(), channel_identifier), 1);
        NextSequenceRecv::insert((port_identifier.clone(), channel_identifier), 1);
        NextSequenceAck::insert((port_identifier.clone(), channel_identifier), 1);
        // return key
        ClientStates::mutate(&connection.client_id, |client_state| {
            (*client_state)
                .channels
                .push((port_identifier.clone(), channel_identifier));
        });
        Self::deposit_event(RawEvent::ChanOpenInit);
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
    pub fn send_packet(packet: Packet) -> dispatch::DispatchResult {
        let channel = Channels::get((packet.source_port.clone(), packet.source_channel));
        // optimistic sends are permitted once the handshake has started
        ensure!(
            channel.state != ChannelState::Closed,
            "channel has been closed"
        );

        // abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(packet.sourcePort, packet.sourceChannel))))
        ensure!(
            packet.dest_port == channel.counterparty_port_identifier,
            Error::<T>::PortIdNotMatch
        );
        ensure!(
            packet.dest_channel == channel.counterparty_channel_identifier,
            Error::<T>::DestChannelIdNotMatch
        );
        let connection = Connections::get(&channel.connection_hops[0]);
        ensure!(
            connection.state != ConnectionState::Closed,
            "connection has been closed"
        );

        // consensusState = provableStore.get(consensusStatePath(connection.clientIdentifier))
        // abortTransactionUnless(consensusState.getHeight() < packet.timeoutHeight)

        let mut next_sequence_send =
            NextSequenceSend::get((packet.source_port.clone(), packet.source_channel));
        ensure!(
            packet.sequence == next_sequence_send,
            Error::<T>::PackedSequenceNotMatch
        );

        // all assertions passed, we can alter state
        next_sequence_send = next_sequence_send + 1;
        NextSequenceSend::insert(
            (packet.source_port.clone(), packet.source_channel),
            next_sequence_send,
        );
        let timeout_height = packet.timeout_height.encode();
        let hash = BlakeTwo256::hash_of(&[&packet.data[..], &timeout_height[..]].concat());

        Packets::insert(
            (
                packet.source_port.clone(),
                packet.source_channel,
                packet.sequence,
            ),
            hash,
        );
        // provableStore.set(packetCommitmentPath(packet.sourcePort, packet.sourceChannel, packet.sequence), hash(packet.data, packet.timeout))

        // log that a packet has been sent
        Self::deposit_event(RawEvent::SendPacket(
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
    pub fn handle_datagram(datagram: Datagram) -> dispatch::DispatchResult {
        #![warn(missing_doc_code_examples)]
        match datagram {
            // Receiving the message containing a block header of other chains from relayers,  IBC module tryies to synchronize the block header.
            Datagram::ClientUpdate { identifier, header } => {
                ensure!(ClientStates::contains_key(&identifier), "Client not found");
                let client_state = ClientStates::get(&identifier);
                ensure!(
                    client_state.latest_height < header.height,
                    "Client already updated"
                );
                ensure!(
                    ConsensusStates::contains_key((identifier, client_state.latest_height)),
                    "ConsensusState not found"
                );
                let consensus_state =
                    ConsensusStates::get((identifier, client_state.latest_height));
                // TODO: verify header using validity_predicate
                let justification = GrandpaJustification::<Block>::decode(
                    &mut &*header.justification,
                );
                if_std! {
                    println!(
                        "consensus_state: {:?}, header: {:?}",
                        consensus_state,
                        header,
                    );
                }
                let authorities = VoterSet::new(consensus_state.authorities.iter().cloned());
                ensure!(authorities.is_some(), "Invalid authorities set");
                let authorities = authorities.unwrap();
                if let Ok(justification) = justification {
                    let result = justification.verify(consensus_state.set_id, &authorities);
                    if_std! {
                        println!("verify result: {:?}", result);
                    }
                    if result.is_ok() {
                        if_std! {
                            println!("block_hash: {:?}", header.block_hash);
                        }
                        assert_eq!(header.block_hash, justification.commit.target_hash);
                        ClientStates::mutate(&identifier, |client_state| {
                            (*client_state).latest_height = header.height;
                        });
                        // TODO
                        let new_consensus_state = grandpa::consensus_state::ConsensusState {
                            root: header.commitment_root,
                            height: header.height,
                            set_id: consensus_state.set_id,
                            authorities: consensus_state.authorities.clone(),
                        };
                        if_std! {
                            println!(
                                "consensus_state inserted: {:?}, {}",
                                identifier,
                                header.height
                            );
                        }
                        ConsensusStates::insert((identifier, header.height), new_consensus_state);

                        let result = read_proof_check::<BlakeTwo256>(
                            header.commitment_root,
                            header.authorities_proof,
                            &GRANDPA_AUTHORITIES_KEY.to_vec(),
                        );
                        // TODO
                        let result = result.unwrap().unwrap();
                        let new_authorities: AuthorityList =
                            VersionedAuthorityList::decode(&mut &*result)
                                .unwrap()
                                .into();
                        if_std! {
                            println!("new_authorities: {:?}", new_authorities);
                        }
                        if new_authorities != consensus_state.authorities {
                            ConsensusStates::mutate(
                                (identifier, header.height),
                                |consensus_state| {
                                    (*consensus_state).set_id += 1;
                                    (*consensus_state).authorities = new_authorities;
                                },
                            );
                        }
                        Self::deposit_event(RawEvent::ClientUpdated);
                    }
                }
            }
            Datagram::ClientMisbehaviour {
                identifier,
                evidence,
            } => {
                Self::deposit_event(RawEvent::ClientMisbehaviourReceived);
            }
            Datagram::ConnOpenTry {
                connection_id,
                counterparty_connection_id,
                counterparty_client_id,
                client_id,
                version,
                counterparty_version,
                proof_init,
                proof_consensus,
                proof_height,
                consensus_height,
            } => {

                let mut new_connection_end;
                if Connections::contains_key(&connection_id) {
                    let old_conn_end = Connections::get(&connection_id);
                    let state_is_consistent = old_conn_end.state.eq(&ConnectionState::Init)
                        && old_conn_end.counterparty_connection_id.eq(&counterparty_connection_id)
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
                if_std! {
                    println!(
                        "query consensus_state: {:?}, {}",
                        client_id,
                        proof_height
                    );
                }
                ensure!(
                    ConsensusStates::contains_key((client_id, proof_height)),
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
                new_connection_end.version = vec![Self::pick_version(intersection)];  // Todo: change the field `version` in `new_connection_end` to `u8`

                let identifier = connection_id;
                Connections::insert(&identifier, new_connection_end);
                // addConnectionToClient(clientIdentifier, identifier)
                ClientStates::mutate(&client_id, |client_state| {
                    (*client_state).connections.push(identifier);
                });
                Self::deposit_event(RawEvent::ConnOpenTry);
            }
            Datagram::ConnOpenAck {
                connection_id,
                counterparty_connection_id,
                version,
                proof_try,
                proof_consensus,
                proof_height,
                consensus_height,
            } => {
                use sp_runtime::traits::SaturatedConversion;
                let current_block_number_self = <frame_system::Module<T>>::block_number().saturated_into::<u32>();
                Self::check_client_consensus_height(current_block_number_self, consensus_height);

                ensure!(
                    Connections::contains_key(&connection_id),
                    "Connection uninitialized"
                );

                let mut new_connection_end;
                {
                    let old_conn_end = Connections::get(&connection_id);
                    let state_is_consistent = old_conn_end.state.eq(&ConnectionState::Init)
                            && old_conn_end.version.contains(&version)
                        || old_conn_end.state.eq(&ConnectionState::TryOpen)
                            && (old_conn_end.version.get(0) == Some(&version));

                    // Check that if the msg's counterparty connection id is not empty then it matches
                    // the old connection's counterparty.
                    // Todo: Ensure connecion id is not empty?
                    let counterparty_matches= old_conn_end.counterparty_connection_id == counterparty_connection_id;

                    ensure!(state_is_consistent && counterparty_matches, "Connection mismatch!");

                    new_connection_end = old_conn_end.clone();
                }

                // expectedConsensusState = getConsensusState(consensusHeight)
                // expected = ConnectionEnd{TRYOPEN, identifier, getCommitmentPrefix(),
                //                          connection.counterpartyClientIdentifier, connection.clientIdentifier,
                //                          version}
                ensure!(
                    ConsensusStates::contains_key((new_connection_end.client_id, proof_height)),
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
                Connections::mutate(&connection_id, |connection| {
                    (*connection).state = ConnectionState::Open;
                });
                // abortTransactionUnless(getCompatibleVersions().indexOf(version) !== -1)
                // connection.version = version
                // provableStore.set(connectionPath(identifier), connection)
                Self::deposit_event(RawEvent::ConnOpenAck);
            }
            Datagram::ConnOpenConfirm {
                connection_id,
                proof_ack,
                proof_height,
            } => {
                ensure!(
                    Connections::contains_key(&connection_id),
                    "Connection uninitialized"
                );

                let mut new_connection_end;
                {
                    let old_conn_end = Connections::get(&connection_id);
                    ensure!(old_conn_end.state.eq(&ConnectionState::TryOpen), "Connection mismatch!");
                    new_connection_end = old_conn_end.clone();
                }

                ensure!(
                    ConsensusStates::contains_key((new_connection_end.client_id, proof_height)),
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
                Connections::mutate(&connection_id, |connection| {
                    (*connection).state = ConnectionState::Open;
                });
                // provableStore.set(connectionPath(identifier), connection)
                Self::deposit_event(RawEvent::ConnOpenConfirm);
            }
            Datagram::ChanOpenTry {
                order,
                connection_hops,
                port_identifier,
                channel_identifier,
                counterparty_port_identifier,
                counterparty_channel_identifier,
                version,
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
                    !Channels::contains_key((port_identifier.clone(), channel_identifier)),
                    "channel identifier already exists"
                );
                // abortTransactionUnless(authenticate(privateStore.get(portPath(portIdentifier))))
                ensure!(
                    Connections::contains_key(&connection_hops[0]),
                    "connection identifier not exists"
                );
                let connection = Connections::get(&connection_hops[0]);
                ensure!(
                    connection.state == ConnectionState::Open,
                    "connection has been closed"
                );

                ensure!(
                    ConsensusStates::contains_key((connection.client_id, proof_height)),
                    "ConsensusState not found"
                );
                let value = Self::verify_channel_state(
                    connection.client_id,
                    proof_height,
                    counterparty_port_identifier.clone(),
                    counterparty_channel_identifier,
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
                let dest_module_index = Ports::get(port_identifier.clone());
                T::ModuleCallbacks::on_chan_open_try(dest_module_index.into(), order.clone(), connection_hops.clone(), port_identifier.clone(), channel_identifier, counterparty_port_identifier.clone(), counterparty_channel_identifier, version.clone(), counterparty_version);

                let channel_end = ChannelEnd {
                    state: ChannelState::TryOpen,
                    ordering: order,
                    counterparty_port_identifier,
                    counterparty_channel_identifier,
                    connection_hops,
                    version,
                };
                Channels::insert((port_identifier.clone(), channel_identifier), channel_end);
                // key = generate()
                // provableStore.set(channelCapabilityPath(portIdentifier, channelIdentifier), key)
                NextSequenceSend::insert((port_identifier.clone(), channel_identifier), 1);
                NextSequenceRecv::insert((port_identifier.clone(), channel_identifier), 1);
                // return key
                ClientStates::mutate(&connection.client_id, |client_state| {
                    (*client_state)
                        .channels
                        .push((port_identifier.clone(), channel_identifier));
                });
                Self::deposit_event(RawEvent::ChanOpenTry);
            }
            Datagram::ChanOpenAck {
                port_identifier,
                channel_identifier,
                version,
                proof_try,
                proof_height,
            } => {
                ensure!(
                    Channels::contains_key((port_identifier.clone(), channel_identifier)),
                    "channel identifier not exists"
                );
                let channel = Channels::get((port_identifier.clone(), channel_identifier));
                ensure!(
                    channel.state == ChannelState::Init || channel.state == ChannelState::TryOpen,
                    "channel is not ready"
                );
                // abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(portIdentifier, channelIdentifier))))
                ensure!(
                    Connections::contains_key(&channel.connection_hops[0]),
                    "connection identifier not exists"
                );
                let connection = Connections::get(&channel.connection_hops[0]);
                ensure!(
                    connection.state == ConnectionState::Open,
                    "connection has been closed"
                );
                ensure!(
                    ConsensusStates::contains_key((connection.client_id, proof_height)),
                    "ConsensusState not found"
                );
                let value = Self::verify_channel_state(
                    connection.client_id,
                    proof_height,
                    channel.counterparty_port_identifier,
                    channel.counterparty_channel_identifier,
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
                let dest_module_index = Ports::get(port_identifier.clone());
                T::ModuleCallbacks::on_chan_open_ack(dest_module_index.into(), port_identifier.clone(), channel_identifier, version.clone());
                Channels::mutate((port_identifier, channel_identifier), |channel| {
                    (*channel).state = ChannelState::Open;
                });
                Self::deposit_event(RawEvent::ChanOpenAck);
            }
            Datagram::ChanOpenConfirm {
                port_identifier,
                channel_identifier,
                proof_ack,
                proof_height,
            } => {
                ensure!(
                    Channels::contains_key((port_identifier.clone(), channel_identifier)),
                    "channel identifier not exists"
                );
                let channel = Channels::get((port_identifier.clone(), channel_identifier));
                ensure!(
                    channel.state == ChannelState::TryOpen,
                    "channel is not ready"
                );
                // abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(portIdentifier, channelIdentifier))))
                ensure!(
                    Connections::contains_key(&channel.connection_hops[0]),
                    "connection identifier not exists"
                );
                let connection = Connections::get(&channel.connection_hops[0]);
                ensure!(
                    connection.state == ConnectionState::Open,
                    "connection has been closed"
                );
                ensure!(
                    ConsensusStates::contains_key((connection.client_id, proof_height)),
                    "ConsensusState not found"
                );
                let value = Self::verify_channel_state(
                    connection.client_id,
                    proof_height,
                    channel.counterparty_port_identifier,
                    channel.counterparty_channel_identifier,
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
                let dest_module_index = Ports::get(port_identifier.clone());
                T::ModuleCallbacks::on_chan_open_confirm(dest_module_index.into(), port_identifier.clone(), channel_identifier);
                Channels::mutate((port_identifier, channel_identifier), |channel| {
                    (*channel).state = ChannelState::Open;
                });
                Self::deposit_event(RawEvent::ChanOpenConfirm);
            }
            Datagram::PacketRecv {
                packet,
                proof,
                proof_height,
            } => {
                ensure!(
                    Channels::contains_key((packet.dest_port.clone(), packet.dest_channel)),
                    "channel identifier not exists"
                );
                let channel = Channels::get((packet.dest_port.clone(), packet.dest_channel));
                ensure!(channel.state == ChannelState::Open, "channel is not ready");
                // abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(packet.destPort, packet.destChannel))))
                ensure!(
                    packet.source_port == channel.counterparty_port_identifier,
                    "source port not match"
                );
                ensure!(
                    packet.source_channel == channel.counterparty_channel_identifier,
                    "source channel not match"
                );

                ensure!(
                    Connections::contains_key(&channel.connection_hops[0]),
                    "connection identifier not exists"
                );
                let connection = Connections::get(&channel.connection_hops[0]);
                ensure!(
                    connection.state == ConnectionState::Open,
                    "connection has been closed"
                );

                // abortTransactionUnless(getConsensusHeight() < packet.timeoutHeight)

                ensure!(
                    ConsensusStates::contains_key((connection.client_id, proof_height)),
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
                let hash = BlakeTwo256::hash_of(&[&packet.data[..], &timeout_height[..]].concat());
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
                let dest_module_index = Ports::get(packet.dest_port.clone());
                if_std! {
                    println!("dest_module_index: {}", dest_module_index);
                }
                T::ModuleCallbacks::on_recv_packet(dest_module_index.into(), packet.clone());
                let acknowledgement: Vec<u8> = vec![1, 3, 3, 7];

                if acknowledgement.len() > 0 || channel.ordering == ChannelOrder::Unordered {
                    let hash = BlakeTwo256::hash_of(&acknowledgement);

                    Acknowledgements::insert(
                        (
                            packet.dest_port.clone(),
                            packet.dest_channel,
                            packet.sequence,
                        ),
                        hash,
                    );
                }

                if channel.ordering == ChannelOrder::Ordered {
                    let mut next_sequence_recv =
                        NextSequenceRecv::get((packet.dest_port.clone(), packet.dest_channel));
                    ensure!(
                        packet.sequence == next_sequence_recv,
                        "recv sequence not match"
                    );
                    next_sequence_recv = next_sequence_recv + 1;
                    NextSequenceRecv::insert(
                        (packet.dest_port.clone(), packet.dest_channel),
                        next_sequence_recv,
                    );
                }

                // log that a packet has been received & acknowledged
                // emitLogEntry("recvPacket", {sequence: packet.sequence, timeout: packet.timeout, data: packet.data, acknowledgement})
                Self::deposit_event(RawEvent::RecvPacket(
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
                    Channels::contains_key((packet.source_port.clone(), packet.source_channel)),
                    "channel identifier not exists"
                );
                let channel = Channels::get((packet.source_port.clone(), packet.source_channel));
                ensure!(channel.state == ChannelState::Open, "channel is not ready");
                // abortTransactionUnless(authenticate(privateStore.get(channelCapabilityPath(packet.sourcePort, packet.sourceChannel))))
                ensure!(
                    packet.dest_channel == channel.counterparty_channel_identifier,
                    "dest channel not match"
                );

                ensure!(
                    Connections::contains_key(&channel.connection_hops[0]),
                    "connection identifier not exists"
                );
                let connection = Connections::get(&channel.connection_hops[0]);
                ensure!(
                    connection.state == ConnectionState::Open,
                    "connection has been closed"
                );
                ensure!(
                    packet.dest_port == channel.counterparty_port_identifier,
                    "dest port not match"
                );

                // verify we sent the packet and haven't cleared it out yet
                // abortTransactionUnless(provableStore.get(packetCommitmentPath(packet.sourcePort, packet.sourceChannel, packet.sequence))
                //        === hash(packet.data, packet.timeout))
                let timeout_height = packet.timeout_height.encode();
                let expect_hash =
                    BlakeTwo256::hash_of(&[&packet.data[..], &timeout_height[..]].concat());

                let hash = Packets::get((
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
                ensure!(
                    value.unwrap() == hash,
                    "packet acknowledgement hash not match"
                );

                // abort transaction unless acknowledgement is processed in order
                if channel.ordering == ChannelOrder::Ordered {
                    let mut next_sequence_ack =
                        NextSequenceAck::get((packet.dest_port.clone(), packet.dest_channel));
                    ensure!(
                        packet.sequence == next_sequence_ack,
                        "recv sequence not match"
                    );
                    next_sequence_ack = next_sequence_ack + 1;
                    NextSequenceAck::insert(
                        (packet.dest_port.clone(), packet.dest_channel),
                        next_sequence_ack,
                    );
                }

                // all assertions passed, we can alter state

                // delete our commitment so we can't "acknowledge" again
                Acknowledgements::remove((
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
        let consensus_state = ConsensusStates::get((client_id, proof_height));
        let key = Connections::hashed_key_for(connection_identifier);
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
                            if_std! {
                                println!("trie value decode error: {:?}", error);
                            }
                        }
                    }
                }
                None => {
                    if_std! {
                        println!("read_proof_check error: value not exists");
                    }
                }
            },
            Err(error) => {
                if_std! {
                    println!("read_proof_check error: {:?}", error);
                }
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
        let consensus_state = ConsensusStates::get((client_id, proof_height));
        let key = Channels::hashed_key_for((port_identifier, channel_identifier));
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
                            if_std! {
                                println!("trie value decode error: {:?}", error);
                            }
                        }
                    }
                }
                None => {
                    if_std! {
                        println!("read_proof_check error: value not exists");
                    }
                }
            },
            Err(error) => {
                if_std! {
                    println!("read_proof_check error: {:?}", error);
                }
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
        let consensus_state = ConsensusStates::get((client_id, proof_height));
        let key = Packets::hashed_key_for((port_identifier, channel_identifier, sequence));
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
                            if_std! {
                                println!("trie value decode error: {:?}", error);
                            }
                        }
                    }
                }
                None => {
                    if_std! {
                        println!("read_proof_check error: value not exists");
                    }
                }
            },
            Err(error) => {
                if_std! {
                    println!("read_proof_check error: {:?}", error);
                }
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
        let consensus_state = ConsensusStates::get((client_id, proof_height));
        let key = Acknowledgements::hashed_key_for((port_identifier, channel_identifier, sequence));
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
                            if_std! {
                                println!("trie value decode error: {:?}", error);
                            }
                        }
                    }
                }
                None => {
                    if_std! {
                        println!("read_proof_check error: value not exists");
                    }
                }
            },
            Err(error) => {
                if_std! {
                    println!("read_proof_check error: {:?}", error);
                }
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
    ) -> dispatch::DispatchResult {
        // Todo: Use Height struct as ibc-rs/modules/src/ics02_client/height.rs?

        ensure!(
                    claimed_height > host_current_height,
                    "Consensus height is too advanced!"
                );

        ensure!(
                    claimed_height < host_current_height - MAX_HISTORY_SIZE,
                    "Consensus height is too old!"
                );

        Ok(())
    }

}
