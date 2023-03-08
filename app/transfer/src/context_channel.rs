use crate::{callback::IbcTransferModule, *};
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
			error::{ChannelError, PacketError},
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};
use sp_std::{boxed::Box, time::Duration, vec::Vec};

impl<T: Config> ChannelReader for IbcTransferModule<T> {
	fn channel_end(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<ChannelEnd, ChannelError> {
		ChannelReader::channel_end(&pallet_ibc::context::Context::<T>::new(), port_id, channel_id)
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
		ChannelReader::connection_end(&pallet_ibc::context::Context::<T>::new(), connection_id)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
		ChannelReader::connection_channels(&pallet_ibc::context::Context::<T>::new(), conn_id)
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
		ChannelReader::client_state(&pallet_ibc::context::Context::<T>::new(), client_id)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		ChannelReader::client_consensus_state(
			&pallet_ibc::context::Context::<T>::new(),
			client_id,
			height,
		)
	}

	fn get_next_sequence_send(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		ChannelReader::get_next_sequence_send(
			&pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
		)
	}

	fn get_next_sequence_recv(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		ChannelReader::get_next_sequence_recv(
			&pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
		)
	}

	fn get_next_sequence_ack(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		ChannelReader::get_next_sequence_ack(
			&pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
		)
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<PacketCommitment, PacketError> {
		ChannelReader::get_packet_commitment(
			&pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	fn get_packet_receipt(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<Receipt, PacketError> {
		ChannelReader::get_packet_receipt(
			&pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<IbcAcknowledgementCommitment, PacketError> {
		ChannelReader::get_packet_acknowledgement(
			&pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: &[u8]) -> Vec<u8> {
		ChannelReader::hash(&pallet_ibc::context::Context::<T>::new(), value)
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Result<Height, ChannelError> {
		ChannelReader::host_height(&pallet_ibc::context::Context::<T>::new())
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		ChannelReader::host_consensus_state(&pallet_ibc::context::Context::<T>::new(), height)
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ChannelError> {
		ChannelReader::pending_host_consensus_state(&pallet_ibc::context::Context::<T>::new())
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ChannelError> {
		ChannelReader::client_update_time(
			&pallet_ibc::context::Context::<T>::new(),
			client_id,
			height,
		)
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Height, ChannelError> {
		ChannelReader::client_update_height(
			&pallet_ibc::context::Context::<T>::new(),
			client_id,
			height,
		)
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ChannelError> {
		ChannelReader::channel_counter(&pallet_ibc::context::Context::<T>::new())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		ChannelReader::max_expected_time_per_block(&pallet_ibc::context::Context::<T>::new())
	}
}

impl<T: Config> ChannelKeeper for IbcTransferModule<T> {
	fn store_packet_commitment(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		commitment: PacketCommitment,
	) -> Result<(), PacketError> {
		ChannelKeeper::store_packet_commitment(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			sequence,
			commitment,
		)
	}

	fn delete_packet_commitment(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<(), PacketError> {
		ChannelKeeper::delete_packet_commitment(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	fn store_packet_receipt(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		receipt: Receipt,
	) -> Result<(), PacketError> {
		ChannelKeeper::store_packet_receipt(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
			receipt,
		)
	}

	fn store_packet_acknowledgement(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), PacketError> {
		ChannelKeeper::store_packet_acknowledgement(
			&mut pallet_ibc::context::Context::<T>::new(),
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
		ChannelKeeper::delete_packet_commitment(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), ChannelError> {
		ChannelKeeper::store_connection_channels(
			&mut pallet_ibc::context::Context::<T>::new(),
			conn_id,
			port_id,
			channel_id,
		)
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Result<(), ChannelError> {
		ChannelKeeper::store_channel(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			channel_end,
		)
	}

	fn store_next_sequence_send(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		ChannelKeeper::store_next_sequence_send(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	fn store_next_sequence_recv(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		ChannelKeeper::store_next_sequence_recv(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	fn store_next_sequence_ack(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		ChannelKeeper::store_next_sequence_ack(
			&mut pallet_ibc::context::Context::<T>::new(),
			port_id,
			channel_id,
			seq,
		)
	}

	fn increase_channel_counter(&mut self) {
		ChannelKeeper::increase_channel_counter(&mut pallet_ibc::context::Context::<T>::new())
	}
}
