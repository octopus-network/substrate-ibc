#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unreachable_patterns)]

//! # Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to
//! interact with other chains in a trustees way via IBC protocol
//!
//! The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  
//! and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs),
//! which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).
extern crate alloc;
extern crate core;

pub use pallet::*;

use crate::prelude::String;
use frame_system::ensure_signed;
use ibc::core::ics24_host::identifier::ChannelId as IbcChannelId;
use sp_std::{fmt::Debug, vec, vec::Vec};

pub mod context;
pub mod errors;
pub mod events;
pub mod module;
pub mod prelude;
pub mod utils;

pub use crate::{
	context::Context,
	module::core::ics24_host::{
		ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId,
	},
};

pub const LOG_TARGET: &str = "runtime::pallet-ibc";
pub const REVISION_NUMBER: u64 = 0;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod type_define {
	use sp_std::vec::Vec;

	pub type OctopusClientStatePath = Vec<u8>;
	pub type OctopusClientState = Vec<u8>;
	pub type OctopusClientId = Vec<u8>;
	pub type OctopusIbcHeight = Vec<u8>;
	pub type OctopusTimeStamp = Vec<u8>;
	pub type OctopusIbcHostHeight = Vec<u8>;
	pub type OctopusClientConsensusStatePath = Vec<u8>;
	pub type OctopusConsensusState = Vec<u8>;
	pub type OctopusConnectionsPath = Vec<u8>;
	pub type OctopusConnectionEnd = Vec<u8>;
	pub type OctopusChannelEndPath = Vec<u8>;
	pub type OctopusChannelEnd = Vec<u8>;
	pub type OctopusSeqSendsPath = Vec<u8>;
	pub type OctopusSeqRecvsPath = Vec<u8>;
	pub type OctopusSeqAcksPath = Vec<u8>;
	pub type OctopusAcksPath = Vec<u8>;
	pub type OctopusAcksHash = Vec<u8>;
	pub type OctopusClientTypePath = Vec<u8>;
	pub type OctopusClientType = Vec<u8>;
	pub type OctopusClientConnectionsPath = Vec<u8>;
	pub type OctopusConnectionId = Vec<u8>;
	pub type OctopusRecipientsPath = Vec<u8>;
	pub type OctopusRecipient = Vec<u8>;
	pub type OctopusCommitmentsPath = Vec<u8>;
	pub type OctopusCommitmentHash = Vec<u8>;
	pub type OctopusSequence = u64;
	pub type PreviousHostHeight = u64;
}

#[frame_support::pallet]
pub mod pallet {
	use super::{errors, type_define::*, *};
	use crate::{events::ModuleEvent, module::core::ics24_host::Height};
	use frame_support::{pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::core::ics26_routing::handler::MsgReceipt;
	use ibc_support::Any;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + Sync + Send + Debug + pallet_ics20_transfer::Config
	{
		/// The aggregated event type of the runtime.
		type RuntimeEvent: Parameter
			+ Member
			+ From<Event<Self>>
			+ Debug
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The provider providing timestamp of host chain
		type TimeProvider: UnixTime;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// ClientStatePath(client_id) => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusClientStatePath, OctopusClientState, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientProcessedTimes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OctopusClientId,
		Blake2_128Concat,
		OctopusIbcHeight,
		OctopusTimeStamp,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientProcessedHeights<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OctopusClientId,
		Blake2_128Concat,
		OctopusIbcHeight,
		OctopusIbcHostHeight,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ClientConsensusStatePath(client_id, Height) => ConsensusState
	pub type ConsensusStates<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OctopusClientConsensusStatePath,
		OctopusConsensusState,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => ConnectionEnd
	pub type Connections<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusConnectionsPath, OctopusConnectionEnd, ValueQuery>;

	#[pallet::storage]
	/// ChannelEndPath(port_id, channel_id) => ChannelEnd
	pub type Channels<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusChannelEndPath, OctopusChannelEnd, ValueQuery>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => Vec<ChannelEndPath(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OctopusConnectionsPath,
		Vec<OctopusChannelEndPath>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// SeqSendsPath(port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusSeqSendsPath, OctopusSequence, ValueQuery>;

	#[pallet::storage]
	/// SeqRecvsPath(port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusSeqRecvsPath, OctopusSequence, ValueQuery>;

	#[pallet::storage]
	/// SeqAcksPath(port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusSeqAcksPath, OctopusSequence, ValueQuery>;

	#[pallet::storage]
	/// AcksPath(port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusAcksPath, OctopusAcksHash, ValueQuery>;

	#[pallet::storage]
	/// ClientTypePath(client_id) => client_type
	pub type Clients<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusClientTypePath, OctopusClientType, ValueQuery>;

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
	/// ClientConnectionsPath(client_id) => connection_id
	pub type ConnectionClient<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OctopusClientConnectionsPath,
		OctopusConnectionId,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ReceiptsPath(port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusRecipientsPath, OctopusRecipient, ValueQuery>;

	#[pallet::storage]
	/// CommitmentsPath(port_id, channel_id, sequence) => hash of (timestamp, height, packet)
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusCommitmentsPath, OctopusCommitmentHash, ValueQuery>;

	#[pallet::storage]
	/// Previous host block height
	pub type OldHeight<T: Config> = StorageValue<_, PreviousHostHeight, ValueQuery>;

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Client created event
		CreateClient { client_id: ClientId, client_type: ClientType, consensus_height: Height },
		/// Client updated event
		UpdateClient {
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
			consensus_heights: Vec<Height>,
			header: Any,
		},
		/// Client upgraded event
		UpgradeClient { client_id: ClientId, client_type: ClientType, consensus_height: Height },
		/// Client misbehaviour event
		ClientMisbehaviour { client_id: ClientId, client_type: ClientType },
		/// Connection open init event
		OpenInitConnection {
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open try event
		OpenTryConnection {
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open acknowledgement event
		OpenAckConnection {
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open confirm event
		OpenConfirmConnection {
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Channel open init event
		OpenInitChannel {
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open try event
		OpenTryChannel {
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open acknowledgement event
		OpenAckChannel {
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open confirm event
		OpenConfirmChannel {
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel close init event
		CloseInitChannel {
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel close confirm event
		CloseConfirmChannel {
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Send packet event
		SendPacket { packet: Packet },
		/// Receive packet event
		ReceivePacket { packet: Packet },
		/// WriteAcknowledgement packet event
		WriteAcknowledgement { packet: Packet, ack: Vec<u8> },
		/// Acknowledgements packet event
		AcknowledgePacket { packet: Packet },
		/// Timeout packet event
		TimeoutPacket { packet: Packet },
		/// TimoutOnClose packet event
		TimeoutOnClosePacket { packet: Packet },
		/// Empty event
		Empty(Vec<u8>),
		/// App Module event
		AppModule(ModuleEvent),
		/// Ibc errors
		IbcErrors { errors: Vec<errors::IbcError> },
	}

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// decode String failed
		DecodeStringFailed,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for most of the IBC request.
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		///
		/// The origin must be Signed and the sender must have sufficient funds fee.
		///
		/// Parameters:
		/// - `messages`: The arbitrary ICS message's representation in Substrate, which contains an
		///   URL and
		///  a serialized protocol buffer message. The URL name that uniquely identifies the type of
		/// the serialized protocol buffer message.
		///
		/// The relevant events are emitted when successful.
		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<ibc_support::Any>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let mut ctx = Context::<T>::new();

			let messages = messages
				.into_iter()
				.map(|message| {
					let type_url = String::from_utf8(message.type_url.clone())
						.map_err(|_| Error::<T>::DecodeStringFailed)?;
					Ok(ibc_proto::google::protobuf::Any { type_url, value: message.value })
				})
				.collect::<Result<Vec<ibc_proto::google::protobuf::Any>, Error<T>>>()?;

			let (events, logs, errors) = messages.into_iter().fold(
				(vec![], vec![], vec![]),
				|(mut events, mut logs, mut errors), msg| {
					match ibc::core::ics26_routing::handler::deliver(&mut ctx, msg) {
						Ok(MsgReceipt { events: temp_events, log: temp_logs }) => {
							events.extend(temp_events);
							logs.extend(temp_logs);
						},
						Err(e) => errors.push(e),
					}
					(events, logs, errors)
				},
			);
			log::trace!(target: "pallet_ibc", "[pallet_ibc_deliver]: logs: {:?}", logs);
			log::trace!(target: "pallet_ibc", "[pallet_ibc_deliver]: errors: {:?}", errors);

			events.into_iter().for_each(|event| {
				Self::deposit_event(event.into());
			});
			Self::deposit_event(errors.into());

			Ok(().into())
		}
	}
}
