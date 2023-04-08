#![cfg_attr(not(feature = "std"), no_std)]

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
use sp_std::{fmt::Debug, vec, vec::Vec};
pub mod channel;
pub mod client;
pub mod connection;
pub mod context;
pub mod errors;
pub mod host;
pub mod port;
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
	use frame_support::{pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::{
		core::{
			ics02_client::{client_type::ClientType, height::Height},
			ics03_connection::connection::ConnectionEnd,
			ics04_channel::{
				channel::ChannelEnd,
				commitment::{
					AcknowledgementCommitment as IbcAcknowledgementCommitment,
					PacketCommitment as IbcPacketCommitment,
				},
				packet::{Receipt, Sequence},
			},
			ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
			ics26_routing::handler::MsgReceipt,
		},
		events::IbcEvent,
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

		const IBC_COMMITMENT_PREFIX: &'static [u8];

		type ExpectedBlockTime: Get<u64>;

		/// benchmarking weight info
		type WeightInfo: WeightInfo<Self>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn client_state)]
	/// Key: client_id
	/// value: ClientState
	pub type ClientStates<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn client_update_time)]
	/// key1: client_id
	/// key2: height
	/// value: timestamp
	pub type ClientProcessedTimes<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClientId, Blake2_128Concat, Height, u64>;

	#[pallet::storage]
	#[pallet::getter(fn client_update_height)]
	/// key1: client_id
	/// key2: height
	/// value: host_height
	pub type ClientProcessedHeights<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClientId, Blake2_128Concat, Height, Height>;

	#[pallet::storage]
	#[pallet::getter(fn consensus_state)]
	/// key1: client_id
	/// key2: height
	/// value: ConsensusState
	pub type ConsensusStates<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, ClientId, Blake2_128Concat, Height, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn connection_end)]
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, ConnectionId, ConnectionEnd>;

	#[pallet::storage]
	#[pallet::getter(fn channel_end)]
	pub type Channels<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, ChannelEnd>;

	#[pallet::storage]
	#[pallet::getter(fn connection_channels)]
	/// key: connection_id
	/// value: Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, ConnectionId, Vec<(PortId, ChannelId)>>;

	#[pallet::storage]
	#[pallet::getter(fn get_next_sequence_send)]
	/// Key1: port_id
	/// key2: channel_id
	/// value: sequence
	pub type NextSequenceSend<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, Sequence>;

	#[pallet::storage]
	#[pallet::getter(fn get_next_sequence_recv)]
	/// key1: port_id
	/// key2: channel_id
	/// value: sequence
	pub type NextSequenceRecv<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, Sequence>;

	#[pallet::storage]
	#[pallet::getter(fn get_next_sequence_ack)]
	/// key1: port_id
	/// key2: channel_id
	/// value: sequence
	pub type NextSequenceAck<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, PortId, Blake2_128Concat, ChannelId, Sequence>;

	#[pallet::storage]
	#[pallet::getter(fn get_packet_acknowledgement)]
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
	#[pallet::getter(fn client_type)]
	/// key: client_id
	/// value: ClientType
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, ClientType>;

	#[pallet::storage]
	#[pallet::getter(fn client_cnt)]
	/// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_cnt)]
	/// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn channel_cnt)]
	/// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_client)]
	/// key: ClientId
	/// value: ConnectionId
	pub type ConnectionClient<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, ConnectionId>;

	#[pallet::storage]
	#[pallet::getter(fn get_packet_receipt)]
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
	#[pallet::getter(fn get_packet_commitment)]
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
		/// Ibc events
		IbcEvents { events: Vec<IbcEvent> },
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
		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<ibc_proto::google::protobuf::Any>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let mut ctx = Context::<T>::new();
			log::info!(
				"☀️ ibc messages type: {:?}",
				messages.iter().map(|v| &v.type_url).collect::<Vec<_>>()
			);

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
			log::info!("🙅🙅 deliver ----> events: {:?}", events);
			log::info!("🙅🙅 🔥 🔥deliver ----> logs: {:?}", logs);
			log::info!("🙅🙅 ❌❌ deliver ----> errors: {:?}", errors);

			log::trace!(target: "pallet_ibc", "[pallet_ibc_deliver]: logs: {:?}", logs);
			log::trace!(target: "pallet_ibc", "[pallet_ibc_deliver]: errors: {:?}", errors);

			Self::deposit_event(Event::IbcEvents { events });
			if !errors.is_empty() {
				Self::deposit_event(errors.into());
			}

			Ok(().into())
		}
	}
}
