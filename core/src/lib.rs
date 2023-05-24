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
pub mod context;
pub mod errors;
pub mod host;
pub mod utils;

pub use crate::context::Context;
use ibc_support::module::AddModule;

pub const LOG_TARGET: &str = "runtime::pallet-ibc";
pub const REVISION_NUMBER: u64 = 0;

#[frame_support::pallet]
pub mod pallet {
	use super::{errors, *};
	use frame_support::{pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::core::{
		events::IbcEvent,
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
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath,
				ClientStatePath, CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath,
				SeqRecvPath, SeqSendPath,
			},
		},
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_timestamp::Config + Sync + Send + Debug
	{
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

		type ChainVersion: Get<u64>;

		type IbcModule: AddModule;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// Key: ClientStatePath
	/// value: ClientState
	pub type ClientStates<T: Config> = StorageMap<_, Blake2_128Concat, ClientStatePath, Vec<u8>>;

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
	/// key: ClientConsensusStatePath
	/// value: ConsensusState
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, ClientConsensusStatePath, Vec<u8>>;

	#[pallet::storage]
	/// key: ConnectionPath
	/// value: ConnectionEnd
	pub type Connections<T: Config> =
		StorageMap<_, Blake2_128Concat, ConnectionPath, ConnectionEnd>;

	#[pallet::storage]
	/// key: CHannelEndsPath
	/// value: ChannelEnd
	pub type Channels<T: Config> = StorageMap<_, Blake2_128Concat, ChannelEndPath, ChannelEnd>;

	#[pallet::storage]
	/// key: connection_id
	/// value: Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, ConnectionId, Vec<(PortId, ChannelId)>>;

	#[pallet::storage]
	/// Key: SeqSendsPath
	/// value: sequence
	pub type NextSequenceSend<T: Config> = StorageMap<_, Blake2_128Concat, SeqSendPath, Sequence>;

	#[pallet::storage]
	/// key: SeqRecvsPath
	/// value: sequence
	pub type NextSequenceRecv<T: Config> = StorageMap<_, Blake2_128Concat, SeqRecvPath, Sequence>;

	#[pallet::storage]
	/// key: SeqAcksPath
	/// value: sequence
	pub type NextSequenceAck<T: Config> = StorageMap<_, Blake2_128Concat, SeqAckPath, Sequence>;

	#[pallet::storage]
	/// key: AcksPath
	/// value: hash of acknowledgement
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, AckPath, IbcAcknowledgementCommitment>;

	#[pallet::storage]
	/// key: ClientId
	/// value: ClientType
	pub type ClientTypeById<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, ClientType>;

	#[pallet::storage]
	/// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// key: ClientId
	/// value: ConnectionId
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, ClientConnectionPath, ConnectionId>;

	#[pallet::storage]
	/// key: ReceiptsPath
	/// value: receipt
	pub type PacketReceipt<T: Config> = StorageMap<_, Blake2_128Concat, ReceiptPath, Receipt>;

	#[pallet::storage]
	/// key: CommitmentsPath
	/// value: hash of (timestamp, height, packet)
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, CommitmentPath, IbcPacketCommitment>;

	#[pallet::storage]
	/// key: height
	/// value: Ibc event height
	pub type IbcEventStore<T: Config> = StorageMap<_, Blake2_128Concat, u64, IbcEvent>;

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
			_messages: Vec<ibc_proto::google::protobuf::Any>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			// let mut ctx = Context::<T>::new();
			// log::info!(
			// 	"☀️ ibc messages type: {:?}",
			// 	messages.iter().map(|v| &v.type_url).collect::<Vec<_>>()
			// );

			// let (events, logs, errors) = messages.into_iter().fold(
			// 	(vec![], vec![], vec![]),
			// 	|(mut events, mut logs, mut errors), msg| {
			// 		match ibc::core::dispatch(&mut ctx, msg) {
			// 			Ok(MsgReceipt { events: temp_events, log: temp_logs }) => {
			// 				events.extend(temp_events);
			// 				logs.extend(temp_logs);
			// 			},
			// 			Err(e) => errors.push(e),
			// 		}
			// 		(events, logs, errors)
			// 	},
			// );
			// log::info!("🙅🙅 deliver ----> events: {:?}", events);
			// log::info!("🙅🙅 🔥 🔥deliver ----> logs: {:?}", logs);
			// log::info!("🙅🙅 ❌❌ deliver ----> errors: {:?}", errors);

			// log::trace!(target: "pallet_ibc", "[pallet_ibc_deliver]: logs: {:?}", logs);
			// log::trace!(target: "pallet_ibc", "[pallet_ibc_deliver]: errors: {:?}", errors);

			// for event in events.clone() {
			// 	<IbcEventStore<T>>::insert(crate::utils::host_height::<T>(), event);
			// }
			// Self::deposit_event(Event::IbcEvents { events });
			// if !errors.is_empty() {
			// 	Self::deposit_event(errors.into());
			// }

			Ok(().into())
		}
	}
}
