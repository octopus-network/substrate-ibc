use crate::{callback::IbcTransferModule, utils::get_channel_escrow_address, *};
use alloc::{
	format,
	string::{String, ToString},
};
use codec::{Decode, Encode};
use frame_support::traits::{
	fungibles::Mutate,
	tokens::{Fortitude, Precision, Preservation},
	ExistenceRequirement::AllowDeath,
};
use ibc::{
	applications::transfer::{
		context::{BankKeeper, TokenTransferContext, TokenTransferReader},
		error::TokenTransferError,
		PrefixedCoin, PrefixedDenom, PORT_ID_STR,
	},
	core::ics24_host::identifier::{ChannelId, PortId},
	signer::Signer,
};
use log::error;
use pallet_ibc_utils::AssetIdAndNameProvider;
use primitive_types::U256;
use scale_info::TypeInfo;
use sha2::{Digest, Sha256};
use sp_runtime::{
	traits::{CheckedConversion, IdentifyAccount, Verify},
	MultiSignature,
};
use sp_std::str::FromStr;

impl<T: Config> BankKeeper for IbcTransferModule<T> {
	type AccountId = <Self as TokenTransferContext>::AccountId;

	fn send_coins(
		&mut self,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		let is_native_asset = amt.denom.trace_path.is_empty();
		match is_native_asset {
			// transfer native token
			true => {
				let amount = U256::from(amt.amount).low_u128().checked_into().unwrap(); // TODO: FIX IN THE FUTURE
				let native_token_name = T::NATIVE_TOKEN_NAME;
				let ibc_token_name = amt.denom.base_denom.as_str().as_bytes();
				log::info!(
					"🐙🐙 pallet_ics20_transfer::impls -> send_coins native_token_name: {:?} amount:{:?}",
					native_token_name,amount
				);
				// assert native token name equal want to send ibc token name
				assert_eq!(
					native_token_name, ibc_token_name,
					"send ibc token name is not native token name"
				);

				<T::Currency as Currency<T::AccountId>>::transfer(
					&from.clone().into_account(),
					&to.clone().into_account(),
					amount,
					AllowDeath,
				)
				.map_err(|error| {
					error!(
						"❌ [send_coins] : Error: ({:?}), from: {:?}, to: {:?}",
						error, from, to
					);
					TokenTransferError::InvalidToken
				})?;

				// add emit transfer native token event
				Pallet::<T>::deposit_event(Event::<T>::TransferNativeToken(
					from.clone(),
					to.clone(),
					amount,
				))
			},
			// transfer non-native token
			false => {
				let amount = U256::from(amt.amount).low_u128().into();
				let denom = amt.denom.base_denom.as_str();
				log::info!(
					"🐙🐙 pallet_ics20_transfer::impls -> send_coins transfer non-native token: {:?} amount:{:?}",
					denom,amount
				);
				// look cross chain asset have register in host chain
				match T::AssetIdByName::try_get_asset_id(denom) {
					Ok(token_id) => {
						log::info!(
							"🐙🐙 pallet_ics20_transfer::impls -> send_coins asset id: {:?} asset name:{:?}",
							token_id,denom
						);
						<T::Fungibles as Mutate<T::AccountId>>::transfer(
							token_id,
							&from.clone().into_account(),
							&to.clone().into_account(),
							amount,
							Preservation::Preserve,
						)
						.map_err(|error| {
							error!("❌ [send_coins] : Error: ({:?})", error);
							TokenTransferError::InvalidToken
						})?;

						// add emit transfer no native token event
						Pallet::<T>::deposit_event(Event::<T>::TransferNoNativeToken(
							from.clone(),
							to.clone(),
							amount,
						));
					},
					Err(_error) => {
						error!("❌ [send_coins]: denom: ({:?})", denom);
						return Err(TokenTransferError::InvalidToken);
					},
				}
			},
		}

		Ok(())
	}

	fn mint_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		let amount = U256::from(amt.amount).low_u128().into();

		let denom_trace_hash = self.denom_hash_string(&amt.denom).unwrap();
		let denom_trace = crate::denom::PrefixedDenom::from(amt.denom.clone());
		log::info!(
			"🐙🐙 pallet_ics20_transfer::impls -> mint_coins  denom_trace_hash:{:?}, denom_trace: {:?} ",
			denom_trace_hash,denom_trace
		);
		// insert denom trace hash, and demo_trace
		<DenomTrace<T>>::insert(denom_trace_hash.as_bytes().to_vec(), denom_trace);
		// look cross chain asset have register in host chain
		let denom: &str = amt.denom.base_denom.as_str();
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				log::info!(
					"🐙🐙 pallet_ics20_transfer::impls -> mint_coins asset id: {:?} asset name:{:?}",
					token_id,denom
				);
				<T::Fungibles as Mutate<T::AccountId>>::mint_into(
					token_id.clone(),
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❌ [mint_coins] : Error: ({:?})", error);
					TokenTransferError::InvalidToken
				})?;
				log::info!("🐙🐙 pallet_ics20_transfer::impls -> token_id: {:?}, mint_into: {:?}, amount: {:?}", token_id,account.clone().into_account(),amount);
				// add mint token event
				Pallet::<T>::deposit_event(Event::<T>::MintToken(
					token_id,
					account.clone(),
					amount,
				));
			},
			Err(_error) => {
				error!("❌ [mint_coins]: denom: ({:?})", denom);
				return Err(TokenTransferError::InvalidToken);
			},
		}
		Ok(())
	}

	fn burn_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		let amount = U256::from(amt.amount).low_u128().into();
		let denom = amt.denom.base_denom.as_str();
		log::info!(
			"🐙🐙 pallet_ics20_transfer::impls -> burn_coins denom: {:?}, amount: {:?} ",
			denom,
			amount
		);
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				log::info!(
					"🐙🐙 pallet_ics20_transfer::impls -> burn_coins asset id: {:?} asset name:{:?}",
					token_id,denom
				);
				<T::Fungibles as Mutate<T::AccountId>>::burn_from(
					token_id.clone(),
					&account.clone().into_account(),
					amount,
					Precision::Exact,
					Fortitude::Polite,
				)
				.map_err(|error| {
					error!("❌ [burn_coins] : Error: ({:?})", error);
					TokenTransferError::InvalidToken
				})?;
				log::info!("🐙🐙 pallet_ics20_transfer::impls ->burn_coins token_id: {:?}, burn_from: {:?}, amount: {:?}", token_id,account.clone().into_account(),amount);

				// add burn token event
				Pallet::<T>::deposit_event(Event::<T>::BurnToken(
					token_id,
					account.clone(),
					amount,
				));
			},
			Err(_error) => {
				error!("❌ [burn_coins]: denom: ({:?})", denom);
				return Err(TokenTransferError::InvalidToken);
			},
		}
		Ok(())
	}
}

impl<T: Config> TokenTransferReader for IbcTransferModule<T> {
	type AccountId = <Self as TokenTransferContext>::AccountId;

	fn get_port(&self) -> Result<PortId, TokenTransferError> {
		PortId::from_str(PORT_ID_STR).map_err(|e| TokenTransferError::InvalidPortId {
			context: PORT_ID_STR.to_string(),
			validation_error: e,
		})
	}

	fn get_channel_escrow_address(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Self::AccountId, TokenTransferError> {
		get_channel_escrow_address(port_id, channel_id)?
			.try_into()
			.map_err(|_| TokenTransferError::ParseAccountFailure)
	}

	fn is_send_enabled(&self) -> bool {
		// TODO(davirain), need according channelEnd def
		true
	}

	fn is_receive_enabled(&self) -> bool {
		// TODO(davirain), need according channelEnd def
		true
	}
	/// Returns a hash of the prefixed denom.
	/// Implement only if the host chain supports hashed denominations.
	fn denom_hash_string(&self, denom: &PrefixedDenom) -> Option<String> {
		use subtle_encoding::hex;
		let mut hasher = Sha256::new();
		let prefix_denom = alloc::format!("{:?}", &denom);
		log::info!("🐙🐙 pallet_ics20_transfer::impls -> denom:{:?} to denom str:{:?}", denom,prefix_denom);
		hasher.update(prefix_denom.as_bytes());

		let denom_bytes = hasher.finalize();
		let denom_hex = String::from_utf8(hex::encode_upper(denom_bytes))
			.map_err(|e| TokenTransferError::UnknownMsgType { msg_type: format!("error: {}", e) })
			.unwrap();

		let denom_str = format!("ibc/{}", denom_hex);
		log::info!(
			"🐙🐙 pallet_ics20_transfer::impls ->PrefixedDenom:{:?} to denom_hash_string : {:?} ",
			denom,
			denom_str
		);
		Some(denom_str)
	}
}

impl<T: Config> TokenTransferContext for IbcTransferModule<T> {
	type AccountId = <T as Config>::AccountIdConversion;
}

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone, Debug, PartialEq, TypeInfo, Encode, Decode)]
pub struct IbcAccount(AccountId);

impl IdentifyAccount for IbcAccount {
	type AccountId = AccountId;
	fn into_account(self) -> Self::AccountId {
		self.0
	}
}

impl TryFrom<Signer> for IbcAccount
where
	AccountId: From<[u8; 32]>,
{
	type Error = &'static str;

	/// Convert a signer to an IBC account.
	/// Only valid hex strings are supported for now.
	fn try_from(signer: Signer) -> Result<Self, Self::Error> {
		let acc_str = signer.as_ref();
		if acc_str.starts_with("0x") {
			match acc_str.strip_prefix("0x") {
				Some(hex_string) => TryInto::<[u8; 32]>::try_into(
					hex::decode(hex_string).map_err(|_| "Error decoding invalid hex string")?,
				)
				.map_err(|_| "Invalid account id hex string")
				.map(|acc| Self(acc.into())),
				_ => Err("Signer does not hold a valid hex string"),
			}
		}
		// Do SS58 decoding instead
		else {
			error!("Convert Signer ❌ : Failed! ");
			Err("invalid ibc address or substrate address")
		}
	}
}

impl<T: Config> IbcTransferModule<T> {
	// GetDenomTrace retreives the full identifiers trace and base denomination from the store.
	pub fn get_denom_trace(
		&self,
		denom_trace_hash: &[u8],
	) -> Result<crate::denom::PrefixedDenom, Error<T>> {
		<DenomTrace<T>>::get(denom_trace_hash).ok_or(Error::<T>::DenomTraceNotFound)

	}

	// HasDenomTrace checks if a the key with the given denomination trace hash exists on the store.
	pub fn has_denom_trace(&self, denom_trace_hash: &[u8]) -> bool {
		<DenomTrace<T>>::contains_key(denom_trace_hash)
	}

	// SetDenomTrace sets a new {trace hash -> denom trace} pair to the store.
	pub fn set_denom_trace(
		&self,
		denom_trace_hash: &[u8],
		denom: crate::denom::PrefixedDenom,
	) -> Result<(), Error<T>> {
		<DenomTrace<T>>::insert(denom_trace_hash, denom);
		Ok(())
	}

	pub fn get_port(&self) -> Result<PortId, Error<T>> {
		Ok(PortId::transfer())
	}
}
