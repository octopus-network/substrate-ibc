use crate::{Config, Context};
pub use alloc::{
	format,
	string::{String, ToString},
};
use core::time::Duration;
use frame_system::pallet_prelude::BlockNumberFor;
use ibc::{
	core::{
		ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{
				AcknowledgementCommitment as IbcAcknowledgementCommitment,
				PacketCommitment as IbcPacketCommitment,
			},
			context::{ChannelKeeper, ChannelReader},
			error::{ChannelError, PacketError},
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};
use pallet_ibc_utils::traits::{ChannelKeeperInterface, ChannelReaderInterface};
use sp_std::{boxed::Box, vec::Vec};

pub mod impls;

impl<T: Config> ChannelReader for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	fn channel_end(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<ChannelEnd, ChannelError> {
		<Context<T> as ChannelReaderInterface>::channel_end(port_id, channel_id)
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
		<Context<T> as ChannelReaderInterface>::connection_end(connection_id)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
		<Context<T> as ChannelReaderInterface>::connection_channels(conn_id)
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
		<Context<T> as ChannelReaderInterface>::client_state(client_id)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		<Context<T> as ChannelReaderInterface>::client_consensus_state(client_id, height)
	}

	fn get_next_sequence_send(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		<Context<T> as ChannelReaderInterface>::get_next_sequence_ack(port_id, channel_id)
	}

	fn get_next_sequence_recv(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		<Context<T> as ChannelReaderInterface>::get_next_sequence_recv(port_id, channel_id)
	}

	fn get_next_sequence_ack(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		<Context<T> as ChannelReaderInterface>::get_next_sequence_ack(port_id, channel_id)
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<IbcPacketCommitment, PacketError> {
		<Context<T> as ChannelReaderInterface>::get_packet_commitment(port_id, channel_id, seq)
	}

	fn get_packet_receipt(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<Receipt, PacketError> {
		<Context<T> as ChannelReaderInterface>::get_packet_receipt(port_id, channel_id, seq)
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<IbcAcknowledgementCommitment, PacketError> {
		<Context<T> as ChannelReaderInterface>::get_packet_acknowledgement(port_id, channel_id, seq)
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: &[u8]) -> Vec<u8> {
		<Context<T> as ChannelReaderInterface>::hash(value)
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Result<Height, ChannelError> {
		<Context<T> as ChannelReaderInterface>::host_height()
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		<Context<T> as ChannelReaderInterface>::host_consensus_state(height)
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ChannelError> {
		<Context<T> as ChannelReaderInterface>::pending_host_consensus_state()
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ChannelError> {
		<Context<T> as ChannelReaderInterface>::client_update_time(client_id, height)
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ChannelError> {
		<Context<T> as ChannelReaderInterface>::client_update_height(client_id, height)
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ChannelError> {
		<Context<T> as ChannelReaderInterface>::channel_counter()
	}

	fn max_expected_time_per_block(&self) -> Duration {
		<Context<T> as ChannelReaderInterface>::max_expected_time_per_block()
	}
}

impl<T: Config> ChannelKeeper for Context<T> {
	fn store_packet_commitment(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		commitment: IbcPacketCommitment,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::store_packet_commitment(
			port_id, channel_id, seq, commitment,
		)
	}

	fn delete_packet_commitment(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::delete_packet_commitment(port_id, channel_id, seq)
	}

	fn store_packet_receipt(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		receipt: Receipt,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::store_packet_receipt(
			port_id, channel_id, seq, receipt,
		)
	}

	fn store_packet_acknowledgement(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::store_packet_acknowledgement(
			port_id,
			channel_id,
			seq,
			ack_commitment,
		)
	}

	fn delete_packet_acknowledgement(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::delete_packet_acknowledgement(
			port_id, channel_id, seq,
		)
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), ChannelError> {
		<Context<T> as ChannelKeeperInterface>::store_connection_channels(
			conn_id, port_id, channel_id,
		)
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Result<(), ChannelError> {
		<Context<T> as ChannelKeeperInterface>::store_channel(port_id, channel_id, channel_end)
	}

	fn store_next_sequence_send(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::store_next_sequence_send(port_id, channel_id, seq)
	}

	fn store_next_sequence_recv(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::store_next_sequence_recv(port_id, channel_id, seq)
	}

	fn store_next_sequence_ack(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		<Context<T> as ChannelKeeperInterface>::store_next_sequence_ack(port_id, channel_id, seq)
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		<Context<T> as ChannelKeeperInterface>::increase_channel_counter()
	}
}
