use core::time::Duration;
use alloc::vec::Vec;
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
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    timestamp::Timestamp,
    Height,
};

pub trait IbcSupportChannelReader {
    fn channel_end(port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, Ics04Error>;

    fn connection_end(connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error>;

    /// Returns the `ChannelsConnection` for the given identifier `conn_id`.
    fn connection_channels(
        conn_id: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, Ics04Error>;

    fn client_state(client_id: &ClientId) -> Result<AnyClientState, Ics04Error>;

    fn client_consensus_state(
        client_id: &ClientId,
        height: Height,
    ) -> Result<AnyConsensusState, Ics04Error>;

    fn get_next_sequence_send(
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, Ics04Error>;

    fn get_next_sequence_recv(
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, Ics04Error>;

    fn get_next_sequence_ack(
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, Ics04Error>;

    /// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
    fn get_packet_commitment(
        key: &(PortId, ChannelId, Sequence),
    ) -> Result<IbcPacketCommitment, Ics04Error>;

    fn get_packet_receipt(
        key: &(PortId, ChannelId, Sequence),
    ) -> Result<Receipt, Ics04Error>;

    /// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
    fn get_packet_acknowledgement(
        key: &(PortId, ChannelId, Sequence),
    ) -> Result<IbcAcknowledgementCommitment, Ics04Error>;

    /// A hashing function for packet commitments
    fn hash(value: Vec<u8>) -> Vec<u8>;

    /// Returns the current height of the local chain.
    fn host_height() -> Height;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp() -> Timestamp;

    /// Returns the `AnyConsensusState` for the given identifier `height`.
    fn host_consensus_state( height: Height) -> Result<AnyConsensusState, Ics04Error> ;
    fn pending_host_consensus_state() -> Result<AnyConsensusState, Ics04Error>;

    /// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
    fn client_update_time(
        client_id: &ClientId,
        height: Height,
    ) -> Result<Timestamp, Ics04Error>;

    fn client_update_height(
        client_id: &ClientId,
        height: Height,
    ) -> Result<Height, Ics04Error>;

    /// Returns a counter on the number of channel ids have been created thus far.
    /// The value of this counter should increase only via method
    /// `ChannelKeeper::increase_channel_counter`.
    fn channel_counter() -> Result<u64, Ics04Error>;

    fn max_expected_time_per_block() -> Duration;
}

pub trait IbcSupportChannelKeeper {
    fn store_packet_commitment(
        key: (PortId, ChannelId, Sequence),
        commitment: IbcPacketCommitment,
    ) -> Result<(), Ics04Error>;

    fn delete_packet_commitment(
        key: (PortId, ChannelId, Sequence),
    ) -> Result<(), Ics04Error>;

    fn store_packet_receipt(
        key: (PortId, ChannelId, Sequence),
        receipt: Receipt,
    ) -> Result<(), Ics04Error>;

    fn store_packet_acknowledgement(
        key: (PortId, ChannelId, Sequence),
        ack_commitment: IbcAcknowledgementCommitment,
    ) -> Result<(), Ics04Error>;

    fn delete_packet_acknowledgement(
        key: (PortId, ChannelId, Sequence),
    ) -> Result<(), Ics04Error>;

    fn store_connection_channels(
        conn_id: ConnectionId,
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<(), Ics04Error>;

    /// Stores the given channel_end at a path associated with the port_id and channel_id.
    fn store_channel(
        port_channel_id: (PortId, ChannelId),
        channel_end: &ChannelEnd,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_next_sequence_send(
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), Ics04Error> ;

    fn store_next_sequence_recv(
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), Ics04Error>;

    fn store_next_sequence_ack(
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), Ics04Error>;

    fn increase_channel_counter();
}
