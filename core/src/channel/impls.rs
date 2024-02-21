use crate::{
	context::Context, Acknowledgements, ChannelCounter, Channels, ChannelsConnection,
	ClientProcessedHeights, ClientProcessedTimes, Config, NextSequenceAck, NextSequenceRecv,
	NextSequenceSend, PacketCommitment as PacketCommitStore, PacketReceipt,
};
use alloc::{format, string::ToString, vec};
use core::time::Duration;
use frame_system::pallet_prelude::BlockNumberFor;
use ibc::{
	core::{
		ics02_client::{
			client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd, context::ConnectionReader, error::ConnectionError,
		},
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
			error::{ChannelError, PacketError},
			packet::{Receipt, Sequence},
		},
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AcksPath, ChannelEndsPath, CommitmentsPath, ReceiptsPath, SeqAcksPath,
				SeqRecvsPath, SeqSendsPath,
			},
		},
	},
	timestamp::Timestamp,
	Height,
};
use pallet_ibc_utils::traits::{ChannelKeeperInterface, ChannelReaderInterface};
use sp_core::Get;
use sp_std::{boxed::Box, vec::Vec};

/// A context supplying all the necessary read-only dependencies for processing any `ChannelMsg`.
impl<T: Config> ChannelReaderInterface for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(port_id: &PortId, channel_id: &ChannelId) -> Result<ChannelEnd, ChannelError> {
		<Channels<T>>::get(ChannelEndsPath(port_id.clone(), channel_id.clone())).ok_or(
			ChannelError::ChannelNotFound {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			},
		)
	}

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(connection_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
		let context = Context::<T>::new();
		ConnectionReader::connection_end(&context, connection_id).map_err(ChannelError::Connection)
	}

	fn connection_channels(cid: &ConnectionId) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
		<ChannelsConnection<T>>::get(&cid)
			.ok_or(ChannelError::ConnectionNotOpen { connection_id: cid.clone() })
	}

	/// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
	/// proof verification.
	fn client_state(client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
		let context = Context::<T>::new();
		ClientReader::client_state(&context, client_id)
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	fn client_consensus_state(
		client_id: &ClientId,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		let context = Context::<T>::new();
		ClientReader::consensus_state(&context, client_id, height)
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	fn get_next_sequence_send(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		<NextSequenceSend<T>>::get(SeqSendsPath(port_id.clone(), channel_id.clone())).ok_or(
			PacketError::MissingNextSendSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			},
		)
	}

	fn get_next_sequence_recv(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		<NextSequenceRecv<T>>::get(SeqRecvsPath(port_id.clone(), channel_id.clone())).ok_or(
			PacketError::MissingNextRecvSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			},
		)
	}

	fn get_next_sequence_ack(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		<NextSequenceAck<T>>::get(SeqAcksPath(port_id.clone(), channel_id.clone())).ok_or(
			PacketError::MissingNextAckSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			},
		)
	}

	fn get_packet_commitment(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: &Sequence,
	) -> Result<PacketCommitment, PacketError> {
		<PacketCommitStore<T>>::get(CommitmentsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: sequence.clone(),
		})
		.ok_or(PacketError::PacketCommitmentNotFound { sequence: *sequence })
	}

	fn get_packet_receipt(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: &Sequence,
	) -> Result<Receipt, PacketError> {
		<PacketReceipt<T>>::get(ReceiptsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: sequence.clone(),
		})
		.ok_or(PacketError::PacketReceiptNotFound { sequence: *sequence })
	}

	fn get_packet_acknowledgement(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: &Sequence,
	) -> Result<AcknowledgementCommitment, PacketError> {
		<Acknowledgements<T>>::get(AcksPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: sequence.clone(),
		})
		.ok_or(PacketError::PacketAcknowledgementNotFound { sequence: *sequence })
	}

	/// A hashing function for packet commitments
	fn hash(value: &[u8]) -> Vec<u8> {
		sp_io::hashing::sha2_256(value).to_vec()
	}

	/// Returns the current height of the local chain.
	fn host_height() -> Result<Height, ChannelError> {
		let block_height = <frame_system::Pallet<T>>::block_number();
		Height::new(T::ChainVersion::get(), u64::from(block_height))
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	/// Returns the `ConsensusState` of the host (local) chain at a specific height.
	fn host_consensus_state(height: &Height) -> Result<Box<dyn ConsensusState>, ChannelError> {
		let context = Context::<T>::new();
		ConnectionReader::host_consensus_state(&context, height).map_err(ChannelError::Connection)
	}

	/// Returns the pending `ConsensusState` of the host (local) chain.
	fn pending_host_consensus_state() -> Result<Box<dyn ConsensusState>, ChannelError> {
		let context = Context::<T>::new();
		ClientReader::pending_host_consensus_state(&context)
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	/// Returns the time when the client state for the given [`ClientId`] was updated with a header
	/// for the given [`Height`]
	fn client_update_time(
		client_id: &ClientId,
		height: &Height,
	) -> Result<Timestamp, ChannelError> {
		let time = <ClientProcessedTimes<T>>::get(client_id, height).ok_or(
			ChannelError::ProcessedTimeNotFound { client_id: client_id.clone(), height: *height },
		)?;

		Timestamp::from_nanoseconds(time)
			.map_err(|e| ChannelError::Other { description: e.to_string() })
	}

	/// Returns the height when the client state for the given [`ClientId`] was updated with a
	/// header for the given [`Height`]
	fn client_update_height(client_id: &ClientId, height: &Height) -> Result<Height, ChannelError> {
		<ClientProcessedHeights<T>>::get(client_id, height).ok_or(
			ChannelError::ProcessedHeightNotFound { client_id: client_id.clone(), height: *height },
		)
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter() -> Result<u64, ChannelError> {
		Ok(<ChannelCounter<T>>::get())
	}

	/// Returns the maximum expected time per block
	fn max_expected_time_per_block() -> Duration {
		Duration::from_secs(T::ExpectedBlockTime::get())
	}
}

/// A context supplying all the necessary write-only dependencies (i.e., storage writing facility)
/// for processing any `ChannelMsg`.
impl<T: Config> ChannelKeeperInterface for Context<T> {
	fn store_packet_commitment(
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		commitment: PacketCommitment,
	) -> Result<(), PacketError> {
		<PacketCommitStore<T>>::insert(
			CommitmentsPath { port_id, channel_id, sequence },
			commitment,
		);

		Ok(())
	}

	fn delete_packet_commitment(
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: &Sequence,
	) -> Result<(), PacketError> {
		<PacketCommitStore<T>>::remove(CommitmentsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: seq.clone(),
		});

		Ok(())
	}

	fn store_packet_receipt(
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		receipt: Receipt,
	) -> Result<(), PacketError> {
		<PacketReceipt<T>>::insert(ReceiptsPath { port_id, channel_id, sequence }, receipt);

		Ok(())
	}

	fn store_packet_acknowledgement(
		port_id: PortId,
		channel_id: ChannelId,
		sequence: Sequence,
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), PacketError> {
		<Acknowledgements<T>>::insert(AcksPath { port_id, channel_id, sequence }, ack_commitment);

		Ok(())
	}

	fn delete_packet_acknowledgement(
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: &Sequence,
	) -> Result<(), PacketError> {
		<Acknowledgements<T>>::remove(AcksPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: sequence.clone(),
		});

		Ok(())
	}

	fn store_connection_channels(
		conn_id: ConnectionId,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), ChannelError> {
		if <ChannelsConnection<T>>::contains_key(&conn_id) {
			let _ =
				<ChannelsConnection<T>>::try_mutate(&conn_id, |val| -> Result<(), ChannelError> {
					if let Some(value) = val {
						value.push((port_id, channel_id));
					}
					Ok(())
				})
				.map_err(|e| ChannelError::Other {
					description: format!("store connection channels failed: {:?}", e),
				});
		} else {
			<ChannelsConnection<T>>::insert(conn_id, vec![(port_id, channel_id)]);
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		port_id: PortId,
		channel_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Result<(), ChannelError> {
		<Channels<T>>::insert(ChannelEndsPath(port_id, channel_id), channel_end);

		Ok(())
	}

	fn store_next_sequence_send(
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		<NextSequenceSend<T>>::insert(SeqSendsPath(port_id, channel_id), seq);

		Ok(())
	}

	fn store_next_sequence_recv(
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		<NextSequenceRecv<T>>::insert(SeqRecvsPath(port_id, channel_id), seq);

		Ok(())
	}

	fn store_next_sequence_ack(
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		<NextSequenceAck<T>>::insert(SeqAcksPath(port_id, channel_id), seq);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter() {
		let _ = ChannelCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}
}
