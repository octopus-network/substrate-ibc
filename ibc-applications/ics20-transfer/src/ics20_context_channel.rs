use crate::{ics20_callback::IbcTransferModule, *};
use ibc::{
	core::{
		ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{
				AcknowledgementCommitment as IbcAcknowledgementCommitment, PacketCommitment,
			},
			context::{ChannelKeeper, ChannelReader},
			error::Error as Ics04Error,
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};
use ibc_support::ibc_trait::{IbcSupportChannelKeeper, IbcSupportChannelReader};
use sp_std::{boxed::Box, time::Duration, vec::Vec};

impl<T: Config> ChannelReader for IbcTransferModule<T> {
	fn channel_end(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<ChannelEnd, Ics04Error> {
		T::IbcContext::channel_end(port_id, channel_id)
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error> {
		T::IbcContext::connection_end(connection_id)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, Ics04Error> {
		T::IbcContext::connection_channels(conn_id)
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics04Error> {
		T::IbcContext::client_state(client_id)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Box<dyn ConsensusState>, Ics04Error> {
		T::IbcContext::client_consensus_state(client_id, height)
	}

	fn get_next_sequence_send(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, Ics04Error> {
		T::IbcContext::get_next_sequence_send(port_id, channel_id)
	}

	fn get_next_sequence_recv(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, Ics04Error> {
		T::IbcContext::get_next_sequence_recv(port_id, channel_id)
	}

	fn get_next_sequence_ack(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, Ics04Error> {
		T::IbcContext::get_next_sequence_ack(port_id, channel_id)
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<PacketCommitment, Ics04Error> {
		T::IbcContext::get_packet_commitment(port_id, channel_id, seq)
	}

	fn get_packet_receipt(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<Receipt, Ics04Error> {
		T::IbcContext::get_packet_receipt(port_id, channel_id, seq)
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<IbcAcknowledgementCommitment, Ics04Error> {
		T::IbcContext::get_packet_acknowledgement(port_id, channel_id, seq)
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: Vec<u8>) -> Vec<u8> {
		T::IbcContext::hash(value)
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		T::IbcContext::host_height()
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics04Error> {
		T::IbcContext::host_consensus_state(height)
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics04Error> {
		T::IbcContext::pending_host_consensus_state()
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, Ics04Error> {
		T::IbcContext::client_update_time(client_id, height)
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, Ics04Error> {
		T::IbcContext::client_update_height(client_id, height)
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, Ics04Error> {
		T::IbcContext::channel_counter()
	}

	fn max_expected_time_per_block(&self) -> Duration {
		T::IbcContext::max_expected_time_per_block()
	}
}

impl<T: Config> ChannelKeeper for IbcTransferModule<T> {
	fn store_packet_commitment(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		commitment: PacketCommitment,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_packet_commitment(port_id, channel_id, sequence, commitment)
	}

	fn delete_packet_commitment(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		T::IbcContext::delete_packet_commitment(port_id, channel_id, seq)
	}

	fn store_packet_receipt(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		receipt: Receipt,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_packet_receipt(port_id, channel_id, seq, receipt)
	}

	fn store_packet_acknowledgement(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_packet_acknowledgement(port_id, channel_id, seq, ack_commitment)
	}

	fn delete_packet_acknowledgement(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		T::IbcContext::delete_packet_commitment(port_id, channel_id, seq)
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_connection_channels(conn_id, port_id, channel_id)
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_channel(port_id, channel_id, channel_end)
	}

	fn store_next_sequence_send(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_next_sequence_send(port_id, channel_id, seq)
	}

	fn store_next_sequence_recv(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_next_sequence_recv(port_id, channel_id, seq)
	}

	fn store_next_sequence_ack(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		T::IbcContext::store_next_sequence_ack(port_id, channel_id, seq)
	}

	fn increase_channel_counter(&mut self) {
		T::IbcContext::increase_channel_counter()
	}
}
