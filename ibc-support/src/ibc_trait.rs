use codec::{Decode, Encode};
use ibc::{
	core::{
		ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
			error::Error as Ics04Error,
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};
use sp_std::{boxed::Box, time::Duration, vec::Vec};

use ibc::core::ics26_routing::context::Module;

pub trait TransferModule: Module + Encode + Decode {}

pub trait IbcSupportChannelReader {
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(port_id: &PortId, channel_id: &ChannelId) -> Result<ChannelEnd, Ics04Error>;

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error>;

	fn connection_channels(cid: &ConnectionId) -> Result<Vec<(PortId, ChannelId)>, Ics04Error>;

	/// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
	/// proof verification.
	fn client_state(client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics04Error>;

	fn client_consensus_state(
		client_id: &ClientId,
		height: Height,
	) -> Result<Box<dyn ConsensusState>, Ics04Error>;

	fn get_next_sequence_send(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, Ics04Error>;

	fn get_next_sequence_recv(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, Ics04Error>;

	fn get_next_sequence_ack(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, Ics04Error>;

	fn get_packet_commitment(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<PacketCommitment, Ics04Error>;

	fn get_packet_receipt(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<Receipt, Ics04Error>;

	fn get_packet_acknowledgement(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<AcknowledgementCommitment, Ics04Error>;

	/// A hashing function for packet commitments
	fn hash(value: Vec<u8>) -> Vec<u8>;

	/// Returns the current height of the local chain.
	fn host_height() -> Height;

	/// Returns the `ConsensusState` of the host (local) chain at a specific height.
	fn host_consensus_state(height: Height) -> Result<Box<dyn ConsensusState>, Ics04Error>;

	/// Returns the pending `ConsensusState` of the host (local) chain.
	fn pending_host_consensus_state() -> Result<Box<dyn ConsensusState>, Ics04Error>;

	/// Returns the time when the client state for the given [`ClientId`] was updated with a header
	/// for the given [`Height`]
	fn client_update_time(client_id: &ClientId, height: Height) -> Result<Timestamp, Ics04Error>;

	/// Returns the height when the client state for the given [`ClientId`] was updated with a
	/// header for the given [`Height`]
	fn client_update_height(client_id: &ClientId, height: Height) -> Result<Height, Ics04Error>;

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter() -> Result<u64, Ics04Error>;

	/// Returns the maximum expected time per block
	fn max_expected_time_per_block() -> Duration;
}

/// A context supplying all the necessary write-only dependencies (i.e., storage writing facility)
/// for processing any `ChannelMsg`.
pub trait IbcSupportChannelKeeper {
	fn store_packet_commitment(
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		commitment: PacketCommitment,
	) -> Result<(), Ics04Error>;

	fn delete_packet_commitment(
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error>;

	fn store_packet_receipt(
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		receipt: Receipt,
	) -> Result<(), Ics04Error>;

	fn store_packet_acknowledgement(
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), Ics04Error>;

	fn delete_packet_acknowledgement(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<(), Ics04Error>;

	fn store_connection_channels(
		conn_id: ConnectionId,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics04Error>;

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		port_id: PortId,
		channel_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Result<(), Ics04Error>;

	fn store_next_sequence_send(
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error>;

	fn store_next_sequence_recv(
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error>;

	fn store_next_sequence_ack(
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), Ics04Error>;

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter();
}
