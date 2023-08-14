use crate::{prelude::*, Config, PacketCommitment as PacketCommitStore, TENDERMINT_CLIENT_TYPE, *};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use sp_core::{Encode, Get};
use sp_std::{boxed::Box, marker::PhantomData};

use derive_more::{From, TryInto};
use ibc::clients::ics07_tendermint::consensus_state::TENDERMINT_CONSENSUS_STATE_TYPE_URL;
use ibc::{
	applications::transfer::{
		MODULE_ID_STR as TRANSFER_MODULE_ID, PORT_ID_STR as TRANSFER_PORT_ID,
	},
	clients::ics07_tendermint::{
		client_state::ClientState as Ics07ClientState,
		consensus_state::ConsensusState as Ics07ConsensusState,
	},
	core::{
		events::IbcEvent,
		ics02_client::{
			client_state::ClientState, client_type::ClientType, consensus_state::ConsensusState,
			error::ClientError,
		},
		ics03_connection::{connection::ConnectionEnd, error::ConnectionError},
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
			error::{ChannelError, PacketError},
			packet::{Receipt, Sequence},
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::{
			identifier::{ClientId, ConnectionId, PortId},
			path::{
				AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath,
				ClientStatePath, CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath,
				SeqRecvPath, SeqSendPath,
			},
		},
		router::{Module, ModuleId},
		timestamp::Timestamp,
		ContextError, ExecutionContext, ValidationContext,
	},
	Height, Signer,
};

use pallet_ibc_utils::module::{AddModule, Router};

#[derive(Debug, Clone, From, TryInto, PartialEq, ConsensusState)]
pub enum AnyConsensusState {
	Tendermint(Ics07ConsensusState),
}

impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
	type Error = ClientError;

	fn try_from(raw: Any) -> Result<Self, Self::Error> {
		if raw.type_url == TENDERMINT_CONSENSUS_STATE_TYPE_URL {
			Ics07ConsensusState::try_from(raw).map(Into::into)
		} else {
			Err(ClientError::Other { description: "failed to deserialize message".to_string() })
		}
	}
}

impl From<AnyConsensusState> for Any {
	fn from(host_consensus_state: AnyConsensusState) -> Self {
		match host_consensus_state {
			AnyConsensusState::Tendermint(cs) => cs.into(),
		}
	}
}

#[derive(Clone, Debug)]
pub struct IbcContext<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: Router,
}

impl<T: Config> IbcContext<T> {
	pub fn new() -> Self {
		let router = Router::new();
		let r = T::IbcModule::add_module(router);
		Self { _pd: PhantomData, router: r }
	}

	pub fn add_route(
		&mut self,
		module_id: ModuleId,
		module: impl Module + 'static,
	) -> Result<(), String> {
		match self.router.router.insert(module_id, Arc::new(module)) {
			None => Ok(()),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}

	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ClientError> {
		let data = <ClientTypeById<T>>::get(client_id.clone()).ok_or(ClientError::Other {
			description: format!("Client({}) not found!", client_id.clone()),
		})?;
		match data.as_str() {
			TENDERMINT_CLIENT_TYPE => ClientType::new(TENDERMINT_CLIENT_TYPE.into())
				.map_err(|e| ClientError::Other { description: format!("{}", e) }),

			unimplemented => {
				return Err(ClientError::UnknownClientStateType {
					client_state_type: unimplemented.to_string(),
				})
			},
		}
	}
}

impl<T: Config> Default for IbcContext<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> ValidationContext for IbcContext<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment>
	+ From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>,

{
    type ClientValidationContext = Self;
    type E = Self;
    type AnyConsensusState = AnyConsensusState;
    type AnyClientState = Ics07ClientState;

    /// Retrieve the context that implements all clients' `ValidationContext`.
    fn get_client_validation_context(&self) -> &Self::ClientValidationContext {
        todo!()
    }

	/// Returns the ClientState for the given identifier `client_id`.
	fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
		let data = <ClientStates<T>>::get(ClientStatePath(client_id.clone())).ok_or(
			ClientError::Other { description: format!("Client({}) not found!", client_id.clone()) },
		)?;
		match self.client_type(client_id)?.as_str() {
			TENDERMINT_CLIENT_TYPE => {
				log::info!("client_state({})", TENDERMINT_CLIENT_TYPE);
				let result: Ics07ClientState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ClientState failed: {:?}", e),
					})?;

				Ok(result)
			},

			unimplemented => {
				log::info!("client_state({})", unimplemented);
				Err(ClientError::Other {
					description: format!("unknow client state type:({})", unimplemented),
				}
				.into())
			},
		}
	}

	/// Tries to decode the given `client_state` into a concrete light client state.
	fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError> {
		if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
			Ok(client_state)
		} else {
			Err(ClientError::UnknownClientStateType { client_state_type: client_state.type_url }
				.into())
		}
	}

	/// Retrieve the consensus state for the given client ID at the specified
	/// height.
	///
	/// Returns an error if no such state exists.
	fn consensus_state(
		&self,
		client_cons_state_path: &ClientConsensusStatePath,
	) -> Result<Self::AnyConsensusState, ContextError> {
		let data = <ConsensusStates<T>>::get(client_cons_state_path).ok_or(
			ClientError::ConsensusStateNotFound {
				client_id: client_cons_state_path.client_id.clone(),
				height: Height::new(client_cons_state_path.epoch, client_cons_state_path.height)
					.map_err(|e| ClientError::Other {
						description: format!("contruct height error({})", e),
					})?,
			},
		)?;
		match self.client_type(&client_cons_state_path.client_id)?.as_str() {
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ConsensusState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ConsensusState failed: {:?}", e),
					})?;
				Ok(AnyConsensusState::Tendermint(result))
			},

			unimplemented => Err(ClientError::Other {
				description: format!("unknow client state type: {}", unimplemented),
			}
			.into()),
		}
	}


	/// Returns the current height of the local chain.
	fn host_height(&self) -> Result<Height, ContextError> {
		let block_height = <frame_system::Pallet<T>>::block_number();
		Height::new(T::ChainVersion::get(), block_height.into()).map_err(|e| {
			ClientError::Other { description: format!("contruct Ibc Height error: {}", e) }.into()
		})
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
		let current_time = <pallet_timestamp::Pallet<T>>::get();
		let duration = Duration::from_millis(current_time.into());
		Timestamp::from_nanoseconds(duration.as_nanos() as u64).map_err(|e| {
			ClientError::Other { description: format!("get host time stamp error: {}", e) }.into()
		})
	}

	/// Returns the `ConsensusState` of the host (local) chain at a specific height.
	fn host_consensus_state(
		&self,
		_height: &Height,
	) -> Result<Self::AnyConsensusState, ContextError> {
		Err(ClientError::Other { description: "no implement host_consensus_state".to_string() }
			.into())
	}

	/// Returns a natural number, counting how many clients have been created
	/// thus far. The value of this counter should increase only via method
	/// `ExecutionContext::increase_client_counter`.
	fn client_counter(&self) -> Result<u64, ContextError> {
		Ok(<ClientCounter<T>>::get())
	}

	/// Returns the ConnectionEnd for the given identifier `conn_id`.
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
		<Connections<T>>::get(ConnectionPath(conn_id.clone())).ok_or(
			ConnectionError::Other {
				description: format!("Can't get ConnectionEnd by ConnectionId({})", conn_id),
			}
			.into(),
		)
	}

	/// Validates the `ClientState` of the client (a client referring to host) stored on the
	/// counterparty chain against the host's internal state.
	///
	/// For more information on the specific requirements for validating the
	/// client state of a host chain, please refer to the [ICS24 host
	/// requirements](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#client-state-validation)
	///
	/// Additionally, implementations specific to individual chains can be found
	/// in the [hosts](crate::hosts) module.
	fn validate_self_client(
		&self,
		_client_state_of_host_on_counterparty: Any,
	) -> Result<(), ContextError> {
		// todo(davirain) need Add
		Ok(())
	}

	/// Returns the prefix that the local chain uses in the KV store.
	fn commitment_prefix(&self) -> CommitmentPrefix {
		CommitmentPrefix::try_from(T::IBC_COMMITMENT_PREFIX.to_vec()).unwrap_or_default()
	}

	/// Returns a counter on how many connections have been created thus far.
	fn connection_counter(&self) -> Result<u64, ContextError> {
		Ok(<ConnectionCounter<T>>::get())
	}

	/// Returns the `ChannelEnd` for the given `port_id` and `chan_id`.
	fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
		<Channels<T>>::get(channel_end_path).ok_or(
			ChannelError::ChannelNotFound {
				port_id: channel_end_path.0.clone(),
				channel_id: channel_end_path.1.clone(),
			}
			.into(),
		)
	}

	/// Returns the sequence number for the next packet to be sent for the given store path
	fn get_next_sequence_send(
		&self,
		seq_send_path: &SeqSendPath,
	) -> Result<Sequence, ContextError> {
		<NextSequenceSend<T>>::get(seq_send_path).ok_or(
			PacketError::MissingNextSendSeq {
				port_id: seq_send_path.0.clone(),
				channel_id: seq_send_path.1.clone(),
			}
			.into(),
		)
	}

	/// Returns the sequence number for the next packet to be received for the given store path
	fn get_next_sequence_recv(
		&self,
		seq_recv_path: &SeqRecvPath,
	) -> Result<Sequence, ContextError> {
		<NextSequenceRecv<T>>::get(seq_recv_path).ok_or(
			PacketError::MissingNextRecvSeq {
				port_id: seq_recv_path.0.clone(),
				channel_id: seq_recv_path.1.clone(),
			}
			.into(),
		)
	}

	/// Returns the sequence number for the next packet to be acknowledged for the given store path
	fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
		<NextSequenceAck<T>>::get(seq_ack_path).ok_or(
			PacketError::MissingNextAckSeq {
				port_id: seq_ack_path.0.clone(),
				channel_id: seq_ack_path.1.clone(),
			}
			.into(),
		)
	}

	/// Returns the packet commitment for the given store path
	fn get_packet_commitment(
		&self,
		commitment_path: &CommitmentPath,
	) -> Result<PacketCommitment, ContextError> {
		<PacketCommitStore<T>>::get(commitment_path).ok_or(
			PacketError::PacketCommitmentNotFound { sequence: commitment_path.sequence }.into(),
		)
	}

	/// Returns the packet receipt for the given store path
	fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
		<PacketReceipt<T>>::get(receipt_path)
			.ok_or(PacketError::PacketReceiptNotFound { sequence: receipt_path.sequence }.into())
	}

	/// Returns the packet acknowledgement for the given store path
	fn get_packet_acknowledgement(
		&self,
		ack_path: &AckPath,
	) -> Result<AcknowledgementCommitment, ContextError> {
		<Acknowledgements<T>>::get(ack_path).ok_or(
			PacketError::PacketAcknowledgementNotFound { sequence: ack_path.sequence }.into(),
		)
	}

	// todo(davirian) Don't Know this correct
	/// Returns the time when the client state for the given [`ClientId`] was updated with a header
	/// for the given [`Height`]
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ContextError> {
		let time = <ClientProcessedTimes<T>>::get(client_id, height).ok_or(
			ChannelError::ProcessedTimeNotFound { client_id: client_id.clone(), height: *height },
		)?;

		Timestamp::from_nanoseconds(time)
			.map_err(|e| ChannelError::Other { description: e.to_string() }.into())
	}

	// todo(davirian) Don't Know this correct
	/// Returns the height when the client state for the given [`ClientId`] was updated with a
	/// header for the given [`Height`]
	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ContextError> {
		<ClientProcessedHeights<T>>::get(client_id, height).ok_or(
			ChannelError::ProcessedHeightNotFound { client_id: client_id.clone(), height: *height }
				.into(),
		)
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ExecutionContext::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ContextError> {
		Ok(<ChannelCounter<T>>::get())
	}

	/// Returns the maximum expected time per block
	fn max_expected_time_per_block(&self) -> Duration {
		Duration::from_secs(T::ExpectedBlockTime::get())
	}

	/// Validates the `signer` field of IBC messages, which represents the address
	/// of the user/relayer that signed the given message.
	fn validate_message_signer(&self, _signer: &Signer) -> Result<(), ContextError> {
		// todo(davirian) need Add
		Ok(())
	}
}

impl<T: Config> ExecutionContext for IbcContext<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment>
	+ From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>,
{
    /// Retrieve the context that implements all clients' `ExecutionContext`.
    fn get_client_execution_context(&mut self) -> &mut Self::E {
        todo!()
    }

	/// Called upon client creation.
	/// Increases the counter which keeps track of how many clients have been created.
	/// Should never fail.
	fn increase_client_counter(&mut self) {
		let _ = <ClientCounter<T>>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	/// Called upon successful client update.
	/// Implementations are expected to use this to record the specified time as the time at which
	/// this update (or header) was processed.
	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ContextError> {
		<ClientProcessedTimes<T>>::insert(client_id, height, timestamp.nanoseconds());
		Ok(())
	}

	/// Called upon successful client update.
	/// Implementations are expected to use this to record the specified height as the height at
	/// at which this update (or header) was processed.
	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ContextError> {
		<ClientProcessedHeights<T>>::insert(client_id, height, host_height);

		Ok(())
	}

	/// Stores the given connection_end at path
	fn store_connection(
		&mut self,
		connection_path: &ConnectionPath,
		connection_end: ConnectionEnd,
	) -> Result<(), ContextError> {
		<Connections<T>>::insert(connection_path, connection_end);

		Ok(())
	}

	/// Stores the given connection_id at a path associated with the client_id.
	fn store_connection_to_client(
		&mut self,
		client_connection_path: &ClientConnectionPath,
		conn_id: ConnectionId,
	) -> Result<(), ContextError> {
		<ConnectionClient<T>>::insert(client_connection_path, conn_id);

		Ok(())
	}

	/// Called upon connection identifier creation (Init or Try process).
	/// Increases the counter which keeps track of how many connections have been created.
	/// Should never fail.
	fn increase_connection_counter(&mut self) {
		let _ = <ConnectionCounter<T>>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	/// Stores the given packet commitment at the given store path
	fn store_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
		commitment: PacketCommitment,
	) -> Result<(), ContextError> {
		<PacketCommitStore<T>>::insert(commitment_path, commitment);

		Ok(())
	}

	/// Deletes the packet commitment at the given store path
	fn delete_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
	) -> Result<(), ContextError> {
		<PacketCommitStore<T>>::remove(commitment_path);

		Ok(())
	}

	/// Stores the given packet receipt at the given store path
	fn store_packet_receipt(
		&mut self,
		receipt_path: &ReceiptPath,
		receipt: Receipt,
	) -> Result<(), ContextError> {
		<PacketReceipt<T>>::insert(receipt_path, receipt);

		Ok(())
	}

	/// Stores the given packet acknowledgement at the given store path
	fn store_packet_acknowledgement(
		&mut self,
		ack_path: &AckPath,
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), ContextError> {
		<Acknowledgements<T>>::insert(ack_path, ack_commitment);

		Ok(())
	}

	/// Deletes the packet acknowledgement at the given store path
	fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
		<Acknowledgements<T>>::remove(ack_path);

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		channel_end_path: &ChannelEndPath,
		channel_end: ChannelEnd,
	) -> Result<(), ContextError> {
		<Channels<T>>::insert(channel_end_path, channel_end);

		Ok(())
	}

	/// Stores the given `nextSequenceSend` number at the given store path
	fn store_next_sequence_send(
		&mut self,
		seq_send_path: &SeqSendPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		<NextSequenceSend<T>>::insert(seq_send_path, seq);

		Ok(())
	}

	/// Stores the given `nextSequenceRecv` number at the given store path
	fn store_next_sequence_recv(
		&mut self,
		seq_recv_path: &SeqRecvPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		<NextSequenceRecv<T>>::insert(seq_recv_path, seq);

		Ok(())
	}

	/// Stores the given `nextSequenceAck` number at the given store path
	fn store_next_sequence_ack(
		&mut self,
		seq_ack_path: &SeqAckPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		<NextSequenceAck<T>>::insert(seq_ack_path, seq);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		let _ = ChannelCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	// todo(davirian) Don't Know this correct
	/// Emit the given IBC event
	fn emit_ibc_event(&mut self, event: IbcEvent) {
		log::info!("⌚️⌚️⌚️ emit_ibc_event:  ({:?})", event);

		let mut key = b"pallet-ibc:ibc-event".to_vec();
		let mut value = sp_io::hashing::sha2_256(&event.encode()).to_vec();
		let _ = key.append(&mut value);

		// store ibc event
		sp_io::offchain_index::set(&key, event.encode().as_slice());

		// store ibc event key
		let _ = IbcEventKey::<T>::try_mutate::<_, (), _>(|val| {
			val.push(key);
			Ok(())
		});

		let _ = <IbcEventStorage<T>>::try_mutate::<_, (), _>(|val| {
			val.push(event.clone());
			Ok(())
		});
		log::trace!("emit ibc event: {:?}", event);
	}

	/// Log the given message.
	fn log_message(&mut self, message: String) {
		log::info!("📒📒 log_message:  ({})", message);

		// store ibc log
		let _ = IbcLogStorage::<T>::try_mutate::<_, (), _>(|val| {
			val.push(message.as_bytes().to_vec());
			Ok(())
		});

		log::trace!("emit ibc event: {:?}", message);
	}
}

#[test]
fn test_public_key_der() {
	let fix_public_key = "{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"A5W0C7iEAuonX56sR81PiwaKTE0GvZlCYuGwHTMpWJo+\"}";
	let _fix_public_key = fix_public_key.parse::<PublicKey>().unwrap();
}
