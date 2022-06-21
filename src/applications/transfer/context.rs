use crate::{context::Context, *};

use ibc::{
	applications::transfer::{
		context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader},
		error::Error as Ics20Error,
		PrefixedCoin, PrefixedDenom,
	},
	core::ics24_host::identifier::{ChannelId as IbcChannelId, PortId},
	signer::Signer,
};

pub struct MocKAccountId;

impl From<Signer> for MocKAccountId {
	fn from(_sig: Signer) -> Self {
		MocKAccountId
	}
}

impl<T: Config> Ics20Context for Context<T> {
	type AccountId = MocKAccountId; // MOCK
}

impl<T: Config> Ics20Keeper for Context<T> {
	type AccountId = MocKAccountId;
}

impl<T: Config> Ics20Reader for Context<T> {
	type AccountId = MocKAccountId;

	/// get_port returns the portID for the transfer module.
	fn get_port(&self) -> Result<PortId, Ics20Error> {
		todo!()
	}

	/// Returns the escrow account id for a port and channel combination
	fn get_channel_escrow_address(
		&self,
		_port_id: &PortId,
		_channel_id: IbcChannelId,
	) -> Result<<Self as Ics20Reader>::AccountId, Ics20Error> {
		todo!()
	}

	/// Returns true iff send is enabled.
	fn is_send_enabled(&self) -> bool {
		todo!()
	}

	/// Returns true iff receive is enabled.
	fn is_receive_enabled(&self) -> bool {
		todo!()
	}

	/// Returns a hash of the prefixed denom.
	/// Implement only if the host chain supports hashed denominations.
	fn denom_hash_string(&self, _denom: &PrefixedDenom) -> Option<String> {
		todo!()
	}
}

impl<T: Config> BankKeeper for Context<T> {
	type AccountId = MocKAccountId;

	/// This function should enable sending ibc fungible tokens from one account to another
	fn send_coins(
		&mut self,
		_from: &Self::AccountId,
		_to: &Self::AccountId,
		_amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function to enable minting ibc tokens to a user account
	fn mint_coins(
		&mut self,
		_account: &Self::AccountId,
		_amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function should enable burning of minted tokens in a user account
	fn burn_coins(
		&mut self,
		_account: &Self::AccountId,
		_amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}
}
