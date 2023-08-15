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

use frame_support::pallet_prelude::*;
use frame_system::{ensure_signed, pallet_prelude::*};
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
			ClientStatePath, CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath,
			SeqSendPath,
		},
	},
	MsgEnvelope, RouterError,
};
use ibc_proto::google::protobuf::Any;
use sp_std::{fmt::Debug, vec, vec::Vec};

pub mod context;
pub mod errors;
pub mod prelude;

pub use crate::context::Context;
use crate::prelude::*;
use pallet_ibc_utils::module::AddModule;

pub const TENDERMINT_CLIENT_TYPE: &'static str = "07-tendermint";
pub const SOLOMACHINE_CLIENT_TYPE: &'static str = "06-solomachine";

pub const LOG_TARGET: &str = "runtime::pallet-ibc";

#[frame_support::pallet]
pub mod pallet {
	use super::{errors, *};

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

		const IBC_COMMITMENT_PREFIX: &'static [u8];

		type ExpectedBlockTime: Get<u64>;

		type IbcModule: AddModule;

		type ChainVersion: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// Key: ClientStatePath
	/// value: ClientState
	pub type ClientStates<T: Config> = StorageMap<_, Blake2_128Concat, ClientStatePath, Vec<u8>>;

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
	#[pallet::getter(fn connection_channels)]
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
	/// host block height for ics06 solomachine
	pub type HostHeight<T: Config> = StorageValue<_, Height>;

	#[pallet::storage]
	pub type IbcEventKey<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	pub type IbcEventStorage<T: Config> = StorageValue<_, Vec<IbcEvent>, ValueQuery>;

	#[pallet::storage]
	pub type IbcLogStorage<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T: Send + Sync,
	{
		fn offchain_worker(_n: BlockNumberFor<T>) {
			// clear ibc event offchain key
			for key in IbcEventKey::<T>::get() {
				sp_io::offchain_index::clear(&key);
			}

			// clear Ibc event key
			IbcEventKey::<T>::set(vec![]);
		}
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u64: From<<T as pallet_timestamp::Config>::Moment>
			+ From<<T as frame_system::Config>::BlockNumber>,
	{
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
		pub fn dispatch(origin: OriginFor<T>, messages: Vec<Any>) -> DispatchResultWithPostInfo {
			log::info!("✍️✍️✍️dispatch messages: {:?}", messages);
			let _ = ensure_signed(origin)?;

			<pallet::Pallet<T> as pallet_ibc_utils::Router>::dispatch(messages)?;

			Ok(().into())
		}
	}
}

impl<T: Config> pallet_ibc_utils::Router for Pallet<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment>
		+ From<<T as frame_system::Config>::BlockNumber>,
{
	fn dispatch(messages: Vec<Any>) -> DispatchResult {
		let mut ctx = Context::<T>::new();

		let errors = messages.into_iter().fold(vec![], |mut errors: Vec<RouterError>, msg| {
			let envelope: MsgEnvelope = msg.try_into().unwrap();
			match ibc::core::dispatch(&mut ctx, envelope) {
				Ok(()) => {},
				Err(e) => errors.push(e),
			}
			errors
		});

		// emit ibc event
		// we don't want emit Message event
		Self::deposit_event(Event::IbcEvents { events: IbcEventStorage::<T>::get() });

		// reset
		IbcEventStorage::<T>::put(Vec::<IbcEvent>::new());

		// emit ibc log
		for ibc_log in IbcLogStorage::<T>::get() {
			let logs = String::from_utf8(ibc_log).unwrap();
			log::info!("📔📔📔[pallet_ibc_deliver]: logs: {:?}", logs);
		}
		// reset
		IbcLogStorage::<T>::put(Vec::<Vec<u8>>::new());

		log::info!("🙅🙅🙅[pallet_ibc_deliver]: errors: {:?}", errors);

		if !errors.is_empty() {
			Self::deposit_event(errors.into());
		}
		Ok(())
	}
}
