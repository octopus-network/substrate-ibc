use crate::Config;
use alloc::{borrow::ToOwned, string::String, sync::Arc};
use sp_std::marker::PhantomData;

use core::time::Duration;
use ibc::{
	core::{
		events::IbcEvent,
		ics02_client::{
			client_state::ClientState, consensus_state::ConsensusState, error::ClientError,
		},
		ics03_connection::{connection::ConnectionEnd, error::ConnectionError},
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
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
use ibc_proto::google::protobuf::Any;
use ibc_support::module::{AddModule, Router};

#[derive(Clone, Debug)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: Router,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let router = Router::new();
		let r = T::IbcModule::add_module(router);
		Self { _pd: PhantomData::default(), router: r }
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
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> ibc::core::router::Router for Context<T> {
	/// Returns a reference to a `Module` registered against the specified `ModuleId`
	fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
		todo!()
	}

	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		todo!()
	}

	/// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
	fn has_route(&self, module_id: &ModuleId) -> bool {
		todo!()
	}

	/// Return the module_id associated with a given port_id
	fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
		todo!()
	}
}

impl<T: Config> ValidationContext for Context<T> {
	/// Returns the ClientState for the given identifier `client_id`.
	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
		todo!()
	}

	/// Tries to decode the given `client_state` into a concrete light client state.
	fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ContextError> {
		todo!()
	}

	/// Retrieve the consensus state for the given client ID at the specified
	/// height.
	///
	/// Returns an error if no such state exists.
	fn consensus_state(
		&self,
		client_cons_state_path: &ClientConsensusStatePath,
	) -> Result<Box<dyn ConsensusState>, ContextError> {
		todo!()
	}

	/// Search for the lowest consensus state higher than `height`.
	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
		todo!()
	}

	/// Search for the highest consensus state lower than `height`.
	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
		todo!()
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Result<Height, ContextError> {
		todo!()
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
		todo!()
	}

	/// Returns the `ConsensusState` of the host (local) chain at a specific height.
	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ContextError> {
		todo!()
	}

	/// Returns a natural number, counting how many clients have been created
	/// thus far. The value of this counter should increase only via method
	/// `ExecutionContext::increase_client_counter`.
	fn client_counter(&self) -> Result<u64, ContextError> {
		todo!()
	}

	/// Returns the ConnectionEnd for the given identifier `conn_id`.
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
		todo!()
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
		client_state_of_host_on_counterparty: Any,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Returns the prefix that the local chain uses in the KV store.
	fn commitment_prefix(&self) -> CommitmentPrefix {
		todo!()
	}

	/// Returns a counter on how many connections have been created thus far.
	fn connection_counter(&self) -> Result<u64, ContextError> {
		todo!()
	}

	/// Returns the `ChannelEnd` for the given `port_id` and `chan_id`.
	fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
		todo!()
	}

	/// Returns the sequence number for the next packet to be sent for the given store path
	fn get_next_sequence_send(
		&self,
		seq_send_path: &SeqSendPath,
	) -> Result<Sequence, ContextError> {
		todo!()
	}

	/// Returns the sequence number for the next packet to be received for the given store path
	fn get_next_sequence_recv(
		&self,
		seq_recv_path: &SeqRecvPath,
	) -> Result<Sequence, ContextError> {
		todo!()
	}

	/// Returns the sequence number for the next packet to be acknowledged for the given store path
	fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
		todo!()
	}

	/// Returns the packet commitment for the given store path
	fn get_packet_commitment(
		&self,
		commitment_path: &CommitmentPath,
	) -> Result<PacketCommitment, ContextError> {
		todo!()
	}

	/// Returns the packet receipt for the given store path
	fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
		todo!()
	}

	/// Returns the packet acknowledgement for the given store path
	fn get_packet_acknowledgement(
		&self,
		ack_path: &AckPath,
	) -> Result<AcknowledgementCommitment, ContextError> {
		todo!()
	}

	/// Returns the time when the client state for the given [`ClientId`] was updated with a header
	/// for the given [`Height`]
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ContextError> {
		todo!()
	}

	/// Returns the height when the client state for the given [`ClientId`] was updated with a
	/// header for the given [`Height`]
	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ContextError> {
		todo!()
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ExecutionContext::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ContextError> {
		todo!()
	}

	/// Returns the maximum expected time per block
	fn max_expected_time_per_block(&self) -> Duration {
		todo!()
	}

	/// Validates the `signer` field of IBC messages, which represents the address
	/// of the user/relayer that signed the given message.
	fn validate_message_signer(&self, signer: &Signer) -> Result<(), ContextError> {
		todo!()
	}
}

impl<T: Config> ExecutionContext for Context<T> {
	/// Called upon successful client creation and update
	fn store_client_state(
		&mut self,
		client_state_path: ClientStatePath,
		client_state: Box<dyn ClientState>,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Called upon successful client creation and update
	fn store_consensus_state(
		&mut self,
		consensus_state_path: ClientConsensusStatePath,
		consensus_state: Box<dyn ConsensusState>,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Called upon client creation.
	/// Increases the counter which keeps track of how many clients have been created.
	/// Should never fail.
	fn increase_client_counter(&mut self) {
		todo!()
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

	/// Stores the given connection_end at path
	fn store_connection(
		&mut self,
		connection_path: &ConnectionPath,
		connection_end: ConnectionEnd,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given connection_id at a path associated with the client_id.
	fn store_connection_to_client(
		&mut self,
		client_connection_path: &ClientConnectionPath,
		conn_id: ConnectionId,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Called upon connection identifier creation (Init or Try process).
	/// Increases the counter which keeps track of how many connections have been created.
	/// Should never fail.
	fn increase_connection_counter(&mut self) {
		todo!()
	}

	/// Stores the given packet commitment at the given store path
	fn store_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
		commitment: PacketCommitment,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Deletes the packet commitment at the given store path
	fn delete_packet_commitment(
		&mut self,
		commitment_path: &CommitmentPath,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given packet receipt at the given store path
	fn store_packet_receipt(
		&mut self,
		receipt_path: &ReceiptPath,
		receipt: Receipt,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given packet acknowledgement at the given store path
	fn store_packet_acknowledgement(
		&mut self,
		ack_path: &AckPath,
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Deletes the packet acknowledgement at the given store path
	fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		channel_end_path: &ChannelEndPath,
		channel_end: ChannelEnd,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given `nextSequenceSend` number at the given store path
	fn store_next_sequence_send(
		&mut self,
		seq_send_path: &SeqSendPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given `nextSequenceRecv` number at the given store path
	fn store_next_sequence_recv(
		&mut self,
		seq_recv_path: &SeqRecvPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Stores the given `nextSequenceAck` number at the given store path
	fn store_next_sequence_ack(
		&mut self,
		seq_ack_path: &SeqAckPath,
		seq: Sequence,
	) -> Result<(), ContextError> {
		todo!()
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		todo!()
	}

	/// Emit the given IBC event
	fn emit_ibc_event(&mut self, event: IbcEvent) {
		todo!()
	}

	/// Log the given message.
	fn log_message(&mut self, message: String) {
		todo!()
	}
}
