use crate::{callback::IbcTransferModule, utils::get_channel_escrow_address, *};
use alloc::string::ToString;
use codec::{Decode, Encode};
use frame_support::traits::{
	fungibles::{Mutate, Transfer},
	ExistenceRequirement::AllowDeath,
};
use ibc::{
	applications::transfer::{
		context::{TokenTransferExecutionContext, TokenTransferValidationContext},
		error::TokenTransferError,
		PrefixedCoin, PORT_ID_STR,
	},
	core::ics24_host::identifier::{ChannelId, PortId},
	signer::Signer,
};
use ibc_support::AssetIdAndNameProvider;
use log::error;
use primitive_types::U256;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{CheckedConversion, IdentifyAccount, Verify},
	MultiSignature,
};
use sp_std::str::FromStr;

impl<T: Config> TokenTransferExecutionContext for IbcTransferModule<T> {
	// type AccountId = <Self as TokenTransferContext>::AccountId;

	fn send_coins_execute(
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
					error!("❌ [send_coins] : Error: ({:?})", error);
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
				// look cross chain asset have register in host chain
				match T::AssetIdByName::try_get_asset_id(denom) {
					Ok(token_id) => {
						<T::Fungibles as Transfer<T::AccountId>>::transfer(
							token_id,
							&from.clone().into_account(),
							&to.clone().into_account(),
							amount,
							true,
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
						return Err(TokenTransferError::InvalidToken)
					},
				}
			},
		}

		Ok(())
	}

	fn mint_coins_execute(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		let amount = U256::from(amt.amount).low_u128().into();
		let denom = amt.denom.base_denom.as_str();
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				<T::Fungibles as Mutate<T::AccountId>>::mint_into(
					token_id,
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❌ [mint_coins] : Error: ({:?})", error);
					TokenTransferError::InvalidToken
				})?;

				// add mint token event
				Pallet::<T>::deposit_event(Event::<T>::MintToken(
					token_id,
					account.clone(),
					amount,
				));
			},
			Err(_error) => {
				error!("❌ [mint_coins]: denom: ({:?})", denom);
				return Err(TokenTransferError::InvalidToken)
			},
		}
		Ok(())
	}

	fn burn_coins_execute(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		let amount = U256::from(amt.amount).low_u128().into();
		let denom = amt.denom.base_denom.as_str();
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				<T::Fungibles as Mutate<T::AccountId>>::burn_from(
					token_id,
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❌ [burn_coins] : Error: ({:?})", error);
					TokenTransferError::InvalidToken
				})?;

				// add burn token event
				Pallet::<T>::deposit_event(Event::<T>::BurnToken(
					token_id,
					account.clone(),
					amount,
				));
			},
			Err(_error) => {
				error!("❌ [burn_coins]: denom: ({:?})", denom);
				return Err(TokenTransferError::InvalidToken)
			},
		}
		Ok(())
	}
}

impl<T: Config> TokenTransferValidationContext for IbcTransferModule<T> {
	type AccountId = <T as Config>::AccountIdConversion;

	fn get_port(&self) -> Result<PortId, TokenTransferError> {
		PortId::from_str(PORT_ID_STR).map_err(|e| TokenTransferError::InvalidPortId {
			context: PORT_ID_STR.to_string(),
			validation_error: e,
		})
	}

	// todo
	fn send_coins_validate(
		&self,
		_from_account: &Self::AccountId,
		_to_account: &Self::AccountId,
		_coin: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		Ok(())
	}

	// todo
	fn mint_coins_validate(
		&self,
		_account: &Self::AccountId,
		_coin: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		Ok(())
	}

	// todo
	fn burn_coins_validate(
		&self,
		_account: &Self::AccountId,
		_coin: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		Ok(())
	}

	fn get_escrow_account(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Self::AccountId, TokenTransferError> {
		get_channel_escrow_address(port_id, channel_id)?
			.try_into()
			.map_err(|_| TokenTransferError::ParseAccountFailure)
	}

	fn can_send_coins(&self) -> Result<(), TokenTransferError> {
		Ok(())
	}

	fn can_receive_coins(&self) -> Result<(), TokenTransferError> {
		Ok(())
	}
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
