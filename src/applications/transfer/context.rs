use crate::{context::Context, utils::get_channel_escrow_address, *};

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
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};
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
	type AccountId = IbcAccount;
}

impl<T: Config> Ics20Keeper for Context<T> {
	type AccountId = IbcAccount;
}

impl<T: Config> Ics20Reader for Context<T> {
	type AccountId = IbcAccount;

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
	type AccountId = IbcAccount;

	/// This function should enable sending ibc fungible tokens from one account to another
	fn send_coins(
		&mut self,
		_from: &Self::AccountId,
		_to: &Self::AccountId,
		_amt: &PrefixedCoin,
	) -> Result<(), ICS20Error> {
		todo!()
	}

	/// This function to enable minting ibc tokens to a user account
	fn mint_coins(
		&mut self,
		_account: &Self::AccountId,
		_amt: &PrefixedCoin,
	) -> Result<(), ICS20Error> {
		todo!()
	}

	/// This function should enable burning of minted tokens in a user account
	fn burn_coins(
		&mut self,
		_account: &Self::AccountId,
		_amt: &PrefixedCoin,
	) -> Result<(), ICS20Error> {
		todo!()
	}
}
