use core::time::Duration;

use crate::{PacketCommitment as PacketCommitStore, *};
use alloc::{borrow::ToOwned, string::String, sync::Arc};
use codec::Encode;
use frame_support::traits::Get;
use ibc::clients::tendermint::client_state::ClientState as Ics07ClientState;
use ibc::core::channel::types::commitment::AcknowledgementCommitment;
use ibc::core::channel::types::error::{ChannelError, PacketError};
use ibc::core::channel::types::packet::Receipt;
use ibc::core::client::context::types::error::ClientError;
use ibc::core::client::context::{client_state::ClientState, consensus_state::ConsensusState};
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::host::ExecutionContext;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::Signer;
use ibc::core::primitives::Timestamp;
use ibc::core::router::module::Module;
use ibc::core::router::types::module::ModuleId;
use ibc::{
	clients::tendermint::consensus_state::ConsensusState as Ics07ConsensusState,
	core::host::types::path::{
		AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
		CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
	},
};
use ibc_proto::{google::protobuf::Any, Protobuf};
use sp_std::{boxed::Box, marker::PhantomData};

use crate::client::AnyClientState;
use crate::client::AnyConsensusState;
/// A struct capturing all the functional dependencies (i.e., context)
/// which the ICS26 module requires to be able to dispatch and process IBC messages.
use crate::router::IbcRouter;
use ibc::apps::transfer::types::MODULE_ID_STR as TRANSFER_MODULE_ID;
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::PacketCommitment;
use ibc::core::client::context::types::Height;
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::host::types::identifiers::ClientType;
use ibc::core::host::types::identifiers::Sequence;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};

#[derive(Clone, Debug)]
pub struct Context<T> {
	pub _pd: PhantomData<T>,
	pub router: IbcRouter,
}

impl<T: Config> ClientValidationContext for Context<T> {
	/// Returns the time when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ContextError> {
		todo!()
	}

	/// Returns the height when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ContextError> {
		todo!()
	}
}

impl<T: Config> ClientExecutionContext for Context<T> {
	type V = Self;
	type AnyClientState = AnyClientState;
	type AnyConsensusState = AnyConsensusState;

	/// Called upon successful client creation and update
	fn store_client_state(
		&mut self,
		client_state_path: ClientStatePath,
		client_state: Self::AnyClientState,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Called upon successful client creation and update
	fn store_consensus_state(
		&mut self,
		consensus_state_path: ClientConsensusStatePath,
		consensus_state: Self::AnyConsensusState,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Delete the consensus state from the store located at the given `ClientConsensusStatePath`
	fn delete_consensus_state(
		&mut self,
		consensus_state_path: ClientConsensusStatePath,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Called upon successful client update.
	/// Implementations are expected to use this to record the specified time as the time at which
	/// this update (or header) was processed.
	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_timestamp: Timestamp,
	) -> Result<(), ContextError> {
		todo!()
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
		todo!()
	}

	/// Delete the update time associated with the client at the specified height. This update
	/// time should be associated with a consensus state through the specified height.
	///
	/// Note that this timestamp is determined by the host.
	fn delete_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Delete the update height associated with the client at the specified height. This update
	/// time should be associated with a consensus state through the specified height.
	fn delete_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
	) -> Result<(), ContextError> {
		todo!()
	}
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		Self { _pd: PhantomData::default(), router: IbcRouter::default() }
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

	pub fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ContextError> {
		let data = <ClientTypeById<T>>::get(client_id.clone()).ok_or(ClientError::Other {
			description: format!("Client({}) not found!", client_id.clone()),
		})?;
		match data.as_str() {
			TENDERMINT_CLIENT_TYPE => ClientType::new(TENDERMINT_CLIENT_TYPE.into())
				.map_err(|e| ClientError::Other { description: format!("{}", e) }.into()),
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}
			.into()),
		}
	}

	pub fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
		match port_id.as_str() {
			TRANSFER_PORT_ID => Some(ModuleId::new(TRANSFER_MODULE_ID.to_string())),
			_ => None,
		}
	}
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> ValidationContext for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	type V = Self;
	type E = Self;
	type AnyConsensusState = AnyConsensusState;
	type AnyClientState = AnyClientState;

	/// Retrieve the context that implements all clients' `ValidationContext`.
	fn get_client_validation_context(&self) -> &Self::V {
		self
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
		let data = <ClientStates<T>>::get(ClientStatePath(client_id.clone())).ok_or(
			ClientError::Other { description: format!("Client({}) not found!", client_id.clone()) },
		)?;
		match self.client_type(&client_id)? {
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ClientState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ClientState failed: {:?}", e),
					})?;

				Ok(result.into())
			},
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}
			.into()),
		}
	}

	fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError> {
		if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
			return Ok(client_state.into());
		}
		Err(ClientError::UnknownClientStateType { client_state_type: client_state.type_url }.into())
	}

	fn consensus_state(
		&self,
		client_cons_state_path: &ClientConsensusStatePath,
	) -> Result<Self::AnyConsensusState, ContextError> {
		let client_id = client_cons_state_path.client_id.clone();
		let epoch = client_cons_state_path.revision_number;
		let height = client_cons_state_path.revision_height;
		let height = Height::new(epoch, height)
			.map_err(|e| ClientError::Other { description: format!("{}", e) })?;
		let data = <ConsensusStates<T>>::get(client_cons_state_path).ok_or(
			ClientError::ConsensusStateNotFound {
				client_id: client_cons_state_path.client_id.clone(),
				height: Height::new(
					client_cons_state_path.revision_number,
					client_cons_state_path.revision_height,
				)
				.map_err(|e| ClientError::Other {
					description: format!("contruct height error({})", e),
				})?,
			},
		)?;
		match self.client_type(&client_id)? {
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ConsensusState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ConsensusState failed: {:?}", e),
					})?;
				Ok(result.into())
			},
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}
			.into()),
		}
	}

	fn host_height(&self) -> Result<Height, ContextError> {
		let block_height = <frame_system::Pallet<T>>::block_number();
		Height::new(REVISION_NUMBER, u64::from(block_height)).map_err(|e| {
			ClientError::Other { description: format!("contruct Ibc Height error: {}", e) }.into()
		})
	}

	fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
		let current_time = <pallet_timestamp::Pallet<T>>::get();
		Timestamp::from_nanoseconds(current_time.into()).map_err(|e| {
			ClientError::Other { description: format!("get host time stamp error: {}", e) }.into()
		})
	}

	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Self::AnyConsensusState, ContextError> {
		Err(ClientError::Other { description: "unimplement host consensus state".to_string() }
			.into())
	}

	fn client_counter(&self) -> Result<u64, ContextError> {
		Ok(Pallet::<T>::client_cnt())
	}

	fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
		<Connections<T>>::get(ConnectionPath(cid.clone())).ok_or(
			ConnectionError::Other {
				description: format!("Can't get ConnectionEnd by ConnectionId({})", cid),
			}
			.into(),
		)
	}

	fn validate_self_client(
		&self,
		client_state_of_host_on_counterparty: Any,
	) -> Result<(), ContextError> {
		Ok(())
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		CommitmentPrefix::try_from(T::IBC_COMMITMENT_PREFIX.to_vec()).unwrap_or_default()
	}

	fn connection_counter(&self) -> Result<u64, ContextError> {
		Ok(Pallet::<T>::connection_cnt())
	}

	fn channel_end(&self, chan_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
		<Channels<T>>::get(chan_end_path).ok_or(
			ChannelError::ChannelNotFound {
				port_id: chan_end_path.0.clone(),
				channel_id: chan_end_path.1.clone(),
			}
			.into(),
		)
	}

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

	fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
		<NextSequenceAck<T>>::get(seq_ack_path).ok_or(
			PacketError::MissingNextAckSeq {
				port_id: seq_ack_path.0.clone(),
				channel_id: seq_ack_path.1.clone(),
			}
			.into(),
		)
	}

	fn get_packet_commitment(
		&self,
		commitment_path: &CommitmentPath,
	) -> Result<PacketCommitment, ContextError> {
		<PacketCommitStore<T>>::get(commitment_path).ok_or(
			PacketError::PacketCommitmentNotFound { sequence: commitment_path.sequence }.into(),
		)
	}

	fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
		<PacketReceipt<T>>::get(receipt_path)
			.ok_or(PacketError::PacketReceiptNotFound { sequence: receipt_path.sequence }.into())
	}

	fn get_packet_acknowledgement(
		&self,
		ack_path: &AckPath,
	) -> Result<AcknowledgementCommitment, ContextError> {
		<Acknowledgements<T>>::get(ack_path).ok_or(
			PacketError::PacketAcknowledgementNotFound { sequence: ack_path.sequence }.into(),
		)
	}

	fn channel_counter(&self) -> Result<u64, ContextError> {
		Ok(Pallet::<T>::channel_cnt())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		Duration::from_secs(T::ExpectedBlockTime::get())
	}

	/// Validates the `signer` field of IBC messages, which represents the address
	/// of the user/relayer that signed the given message.
	fn validate_message_signer(&self, signer: &Signer) -> Result<(), ContextError> {
		// todo(davirian) need Add
		Ok(())
	}
}

impl<T: Config> ExecutionContext for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	/// Retrieve the context that implements all clients' `ExecutionContext`.
	fn get_client_execution_context(&mut self) -> &mut Self::E {
		self
	}

	fn increase_client_counter(&mut self) -> Result<(), ContextError> {
		let _ = ClientCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
		Ok(())
	}

	fn store_connection(
		&mut self,
		connection_path: &ConnectionPath,
		connection_end: ConnectionEnd,
	) -> Result<(), ContextError> {
		<Connections<T>>::insert(connection_path, connection_end);

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		client_connection_path: &ClientConnectionPath,
		conn_id: ConnectionId,
	) -> Result<(), ContextError> {
		let client_id = client_connection_path.0.clone();

		<ConnectionClient<T>>::insert(client_id, conn_id);

		Ok(())
	}

	fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
		let _ = ConnectionCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
		Ok(())
	}

	fn store_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
		commitment: PacketCommitment,
	) -> Result<(), ContextError> {
		<PacketCommitStore<T>>::insert(commitment_path, commitment);

		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
	) -> Result<(), ContextError> {
		<PacketCommitStore<T>>::remove(commitment_path);

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		path: &ReceiptPath,
		receipt: Receipt,
	) -> Result<(), ContextError> {
		<PacketReceipt<T>>::insert(path, receipt);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		ack_path: &AckPath,
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), ContextError> {
		<Acknowledgements<T>>::insert(ack_path, ack_commitment);

		Ok(())
	}

	fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
		<Acknowledgements<T>>::remove(ack_path);

		Ok(())
	}

	fn store_channel(
		&mut self,
		channel_end_path: &ChannelEndPath,
		channel_end: ChannelEnd,
	) -> Result<(), ContextError> {
		<Channels<T>>::insert(channel_end_path, channel_end);

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		seq_send_path: &SeqSendPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		<NextSequenceSend<T>>::insert(seq_send_path, seq);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		seq_recv_path: &SeqRecvPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		<NextSequenceRecv<T>>::insert(seq_recv_path, seq);

		Ok(())
	}

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
	///
	fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
		let _ = ChannelCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
		Ok(())
	}

	// Emit the given IBC event
	fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError> {
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

		log::trace!("emit ibc event: {:?}", event);
		Ok(())
	}

	// Log the given message.
	fn log_message(&mut self, message: String) -> Result<(), ContextError> {
		let mut key = b"pallet-ibc:ibc-log".to_vec();
		let mut value = sp_io::hashing::sha2_256(&message.as_ref()).to_vec();
		let _ = key.append(&mut value);

		// store ibc log
		sp_io::offchain_index::set(&key, message.as_ref());

		// store ibc log key
		let _ = IbcLogKey::<T>::try_mutate::<_, (), _>(|val| {
			val.push(key);
			Ok(())
		});
		log::trace!("emit ibc event: {:?}", message);
		Ok(())
	}
}
