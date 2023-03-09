use core::time::Duration;

use crate::{PacketCommitment as PalletPacketCommitment, *};
use alloc::{borrow::ToOwned, string::String, sync::Arc};
use codec::Encode;
use frame_support::traits::Get;
use ibc::{
	clients::ics07_tendermint::{
		client_state::ClientState as Ics07ClientState,
		consensus_state::ConsensusState as Ics07ConsensusState,
	},
	core::{
		ics02_client::{
			client_state::ClientState, consensus_state::ConsensusState, error::ClientError,
		},
		ics03_connection::error::ConnectionError,
		ics04_channel::{
			commitment::AcknowledgementCommitment,
			error::{ChannelError, PacketError},
			packet::Receipt,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::path::{
			AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath,
			ClientStatePath, ClientTypePath, CommitmentPath, ConnectionPath, ReceiptPath,
			SeqAckPath, SeqRecvPath, SeqSendPath,
		},
		ics26_routing::context::{Module, ModuleId},
		ContextError, ExecutionContext, ValidationContext,
	},
	events::IbcEvent,
	mock::consensus_state::MockConsensusState,
	timestamp::Timestamp,
};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use sp_std::{boxed::Box, marker::PhantomData};

/// A struct capturing all the functional dependencies (i.e., context)
/// which the ICS26 module requires to be able to dispatch and process IBC messages.
use crate::routing::{Router, SubstrateRouterBuilder};
use ibc::{
	core::{
		ics02_client::client_type::ClientType,
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{channel::ChannelEnd, commitment::PacketCommitment, packet::Sequence},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	mock::client_state::{client_type as mock_client_type, MockClientState},
	Height,
};

#[derive(Clone, Debug)]
pub struct Context<T> {
	pub _pd: PhantomData<T>,
	pub router: Router,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let r = SubstrateRouterBuilder::default().build();

		Self { _pd: PhantomData::default(), router: r }
	}

	pub fn add_route(&mut self, module_id: ModuleId, module: impl Module) -> Result<(), String> {
		match self.router.0.insert(module_id, Arc::new(module)) {
			None => Ok(()),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> ValidationContext for Context<T> {
	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
		let data = Pallet::<T>::client_state(&client_id).ok_or::<ContextError>(
			ClientError::ClientNotFound { client_id: client_id.clone() }.into(),
		)?;
		match <Pallet<T>>::client_type(client_id)
			.ok_or(ClientError::Other { description: "Cannt get Client type".to_string() })?
			.as_str()
		{
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ClientState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ClientState failed: {:?}", e),
					})?;

				Ok(Box::new(result))
			},
			MOCK_CLIENT_TYPE => {
				let result: MockClientState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Deocode Ics10ClientState failed: {:?}", e),
					})?;
				Ok(Box::new(result))
			},
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}
			.into()),
		}
	}

	fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ContextError> {
		if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
			return Ok(client_state.into_box())
		}
		#[cfg(test)]
		if let Ok(client_state) = MockClientState::try_from(client_state.clone()) {
			return Ok(client_state.into_box())
		}
		Err(ClientError::UnknownClientStateType { client_state_type: client_state.type_url }.into())
	}

	fn consensus_state(
		&self,
		client_cons_state_path: &ClientConsensusStatePath,
	) -> Result<Box<dyn ConsensusState>, ContextError> {
		let client_id = client_cons_state_path.client_id.clone();
		let epoch = client_cons_state_path.epoch;
		let height = client_cons_state_path.height;
		let height = Height::new(epoch, height)
			.map_err(|e| ClientError::Other { description: format!("{}", e) })?;
		let data = Pallet::<T>::consensus_state(client_id.clone(), height)
			.ok_or(ClientError::ConsensusStateNotFound { client_id: client_id.clone(), height })?;
		match <Pallet<T>>::client_type(client_id)
			.ok_or(ClientError::Other { description: "Cannt get Client type".to_string() })?
			.as_str()
		{
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ConsensusState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ConsensusState failed: {:?}", e),
					})?;
				Ok(Box::new(result))
			},
			MOCK_CLIENT_TYPE => {
				let result: MockConsensusState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode MockConsensusState failed: {:?}", e),
					})?;
				Ok(Box::new(result))
			},
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}
			.into()),
		}
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
		let client_consensus_state_key =
			<ConsensusStates<T>>::iter_keys().collect::<Vec<(ClientId, Height)>>();
		let mut heights = client_consensus_state_key
			.into_iter()
			.map(|(_, height)| height)
			.collect::<Vec<Height>>();

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h > *height {
				let data = <ConsensusStates<T>>::get(client_id, height).ok_or(
					ClientError::ConsensusStateNotFound {
						client_id: client_id.clone(),
						height: *height,
					},
				)?;
				match <Pallet<T>>::client_type(client_id)
					.ok_or(ClientError::Other { description: "Cannt get Client type".to_string() })?
					.as_str()
				{
					TENDERMINT_CLIENT_TYPE => {
						let result: Ics07ConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| ClientError::Other {
								description: format!("Decode Ics07ConsensusState failed: {:?}", e),
							})?;
						return Ok(Some(Box::new(result)))
					},
					MOCK_CLIENT_TYPE => {
						let result: MockConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| ClientError::Other {
								description: format!("Decode MockConsensusState failed: {:?}", e),
							})?;
						return Ok(Some(Box::new(result)))
					},
					_ => {},
				}
			}
		}
		Ok(None)
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
		let client_consensus_state_key =
			<ConsensusStates<T>>::iter_keys().collect::<Vec<(ClientId, Height)>>();
		let mut heights = client_consensus_state_key
			.into_iter()
			.map(|(_, height)| height)
			.collect::<Vec<Height>>();

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h < *height {
				let data = <ConsensusStates<T>>::get(client_id, height).ok_or(
					ClientError::ConsensusStateNotFound {
						client_id: client_id.clone(),
						height: *height,
					},
				)?;
				match <Pallet<T>>::client_type(client_id)
					.ok_or(ClientError::Other { description: "Cannt get Client type".to_string() })?
					.as_str()
				{
					TENDERMINT_CLIENT_TYPE => {
						let result: Ics07ConsensusState = ibc_proto::protobuf::Protobuf::<
							ibc_proto::google::protobuf::Any,
						>::decode_vec(&data)
						.map_err(|e| ClientError::Other {
							description: format!("Decode Ics07ConsensusState failed: {:?}", e),
						})?;
						return Ok(Some(Box::new(result)))
					},
					MOCK_CLIENT_TYPE => {
						let result: MockConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| ClientError::Other {
								description: format!("Decode MockConsensusState failed: {:?}", e),
							})?;
						return Ok(Some(Box::new(result)))
					},
					_ => {},
				}
			}
		}
		Ok(None)
	}

	fn host_height(&self) -> Result<Height, ContextError> {
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		Height::new(REVISION_NUMBER, current_height)
			.map_err(|e| ClientError::Other { description: format!("{}", e) }.into())
	}

	fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
		#[cfg(not(test))]
		{
			use frame_support::traits::UnixTime;
			let nanoseconds = <T as Config>::TimeProvider::now().as_nanos();
			return Ok(Timestamp::from_nanoseconds(nanoseconds as u64).unwrap())
		}
		#[cfg(test)]
		{
			Ok(Timestamp::now())
		}
	}

	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ContextError> {
		#[cfg(not(test))]
		{
			Err(ClientError::ImplementationSpecific.into())
		}
		#[cfg(test)]
		{
			use ibc::mock::header::MockHeader;
			let mock_header =
				MockHeader { height: self.host_height()?, timestamp: Default::default() };
			Ok(Box::new(MockConsensusState::new(mock_header)))
		}
	}

	fn client_counter(&self) -> Result<u64, ContextError> {
		Ok(Pallet::<T>::client_cnt())
	}

	fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
		Pallet::<T>::connection_end(cid)
			.ok_or(ConnectionError::ConnectionNotFound { connection_id: cid.clone() }.into())
	}

	fn validate_self_client(
		&self,
		client_state_of_host_on_counterparty: Any,
	) -> Result<(), ContextError> {
		// let mock_client_state = MockClientState::try_from(client_state_of_host_on_counterparty)
		// 	.map_err(|_| ConnectionError::InvalidClientState {
		// 		reason: "client must be a mock client".to_string(),
		// 	})?;

		// if mock_client_state.is_frozen() {
		// 	return Err(ConnectionError::InvalidClientState {
		// 		reason: "client is frozen".to_string(),
		// 	})
		// }

		// CommitmentPrefix::try_from(b"mock".to_vec()).unwrap()
		Ok(())
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		CommitmentPrefix::try_from(T::IBC_COMMITMENT_PREFIX.to_vec()).unwrap_or_default()
	}

	fn connection_counter(&self) -> Result<u64, ContextError> {
		Ok(Pallet::<T>::connection_cnt())
	}

	fn channel_end(&self, chan_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
		let port_id = &chan_end_path.0;
		let channel_id = &chan_end_path.1;

		Pallet::<T>::channel_end(port_id, channel_id).ok_or(
			ChannelError::ChannelNotFound {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			}
			.into(),
		)
	}

	fn get_next_sequence_send(
		&self,
		seq_send_path: &SeqSendPath,
	) -> Result<Sequence, ContextError> {
		let port_id = &seq_send_path.0;
		let channel_id = &seq_send_path.1;

		Pallet::<T>::get_next_sequence_send(port_id, channel_id).ok_or(
			PacketError::MissingNextSendSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			}
			.into(),
		)
	}

	fn get_next_sequence_recv(
		&self,
		seq_recv_path: &SeqRecvPath,
	) -> Result<Sequence, ContextError> {
		let port_id = &seq_recv_path.0;
		let channel_id = &seq_recv_path.1;

		Pallet::<T>::get_next_sequence_recv(port_id, channel_id).ok_or(
			PacketError::MissingNextRecvSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			}
			.into(),
		)
	}

	fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
		let port_id = &seq_ack_path.0;
		let channel_id = &seq_ack_path.1;

		Pallet::<T>::get_next_sequence_ack(port_id, channel_id).ok_or(
			PacketError::MissingNextAckSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			}
			.into(),
		)
	}

	fn get_packet_commitment(
		&self,
		commitment_path: &CommitmentPath,
	) -> Result<PacketCommitment, ContextError> {
		let port_id = &commitment_path.port_id;
		let channel_id = &commitment_path.channel_id;
		let seq = &commitment_path.sequence;

		Pallet::<T>::get_packet_commitment((port_id, channel_id, seq))
			.ok_or(PacketError::PacketCommitmentNotFound { sequence: *seq }.into())
	}

	fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
		let port_id = &receipt_path.port_id;
		let channel_id = &receipt_path.channel_id;
		let seq = &receipt_path.sequence;

		Pallet::<T>::get_packet_receipt((port_id, channel_id, seq))
			.ok_or(PacketError::PacketReceiptNotFound { sequence: *seq }.into())
	}

	fn get_packet_acknowledgement(
		&self,
		ack_path: &AckPath,
	) -> Result<AcknowledgementCommitment, ContextError> {
		let port_id = &ack_path.port_id;
		let channel_id = &ack_path.channel_id;
		let seq = &ack_path.sequence;

		Pallet::<T>::get_packet_acknowledgement((port_id, channel_id, seq))
			.ok_or(PacketError::PacketAcknowledgementNotFound { sequence: *seq }.into())
	}

	fn hash(&self, value: &[u8]) -> Vec<u8> {
		sp_io::hashing::sha2_256(value).to_vec()
	}

	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ContextError> {
		let time = Pallet::<T>::client_update_time(client_id, height).ok_or(
			ChannelError::ProcessedTimeNotFound { client_id: client_id.clone(), height: *height },
		)?;

		Timestamp::from_nanoseconds(time)
			.map_err(|e| ChannelError::Other { description: e.to_string() }.into())
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ContextError> {
		Pallet::<T>::client_update_height(client_id, height).ok_or(
			ChannelError::ProcessedHeightNotFound { client_id: client_id.clone(), height: *height }
				.into(),
		)
	}

	fn channel_counter(&self) -> Result<u64, ContextError> {
		Ok(Pallet::<T>::channel_cnt())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		Duration::from_secs(T::ExpectedBlockTime::get())
	}
}

impl<T: Config> ExecutionContext for Context<T> {
	fn store_client_type(
		&mut self,
		client_type_path: ClientTypePath,
		client_type: ClientType,
	) -> Result<(), ContextError> {
		let client_id = client_type_path.0;

		<Clients<T>>::insert(client_id, client_type);

		Ok(())
	}

	fn store_client_state(
		&mut self,
		client_state_path: ClientStatePath,
		client_state: Box<dyn ClientState>,
	) -> Result<(), ContextError> {
		let client_id = client_state_path.0;
		let data = client_state.encode_vec().map_err(|e| ClientError::Other {
			description: format!("Encode ClientState Failed: {:?}", e),
		})?;

		<ClientStates<T>>::insert(client_id, data);
		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		consensus_state_path: ClientConsensusStatePath,
		consensus_state: Box<dyn ConsensusState>,
	) -> Result<(), ContextError> {
		let client_id = consensus_state_path.client_id.clone();
		let height = Height::new(consensus_state_path.epoch, consensus_state_path.height)
			.map_err(|e| ClientError::Other { description: format!("{}", e) })?;

		let consensus_state = consensus_state.encode_vec().map_err(|e| ClientError::Other {
			description: format!("Encode ConsensusStates failed: {:?}", e),
		})?;

		<ConsensusStates<T>>::insert(client_id, height, consensus_state);

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		let _ = ClientCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ContextError> {
		<ClientProcessedTimes<T>>::insert(client_id, height, timestamp.nanoseconds());

		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ContextError> {
		<ClientProcessedHeights<T>>::insert(client_id, height, host_height);

		Ok(())
	}

	fn store_connection(
		&mut self,
		connection_path: &ConnectionPath,
		connection_end: ConnectionEnd,
	) -> Result<(), ContextError> {
		let connection_id = connection_path.0.clone();

		<Connections<T>>::insert(connection_id, connection_end);

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

	fn increase_connection_counter(&mut self) {
		let _ = ConnectionCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	fn store_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
		commitment: PacketCommitment,
	) -> Result<(), ContextError> {
		let port_id = commitment_path.port_id.clone();
		let channel_id = commitment_path.channel_id.clone();
		let sequence = commitment_path.sequence.clone();

		<PalletPacketCommitment<T>>::insert((port_id, channel_id, sequence), commitment);

		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
	) -> Result<(), ContextError> {
		let port_id = commitment_path.port_id.clone();
		let channel_id = commitment_path.channel_id.clone();
		let sequence = commitment_path.sequence.clone();

		<PalletPacketCommitment<T>>::remove((port_id, channel_id, sequence));

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		path: &ReceiptPath,
		receipt: Receipt,
	) -> Result<(), ContextError> {
		let port_id = path.port_id.clone();
		let channel_id = path.channel_id.clone();
		let sequence = path.sequence.clone();

		<PacketReceipt<T>>::insert((port_id, channel_id, sequence), receipt);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		ack_path: &AckPath,
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), ContextError> {
		let port_id = ack_path.port_id.clone();
		let channel_id = ack_path.channel_id.clone();
		let seq = ack_path.sequence;

		<Acknowledgements<T>>::insert((port_id, channel_id, seq), ack_commitment);

		Ok(())
	}

	fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
		let port_id = ack_path.port_id.clone();
		let channel_id = ack_path.channel_id.clone();
		let sequence = ack_path.sequence;

		<Acknowledgements<T>>::remove((port_id, channel_id, sequence));

		Ok(())
	}

	fn store_channel(
		&mut self,
		channel_end_path: &ChannelEndPath,
		channel_end: ChannelEnd,
	) -> Result<(), ContextError> {
		let port_id = channel_end_path.0.clone();
		let channel_id = channel_end_path.1.clone();

		<Channels<T>>::insert(port_id, channel_id, channel_end);

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		seq_send_path: &SeqSendPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		let port_id = seq_send_path.0.clone();
		let channel_id = seq_send_path.1.clone();

		<NextSequenceSend<T>>::insert(port_id, channel_id, seq);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		seq_recv_path: &SeqRecvPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		let port_id = seq_recv_path.0.clone();
		let channel_id = seq_recv_path.1.clone();

		<NextSequenceRecv<T>>::insert(port_id, channel_id, seq);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		seq_ack_path: &SeqAckPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		let port_id = seq_ack_path.0.clone();
		let channel_id = seq_ack_path.1.clone();

		<NextSequenceAck<T>>::insert(port_id, channel_id, seq);

		Ok(())
	}

	fn increase_channel_counter(&mut self) {
		let _ = ChannelCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	fn emit_ibc_event(&mut self, event: IbcEvent) {
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
	}

	fn log_message(&mut self, message: String) {
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
	}
}
