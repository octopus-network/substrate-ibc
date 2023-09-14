#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use frame_support::traits::Currency;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod callback;
pub mod context_channel;
pub mod denom;
pub mod impls;
pub mod utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

use pallet_ibc_utils::AssetIdAndNameProvider;
use sp_std::vec::Vec;

pub const LOG_TARGET: &str = "runtime::pallet-ics20-transfer";
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::callback::IbcTransferModule;
	// use crate::{callback::IbcTransferModule, LOG_TARGET};
	use alloc::string::String;
	use crate::alloc::string::ToString;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungibles::{Balanced, Mutate},
			tokens::{AssetId, Balance as AssetBalance},
			Currency,
		},
	};
	use frame_system::pallet_prelude::*;
	use ibc::{
		applications::transfer::denom::PrefixedDenom as IbcPrefixedDenom,
		// applications::transfer::error::TokenTransferError,
		applications::transfer::msgs::transfer::MsgTransfer,
		core::ics04_channel::events::SendPacket as IbcSendPacket,
		events::IbcEvent,
		handler::{HandlerOutput, HandlerOutputBuilder},
		signer::Signer,
	};
	use pallet_ibc_utils::AssetIdAndNameProvider;
	use sp_runtime::traits::IdentifyAccount;
	use sp_std::{fmt::Debug, vec::Vec};
	// use utils::SendPacket;
	#[pallet::pallet]
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
		type Fungibles: Balanced<Self::AccountId>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>;

		/// Map of cross-chain asset ID & name
		type AssetIdByName: AssetIdAndNameProvider<Self::AssetId>;

		/// Account Id Conversion from SS58 string or hex string
		type AccountIdConversion: TryFrom<Signer>
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ Clone
			+ PartialEq
			+ Debug;

		type IbcContext: pallet_ibc_utils::traits::ChannelKeeperInterface
			+ pallet_ibc_utils::traits::ChannelReaderInterface;

		// The native token name
		const NATIVE_TOKEN_NAME: &'static [u8];
	}

	type AssetName = Vec<u8>;

	#[pallet::storage]
	/// (asset name) => asset id
	pub type AssetIdByName<T: Config> =
		StorageMap<_, Twox64Concat, AssetName, T::AssetId, ValueQuery>;

	#[pallet::storage]
	// key: denom trace hash
	// value: denom trace
	pub type DenomTrace<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, denom::PrefixedDenom>;

	#[pallet::storage]
	/// key: height
	/// value: Ibc event height
	pub type SendPacketStore<T: Config> = StorageMap<_, Blake2_128Concat, u64, IbcEvent>;
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
		SendPacket(IbcSendPacket),
		// SendPacket(SendPacket),
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
		///
		DecodeStringFailed,
		///  transfer error
		TokenTransferError,
		/// not found denom trace
		DenomTraceNotFound,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u64: From<<T as frame_system::Config>::BlockNumber>,
	{
		/// ICS20 fungible token transfer.
		/// Handling transfer request as sending chain or receiving chain.
		///
		/// Parameters:
		/// - `messages`: A serialized protocol buffer message containing the transfer request.
		///
		/// The relevant events are emitted when successful.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn raw_transfer(
			origin: OriginFor<T>,
			messages: Vec<ibc_proto::google::protobuf::Any>,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;
			let mut ctx: IbcTransferModule<T> = IbcTransferModule(PhantomData::<T>);

			for message in messages {
				let mut handle_out = HandlerOutputBuilder::new();
				let raw_msg_transfer = MsgTransfer::try_from(message)
					.map_err(|_| Error::<T>::ParserMsgTransferError)?;
				log::info!(
					"🐙🐙 pallet_ics20_transfer -> raw_transfer raw msg transfer: {:?}",
					raw_msg_transfer
				);
				// convert denom trace
				let raw_denom = raw_msg_transfer.clone().token.denom;
				let new_transfer_msg = match raw_denom.starts_with("ibc/") {
					true => {
						let denom_trace_hash = raw_denom.as_bytes();
						let full_denom_path = ctx.get_denom_trace(denom_trace_hash)?;
						let ibc_prefixed_denom: IbcPrefixedDenom = full_denom_path.into();
						log::info!(
							"🐙🐙 pallet_ics20_transfer -> raw_transfer ibc_prefixed_denom: {:?}",
							ibc_prefixed_denom
						);

						let coin = ibc_proto::cosmos::base::v1beta1::Coin {
							denom: ibc_prefixed_denom.to_string(),
							amount: raw_msg_transfer.token.amount,
						};
						MsgTransfer {
							port_on_a: raw_msg_transfer.port_on_a,
							chan_on_a: raw_msg_transfer.chan_on_a,
							token: coin,
							sender: raw_msg_transfer.sender,
							receiver: raw_msg_transfer.receiver,
							timeout_height_on_b: raw_msg_transfer.timeout_height_on_b,
							timeout_timestamp_on_b: raw_msg_transfer.timeout_timestamp_on_b,
						}
					},
					false => raw_msg_transfer.clone(),
				};
				log::info!(
					"🐙🐙 pallet_ics20_transfer -> raw_transfer new_transfer_msg: {:?}",
					new_transfer_msg
				);

				let result = ibc::applications::transfer::relay::send_transfer::send_transfer(
					&mut ctx,
					&mut handle_out,
					new_transfer_msg,
				);
				match result {
					Ok(_value) => {
						let HandlerOutput::<()> { result: _, log, events } =
							handle_out.with_result(());

						log::info!("🐙🐙 pallet_ics20_transfer -> raw_transfer log : {:?} ", log);

						// deposit events about send packet event and ics20 transfer event
						for event in events {
							log::info!(
								"🐙🐙 pallet_ics20_transfer -> raw_transfer event : {:?} ",
								event
							);

							match event {
								IbcEvent::SendPacket(ref ibc_send_packet) => {
									log::info!("🐙🐙 pallet_ics20_transfer -> covert ibc_send_packet to pallet send_packet : {:?} ", ibc_send_packet);
									Self::deposit_event(Event::SendPacket(ibc_send_packet.clone()));
									let block_height = <frame_system::Pallet<T>>::block_number();
									// save event
									<SendPacketStore<T>>::insert(u64::from(block_height), event);
								},
								_ => {},
							}
						}
					},
					Err(error) => {
						log::error!(
							"🐙🐙 pallet_ics20_transfer -> raw_transfer Error : {:?} ",
							error
						);
						return Err(Error::<T>::TokenTransferError.into());
					},
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
