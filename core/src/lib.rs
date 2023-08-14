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

use crate::traits::AssetIdAndNameProvider;
use frame_support::pallet_prelude::*;
use frame_support::traits::fungibles::Mutate;
use frame_support::traits::tokens::AssetId;
use frame_support::traits::tokens::Balance as AssetBalance;
use frame_support::traits::Currency;
use frame_support::traits::GenesisBuild;
use frame_system::{ensure_signed, pallet_prelude::*};
use ibc::applications::transfer::msgs::transfer::MsgTransfer;
use ibc::applications::transfer::send_transfer;
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
	MsgEnvelope,
};
use ibc::Signer;
use ibc_proto::google::protobuf::Any;
use sp_runtime::traits::IdentifyAccount;
use sp_std::{fmt::Debug, vec, vec::Vec};

pub mod app;
pub mod client_context;
pub mod constant;
pub mod errors;
pub mod impls;
pub mod prelude;
pub mod router;
pub mod traits;

pub use crate::impls::IbcContext;
use crate::prelude::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use crate::app::transfer::IbcTransferModule;

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

		type ChainVersion: Get<u64>;

		/// The currency type of the runtime
		type Currency: Currency<Self::AccountId>;

		/// Identifier for the class of asset.
		type AssetId: AssetId + MaybeSerializeDeserialize + Default;

		/// The units in which we record balances.
		type AssetBalance: AssetBalance + From<u128> + Into<u128>;

		/// Expose customizable associated type of asset transfer, lock and unlock
		type Fungibles: Mutate<
			Self::AccountId,
			AssetId = Self::AssetId,
			Balance = Self::AssetBalance,
		>;

		/// Map of cross-chain asset ID & name
		type AssetIdByName: AssetIdAndNameProvider<Self::AssetId>;

		/// Account Id Conversion from SS58 string or hex string
		type AccountIdConversion: TryFrom<Signer>
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ Clone
			+ PartialEq
			+ Debug;

		// The native token name
		const NATIVE_TOKEN_NAME: &'static [u8];
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

	type AssetName = Vec<u8>;

	#[pallet::storage]
	/// (asset name) => asset id
	pub type AssetIdByName<T: Config> =
		StorageMap<_, Twox64Concat, AssetName, T::AssetId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub asset_id_by_name: Vec<(String, T::AssetId)>,
	}

	impl<T: Config> core::default::Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { asset_id_by_name: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (token_id, id) in self.asset_id_by_name.iter() {
				<AssetIdByName<T>>::insert(token_id.as_bytes(), id);
			}
		}
	}

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Ibc events
		IbcEvents {
			events: Vec<IbcEvent>,
		},
		/// Ibc errors
		IbcErrors {
			errors: Vec<errors::IbcError>,
		},
		/// transfer ibc token successful
		TransferIbcTokenSuccessful,
		/// transfer ibc token have error
		TransferIbcTokenErr(String),
		// unsupported event
		UnsupportedEvent,
		/// Transfer native token  event
		TransferNativeToken(T::AccountIdConversion, T::AccountIdConversion, BalanceOf<T>),
		/// Transfer non-native token event
		TransferNoNativeToken(
			T::AccountIdConversion,
			T::AccountIdConversion,
			<T as Config>::AssetBalance,
		),
		/// Burn cross chain token event
		BurnToken(T::AssetId, T::AccountIdConversion, T::AssetBalance),
		/// Mint chairperson token event
		MintToken(T::AssetId, T::AccountIdConversion, T::AssetBalance),
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
		// Parser Msg Transfer Error
		ParserMsgTransferError,
		/// Invalid token id
		InvalidTokenId,
		/// Wrong assert id
		WrongAssetId,
		/// other error
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
		+ From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>,

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
		pub fn dispatch(origin: OriginFor<T>, message: Any) -> DispatchResultWithPostInfo {
			log::info!("✍️✍️✍️dispatch messages: {:?}", message);
			let _ = ensure_signed(origin)?;

			let mut ctx = IbcContext::<T>::new();
			let mut router = ctx.router.clone();

    		if let Ok(msg) = MsgEnvelope::try_from(message.clone()) {
                ibc::core::dispatch(&mut ctx, &mut router, msg).unwrap(); // todo
            } else if let Ok(transfer_msg) = MsgTransfer::try_from(message) {
                let mut transfer_module = IbcTransferModule::<T>::new();
                send_transfer(&mut ctx, &mut transfer_module, transfer_msg).unwrap(); // todo
            } else {
                todo!()
            }

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

			Ok(().into())
		}
	}
}

impl<T: Config> AssetIdAndNameProvider<T::AssetId> for Pallet<T> {
	type Err = Error<T>;

	fn try_get_asset_id(name: impl AsRef<[u8]>) -> Result<<T as Config>::AssetId, Self::Err> {
		let asset_id = <AssetIdByName<T>>::try_get(name.as_ref().to_vec());
		match asset_id {
			Ok(id) => Ok(id),
			_ => Err(Error::<T>::InvalidTokenId),
		}
	}

	fn try_get_asset_name(asset_id: T::AssetId) -> Result<Vec<u8>, Self::Err> {
		let token_id = <AssetIdByName<T>>::iter().find(|p| p.1 == asset_id).map(|p| p.0);
		match token_id {
			Some(id) => Ok(id),
			_ => Err(Error::<T>::WrongAssetId),
		}
	}
}
