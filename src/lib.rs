#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unreachable_patterns)]
#![allow(unused_imports)]
#![allow(deprecated)]

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

pub use alloc::{
	format,
	string::{String, ToString},
};
use frame_system::ensure_signed;
use ibc::core::ics24_host::identifier::ChannelId as IbcChannelId;
use sp_std::{fmt::Debug, vec, vec::Vec};
pub mod channel;
pub mod client;
pub mod connection;
pub mod context;
pub mod errors;
pub mod events;
pub mod host;
pub mod port;
pub mod relayer;
pub mod routing;
pub mod utils;

pub use crate::context::Context;

pub const LOG_TARGET: &str = "runtime::pallet-ibc";
pub const REVISION_NUMBER: u64 = 0;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarks;
mod weights;

pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::{errors, *};
	use crate::events::ModuleEvent;
	use frame_support::{pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::{
		core::{
			ics02_client::{client_type::ClientType, height::Height},
			ics03_connection::connection::ConnectionEnd,
			ics04_channel::{
				channel::{ChannelEnd, Order},
				commitment::{
					AcknowledgementCommitment as IbcAcknowledgementCommitment,
					PacketCommitment as IbcPacketCommitment,
				},
				packet::{Receipt, Sequence},
				timeout::TimeoutHeight,
				Version as ChannelVersion,
			},
			ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
			ics26_routing::handler::MsgReceipt,
		},
		timestamp::Timestamp,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Sync + Send + Debug {
		/// The aggregated event type of the runtime.
		type RuntimeEvent: Parameter
			+ Member
			+ From<Event<Self>>
			+ Debug
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The provider providing timestamp of host chain
		type TimeProvider: UnixTime;

		type ExpectedBlockTime: Get<u64>;

		/// benchmarking weight info
		type WeightInfo: WeightInfo<Self>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// Key: client_id
	/// value: ClientState
	pub type ClientStates<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, Vec<u8>>;

	#[pallet::storage]
	/// key1: client_id
	/// key2: height
	/// value: timestamp
	pub type ClientProcessedTimes<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClientId, Blake2_128Concat, Height, u64>;

	#[pallet::storage]
	/// key1: client_id
	/// key2: height
	/// value: host_height
	pub type ClientProcessedHeights<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClientId, Blake2_128Concat, Height, Height>;

	#[pallet::storage]
	/// key1: client_id
	/// key2: height
	/// value: ConsensusState
	pub type ConsensusStates<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClientId, Blake2_128Concat, Height, Vec<u8>>;

	#[pallet::storage]
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, ConnectionId, ConnectionEnd>;

	#[pallet::storage]
	pub type Channels<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, ChannelEnd>;

	#[pallet::storage]
	/// key: connection_id
	/// value: Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, ConnectionId, Vec<(PortId, ChannelId)>>;

	#[pallet::storage]
	/// Key1: port_id
	/// key2: channel_id
	/// value: sequence
	pub type NextSequenceSend<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, Sequence>;

	#[pallet::storage]
	/// key1: port_id
	/// key2: channel_id
	/// value: sequence
	pub type NextSequenceRecv<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, Sequence>;

	#[pallet::storage]
	/// key1: port_id
	/// key2: channel_id
	/// value: sequence
	pub type NextSequenceAck<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, Sequence>;

	#[pallet::storage]
	/// key1: port_id
	/// key2: channel_id
	/// key3: sequence
	/// value: hash of acknowledgement
	pub type Acknowledgements<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, PortId>,
			NMapKey<Blake2_128Concat, ChannelId>,
			NMapKey<Blake2_128Concat, Sequence>,
		),
		IbcAcknowledgementCommitment,
	>;

	#[pallet::storage]
	/// key: client_id
	/// value: ClientType
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, ClientType>;

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
	/// key: ClientId
	/// value: ConnectionId
	pub type ConnectionClient<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, ConnectionId>;

	#[pallet::storage]
	/// ReceiptsPath(port_id, channel_id, sequence) => receipt
	/// key1: port_id
	/// key2: channel_id
	/// key3: sequence
	/// value: receipt
	pub type PacketReceipt<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, PortId>,
			NMapKey<Blake2_128Concat, ChannelId>,
			NMapKey<Blake2_128Concat, Sequence>,
		),
		Receipt,
	>;

	#[pallet::storage]
	/// key1: port_id
	/// key2: channel_id
	/// key3: sequence
	/// value: hash of (timestamp, height, packet)
	pub type PacketCommitment<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, PortId>,
			NMapKey<Blake2_128Concat, ChannelId>,
			NMapKey<Blake2_128Concat, Sequence>,
		),
		IbcPacketCommitment,
	>;

	#[pallet::storage]
	/// Previous host block height
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery>;

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
			header: ibc_support::Any,
		},
		/// Client upgraded event
		UpgradeClient { client_id: ClientId, client_type: ClientType, consensus_height: Height },
		/// Client misbehaviour event
		ClientMisbehaviour { client_id: ClientId, client_type: ClientType },
		/// Connection open init event
		OpenInitConnection {
			connection_id: ConnectionId,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open try event
		OpenTryConnection {
			connection_id: ConnectionId,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open acknowledgement event
		OpenAckConnection {
			connection_id: ConnectionId,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open confirm event
		OpenConfirmConnection {
			connection_id: ConnectionId,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Channel open init event
		OpenInitChannel {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			connection_id: ConnectionId,
			version: ChannelVersion,
		},
		/// Channel open try event
		OpenTryChannel {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			counterparty_channel_id: ChannelId,
			connection_id: ConnectionId,
			version: ChannelVersion,
		},
		/// Channel open acknowledgement event
		OpenAckChannel {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			counterparty_channel_id: ChannelId,
			connection_id: ConnectionId,
		},
		/// Channel open confirm event
		OpenConfirmChannel {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			counterparty_channel_id: ChannelId,
			connection_id: ConnectionId,
		},
		/// Channel close init event
		CloseInitChannel {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			counterparty_channel_id: ChannelId,
			connection_id: ConnectionId,
		},
		/// Channel close confirm event
		CloseConfirmChannel {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			counterparty_channel_id: ChannelId,
			connection_id: ConnectionId,
		},
		/// Send packet event
		SendPacket {
			packet_data: Vec<u8>,
			timeout_height: TimeoutHeight,
			timeout_timestamp: Timestamp,
			sequence: Sequence,
			src_port_id: PortId,
			src_channel_id: ChannelId,
			dst_port_id: PortId,
			dst_channel_id: ChannelId,
			channel_ordering: Order,
			src_connection_id: ConnectionId,
		},
		/// Receive packet event
		ReceivePacket {
			packet_data: Vec<u8>,
			timeout_height: TimeoutHeight,
			timeout_timestamp: Timestamp,
			sequence: Sequence,
			src_port_id: PortId,
			src_channel_id: ChannelId,
			dst_port_id: PortId,
			dst_channel_id: ChannelId,
			channel_ordering: Order,
			dst_connection_id: ConnectionId,
		},
		/// WriteAcknowledgement packet event
		WriteAcknowledgement {
			packet_data: Vec<u8>,
			timeout_height: TimeoutHeight,
			timeout_timestamp: Timestamp,
			sequence: Sequence,
			src_port_id: PortId,
			src_channel_id: ChannelId,
			dst_port_id: PortId,
			dst_channel_id: ChannelId,
			acknowledgement: Vec<u8>,
			dst_connection_id: ConnectionId,
		},
		/// Acknowledgements packet event
		AcknowledgePacket {
			timeout_height: TimeoutHeight,
			timeout_timestamp: Timestamp,
			sequence: Sequence,
			src_port_id: PortId,
			src_channel_id: ChannelId,
			dst_port_id: PortId,
			dst_channel_id: ChannelId,
			channel_ordering: Order,
			src_connection_id: ConnectionId,
		},
		/// Timeout packet event
		TimeoutPacket {
			timeout_height: TimeoutHeight,
			timeout_timestamp: Timestamp,
			sequence: Sequence,
			src_port_id: PortId,
			src_channel_id: ChannelId,
			dst_port_id: PortId,
			dst_channel_id: ChannelId,
		},
		/// TimoutOnClose packet event
		ChannelClosed {
			port_id: PortId,
			channel_id: ChannelId,
			counterparty_port_id: PortId,
			maybe_counterparty_channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			channel_ordering: Order,
		},
		/// App Module event
		AppModule(ModuleEvent<T>),
		/// Ibc errors
		IbcErrors { errors: Vec<errors::IbcError> },
	}

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// decode String failed
		DecodeStringFailed,
		/// unknow Client type
		UnknownClientType,
		/// invalid portid
		InvalidPortId,
		/// invalid channel id
		InvalidChannelId,
		/// invalid height
		InvalidHeight,
		/// invalid client id
		InvalidClientId,
		/// invalid connection id
		InvalidConnectionId,
		/// invalid timestamp
		InvalidTimestamp,
		/// invalid version
		InvalidVersion,
		/// Invalid module id
		InvalidModuleId,
		///
		Other,
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
		#[pallet::call_index(0)]
		#[pallet::weight(crate::weights::deliver::<T>(messages))]
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

			for event in events.into_iter() {
				Self::deposit_event(event.try_into()?);
			}
			Self::deposit_event(errors.into());

			Ok(().into())
		}
	}
}
