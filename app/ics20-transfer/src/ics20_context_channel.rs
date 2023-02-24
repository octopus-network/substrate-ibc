use crate::{ics20_callback::IbcTransferModule, *};
use ibc::core::{ExecutionContext, ValidationContext};
use ibc::{
	core::{
		ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{
				AcknowledgementCommitment as IbcAcknowledgementCommitment, PacketCommitment,
			},
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};
use sp_std::{boxed::Box, time::Duration, vec::Vec};

impl<T: Config> ValidationContext for IbcTransferModule<T> {
	fn client_state(
		&self,
		client_id: &ClientId,
	) -> Result<Box<dyn ClientState>, ibc::core::ContextError> {
		ValidationContext::client_state(&self.ibc_core_context, client_id)
	}

	fn decode_client_state(
		&self,
		client_state: ibc_proto::google::protobuf::Any,
	) -> Result<Box<dyn ClientState>, ibc::core::ContextError> {
		ValidationContext::decode_client_state(&self.ibc_core_context, client_state)
	}

	fn consensus_state(
		&self,
		client_cons_state_path: &ibc::core::ics24_host::path::ClientConsensusStatePath,
	) -> Result<Box<dyn ConsensusState>, ibc::core::ContextError> {
		ValidationContext::consensus_state(&self.ibc_core_context, client_cons_state_path)
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ibc::core::ContextError> {
		ValidationContext::next_consensus_state(&self.ibc_core_context, client_id, height)
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ibc::core::ContextError> {
		ValidationContext::prev_consensus_state(&self.ibc_core_context, client_id, height)
	}

	fn host_height(&self) -> Result<Height, ibc::core::ContextError> {
		ValidationContext::host_height(&self.ibc_core_context)
	}

	fn host_timestamp(&self) -> Result<Timestamp, ibc::core::ContextError> {
		ValidationContext::host_timestamp(&self.ibc_core_context)
	}

	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ibc::core::ContextError> {
		ValidationContext::host_consensus_state(&self.ibc_core_context, height)
	}

	fn client_counter(&self) -> Result<u64, ibc::core::ContextError> {
		ValidationContext::client_counter(&self.ibc_core_context)
	}

	fn connection_end(
		&self,
		conn_id: &ConnectionId,
	) -> Result<ConnectionEnd, ibc::core::ContextError> {
		ValidationContext::connection_end(&self.ibc_core_context, conn_id)
	}

	fn validate_self_client(
		&self,
		client_state_of_host_on_counterparty: ibc_proto::google::protobuf::Any,
	) -> Result<(), ibc::core::ics03_connection::error::ConnectionError> {
		ValidationContext::validate_self_client(
			&self.ibc_core_context,
			client_state_of_host_on_counterparty,
		)
	}

	fn commitment_prefix(&self) -> ibc::core::ics23_commitment::commitment::CommitmentPrefix {
		ValidationContext::commitment_prefix(&self.ibc_core_context)
	}

	fn connection_counter(&self) -> Result<u64, ibc::core::ContextError> {
		ValidationContext::connection_counter(&self.ibc_core_context)
	}

	fn channel_end(
		&self,
		channel_end_path: &ibc::core::ics24_host::path::ChannelEndPath,
	) -> Result<ChannelEnd, ibc::core::ContextError> {
		ValidationContext::channel_end(&self.ibc_core_context, channel_end_path)
	}

	fn connection_channels(
		&self,
		cid: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ibc::core::ContextError> {
		ValidationContext::connection_channels(&self.ibc_core_context, cid)
	}

	fn get_next_sequence_send(
		&self,
		seq_send_path: &ibc::core::ics24_host::path::SeqSendPath,
	) -> Result<Sequence, ibc::core::ContextError> {
		ValidationContext::get_next_sequence_send(&self.ibc_core_context, seq_send_path)
	}

	fn get_next_sequence_recv(
		&self,
		seq_recv_path: &ibc::core::ics24_host::path::SeqRecvPath,
	) -> Result<Sequence, ibc::core::ContextError> {
		ValidationContext::get_next_sequence_recv(&self.ibc_core_context, seq_recv_path)
	}

	fn get_next_sequence_ack(
		&self,
		seq_ack_path: &ibc::core::ics24_host::path::SeqAckPath,
	) -> Result<Sequence, ibc::core::ContextError> {
		ValidationContext::get_next_sequence_ack(&self.ibc_core_context, seq_ack_path)
	}

	fn get_packet_commitment(
		&self,
		commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
	) -> Result<PacketCommitment, ibc::core::ContextError> {
		ValidationContext::get_packet_commitment(&self.ibc_core_context, commitment_path)
	}

	fn get_packet_receipt(
		&self,
		receipt_path: &ibc::core::ics24_host::path::ReceiptPath,
	) -> Result<Receipt, ibc::core::ContextError> {
		ValidationContext::get_packet_receipt(&self.ibc_core_context, receipt_path)
	}

	fn get_packet_acknowledgement(
		&self,
		ack_path: &ibc::core::ics24_host::path::AckPath,
	) -> Result<IbcAcknowledgementCommitment, ibc::core::ContextError> {
		ValidationContext::get_packet_acknowledgement(&self.ibc_core_context, ack_path)
	}

	fn hash(&self, value: &[u8]) -> Vec<u8> {
		ValidationContext::hash(&self.ibc_core_context, value)
	}

	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ibc::core::ContextError> {
		ValidationContext::client_update_time(&self.ibc_core_context, client_id, height)
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ibc::core::ContextError> {
		ValidationContext::client_update_height(&self.ibc_core_context, client_id, height)
	}

	fn channel_counter(&self) -> Result<u64, ibc::core::ContextError> {
		ValidationContext::channel_counter(&self.ibc_core_context)
	}

	fn max_expected_time_per_block(&self) -> Duration {
		ValidationContext::max_expected_time_per_block(&self.ibc_core_context)
	}
}

impl<T: Config> ExecutionContext for IbcTransferModule<T> {
	fn store_client_type(
		&mut self,
		client_type_path: ibc::core::ics24_host::path::ClientTypePath,
		client_type: ibc::core::ics02_client::client_type::ClientType,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_client_type(
			&mut self.ibc_core_context,
			client_type_path,
			client_type,
		)
	}

	fn store_client_state(
		&mut self,
		client_state_path: ibc::core::ics24_host::path::ClientStatePath,
		client_state: Box<dyn ClientState>,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_client_state(
			&mut self.ibc_core_context,
			client_state_path,
			client_state,
		)
	}

	fn store_consensus_state(
		&mut self,
		consensus_state_path: ibc::core::ics24_host::path::ClientConsensusStatePath,
		consensus_state: Box<dyn ConsensusState>,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_consensus_state(
			&mut self.ibc_core_context,
			consensus_state_path,
			consensus_state,
		)
	}

	fn increase_client_counter(&mut self) {
		ExecutionContext::increase_client_counter(&mut self.ibc_core_context)
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_update_time(
			&mut self.ibc_core_context,
			client_id,
			height,
			timestamp,
		)
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_update_height(
			&mut self.ibc_core_context,
			client_id,
			height,
			host_height,
		)
	}

	fn store_connection(
		&mut self,
		connection_path: &ibc::core::ics24_host::path::ConnectionPath,
		connection_end: ConnectionEnd,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_connection(
			&mut self.ibc_core_context,
			connection_path,
			connection_end,
		)
	}

	fn store_connection_to_client(
		&mut self,
		client_connection_path: &ibc::core::ics24_host::path::ClientConnectionPath,
		conn_id: ConnectionId,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_connection_to_client(
			&mut self.ibc_core_context,
			client_connection_path,
			conn_id,
		)
	}

	fn increase_connection_counter(&mut self) {
		ExecutionContext::increase_connection_counter(&mut self.ibc_core_context)
	}

	fn store_packet_commitment(
		&mut self,
		commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
		commitment: PacketCommitment,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_packet_commitment(
			&mut self.ibc_core_context,
			commitment_path,
			commitment,
		)
	}

	fn delete_packet_commitment(
		&mut self,
		commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::delete_packet_commitment(&mut self.ibc_core_context, commitment_path)
	}

	fn store_packet_receipt(
		&mut self,
		receipt_path: &ibc::core::ics24_host::path::ReceiptPath,
		receipt: Receipt,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_packet_receipt(&mut self.ibc_core_context, receipt_path, receipt)
	}

	fn store_packet_acknowledgement(
		&mut self,
		ack_path: &ibc::core::ics24_host::path::AckPath,
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_packet_acknowledgement(
			&mut self.ibc_core_context,
			ack_path,
			ack_commitment,
		)
	}

	fn delete_packet_acknowledgement(
		&mut self,
		ack_path: &ibc::core::ics24_host::path::AckPath,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::delete_packet_acknowledgement(&mut self.ibc_core_context, ack_path)
	}

	fn store_channel(
		&mut self,
		channel_end_path: &ibc::core::ics24_host::path::ChannelEndPath,
		channel_end: ChannelEnd,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_channel(&mut self.ibc_core_context, channel_end_path, channel_end)
	}

	fn store_next_sequence_send(
		&mut self,
		seq_send_path: &ibc::core::ics24_host::path::SeqSendPath,
		seq: Sequence,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_next_sequence_send(&mut self.ibc_core_context, seq_send_path, seq)
	}

	fn store_next_sequence_recv(
		&mut self,
		seq_recv_path: &ibc::core::ics24_host::path::SeqRecvPath,
		seq: Sequence,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_next_sequence_recv(&mut self.ibc_core_context, seq_recv_path, seq)
	}

	fn store_next_sequence_ack(
		&mut self,
		seq_ack_path: &ibc::core::ics24_host::path::SeqAckPath,
		seq: Sequence,
	) -> Result<(), ibc::core::ContextError> {
		ExecutionContext::store_next_sequence_ack(&mut self.ibc_core_context, seq_ack_path, seq)
	}

	fn increase_channel_counter(&mut self) {
		ExecutionContext::increase_channel_counter(&mut self.ibc_core_context)
	}

	fn emit_ibc_event(&mut self, event: ibc::events::IbcEvent) {
		ExecutionContext::emit_ibc_event(&mut self.ibc_core_context, event)
	}

	fn log_message(&mut self, message: String) {
		ExecutionContext::log_message(&mut self.ibc_core_context, message)
	}
}

impl<T: Config> ibc::core::context::Router for IbcTransferModule<T> {
	fn get_route(
		&self,
		module_id: &ibc::core::ics26_routing::context::ModuleId,
	) -> Option<&dyn ibc::core::ics26_routing::context::Module> {
		ibc::core::context::Router::get_route(&self.ibc_core_context, module_id)
	}

	fn get_route_mut(
		&mut self,
		module_id: &ibc::core::ics26_routing::context::ModuleId,
	) -> Option<&mut dyn ibc::core::ics26_routing::context::Module> {
		ibc::core::context::Router::get_route_mut(&mut self.ibc_core_context, module_id)
	}

	fn has_route(&self, module_id: &ibc::core::ics26_routing::context::ModuleId) -> bool {
		ibc::core::context::Router::has_route(&self.ibc_core_context, module_id)
	}

	fn lookup_module_by_port(
		&self,
		port_id: &PortId,
	) -> Option<ibc::core::ics26_routing::context::ModuleId> {
		ibc::core::context::Router::lookup_module_by_port(&self.ibc_core_context, port_id)
	}
}
