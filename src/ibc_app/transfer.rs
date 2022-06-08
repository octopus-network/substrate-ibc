use crate::{context::Context, *};
use log::trace;

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
	fn from(sig: Signer) -> Self {
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
		port_id: &PortId,
		channel_id: IbcChannelId,
	) -> Result<<Self as Ics20Reader>::AccountId, Ics20Error> {
		// let hash = cosmos_adr028_escrow_address(port_id, channel_id);
		// String::from_utf8(hex::encode_upper(hash))
		//     .expect("hex encoded bytes are not valid UTF8")
		//     .parse::<Signer>()
		//     .map_err(Ics20Error::signer)?
		//     .try_into()
		//     .map_err(|_| Ics20Error::parse_account_failure())
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
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function to enable minting ibc tokens to a user account
	fn mint_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function should enable burning of minted tokens in a user account
	fn burn_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}
}
