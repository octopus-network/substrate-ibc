#![cfg_attr(not(feature = "std"), no_std)]
// todo need in future to remove
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(clippy::type_complexity)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::too_many_arguments)]

//! # Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to
//! interact with other chains in a trustees way via IBC protocol
//!
//! The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  
//! and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs),
//! which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).

extern crate alloc;

pub use pallet::*;

use alloc::{
	format,
	string::{String, ToString},
};
use core::{marker::PhantomData, str::FromStr};
use scale_info::{prelude::vec, TypeInfo};
use serde::{Deserialize, Serialize};

use beefy_light_client::commitment;
use codec::{Codec, Decode, Encode};

use frame_support::{
	sp_runtime::traits::{AtLeast32BitUnsigned, CheckedConversion},
	sp_std::fmt::Debug,
	traits::{tokens::fungibles, Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, DispatchError, RuntimeDebug, TypeId};
use sp_std::prelude::*;

use ibc::{
	applications::ics20_fungible_token_transfer::msgs::transfer::MsgTransfer,
	clients::ics10_grandpa::{
		client_state::ClientState,
		help::{self, BlockHeader, Commitment},
	},
	core::{
		ics02_client::{client_state::AnyClientState, height},
		ics24_host::identifier::{self, ChainId as ICS24ChainId, ChannelId as IbcChannelId},
		ics26_routing::msgs::Ics26Envelope,
	},
	timestamp,
	tx_msg::Msg,
};
use tendermint_proto::Protobuf;

pub mod context;
pub mod event;
pub mod ibc_app;
pub mod ibc_core;
pub mod ibc_help;
pub mod utils;

use crate::{
	context::Context,
	ibc_app::ics20_ibc_module_impl::Ics20IBCModule,
	ibc_help::{event_from_ibc_event, get_signer},
	utils::AssetIdAndNameProvider,
};

use event::primitive::{
	ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height, Packet,
	PortId, Timestamp,
};

pub(crate) const LOG_TARGET: &str = "runtime::pallet-ibc";

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// A struct corresponds to `Any` in crate "prost-types", used in ibc-rs.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Any {
	pub type_url: Vec<u8>,
	pub value: Vec<u8>,
}

impl From<ibc_proto::google::protobuf::Any> for Any {
	fn from(any: ibc_proto::google::protobuf::Any) -> Self {
		Self { type_url: any.type_url.as_bytes().to_vec(), value: any.value }
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::event::primitive::Sequence;
	use event::primitive::{
		ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height,
		Packet, PortId, Timestamp,
	};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::{
		applications::ics20_fungible_token_transfer::context::Ics20Context,
		clients::ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
		core::{
			ics02_client::client_consensus::AnyConsensusState,
			ics04_channel::{
				channel::{Counterparty, Order},
				context::ChannelKeeper,
				events::WriteAcknowledgement,
				Version,
			},
			ics05_port::capabilities::Capability,
			ics24_host::identifier::{ChannelId as IbcChannelId, PortId as IbcPortId},
			ics26_routing::error::Error as Ics26Error,
		},
		events::IbcEvent,
		signer::Signer,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type TimeProvider: UnixTime;

		/// Currency type of the runtime
		type Currency: Currency<Self::AccountId>;

		type AssetId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Codec
			+ Copy
			+ Debug
			+ Default
			+ MaybeSerializeDeserialize;

		type AssetBalance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ From<u128>
			+ Into<u128>
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug;

		type Assets: fungibles::Mutate<
			<Self as frame_system::Config>::AccountId,
			AssetId = Self::AssetId,
			Balance = Self::AssetBalance,
		>;

		type AssetIdByName: AssetIdAndNameProvider<Self::AssetId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// vector client id for rpc
	pub type ClientStatesKeys<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientProcessedTimes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientProcessedHeights<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// client_id => Vector<(Height, ConsensusState)>
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// connection_id => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// vector connection id for rpc
	pub type ConnectionsKeys<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// (port_identifier, channel_identifier) => ChannelEnd
	pub type Channels<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// vector of (port id, channel id) for rpc
	pub type ChannelsKeys<T: Config> = StorageValue<_, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// connection_id => Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// vector of (port_identifier, channel_identifier, sequence) for rpc
	pub type AcknowledgementsKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>, u64)>, ValueQuery>;

	#[pallet::storage]
	/// client_id => client_type
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn client_counter)]
	/// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_counter)]
	/// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// client_id => connection_id
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => hash of (timestamp, heigh, packet)
	pub type PacketCommitment<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// vector of (port_id, channel_id, sequence) for rpc
	pub type PacketCommitmentKeys<T: Config> =
		StorageValue<_, Vec<(Vec<u8>, Vec<u8>, u64)>, ValueQuery>;

	#[pallet::storage]
	/// (height, port_id, channel_id, sequence) => sendpacket event
	pub type SendPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => writeack event
	pub type WriteAckPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// store latest height
	pub type LatestHeight<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// sha256(tracePath + "/" + baseDenom) => DenomTrace
	pub type Denomination<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	// port, channel -> escrow address
	pub type EscrowAddresses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PortId,
		Blake2_128Concat,
		ChannelId,
		T::AccountId,
		ValueQuery,
	>;

	#[pallet::storage]
	/// key-value asserid with asset name
	pub type AssetIdByName<T: Config> =
		StorageMap<_, Twox64Concat, Vec<u8>, T::AssetId, ValueQuery>;

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

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// emit new block event
		NewBlock(Height),
		/// emit create client event
		CreateClient(Height, ClientId, ClientType, Height),
		/// emit updte client event
		UpdateClient(Height, ClientId, ClientType, Height),
		/// emit update client state event
		UpdateClientState(Height, EventClientState),
		/// emit upgrade client event
		UpgradeClient(Height, ClientId, ClientType, Height),
		/// emit client misbehaviour event
		ClientMisbehaviour(Height, ClientId, ClientType, Height),
		/// emit open init connection event
		OpenInitConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		/// emit open try connection event
		OpenTryConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		/// emit open ack connection event
		OpenAckConnection(Height, Option<ConnectionId>, ClientId, Option<ConnectionId>, ClientId),
		/// emit open confirm connection event
		OpenConfirmConnection(
			Height,
			Option<ConnectionId>,
			ClientId,
			Option<ConnectionId>,
			ClientId,
		),
		/// emit open init channel event
		OpenInitChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		/// emit open try channel event
		OpenTryChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		/// emit open ack channel event
		OpenAckChannel(Height, PortId, Option<ChannelId>, ConnectionId, PortId, Option<ChannelId>),
		/// emit open confirm channel event
		OpenConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		/// emit close init channel event
		CloseInitChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		/// emit close confirm channel event
		CloseConfirmChannel(
			Height,
			PortId,
			Option<ChannelId>,
			ConnectionId,
			PortId,
			Option<ChannelId>,
		),
		/// emit send packet event
		SendPacket(Height, Packet),
		/// emit receive packet
		ReceivePacket(Height, Packet),
		/// emit write acknowledgement packet event
		WriteAcknowledgement(Height, Packet, Vec<u8>),
		/// emit acknowledgement packet event
		AcknowledgePacket(Height, Packet),
		/// emit timeout packet event
		TimeoutPacket(Height, Packet),
		/// emit timeout on close packet event
		TimeoutOnClosePacket(Height, Packet),
		/// emit empty event
		Empty(Vec<u8>),
		/// emit chain error event
		ChainError(Vec<u8>),
		/// emit escrow token
		EscrowToken(T::AccountId, T::AccountId, BalanceOf<T>),
		/// emit burn token
		BurnToken(T::AssetId, T::AccountId, T::AssetBalance),
		/// unescrow token
		UnEscrowToken(T::AccountId, T::AccountId, BalanceOf<T>),
		/// mint token
		MintToken(T::AssetId, T::AccountId, T::AssetBalance),
	}

	/// Convert events of ibc-rs to the corresponding events in substrate-ibc
	impl<T: Config> From<ibc::events::IbcEvent> for Event<T> {
		fn from(value: ibc::events::IbcEvent) -> Self {
			event_from_ibc_event(value)
		}
	}

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// update the beefy light client failure!
		UpdateBeefyLightClientFailure,
		/// receive mmr root block number less than client_state.latest_commitment.block_number
		ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber,
		/// client id not found
		ClientIdNotFound,
		/// Encode error
		InvalidEncode,
		/// Decode Error
		InvalidDecode,
		/// FromUtf8Error
		InvalidFromUtf8,
		/// ics26router error
		Ics26Error,
		/// invalid signer
		InvalidSigner,
		/// empty channel id
		EmptyChannelId,
		/// ics20 error
		Ics20Error,
		/// parse ibc packet error
		InvalidPacket,
		/// invalid signed_commitment
		InvalidSignedCommitment,
		/// invalid identifier
		InvalidIdentifier,
		/// invalid timestamp
		InvalidTimestamp,
		/// empty latest_commitment
		EmptyLatestCommitment,
		/// send packet error
		SendPacketError,
		/// ReceivePacket error
		ReceivePacketError,
		/// TimeoutPacket error
		TimeoutPacketError,
		/// AcknowledgePacket error
		AcknowledgePacketError,
		/// OpenInitChannel error
		OpenInitChannelError,
		/// OpenTryChannel error
		OpenTryChannelError,
		/// OpenAckChannel error
		OpenAckChannelError,
		/// OpenConfirmChannel error
		OpenConfirmChannelError,
		/// CloseInitChannel error
		CloseInitChannelError,
		/// CloseConfirmChannel error
		CloseConfirmChannelError,
		/// AmountOverflow
		AmountOverflow,
		// Serde IBCFungibleTokenPacketData error
		SerdeIBCFungibleTokenPacketDataError,
		/// Invalid parse
		InvalidParse,
		/// parse denom trace error
		ParseDenomTraceError,
		/// acknowledgement_response_empty
		AcknowledgementResponseEmpty,
		/// Get Ibc denom Error
		GetIbcDenomError,
		/// invalid_validation
		InvalidValidation,
		/// store packet result error
		StorePacketResultError,
		/// invalid token id
		InvalidTokenId,
		/// wrong assert id
		WrongAssetId,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for all of the IBC request(except MMR root update).
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<Any>,
			_tmp: u8,
		) -> DispatchResultWithPostInfo {
			sp_tracing::within_span!(
			sp_tracing::Level::TRACE, "deliver";
			{
				let _sender = ensure_signed(origin)?;
				let mut ctx = Context::<T>::new();

				let messages: Vec<ibc_proto::google::protobuf::Any> = messages
					.into_iter()
					.map(|message| ibc_proto::google::protobuf::Any {
						type_url: String::from_utf8(message.type_url.clone()).unwrap(),
						value: message.value,
					})
					.collect();

				log::trace!(target: LOG_TARGET, "received deliver : {:?} ", messages.iter().map(|message| message.type_url.clone()).collect::<Vec<_>>());

				let mut results: Vec<IbcEvent> = vec![];
				for (index, message) in messages.clone().into_iter().enumerate() {

					let mut result = Vec::new();
					match ibc::core::ics26_routing::handler::deliver(&mut ctx, message.clone()) {
						Ok(value) => {
							log::trace!(target: LOG_TARGET, "deliver event  : {:?} ", value.0);
							result = value.0;

						}
						Err(error) => {
							log::trace!(target: LOG_TARGET, "deliver error  : {:?} ", error);
						}
					};

					log::info!("result: {:?}", result);

					results.append(&mut result);
				}

				let ret = Self::handle_result(&mut ctx, messages, results)?;

				Ok(().into())
			})
		}

		/// Update the MMR root stored in client_state
		/// Example of invoking this function via subxt
		#[pallet::weight(0)]
		pub fn update_client_state(
			origin: OriginFor<T>,
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			log::trace!(target: LOG_TARGET, "update_client_state: update_client_state request.");
			let _who = ensure_signed(origin)?;

			Self::inner_update_mmr_root(client_id, mmr_root)
		}

		/// Transfer interface for user test by explore
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			source_port: Vec<u8>,
			source_channel: Vec<u8>,
			token: Vec<u8>,
			amount: u128,
			receiver: Vec<u8>,
			timeout_height: u64,
			timeout_timestamp: u64,
		) -> DispatchResult {
			sp_tracing::within_span!(
				sp_tracing::Level::TRACE, "transfer";
				{
					let source_port = identifier::PortId::from_str(
						&String::from_utf8(source_port).map_err(|_| Error::<T>::InvalidFromUtf8)?,
					)
					.map_err(|_| Error::<T>::InvalidIdentifier)?;
					log::info!(target: LOG_TARGET,"transfer : source_port: {}", source_port);

					let source_channel = identifier::ChannelId::from_str(
						&String::from_utf8(source_channel).map_err(|_| Error::<T>::InvalidFromUtf8)?,
					)
					.map_err(|_| Error::<T>::InvalidIdentifier)?;
					log::info!(target: LOG_TARGET,"transfer : source_channel : {}", source_channel);

					let token = Some(ibc_proto::cosmos::base::v1beta1::Coin {
						denom: String::from_utf8(token).map_err(|_| Error::<T>::InvalidFromUtf8)?,
						amount: amount.to_string(),
					});
					log::info!(target: LOG_TARGET,"transfer : token : {:?}", token);

					let sender: T::AccountId = ensure_signed(origin)?;
					let encode_sender = T::AccountId::encode(&sender);
					let hex_sender = hex::encode(encode_sender);
					log::info!(target: LOG_TARGET,"transfer : hex sender : 0x{}", hex_sender);

					let sender = Signer::from(hex_sender);
					log::info!(target: LOG_TARGET,"transfer : sender : {}", sender);

					let receiver = String::from_utf8(receiver).map_err(|_| Error::<T>::InvalidFromUtf8)?;
					let receiver = Signer::new(receiver);
					log::info!(target: LOG_TARGET,"transfer : receiver : {}", receiver);


					let timeout_height =
						height::Height { revision_number: 0, revision_height: timeout_height };
					log::info!(target: LOG_TARGET,"transfer : timeout height : {}", timeout_height);

					let timeout_timestamp = timestamp::Timestamp::from_nanoseconds(timeout_timestamp)
						.map_err(|_| Error::<T>::InvalidTimestamp)?;
					log::info!(target: LOG_TARGET,"transfer : timeout timestamp : {}", timeout_timestamp);

					let msg = MsgTransfer {
						source_port,
						source_channel,
						token,
						sender,
						receiver,
						timeout_height,
						timeout_timestamp,
					};

					// send to router
					let mut ctx = Context::<T>::new();
					let send_transfer_result = ibc::applications::ics20_fungible_token_transfer::relay_application_logic::send_transfer::send_transfer(&ctx, msg.clone()).unwrap();

					ctx.store_packet_result(send_transfer_result.result)
						.map_err(|_| Error::<T>::StorePacketResultError)?;

					let send_transfer_result_event = send_transfer_result.events;

					// handle the result
					log::info!(target: LOG_TARGET,"result: {:?}", send_transfer_result_event);

					Self::handle_result(&mut ctx, vec![msg.to_any()], send_transfer_result_event)?;

					Ok(())
				}
			)
		}

		#[pallet::weight(0)]
		pub fn delete_send_packet_event(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			log::trace!(target: LOG_TARGET, "delete_send_packet_event");

			let _who = ensure_signed(origin)?;
			<SendPacketEvent<T>>::drain();

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn delete_ack_packet_event(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			log::trace!(target: LOG_TARGET, "delete_ack_packet_event");

			let _who = ensure_signed(origin)?;
			<WriteAckPacketEvent<T>>::drain();

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// handle the event returned by ics26 route module
		fn handle_result<Ctx>(
			ctx: &mut Ctx,
			messages: Vec<ibc_proto::google::protobuf::Any>,
			result: Vec<IbcEvent>,
		) -> DispatchResult
		where
			Ctx: Ics20Context,
		{
			for (index, event) in result.into_iter().enumerate() {
				match event.clone() {
					IbcEvent::SendPacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/f5962c3324ee7e69eeaa9918b65eb1b089da6095/modules/apps/transfer/keeper/msg_server.go#L16
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] send packet is : {:?}",
							value.packet
						);

						let ret = ibc_app::ics20_handler::handle_transfer::<Ctx, T>(
							ctx,
							value.clone().packet,
						)?;

						Self::deposit_event(event.clone().into());
						store_send_packet::<T>(&value);
					},

					IbcEvent::ReceivePacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L364
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] receive packet is : {:?}",
							value.packet
						);
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = Ics20IBCModule::<T>::new();
						let ack = ibc::core::ics26_routing::ibc_module::IBCModule::on_recv_packet(
							&ics20_modlue,
							ctx,
							value.clone().packet,
							relayer_signer,
						)
						.map_err(|_| Error::<T>::ReceivePacketError)?;

						// Emit recv event
						Self::deposit_event(event.clone().into());

						let packet = value.packet;

						let write_ack_event =
							ibc::core::ics04_channel::handler::write_acknowledgement::process(
								ctx,
								packet.clone(),
								ack.clone(),
							)
							.map_err(|_| Error::<T>::ReceivePacketError)?;

						use ibc::core::ics04_channel::packet::PacketResult;

						let write_ack_event_result =
							if let PacketResult::WriteAck(write_ack_event_result) =
								write_ack_event.result
							{
								write_ack_event_result
							} else {
								todo!()
							};

						ctx.store_packet_acknowledgement(
							(
								write_ack_event_result.port_id.clone(),
								write_ack_event_result.channel_id,
								write_ack_event_result.seq,
							),
							write_ack_event_result.ack_commitment,
						);

						// Emit write acknowledgement event
						// todo this
						let block_number =
							format!("{:?}", <frame_system::Pallet<T>>::block_number());
						let current_height: u64 = block_number.parse().unwrap_or_default();
						Self::deposit_event(Event::<T>::WriteAcknowledgement(
							Height::new(0, current_height),
							packet.into(),
							ack,
						));

						// write ack acknowledgement
						let write_ack_event =
							if let IbcEvent::WriteAcknowledgement(write_ack_event) =
								write_ack_event.events.first().unwrap()
							{
								write_ack_event
							} else {
								todo!()
							};
						store_write_ack::<T>(write_ack_event);
					},
					IbcEvent::TimeoutPacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L442
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] timeout packet is : {:?}",
							value.packet
						);
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_module = Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_timeout_packet(
								&ics20_module,
								ctx,
								value.clone().packet,
								relayer_signer,
							)
							.map_err(|_| Error::<T>::TimeoutPacketError)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::AcknowledgePacket(value) => {
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L581
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] ack packet is : {:?}",
							value.packet
						);
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_module = Ics20IBCModule::<T>::new();

						let ret = ibc::core::ics26_routing::ibc_module::IBCModule::on_acknowledgement_packet(&ics20_module, ctx, value.clone().packet, vec![], relayer_signer).map_err(|_| Error::<T>::AcknowledgePacketError)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenInitChannel(value) => {
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] open init channel : {:?}",
							value
						);
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L163
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let height = value.clone().height;
						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;
						let connection_id = value.clone().connection_id;
						let counterparty_port_id = value.clone().counterparty_port_id;
						let counterparty_channel_id = value.clone().counterparty_channel_id;

						let ics20_modlue = Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_init(
								&ics20_modlue,
								ctx,
								Order::Unordered,
								vec![connection_id],
								port_id,
								channel_id,
								&Capability::default(), // todo
								Counterparty {
									port_id: counterparty_port_id,
									channel_id: counterparty_channel_id,
								},
								Version::ics20(),
							)
							.map_err(|_| Error::<T>::OpenInitChannelError)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenTryChannel(value) => {
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] open try channel : {:?}",
							value
						);
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L203

						let height = value.clone().height;
						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;
						let connection_id = value.clone().connection_id;
						let counterparty_port_id = value.clone().counterparty_port_id;
						let counterparty_channel_id = value.clone().counterparty_channel_id;

						let ics20_modlue = Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_try(
								&ics20_modlue,
								ctx,
								Order::Unordered,
								vec![connection_id],
								port_id,
								channel_id,
								&Capability::default(), // todo
								Counterparty {
									port_id: counterparty_port_id,
									channel_id: counterparty_channel_id,
								},
								Version::ics20(),
							)
							.map_err(|_| Error::<T>::OpenTryChannelError)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenAckChannel(value) => {
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] open ack channel : {:?}",
							value
						);
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L241

						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = Ics20IBCModule::<T>::new();
						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_ack(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
								Version::ics20(),
							)
							.map_err(|_| Error::<T>::OpenAckChannelError)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::OpenConfirmChannel(value) => {
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] open confirm channel : {:?}",
							value
						);
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L277

						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_open_confirm(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
							)
							.map_err(|_| Error::<T>::OpenConfirmChannelError)?;

						Self::deposit_event(event.clone().into());
					},
					IbcEvent::CloseInitChannel(value) => {
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] close init channel : {:?}",
							value
						);
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L309

						let port_id = value.clone().port_id;
						let channel_id = value.clone().channel_id;
						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = Ics20IBCModule::<T>::new();

						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_close_init(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
							)
							.map_err(|_| Error::<T>::CloseInitChannelError)?;

						Self::deposit_event(event.clone().into());
					},

					IbcEvent::CloseConfirmChannel(value) => {
						log::trace!(
							target: LOG_TARGET,
							"[handle_result] close confirm channel : {:?}",
							value
						);
						// refer to https://github.com/octopus-network/ibc-go/blob/acbc9b61d10bf892528a392595782ac17aeeca30/modules/core/keeper/msg_server.go#L336

						let port_id = value.clone().port_id;
						let channel_id =
							value.clone().channel_id.ok_or(Error::<T>::EmptyChannelId)?;

						let relayer_signer = get_signer::<T>(messages[index].clone())
							.map_err(|_| Error::<T>::InvalidSigner)?;

						let ics20_modlue = Ics20IBCModule::<T>::new();
						let ret =
							ibc::core::ics26_routing::ibc_module::IBCModule::on_chan_close_confirm(
								&ics20_modlue,
								ctx,
								port_id,
								channel_id,
							)
							.map_err(|_| Error::<T>::CloseConfirmChannelError)?;

						Self::deposit_event(event.clone().into());
					},
					_ => {
						log::warn!(
							target: LOG_TARGET,
							"[handle_result] Unhandled event: {:?}",
							event
						);
						Self::deposit_event(event.clone().into());
					},
				}
			}
			Ok(())
		}

		/// inner update mmr root
		fn inner_update_mmr_root(
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			log::trace!(
				target: LOG_TARGET,
				"inner_update_client_state: update_client_state request."
			);

			// check if the client id exist?
			let client_id_str =
				String::from_utf8(client_id.clone()).map_err(|_| Error::<T>::InvalidFromUtf8)?;
			log::trace!(
				target: LOG_TARGET,
				"inner_update_client_state:  client id is {:?}",
				client_id_str
			);

			let decode_received_mmr_root =
				help::MmrRoot::decode(&mut &mmr_root[..]).map_err(|_| Error::<T>::InvalidDecode)?;
			log::trace!(
				target: LOG_TARGET,
				"inner_update_client_state:  decode mmr root is {:?}",
				decode_received_mmr_root
			);

			let mut client_state = ClientState::default();

			if !<ClientStates<T>>::contains_key(client_id.clone()) {
				log::error!(
					"in inner_update_client_state: {:?} client_state not found !",
					client_id_str
				);

				return Err(Error::<T>::ClientIdNotFound.into())
			} else {
				// get client state from chain storage
				let data = <ClientStates<T>>::get(client_id.clone());
				let any_client_state =
					AnyClientState::decode_vec(&*data).map_err(|_| Error::<T>::InvalidDecode)?;
				client_state = match any_client_state {
					AnyClientState::Grandpa(value) => value,
					_ => unimplemented!(),
				};

				log::trace!(
					target: LOG_TARGET,
					"in inner_update_client_state: get client_state from chain storage: {:?}",
					client_state
				);
			}

			let signed_commitment =
				commitment::SignedCommitment::try_from(decode_received_mmr_root.signed_commitment)
					.map_err(|_| Error::<T>::InvalidSignedCommitment)?;

			let rev_block_number = signed_commitment.commitment.block_number;
			if rev_block_number <= client_state.latest_commitment.block_number {
				log::trace!(target: LOG_TARGET,"receive mmr root block number({}) less than client_state.latest_commitment.block_number({})",
				rev_block_number,client_state.latest_commitment.block_number);

				return Err(Error::<T>::ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber.into());
			}
			// build new beefy light client by client_state
			let mut light_client = beefy_light_client::LightClient {
				latest_commitment: Some(client_state.latest_commitment.clone().into()),
				validator_set: client_state.validator_set.clone().into(),
				in_process_state: None,
			};
			log::trace!(
				target: LOG_TARGET,
				"build new beefy_light_client from client_state store in chain \n {:?}",
				light_client
			);

			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs = decode_received_mmr_root.validator_merkle_proofs;
			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs: Vec<beefy_light_client::ValidatorMerkleProof> = validator_proofs
				.into_iter()
				.map(|validator_proof| validator_proof.into())
				.collect();

			// encode signed_commitment
			let encoded_signed_commitment =
				commitment::SignedCommitment::encode(&signed_commitment);

			let mmr_leaf = decode_received_mmr_root.mmr_leaf;
			let mmr_leaf_proof = decode_received_mmr_root.mmr_leaf_proof;

			// verfiy mmr proof and update lc state
			let result = light_client.update_state(
				&encoded_signed_commitment,
				&validator_proofs,
				&mmr_leaf,
				&mmr_leaf_proof,
			);

			match result {
				Ok(_) => {
					log::trace!(target:"runtime::pallet-ibc","update the beefy light client sucesse! and the beefy light client state is : {:?} \n",light_client);

					// update client_client block number and latest commitment
					let latest_commitment =
						light_client.latest_commitment.ok_or(Error::<T>::EmptyLatestCommitment)?;
					client_state.block_number = latest_commitment.block_number;
					client_state.latest_commitment = help::Commitment::from(latest_commitment);

					// update validator_set
					client_state.validator_set =
						help::ValidatorSet::from(light_client.validator_set.clone());

					// update block header
					client_state.block_header = decode_received_mmr_root.block_header;

					// save to chain
					let any_client_state = AnyClientState::Grandpa(client_state.clone());
					let data =
						any_client_state.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					// store client states key-value
					<ClientStates<T>>::insert(client_id.clone(), data);

					// store client states keys
					let _ = <ClientStatesKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
						if let Some(_value) = val.iter().find(|&x| x == &client_id.clone()) {
						} else {
							val.push(client_id.clone());
						}
						Ok(())
					});

					log::trace!(
						target: LOG_TARGET,
						"the updated client state is : {:?}",
						client_state
					);

					use ibc::{
						clients::ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
						core::ics02_client::client_consensus::AnyConsensusState,
					};

					let mut consensus_state =
						GPConsensusState::new(client_state.block_header.clone());
					consensus_state.digest = client_state.latest_commitment.payload.clone();
					let any_consensus_state = AnyConsensusState::Grandpa(consensus_state);

					let height = ibc::Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};

					log::trace!(target: LOG_TARGET,"in ibc-lib : [store_consensus_state] >> client_id: {:?}, height = {:?}, consensus_state = {:?}", client_id, height, any_consensus_state);

					let height = height.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					let data =
						any_consensus_state.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;

					if <ConsensusStates<T>>::contains_key(client_id.clone()) {
						// if consensus_state is no empty use push insert an exist
						// ConsensusStates
						let _ = <ConsensusStates<T>>::try_mutate(
							client_id,
							|val| -> Result<(), &'static str> {
								val.push((height, data));
								Ok(())
							},
						);
					} else {
						// if consensus state is empty insert a new item.
						<ConsensusStates<T>>::insert(client_id, vec![(height, data)]);
					}

					// emit update state sucesse event
					let event_height = Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};
					let event_client_state = EventClientState::from(client_state);
					Self::deposit_event(Event::<T>::UpdateClientState(
						event_height,
						event_client_state,
					));
				},
				Err(e) => {
					log::error!(
						target: LOG_TARGET,
						"update the beefy light client failure! : {:?}",
						e
					);

					return Err(Error::<T>::UpdateBeefyLightClientFailure.into())
				},
			}

			Ok(().into())
		}
	}
}

fn store_send_packet<T: Config>(_send_packet_event: &ibc::core::ics04_channel::events::SendPacket) {
	use crate::event::primitive::Sequence;

	// store send-packet
	let send_packet_event = _send_packet_event.clone();
	let packet = Packet {
		sequence: Sequence::from(send_packet_event.packet.sequence),
		source_channel: ChannelId::from(send_packet_event.packet.source_channel),
		source_port: PortId::from(send_packet_event.packet.source_port.clone()),
		destination_channel: ChannelId::from(send_packet_event.packet.destination_channel),
		destination_port: PortId::from(send_packet_event.packet.destination_port),
		data: send_packet_event.packet.data,
		timeout_timestamp: Timestamp::from(send_packet_event.packet.timeout_timestamp),
		timeout_height: Height::from(send_packet_event.packet.timeout_height),
	};
	let packet = packet.to_ibc_packet().unwrap().encode_vec().unwrap();

	let port_id = send_packet_event.packet.source_port.as_bytes().to_vec();
	let channel_id = from_channel_id_to_vec(send_packet_event.packet.source_channel);

	log::trace!(target: LOG_TARGET, "in lib: [store_send_packet]. _send_packet_event={:?}", _send_packet_event.clone());
	<SendPacketEvent<T>>::insert(
		(port_id, channel_id, u64::from(send_packet_event.packet.sequence)),
		packet,
	);
}

fn store_write_ack<T: Config>(
	write_ack_event: &ibc::core::ics04_channel::events::WriteAcknowledgement,
) {
	use ibc::core::ics04_channel::events::WriteAcknowledgement;

	// store ack
	let port_id = write_ack_event.packet.source_port.as_bytes().to_vec();
	let channel_id = from_channel_id_to_vec(write_ack_event.packet.source_channel);
	let sequence = u64::from(write_ack_event.packet.sequence);
	let write_ack = write_ack_event.encode_vec().unwrap();
	let _write_ack = WriteAcknowledgement::decode(&*write_ack).unwrap();
	// store.Set((portID, channelID, sequence), WriteAckEvent)
	<WriteAckPacketEvent<T>>::insert((port_id, channel_id, sequence), write_ack);
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

/// FungibleTokenPacketData defines a struct for the packet payload
/// See FungibleTokenPacketData spec: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures
#[derive(Decode, Encode, Debug, PartialEq)]
pub struct FungibleTokenPacketData<T: Config> {
	// the token denomination to be transferred
	pub denomination: Vec<u8>,
	// the token amount to be transferred
	pub amount: u128,
	// the sender address
	pub sender: T::AccountId,
	// the recipient address on the destination chain
	pub receiver: T::AccountId,
}

use ibc::applications::ics20_fungible_token_transfer::msgs::fungible_token_packet_data::FungibleTokenPacketData as IBCFungibleTokenPacketData;

impl<T: Config> From<IBCFungibleTokenPacketData> for FungibleTokenPacketData<T> {
	fn from(value: IBCFungibleTokenPacketData) -> Self {
		use core::str;
		use hex::FromHex;

		let sender = <Vec<u8>>::from_hex(value.sender.as_str()).unwrap();
		let receiver = <Vec<u8>>::from_hex(value.receiver.as_str()).unwrap();

		Self {
			denomination: value.denom.as_bytes().to_vec(),
			amount: value.amount.parse::<u128>().unwrap_or_default(),
			sender: T::AccountId::decode(&mut sender.as_ref()).unwrap(),
			receiver: T::AccountId::decode(&mut receiver.as_ref()).unwrap(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FungibleTokenPacketAcknowledgement {
	Success(FungibleTokenPacketSuccess),
	Err(FungibleTokenPacketError),
}

impl FungibleTokenPacketAcknowledgement {
	pub fn new() -> Self {
		Self::Success(FungibleTokenPacketSuccess::new())
	}
}

impl Default for FungibleTokenPacketAcknowledgement {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleTokenPacketSuccess {
	result: AQ,
}

impl Default for FungibleTokenPacketSuccess {
	fn default() -> Self {
		Self::new()
	}
}

impl FungibleTokenPacketSuccess {
	pub fn new() -> Self {
		let aq = AQ;
		Self { result: aq }
	}
	pub fn result(&self) -> &str {
		// this is binary 0x01 base64 encoded
		"AQ=="
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct AQ;

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleTokenPacketError {
	pub error: String,
}

pub fn from_channel_id_to_vec(value: IbcChannelId) -> Vec<u8> {
	format!("{}", value).as_bytes().to_vec()
}
