use crate::{context::Context, *};
use log::trace;

use ibc::{
	applications::transfer::{
		context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader},
		error::Error as Ics20Error,
		PrefixedCoin,
	},
	core::ics24_host::identifier::PortId,
	signer::Signer,
};

impl<T: Config> Ics20Keeper for Context<T> {
	type AccountId = <Self as Ics20Context>::AccountId;
}

impl<T: Config> BankKeeper for Context<T> {
	type AccountId = <Self as Ics20Context>::AccountId;

	fn send_coins(
		&mut self,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	fn mint_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	fn burn_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &PrefixedCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}
}

impl<T: Config> Ics20Reader for Context<T> {
	type AccountId = <Self as Ics20Context>::AccountId;

	fn get_port(&self) -> Result<PortId, Ics20Error> {
		todo!()
	}

	fn get_channel_escrow_address(
		&self,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<Self::AccountId, Ics20Error> {
		todo!()
	}

	fn is_send_enabled(&self) -> bool {
		todo!()
	}

	fn is_receive_enabled(&self) -> bool {
		todo!()
	}
}

impl<T: Config> Ics20Context for Context<T> {
	type AccountId = AccountId32; // Need Setting Account TODO(davirian)
}

pub struct AccountId32;

impl From<Signer> for AccountId32 {
	fn from(_signer: Signer) -> Self {
		Self
	}
}
