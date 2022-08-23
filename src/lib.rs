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
extern crate core;

pub use pallet::*;

use alloc::{
	format,
	string::{String, ToString},
};
use core::{marker::PhantomData, str::FromStr};
use scale_info::{prelude::vec, TypeInfo};
use serde::{Deserialize, Serialize};

use beefy_light_client::commitment::{self, known_payload_ids::MMR_ROOT_ID};

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
	applications::transfer::msgs::transfer::MsgTransfer,
	clients::ics10_grandpa::{
		client_state::ClientState,
		help::{self, BlockHeader, Commitment},
	},
	core::{
		ics02_client::{client_state::AnyClientState, height},
		ics24_host::identifier::{self, ChainId as ICS24ChainId, ChannelId as IbcChannelId},
		ics26_routing::{handler, msgs::Ics26Envelope},
	},
	events::IbcEvent,
	timestamp,
	tx_msg::Msg,
};

use tendermint_proto::Protobuf;

pub mod context;
pub mod events;
pub mod module;
pub mod traits;
pub mod utils;

use crate::{context::Context, traits::AssetIdAndNameProvider};

use crate::module::{
	clients::ics10_grandpa::ClientState as EventClientState,
	core::ics24_host::{
		ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId, Timestamp,
	},
};

pub const LOG_TARGET: &str = "runtime::pallet-ibc";
pub const REVISION_NUMBER: u64 = 8888;

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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{
		events::ModuleEvent,
		module::{
			applications::transfer::transfer_handle_callback::TransferModule,
			clients::ics10_grandpa::ClientState as EventClientState,
			core::ics24_host::{
				ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId, Sequence,
				Timestamp,
			},
		},
	};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			fungibles::{Inspect, Mutate, Transfer},
			UnixTime,
		},
	};
	use frame_system::pallet_prelude::*;
	use ibc::{
		applications::transfer::context::Ics20Context,
		clients::ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
		core::{
			ics02_client::client_consensus::AnyConsensusState,
			ics04_channel::{
				channel::{Counterparty, Order},
				context::ChannelKeeper,
				events::WriteAcknowledgement,
				Version,
			},
			ics24_host::{
				identifier::{ChannelId as IbcChannelId, PortId as IbcPortId},
				path::{ClientConsensusStatePath, ClientStatePath},
			},
			ics26_routing::error::Error as Ics26Error,
		},
		events::IbcEvent,
		handler::{HandlerOutput, HandlerOutputBuilder},
		signer::Signer,
	};
	use sp_runtime::traits::IdentifyAccount;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Sync + Send + Debug {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The provider providing timestamp of host chain
		type TimeProvider: UnixTime;

		/// The currency type of the runtime
		type Currency: Currency<Self::AccountId>;

		/// Identifier for the class of asset.
		type AssetId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Codec
			+ Copy
			+ Debug
			+ Default
			+ MaybeSerializeDeserialize;

		/// The units in which we record balances.
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

		/// Expose customizable associated type of asset transfer, lock and unlock
		type Assets: Transfer<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>;

		/// Map of cross-chain asset ID & name
		type AssetIdByName: AssetIdAndNameProvider<Self::AssetId>;

		/// Account Id Conversion from SS58 string or hex string
		type AccountIdConversion: TryFrom<Signer>
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ Clone
			+ PartialEq
			+ Debug;

		// The native token name
		const NATIVE_TOKEN_NAME: &'static [u8];
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// ClientStatePath(client_id) => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

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
	/// ClientConsensusStatePath(client_id, Height) => ConsensusState
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ChannelEndPath(port_id, channel_id) => ChannelEnd
	pub type Channels<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => Vec<ChannelEndPath(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// SeqSendsPath(port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// SeqRecvsPath(port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// SeqAcksPath(port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// AcksPath(port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ClientTypePath(client_id) => client_type
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
	/// ClientConnectionsPath(client_id) => connection_id
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ReceiptsPath(port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// CommitmentsPath(port_id, channel_id, sequence) => hash of (timestamp, height, packet)
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (height, port_id, channel_id, sequence) => send-packet event
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
	/// (port_id, channel_id, sequence) => writ ack event
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
	/// Latest height
	pub type LatestHeight<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// Previous host block height
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// (asset name) => asset id
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
		/// New block event
		NewBlock { height: Height },
		/// Client created event
		CreateClient {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Client updated event
		UpdateClient {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Client upgraded event
		UpgradeClient {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Client misbehaviour event
		ClientMisbehaviour {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Connection open init event
		OpenInitConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open try event
		OpenTryConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open acknowledgement event
		OpenAckConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open confirm event
		OpenConfirmConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Channel open init event
		OpenInitChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open try event
		OpenTryChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open acknowledgement event
		OpenAckChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open confirm event
		OpenConfirmChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel close init event
		CloseInitChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel close confirm event
		CloseConfirmChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Send packet event
		SendPacket { height: Height, packet: Packet },
		/// Receive packet event
		ReceivePacket { height: Height, packet: Packet },
		/// WriteAcknowledgement packet event
		WriteAcknowledgement { height: Height, packet: Packet, ack: Vec<u8> },
		/// Acknowledgements packet event
		AcknowledgePacket { height: Height, packet: Packet },
		/// Timeout packet event
		TimeoutPacket { height: Height, packet: Packet },
		/// TimoutOnClose packet event
		TimeoutOnClosePacket { height: Height, packet: Packet },
		/// Empty event
		Empty(Vec<u8>),
		/// Chain Error event
		ChainError(Vec<u8>),
		/// App Module event
		AppModule(ModuleEvent),
		/// Emit update client state event
		UpdateClientState(Height, EventClientState),
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

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Update the beefy light client failure!
		UpdateBeefyLightClientFailure,
		/// Receive mmr root block number less than client_state.latest_commitment.block_number
		ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber,
		/// Client id not found
		ClientIdNotFound,
		/// Encode error
		InvalidEncode,
		/// Decode Error
		InvalidDecode,
		/// FromUtf8Error
		InvalidFromUtf8,
		/// Invalid signed_commitment
		InvalidSignedCommitment,
		/// Empty latest_commitment
		EmptyLatestCommitment,
		/// Invalid token id
		InvalidTokenId,
		/// Wrong assert id
		WrongAssetId,
		// Parser Msg Transfer Error
		ParserMsgTransferError,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for most of the IBC request.
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		///
		/// The origin must be Signed and the sender must have sufficient funds fee.
		///
		/// Parameters:
		/// - `messages`: The arbitrary ICS message's representation in Substrate, which contains an URL and
		///  a serialized protocol buffer message. The URL name that uniquely identifies the type of the serialized protocol buffer message.
		///
		/// The relevant events are emitted when successful.
		#[pallet::weight(0)]
		pub fn deliver(origin: OriginFor<T>, messages: Vec<Any>) -> DispatchResultWithPostInfo {
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

				for (_, message) in messages.into_iter().enumerate() {

					match ibc::core::ics26_routing::handler::deliver(&mut ctx, message.clone()) {
						Ok(ibc::core::ics26_routing::handler::MsgReceipt { events, log: _log}) => {
							log::trace!(target: LOG_TARGET, "deliver events  : {:?} ", events);
							// deposit events about send packet event and ics20 transfer event
							for event in events {
								match event {
									IbcEvent::WriteAcknowledgement(ref write_ack) => {
										store_write_ack::<T>(write_ack);
										Self::deposit_event(event.clone().into());
									}
									_ => {
										log::trace!(target: LOG_TARGET, "raw_transfer event : {:?} ", event);
										Self::deposit_event(event.clone().into());
									}
								}
							}
						}
						Err(error) => {
							log::trace!(target: LOG_TARGET, "deliver error  : {:?} ", error);
						}
					};
				}

				Ok(().into())
			})
		}

		/// ICS20 fungible token transfer.
		/// Handling transfer request as sending chain or receiving chain.
		///
		/// Parameters:
		/// - `messages`: A serialized protocol buffer message containing the transfer request.
		///
		/// The relevant events are emitted when successful.
		#[pallet::weight(0)]
		pub fn raw_transfer(
			origin: OriginFor<T>,
			messages: Vec<Any>,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;
			let mut ctx = TransferModule(PhantomData::<T>);

			let messages: Vec<ibc_proto::google::protobuf::Any> = messages
				.into_iter()
				.map(|message| ibc_proto::google::protobuf::Any {
					type_url: String::from_utf8(message.type_url.clone()).unwrap(),
					value: message.value,
				})
				.collect();

			log::trace!(
				target: LOG_TARGET,
				"raw_transfer : {:?} ",
				messages.iter().map(|message| message.type_url.clone()).collect::<Vec<_>>()
			);

			for message in messages {
				let mut handle_out = HandlerOutputBuilder::new();
				let msg_transfer = MsgTransfer::try_from(message)
					.map_err(|_| Error::<T>::ParserMsgTransferError)?;
				let result = ibc::applications::transfer::relay::send_transfer::send_transfer(
					&mut ctx,
					&mut handle_out,
					msg_transfer,
				);
				match result {
					Ok(_value) => {
						log::trace!(target: LOG_TARGET, "raw_transfer Successful!");
					},
					Err(error) => {
						log::trace!(target: LOG_TARGET, "raw_transfer Error : {:?} ", error);
					},
				}

				let HandlerOutput::<()> { result, log, events } = handle_out.with_result(());

				log::trace!(target: LOG_TARGET, "raw_transfer log : {:?} ", log);

				// deposit events about send packet event and ics20 transfer event
				for event in events {
					match event {
						IbcEvent::SendPacket(ref send_packet) => {
							store_send_packet::<T>(send_packet);
							Self::deposit_event(event.clone().into());
						},
						IbcEvent::WriteAcknowledgement(ref write_ack) => {
							store_write_ack::<T>(write_ack);
							Self::deposit_event(event.clone().into());
						},
						_ => {
							log::trace!(target: LOG_TARGET, "raw_transfer event : {:?} ", event);
							Self::deposit_event(event.clone().into());
						},
					}
				}
			}

			Ok(().into())
		}
	}
}

fn store_send_packet<T: Config>(send_packet_event: &ibc::core::ics04_channel::events::SendPacket) {
	// store key port_id and channel_id
	let port_id = send_packet_event.packet.source_port.as_bytes().to_vec();
	let channel_id = from_channel_id_to_vec(send_packet_event.packet.source_channel.clone());
	// store value packet
	let packet = send_packet_event.packet.encode_vec().unwrap();
	<SendPacketEvent<T>>::insert(
		(port_id, channel_id, u64::from(send_packet_event.packet.sequence)),
		packet,
	);
}

fn store_write_ack<T: Config>(
	write_ack_event: &ibc::core::ics04_channel::events::WriteAcknowledgement,
) {
	// store ack
	let port_id = write_ack_event.packet.source_port.as_bytes().to_vec();
	let channel_id = from_channel_id_to_vec(write_ack_event.packet.source_channel.clone());
	let sequence = u64::from(write_ack_event.packet.sequence);
	let write_ack = write_ack_event.encode_vec().unwrap();

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

pub fn from_channel_id_to_vec(value: IbcChannelId) -> Vec<u8> {
	value.to_string().as_bytes().to_vec()
}
