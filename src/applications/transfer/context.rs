use crate::{context::Context, utils::get_channel_escrow_address, *};
use frame_support::traits::{
	fungibles::{Mutate, Transfer},
	ExistenceRequirement::AllowDeath,
};

use ibc::{
	applications::transfer::{
		context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader},
		error::Error as ICS20Error,
		PrefixedCoin, PORT_ID_STR,
	},
	core::ics24_host::identifier::{ChannelId as IbcChannelId, PortId},
	signer::Signer,
};
use sp_runtime::{
	traits::{CheckedConversion, IdentifyAccount, Verify},
	MultiSignature,
};

// TODO IBC Account ID
type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone)]
pub struct IbcAccount(AccountId);

impl IdentifyAccount for IbcAccount {
	type AccountId = AccountId;
	fn into_account(self) -> Self::AccountId {
		self.0
	}
}

impl TryFrom<Signer> for IbcAccount {
	type Error = &'static str;
	// TODO
	fn try_from(_: Signer) -> Result<Self, Self::Error> {
		Ok(IbcAccount(AccountId::new([0u8; 32])))
	}
}

/// Captures all the dependencies which the ICS20 module requires to be able to dispatch and
/// process IBC messages.
impl<T: Config> Ics20Context for Context<T> {
	type AccountId = <T as Config>::AccountIdConversion;
}

impl<T: Config> Ics20Keeper for Context<T> {
	type AccountId = <T as Config>::AccountIdConversion;
}

impl<T: Config> Ics20Reader for Context<T> {
	type AccountId = <T as Config>::AccountIdConversion;

	/// get_port returns the portID for the transfer module.
	fn get_port(&self) -> Result<PortId, ICS20Error> {
		PortId::from_str(PORT_ID_STR)
			.map_err(|e| ICS20Error::invalid_port_id(PORT_ID_STR.to_string(), e))
	}

	/// Returns the escrow account id for a port and channel combination
	fn get_channel_escrow_address(
		&self,
		port_id: &PortId,
		channel_id: IbcChannelId,
	) -> Result<<Self as Ics20Reader>::AccountId, ICS20Error> {
		get_channel_escrow_address(port_id, channel_id)?
			.try_into()
			.map_err(|_| ICS20Error::parse_account_failure())
	}

	/// Returns true iff send is enabled.
	fn is_send_enabled(&self) -> bool {
		true
	}

	/// Returns true iff receive is enabled.
	fn is_receive_enabled(&self) -> bool {
		true
	}
}

impl<T: Config> BankKeeper for Context<T> {
	type AccountId = <T as Config>::AccountIdConversion;

	/// This function should enable sending ibc fungible tokens from one account to another
	fn send_coins(
		&mut self,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), ICS20Error> {
		let is_native_asset = amt.denom.trace_path().is_empty();
		match is_native_asset {
			// transfer native token
			true => {
				let amount = amt.amount.as_u256().low_u128().checked_into().unwrap(); // TODO: FIX IN THE FUTURE
				let native_token_name = T::NATIVE_TOKEN_NAME;
				let ibc_token_name = amt.denom.base_denom().as_str().as_bytes();

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
					error!("❎ [send_coins] : Error: ({:?})", error);
					ICS20Error::invalid_token()
				})?;
			},
			// transfer non-native token
			false => {
				let amount = amt.amount.as_u256().low_u128().into();
				let denom = amt.denom.base_denom().as_str();
				// look cross chain asset have register in host chain
				match T::AssetIdByName::try_get_asset_id(denom) {
					Ok(token_id) => {
						<T::Assets as Transfer<T::AccountId>>::transfer(
							token_id.into(),
							&from.clone().into_account(),
							&to.clone().into_account(),
							amount,
							true,
						)
						.map_err(|error| {
							error!("❎ [send_coins] : Error: ({:?})", error);
							ICS20Error::invalid_token()
						})?;
					},
					Err(error) => {
						error!("❎ [send_coins]: Error({:?}), denom: ({:?})", error, denom);
						return Err(ICS20Error::invalid_token())
					},
				}
			},
		}

		Ok(())
	}

	/// This function to enable minting ibc tokens to a user account
	fn mint_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), ICS20Error> {
		let amount = amt.amount.as_u256().low_u128().into();
		let denom = amt.denom.base_denom().as_str();
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				<T::Assets as Mutate<T::AccountId>>::mint_into(
					token_id.into(),
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❎ [mint_coins] : Error: ({:?})", error);
					ICS20Error::invalid_token()
				})?;
			},
			Err(error) => {
				error!("❎ [mint_coins]: Error({:?}), denom: ({:?})", error, denom);
				return Err(ICS20Error::invalid_token())
			},
		}
		Ok(())
	}

	/// This function should enable burning of minted tokens in a user account
	fn burn_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), ICS20Error> {
		let amount = amt.amount.as_u256().low_u128().into();
		let denom = amt.denom.base_denom().as_str();
		// look cross chain asset have register in host chain
		match T::AssetIdByName::try_get_asset_id(denom) {
			Ok(token_id) => {
				<T::Assets as Mutate<T::AccountId>>::burn_from(
					token_id.into(),
					&account.clone().into_account(),
					amount,
				)
				.map_err(|error| {
					error!("❎ [burn_coins] : Error: ({:?})", error);
					ICS20Error::invalid_token()
				})?;
			},
			Err(error) => {
				error!("❎ [burn_coins]: Error({:?}), denom: ({:?})", error, denom);
				return Err(ICS20Error::invalid_token())
			},
		}
		Ok(())
	}
}
