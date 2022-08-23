use super::transfer_handle_callback::TransferModule;
use crate::*;
use core::{str::FromStr, time::Duration};
use log::{error, info, trace, warn};

use crate::context::Context;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState, client_state::AnyClientState,
			context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd, context::ConnectionReader, error::Error as ICS03Error,
		},
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{
				AcknowledgementCommitment as IbcAcknowledgementCommitment,
				PacketCommitment as IbcPacketCommitment,
			},
			context::{ChannelKeeper, ChannelReader},
			error::Error as Ics04Error,
			packet::{Receipt, Sequence},
		},
		ics05_port::{context::PortReader, error::Error as Ics05Error},
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AcksPath, ChannelEndsPath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
				SeqAcksPath, SeqRecvsPath, SeqSendsPath,
			},
			Path,
		},
		ics26_routing::context::ModuleId,
	},
	timestamp::Timestamp,
	Height,
};

impl<T: Config> ChannelReader for TransferModule<T> {
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, Ics04Error> {
		let connect = Context::<T>::new();
		connect.channel_end(port_channel_id)
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error> {
		let connect = Context::<T>::new();
		ChannelReader::connection_end(&connect, connection_id)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, Ics04Error> {
		let connect = Context::<T>::new();
		connect.connection_channels(conn_id)
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics04Error> {
		let connect = Context::<T>::new();

		ChannelReader::client_state(&connect, client_id)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics04Error> {
		let connect = Context::<T>::new();
		ChannelReader::client_consensus_state(&connect, client_id, height)
	}

	fn get_next_sequence_send(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		let connect = Context::<T>::new();
		connect.get_next_sequence_send(port_channel_id)
	}

	fn get_next_sequence_recv(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		let connect = Context::<T>::new();
		connect.get_next_sequence_recv(port_channel_id)
	}

	fn get_next_sequence_ack(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		let connect = Context::<T>::new();
		connect.get_next_sequence_ack(port_channel_id)
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<IbcPacketCommitment, Ics04Error> {
		let connect = Context::<T>::new();
		connect.get_packet_commitment(key)
	}

	fn get_packet_receipt(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<Receipt, Ics04Error> {
		let connect = Context::<T>::new();
		connect.get_packet_receipt(key)
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<IbcAcknowledgementCommitment, Ics04Error> {
		let connect = Context::<T>::new();
		connect.get_packet_acknowledgement(key)
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: Vec<u8>) -> Vec<u8> {
		let connect = Context::<T>::new();
		connect.hash(value)
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		let connect = Context::<T>::new();
		ChannelReader::host_height(&connect)
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		let connect = Context::<T>::new();
		ChannelReader::host_timestamp(&connect)
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(&self, height: Height) -> Result<AnyConsensusState, Ics04Error> {
		let connect = Context::<T>::new();
		ConnectionReader::host_consensus_state(&connect, height)
			.map_err(Ics04Error::ics03_connection)
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, Ics04Error> {
		let connect = Context::<T>::new();
		ClientReader::pending_host_consensus_state(&connect)
			.map_err(|e| Ics04Error::ics03_connection(ICS03Error::ics02_client(e)))
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, Ics04Error> {
		let connect = Context::<T>::new();
		connect.client_update_time(client_id, height)
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, Ics04Error> {
		let connect = Context::<T>::new();
		connect.client_update_height(client_id, height)
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, Ics04Error> {
		let connect = Context::<T>::new();
		connect.channel_counter()
	}

	fn max_expected_time_per_block(&self) -> Duration {
		let connect = Context::<T>::new();
		connect.max_expected_time_per_block()
	}
}

impl<T: Config> ChannelKeeper for TransferModule<T> {
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		commitment: IbcPacketCommitment,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_packet_commitment(key, commitment)
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.delete_packet_commitment(key)
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_packet_receipt(key, receipt)
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();

		connect.store_packet_acknowledgement(key, ack_commitment)
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.delete_packet_acknowledgement(key)
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_connection_channels(conn_id, port_channel_id)
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_channel(port_channel_id, channel_end)
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_next_sequence_send(port_channel_id, seq)
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_next_sequence_recv(port_channel_id, seq)
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		let mut connect = Context::<T>::new();
		connect.store_next_sequence_ack(port_channel_id, seq)
	}

	fn increase_channel_counter(&mut self) {
		let mut connect = Context::<T>::new();
		connect.increase_channel_counter()
	}
}
