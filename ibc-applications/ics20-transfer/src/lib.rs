#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::traits::Currency;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod ics20_callback;
pub mod ics20_context_channel;
pub mod ics20_impl;
pub mod utils;

use ibc_support::AssetIdAndNameProvider;

pub const LOG_TARGET: &str = "runtime::pallet-ics20-transfer";
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{ics20_callback::IbcTransferModule, LOG_TARGET};
	use alloc::string::{String, ToString};
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungibles::{Mutate, Transfer},
			tokens::{AssetId, Balance as AssetBalance},
			Currency,
		},
	};
	use frame_system::pallet_prelude::*;
	use ibc::{
		applications::transfer::msgs::transfer::MsgTransfer,
		events::IbcEvent,
		handler::{HandlerOutput, HandlerOutputBuilder},
		signer::Signer,
	};
	use ibc_support::AssetIdAndNameProvider;
	use sp_runtime::traits::IdentifyAccount;
	use sp_std::{fmt::Debug, vec::Vec};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Sync + Send + Debug {
		/// The aggregated event type of the runtime.
		type RuntimeEvent: Parameter
			+ Member
			+ From<Event<Self>>
			+ Debug
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type of the runtime
		type Currency: Currency<Self::AccountId>;

		/// Identifier for the class of asset.
		type AssetId: AssetId + MaybeSerializeDeserialize + Default;

		/// The units in which we record balances.
		type AssetBalance: AssetBalance + From<u128> + Into<u128>;

		/// Expose customizable associated type of asset transfer, lock and unlock
		type Fungibles: Transfer<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>;

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

		/// IbcContext need to implements to ics20
		type IbcContext: ibc_support::ibc_trait::IbcSupportChannelKeeper
			+ ibc_support::ibc_trait::IbcSupportChannelReader;
	}

	type AssetName = Vec<u8>;

	#[pallet::storage]
	/// (asset name) => asset id
	pub type AssetIdByName<T: Config> =
		StorageMap<_, Twox64Concat, AssetName, T::AssetId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub asset_id_by_name: Vec<(String, T::AssetId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
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

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Send packet event
		SendPacket {
			sequence: Vec<u8>,
			source_port: Vec<u8>,
			source_channel: Vec<u8>,
			destination_port: Vec<u8>,
			destination_channel: Vec<u8>,
			data: Vec<u8>,
			timeout_height: Vec<u8>,
			timeout_timestamp: Vec<u8>,
		},
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

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// Parser Msg Transfer Error
		ParserMsgTransferError,
		/// Invalid token id
		InvalidTokenId,
		/// Wrong assert id
		WrongAssetId,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// ICS20 fungible token transfer.
		/// Handling transfer request as sending chain or receiving chain.
		///
		/// Parameters:
		/// - `messages`: A serialized protocol buffer message containing the transfer request.
		///
		/// The relevant events are emitted when successful.
		#[pallet::weight(0)]
		pub fn raw_transfer(
			origin: OriginFor<T>,
			messages: Vec<ibc_support::Any>,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;
			let mut ctx = IbcTransferModule(PhantomData::<T>);

			let messages: Vec<ibc_proto::google::protobuf::Any> = messages
				.into_iter()
				.map(|message| ibc_proto::google::protobuf::Any {
					type_url: String::from_utf8(message.type_url.clone()).unwrap(),
					value: message.value,
				})
				.collect();

			log::trace!(
				target: LOG_TARGET,
				"raw_transfer : {:?} ",
				messages.iter().map(|message| message.type_url.clone()).collect::<Vec<_>>()
			);

			for message in messages {
				let mut handle_out = HandlerOutputBuilder::new();
				let msg_transfer = MsgTransfer::try_from(message)
					.map_err(|_| Error::<T>::ParserMsgTransferError)?;
				let result = ibc::applications::transfer::relay::send_transfer::send_transfer(
					&mut ctx,
					&mut handle_out,
					msg_transfer,
				);
				match result {
					Ok(_value) => {
						log::trace!(target: LOG_TARGET, "raw_transfer Successful!");
					},
					Err(error) => {
						log::trace!(target: LOG_TARGET, "raw_transfer Error : {:?} ", error);
					},
				}

				let HandlerOutput::<()> { result: _, log, events } = handle_out.with_result(());

				log::trace!(target: LOG_TARGET, "raw_transfer log : {:?} ", log);

				// deposit events about send packet event and ics20 transfer event
				for event in events {
					log::trace!(target: LOG_TARGET, "raw_transfer event : {:?} ", event);
					match event {
						IbcEvent::SendPacket(ref send_packet) => {
							let sequence =
								send_packet.packet.sequence.to_string().as_bytes().to_vec();
							let source_port =
								send_packet.packet.source_port.to_string().as_bytes().to_vec();
							let source_channel =
								send_packet.packet.source_channel.to_string().as_bytes().to_vec();
							let destination_port =
								send_packet.packet.destination_port.to_string().as_bytes().to_vec();
							let destination_channel = send_packet
								.packet
								.destination_channel
								.to_string()
								.as_bytes()
								.to_vec();
							let data = send_packet.packet.data.clone();
							let timeout_height =
								send_packet.packet.timeout_height.to_string().as_bytes().to_vec();
							let timeout_timestamp = send_packet
								.packet
								.timeout_timestamp
								.to_string()
								.as_bytes()
								.to_vec();
							Self::deposit_event(Event::SendPacket {
								sequence,
								source_port,
								source_channel,
								destination_port,
								destination_channel,
								data,
								timeout_height,
								timeout_timestamp,
							});
						},
						_ => {
							Self::deposit_event(Event::UnsupportedEvent);
						},
					}
				}
			}

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
