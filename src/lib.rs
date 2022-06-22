#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(unused_assignments)]

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

use crate::{
	context::Context,
	ibc_help::event_from_ibc_event,
	utils::{AssetIdAndNameProvider, LOG_TARGET},
};
use alloc::{
	format,
	string::{String, ToString},
};
use core::{fmt::Debug, marker::PhantomData, str::FromStr};

use codec::{Codec, Decode, Encode};
use scale_info::{prelude::vec, TypeInfo};

use beefy_light_client::commitment::{self, known_payload_ids::MMR_ROOT_ID};

use log::{error, info, trace};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, IdentifyAccount},
	RuntimeDebug,
};
use sp_std::prelude::*;

use event::primitive::{
	ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height, Packet,
	PortId,
};
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect, Mutate, Transfer},
		Currency, UnixTime,
	},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use ibc::{
	applications::transfer::msgs::transfer::MsgTransfer,
	clients::ics10_grandpa::{client_state::ClientState, help},
	core::{
		ics02_client::{client_state::AnyClientState, height},
		ics24_host::identifier,
	},
	events::IbcEvent,
	signer::Signer,
	timestamp,
};
use tendermint_proto::Protobuf;

pub mod context;
pub mod event;
pub mod ibc_help;
pub mod utils;

// ibc protocol implement
pub mod applications;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod clients;
pub mod ibc_core;
#[cfg(test)]
mod mock;
pub mod relayer;
#[cfg(test)]
mod tests;

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

#[frame_support::pallet]
pub mod pallet {
	use super::*;

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

		type Assets: Transfer<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>;

		type AssetIdByName: AssetIdAndNameProvider<Self::AssetId>;

		/// Account Id Conversion from SS58 string or hex string
		type AccountIdConversion: TryFrom<Signer>
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ Clone;

		// config native token name
		const NATIVE_TOKEN_NAME: &'static [u8];

		/// Prefix for events stored in the Off-chain DB via Indexing API.
		const INDEXING_PREFIX: &'static [u8];

		/// Prefix for ibc connection, should be valid utf8 string bytes
		const CONNECTION_PREFIX: &'static [u8];
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// client_id => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientUpdateTime<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientUpdateHeight<T: Config> = StorageDoubleMap<
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
	/// ClientId => Vec<ConnectionId>
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Vec<u8>>, ValueQuery>;

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
	/// (port_id, channel_id, sequence) => hash of (timestamp, height, packet)
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
		/// New block
		NewBlock { height: Height },
		/// Client Created
		CreateClient {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Client updated
		UpdateClient {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// emit update client state event
		UpdateClientState { height: Height, client_state: EventClientState },
		/// Client upgraded
		UpgradeClient {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Client misbehaviour
		ClientMisbehaviour {
			height: Height,
			client_id: ClientId,
			client_type: ClientType,
			consensus_height: Height,
		},
		/// Connection open init
		OpenInitConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open try
		OpenTryConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open acknowledgement
		OpenAckConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Connection open confirm
		OpenConfirmConnection {
			height: Height,
			connection_id: Option<ConnectionId>,
			client_id: ClientId,
			counterparty_connection_id: Option<ConnectionId>,
			counterparty_client_id: ClientId,
		},
		/// Channel open init
		OpenInitChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open try
		OpenTryChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open acknowledgement
		OpenAckChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel open confirm
		OpenConfirmChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel close init
		CloseInitChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Channel close confirm
		CloseConfirmChannel {
			height: Height,
			port_id: PortId,
			channel_id: Option<ChannelId>,
			connection_id: ConnectionId,
			counterparty_port_id: PortId,
			counterparty_channel_id: Option<ChannelId>,
		},
		/// Send packet
		SendPacket { height: Height, packet: Packet },
		/// Receive packet
		ReceivePacket { height: Height, packet: Packet },
		/// WriteAcknowledgement packet
		WriteAcknowledgement { height: Height, packet: Packet, ack: Vec<u8> },
		/// Acknowledgements packet
		AcknowledgePacket { height: Height, packet: Packet },
		/// Timeout packet
		TimeoutPacket { height: Height, packet: Packet },
		/// TimoutOnClose packet
		TimeoutOnClosePacket { height: Height, packet: Packet },
		/// Empty
		Empty(Vec<u8>),
		/// Chain Error
		ChainError(Vec<u8>),
		/// Escrow token
		EscrowToken { sender: T::AccountId, escrow_account: T::AccountId, amount: BalanceOf<T> },
		/// Burn token
		BurnToken { token_id: T::AssetId, sender: T::AccountId, amount: T::AssetBalance },
		/// Unescrow token
		UnEscrowToken { escrow_account: T::AccountId, receive: T::AccountId, amount: BalanceOf<T> },
		/// Mint token
		MintToken { token_id: T::AssetId, receive: T::AccountId, amount: T::AssetBalance },
		/// App Module
		AppModule,
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
				let mut ctx = Context::<T>::default();

				let messages: Vec<ibc_proto::google::protobuf::Any> = messages
					.into_iter()
					.map(|message| ibc_proto::google::protobuf::Any {
						type_url: String::from_utf8(message.type_url.clone()).unwrap(),
						value: message.value,
					})
					.collect();

				trace!(
					target: LOG_TARGET,
					"received deliver : {:?} ",
					messages.iter().map(|message| message.type_url.clone()).collect::<Vec<_>>()
				);

				let mut results: Vec<IbcEvent> = vec![];
				for (_index, message) in messages.into_iter().enumerate() {

					let mut result = Vec::new();
					match ibc::core::ics26_routing::handler::deliver(&mut ctx, message.clone()) {
						Ok(value) => {
							trace!(
								target: LOG_TARGET,
								"deliver event  : {:?} ",
								value.events
							);
							result = value.events;

						}
						Err(error) => {
							trace!(
								target: LOG_TARGET,
								"deliver error  : {:?} ",
								error
							);
						}
					};

					info!("result: {:?}", result);

					results.append(&mut result);
				}

				// Self::handle_result(&mut ctx, messages, results)?;

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
			trace!(target: LOG_TARGET, "[update_client_state] : update_client_state request.");

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
					info!(target: LOG_TARGET,"[transfer] : source_port: {}", source_port);

					let source_channel = identifier::ChannelId::from_str(
						&String::from_utf8(source_channel).map_err(|_| Error::<T>::InvalidFromUtf8)?,
					)
					.map_err(|_| Error::<T>::InvalidIdentifier)?;
					info!(target: LOG_TARGET,"[transfer] : source_channel : {}", source_channel);

					let token = Some(ibc_proto::cosmos::base::v1beta1::Coin {
						denom: String::from_utf8(token).map_err(|_| Error::<T>::InvalidFromUtf8)?,
						amount: amount.to_string(),
					});
					info!(target: LOG_TARGET,"[transfer] : token : {:?}", token);

					let sender: T::AccountId = ensure_signed(origin)?;
					let encode_sender = T::AccountId::encode(&sender);
					let hex_sender = hex::encode(encode_sender);
					info!(target: LOG_TARGET,"[transfer] : hex sender : 0x{}", hex_sender);

					let sender = Signer::from_str(&hex_sender).unwrap();
					info!(target: LOG_TARGET,"[transfer] : sender : {}", sender);

					let receiver = String::from_utf8(receiver).map_err(|_| Error::<T>::InvalidFromUtf8)?;
					let receiver = Signer::from_str(&receiver).unwrap();
					info!(target: LOG_TARGET,"[transfer] : receiver : {}", receiver);


					let timeout_height =
						height::Height { revision_number: 0, revision_height: timeout_height };
					info!(target: LOG_TARGET,"[transfer] : timeout height : {}", timeout_height);

					let timeout_timestamp = timestamp::Timestamp::from_nanoseconds(timeout_timestamp)
						.map_err(|_| Error::<T>::InvalidTimestamp)?;
					info!(target: LOG_TARGET,"[transfer] : timeout timestamp : {}", timeout_timestamp);

					let _msg = MsgTransfer {
						source_port,
						source_channel,
						token,
						sender,
						receiver,
						timeout_height,
						timeout_timestamp,
					};

					// send to router
					// let mut ctx = Context::<T>::new();
					// let mut output = HandlerOutputBuilder::new();
					// let send_transfer_result = ibc::applications::transfer::relay::send_transfer::send_transfer(&mut ctx, &mut output,  msg).unwrap();

					// ctx.store_packet_result(send_transfer_result.result)
					// 	.map_err(|_| Error::<T>::StorePacketResultError)?;

					// let send_transfer_result_event = send_transfer_result.events;

					// handle the result
					// info!(target: LOG_TARGET,"result: {:?}", send_transfer_result_event);

					// Self::handle_result(&mut ctx, vec![msg.to_any()], send_transfer_result_event)?;

					Ok(())
				}
			)
		}
	}

	impl<T: Config> Pallet<T> {
		/// inner update mmr root
		fn inner_update_mmr_root(
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			trace!(target: LOG_TARGET, "[inner_update_mmr_root]: update_client_state request.");

			// check if the client id exist?
			let client_id_str =
				String::from_utf8(client_id.clone()).map_err(|_| Error::<T>::InvalidFromUtf8)?;
			trace!(
				target: LOG_TARGET,
				"[inner_update_mmr_root]:  client id is {:?}",
				client_id_str
			);

			let decode_received_mmr_root =
				help::MmrRoot::decode(&mut &mmr_root[..]).map_err(|_| Error::<T>::InvalidDecode)?;
			trace!(
				target: LOG_TARGET,
				"[inner_update_mmr_root]:  decode mmr root is {:?}",
				decode_received_mmr_root
			);

			let mut client_state = ClientState::default();

			if !<ClientStates<T>>::contains_key(client_id.clone()) {
				error!("[inner_update_mmr_root]: {:?} client_state not found !", client_id_str);

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

				trace!(
					target: LOG_TARGET,
					"[inner_update_mmr_root]: get client_state from chain storage: {:?}",
					client_state
				);
			}

			let signed_commitment =
				commitment::SignedCommitment::try_from(decode_received_mmr_root.signed_commitment)
					.map_err(|_| Error::<T>::InvalidSignedCommitment)?;

			let rev_block_number = signed_commitment.commitment.block_number;
			if rev_block_number <= client_state.latest_commitment.block_number {
				trace!(
					target: LOG_TARGET,
					"receive mmr root block number({}) less than client_state.latest_commitment.block_number({})",
					rev_block_number, client_state.latest_commitment.block_number
				);

				return Err(Error::<T>::ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber.into());
			}
			// build new beefy light client by client_state
			let mut light_client = beefy_light_client::LightClient {
				latest_commitment: Some(client_state.latest_commitment.clone().into()),
				validator_set: client_state.validator_set.clone().into(),
				in_process_state: None,
			};
			trace!(
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

			// verify mmr proof and update lc state
			let result = light_client.update_state(
				&encoded_signed_commitment,
				&validator_proofs,
				&mmr_leaf,
				&mmr_leaf_proof,
			);

			match result {
				Ok(_) => {
					trace!(
						target: LOG_TARGET ,
						"update the beefy light client success! and the beefy light client state is : {:?} \n",
						light_client
					);

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

					trace!(target: LOG_TARGET, "the updated client state is : {:?}", client_state);

					use ibc::{
						clients::ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
						core::ics02_client::client_consensus::AnyConsensusState,
					};

					let mut consensus_state =
						GPConsensusState::new(client_state.block_header.clone());

					consensus_state.digest = client_state
						.latest_commitment
						.payload
						.get_raw(&MMR_ROOT_ID)
						.cloned()
						.unwrap_or_default();
					let any_consensus_state = AnyConsensusState::Grandpa(consensus_state);

					let height = ibc::Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};

					trace!(
						target: LOG_TARGET,
						"in ibc-lib : [store_consensus_state]\
						 >> client_id: {:?}, height = {:?}, consensus_state = {:?}",
						client_id,
						height,
						any_consensus_state
					);

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

					// emit update state success event
					let event_height = Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};
					let event_client_state = EventClientState::from(client_state);
					Self::deposit_event(Event::<T>::UpdateClientState {
						height: event_height,
						client_state: event_client_state,
					});
				},
				Err(e) => {
					error!(target: LOG_TARGET, "update the beefy light client failure! : {:?}", e);

					return Err(Error::<T>::UpdateBeefyLightClientFailure.into())
				},
			}

			Ok(().into())
		}
	}
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
