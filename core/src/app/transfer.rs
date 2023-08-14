use core::fmt::Debug;
use ibc::applications::transfer::VERSION;
use ibc::{
	applications::transfer::{
		context::{
			cosmos_adr028_escrow_address, TokenTransferExecutionContext,
			TokenTransferValidationContext,
		},
		error::TokenTransferError,
		PrefixedCoin,
	},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::{ChannelError, PacketError},
			packet::Packet,
			Version as ChannelVersion,
		},
		ics24_host::identifier::{ChannelId, ConnectionId},
		router::{Module as IbcModule, ModuleExtras},
	},
};
use ibc::{
	core::{ics04_channel::acknowledgement::Acknowledgement, ics24_host::identifier::PortId},
	Signer,
};

use ibc::applications::transfer::context::{
	on_acknowledgement_packet_validate, on_chan_open_ack_validate, on_chan_open_confirm_validate,
	on_chan_open_init_execute, on_chan_open_init_validate, on_chan_open_try_execute,
	on_chan_open_try_validate, on_recv_packet_execute, on_timeout_packet_execute,
	on_timeout_packet_validate,
};

use crate::traits::AssetIdAndNameProvider;
use crate::Config;
use crate::Event;
use crate::Pallet;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::traits::fungibles::Mutate;
use frame_support::traits::tokens::Fortitude;
use frame_support::traits::tokens::Precision;
use frame_support::traits::tokens::Preservation;
use frame_support::traits::Currency;
use frame_support::traits::ExistenceRequirement::AllowDeath;
use log::error;
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::traits::CheckedConversion;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::Verify;
use sp_runtime::MultiSignature;

#[derive(Clone, Debug)]
pub struct IbcTransferModule<T> {
	pub phatom_data: PhantomData<T>,
}

impl<T: Config> IbcTransferModule<T> {
	pub fn new() -> Self {
		Self { phatom_data: PhantomData }
	}
}

impl<T: Config> crate::router::IbcModule for IbcTransferModule<T> {}

impl<T: Config> IbcModule for IbcTransferModule<T> {
	fn on_chan_open_init_validate(
		&self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &ChannelVersion,
	) -> Result<ChannelVersion, ChannelError> {
		on_chan_open_init_validate(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
		)
		.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })?;
		Ok(ChannelVersion::new(VERSION.to_string()))
	}

	fn on_chan_open_init_execute(
		&mut self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &ChannelVersion,
	) -> Result<(ModuleExtras, ChannelVersion), ChannelError> {
		on_chan_open_init_execute(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
		)
		.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_try_validate(
		&self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		counterparty_version: &ChannelVersion,
	) -> Result<ChannelVersion, ChannelError> {
		on_chan_open_try_validate(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			counterparty_version,
		)
		.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })?;
		Ok(ChannelVersion::new(VERSION.to_string()))
	}

	fn on_chan_open_try_execute(
		&mut self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		counterparty_version: &ChannelVersion,
	) -> Result<(ModuleExtras, ChannelVersion), ChannelError> {
		on_chan_open_try_execute(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			counterparty_version,
		)
		.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_ack_validate(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty_version: &ChannelVersion,
	) -> Result<(), ChannelError> {
		on_chan_open_ack_validate(self, port_id, channel_id, counterparty_version)
			.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_ack_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty_version: &ChannelVersion,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_open_confirm_validate(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		on_chan_open_confirm_validate(self, port_id, channel_id)
			.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_confirm_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_close_init_validate(
		&self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		Ok(())
	}

	fn on_chan_close_init_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_close_confirm_validate(
		&self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		Ok(())
	}

	fn on_chan_close_confirm_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_recv_packet_execute(
		&mut self,
		packet: &Packet,
		_relayer: &Signer,
	) -> (ModuleExtras, Acknowledgement) {
		on_recv_packet_execute(self, packet)
	}

	fn on_acknowledgement_packet_validate(
		&self,
		packet: &Packet,
		acknowledgement: &Acknowledgement,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		on_acknowledgement_packet_validate(self, packet, acknowledgement, relayer)
			.map_err(|e: TokenTransferError| PacketError::AppModule { description: e.to_string() })
	}

	fn on_acknowledgement_packet_execute(
		&mut self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
		_relayer: &Signer,
	) -> (ModuleExtras, Result<(), PacketError>) {
		(ModuleExtras::empty(), Ok(()))
	}

	/// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback
	fn on_timeout_packet_validate(
		&self,
		packet: &Packet,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		on_timeout_packet_validate(self, packet, relayer)
			.map_err(|e: TokenTransferError| PacketError::AppModule { description: e.to_string() })
	}

	/// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback
	fn on_timeout_packet_execute(
		&mut self,
		packet: &Packet,
		relayer: &Signer,
	) -> (ModuleExtras, Result<(), PacketError>) {
		let res = on_timeout_packet_execute(self, packet, relayer);
		(
			res.0,
			res.1.map_err(|e: TokenTransferError| PacketError::AppModule {
				description: e.to_string(),
			}),
		)
	}
}

impl<T: Config> TokenTransferExecutionContext for IbcTransferModule<T> {
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
					token_id.clone(),
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
				return Err(TokenTransferError::InvalidToken);
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
					token_id.clone(),
					&account.clone().into_account(),
					amount,
					Precision::BestEffort,
					Fortitude::Force,
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
				return Err(TokenTransferError::InvalidToken);
			},
		}
		Ok(())
	}
}

impl<T: Config> TokenTransferValidationContext for IbcTransferModule<T> {
	type AccountId = <T as Config>::AccountIdConversion;

	fn get_port(&self) -> Result<PortId, TokenTransferError> {
		Ok(PortId::transfer())
	}

	fn get_escrow_account(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Self::AccountId, TokenTransferError> {
		let data = cosmos_adr028_escrow_address(port_id, channel_id);
		let mut hex_string = hex::encode_upper(data);
		hex_string.insert_str(0, "0x");

		Signer::from(hex_string)
			.try_into()
			.map_err(|_| TokenTransferError::ParseAccountFailure)
	}

	fn send_coins_validate(
		&self,
		_from_account: &Self::AccountId,
		_to_account: &Self::AccountId,
		_coin: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		// Architectures that don't use `dispatch()` and care about the
		// distinction between `validate()` and `execute()` would want to check
		// that we can also send the coins between the 2 accounts.
		// However we use `dispatch()` and simply do all our checks in the `execute()` phase.
		Ok(())
	}

	fn mint_coins_validate(
		&self,
		_account: &Self::AccountId,
		_coin: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		// Architectures that don't use `dispatch()` and care about the
		// distinction between `validate()` and `execute()` would want to check
		// that we can also send the coins between the 2 accounts.
		// However we use `dispatch()` and simply do all our checks in the `execute()` phase.
		Ok(())
	}

	fn burn_coins_validate(
		&self,
		_account: &Self::AccountId,
		_coin: &PrefixedCoin,
	) -> Result<(), TokenTransferError> {
		// Architectures that don't use `dispatch()` and care about the
		// distinction between `validate()` and `execute()` would want to check
		// that we can also send the coins between the 2 accounts.
		// However we use `dispatch()` and simply do all our checks in the `execute()` phase.
		Ok(())
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

/// In ICS20 fungible token transfer, get the escrow address by channel ID and port ID
///
/// Parameters:
/// - `port_id`: The ID of the port corresponding to the escrow.
/// - `channel_id`: The ID of the channel corresponding to the escrow.
pub fn get_channel_escrow_address(
	port_id: &PortId,
	channel_id: &ChannelId,
) -> Result<Signer, TokenTransferError> {
	let contents = format!("{}/{}", port_id, channel_id);
	let mut data = VERSION.as_bytes().to_vec();
	data.extend_from_slice(&[0]);
	data.extend_from_slice(contents.as_bytes());

	let hash = sp_io::hashing::sha2_256(&data).to_vec();
	let mut hex_string = hex::encode_upper(hash);
	hex_string.insert_str(0, "0x");
	Ok(Signer::from(hex_string))
}
