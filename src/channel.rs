use crate::{
	context::Context,
	prelude::{format, String, ToString},
	Acknowledgements, ChannelCounter, Channels, ChannelsConnection, ClientProcessedHeights,
	ClientProcessedTimes, Config, IbcChannelId, NextSequenceAck, NextSequenceRecv,
	NextSequenceSend, PacketCommitment, PacketReceipt, Pallet, Store, REVISION_NUMBER,
};
use sp_std::{boxed::Box, vec, vec::Vec};

use core::{str::FromStr, time::Duration};
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
			commitment::{
				AcknowledgementCommitment as IbcAcknowledgementCommitment,
				PacketCommitment as IbcPacketCommitment,
			},
			context::{ChannelKeeper, ChannelReader},
			error::{ChannelError, PacketError},
			packet::{Receipt, Sequence},
		},
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AcksPath, ChannelEndsPath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
				SeqAcksPath, SeqRecvsPath, SeqSendsPath,
			},
			Path,
		},
	},
	timestamp::Timestamp,
	Height,
};
use ibc_proto::protobuf::Protobuf;

impl<T: Config> ChannelReader for Context<T> {
	fn channel_end(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<ChannelEnd, ChannelError> {
		let channel_end_path = ChannelEndsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <Channels<T>>::contains_key(&channel_end_path) {
			let data = <Channels<T>>::get(&channel_end_path);

			let channel_end =
				ChannelEnd::decode_vec(&data).map_err(|_| ChannelError::ChannelNotFound {
					port_id: port_id.clone(),
					channel_id: channel_id.clone(),
				})?;

			Ok(channel_end)
		} else {
			Err(ChannelError::ChannelNotFound {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			})
		}
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
		let context = Context::<T>::new();
		ConnectionReader::connection_end(&context, connection_id).map_err(ChannelError::Connection)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
		let connections_path = ConnectionsPath(conn_id.clone()).to_string().as_bytes().to_vec();

		if <ChannelsConnection<T>>::contains_key(&connections_path) {
			let channel_ends_paths = <ChannelsConnection<T>>::get(&connections_path);

			let mut result = vec![];

			for item in channel_ends_paths.into_iter() {
				let raw_path = String::from_utf8(item).map_err(|e| ChannelError::Other {
					description: format!("Decode ChannelEnds Paths String format failed: {:?}", e),
				})?;
				let path = Path::from_str(&raw_path).map_err(|e| ChannelError::Other {
					description: format!("Decode ChannelEnds Path format Failed: {:?}", e),
				})?;
				match path {
					Path::ChannelEnds(channel_ends_path) => {
						let ChannelEndsPath(port_id, channel_id) = channel_ends_path;
						result.push((port_id, channel_id));
					},
					_ => unimplemented!(),
				}
			}
			Ok(result)
		} else {
			Err(ChannelError::ConnectionNotOpen { connection_id: conn_id.clone() })
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
		let context = Context::<T>::new();
		ClientReader::client_state(&context, client_id)
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		let context = Context::<T>::new();
		ClientReader::consensus_state(&context, client_id, height)
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	fn get_next_sequence_send(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		let seq_sends_path = SeqSendsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <NextSequenceSend<T>>::contains_key(&seq_sends_path) {
			let sequence = <NextSequenceSend<T>>::get(&seq_sends_path);
			Ok(Sequence::from(sequence))
		} else {
			Err(PacketError::MissingNextSendSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			})
		}
	}

	fn get_next_sequence_recv(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		let seq_recvs_path = SeqRecvsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <NextSequenceRecv<T>>::contains_key(&seq_recvs_path) {
			let sequence = <NextSequenceRecv<T>>::get(&seq_recvs_path);
			Ok(Sequence::from(sequence))
		} else {
			Err(PacketError::MissingNextRecvSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			})
		}
	}

	fn get_next_sequence_ack(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<Sequence, PacketError> {
		let seq_acks_path =
			SeqAcksPath(port_id.clone(), channel_id.clone()).to_string().as_bytes().to_vec();

		if <NextSequenceAck<T>>::contains_key(&seq_acks_path) {
			let sequence = <NextSequenceAck<T>>::get(&seq_acks_path);

			Ok(Sequence::from(sequence))
		} else {
			Err(PacketError::MissingNextAckSeq {
				port_id: port_id.clone(),
				channel_id: channel_id.clone(),
			})
		}
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<IbcPacketCommitment, PacketError> {
		let packet_commitments_path = CommitmentsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: seq,
		}
		.to_string()
		.as_bytes()
		.to_vec();

		if <PacketCommitment<T>>::contains_key(&packet_commitments_path) {
			let data = <PacketCommitment<T>>::get(&packet_commitments_path);
			let packet_commitment = IbcPacketCommitment::from(data);
			Ok(packet_commitment)
		} else {
			Err(PacketError::PacketCommitmentNotFound { sequence: seq })
		}
	}

	fn get_packet_receipt(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<Receipt, PacketError> {
		let packet_receipt_path = ReceiptsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: seq,
		}
		.to_string()
		.as_bytes()
		.to_vec();

		if <PacketReceipt<T>>::contains_key(&packet_receipt_path) {
			let data = <PacketReceipt<T>>::get(&packet_receipt_path);
			let data = String::from_utf8(data).map_err(|e| PacketError::AppModule {
				description: format!("Decode packet receipt failed: {:?}", e),
			})?;
			let data = match data.as_ref() {
				"Ok" => Receipt::Ok,
				e =>
					return Err(PacketError::AppModule {
						description: format!("Unknown Receipts {:?}", e),
					}),
			};
			Ok(data)
		} else {
			Err(PacketError::PacketReceiptNotFound { sequence: seq })
		}
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<IbcAcknowledgementCommitment, PacketError> {
		let acks_path =
			AcksPath { port_id: port_id.clone(), channel_id: channel_id.clone(), sequence: seq }
				.to_string()
				.as_bytes()
				.to_vec();

		if <Acknowledgements<T>>::contains_key(&acks_path) {
			let data = <Acknowledgements<T>>::get(&acks_path);
			let acknowledgement = IbcAcknowledgementCommitment::from(data);

			Ok(acknowledgement)
		} else {
			Err(PacketError::PacketAcknowledgementNotFound { sequence: seq })
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: Vec<u8>) -> Vec<u8> {
		sp_io::hashing::sha2_256(&value).to_vec()
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Result<Height, ChannelError> {
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		Ok(Height::new(REVISION_NUMBER, current_height).unwrap())
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(
		&self,
		height: Height,
	) -> Result<Box<dyn ConsensusState>, ChannelError> {
		let context = Context::<T>::new();
		ConnectionReader::host_consensus_state(&context, height).map_err(ChannelError::Connection)
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ChannelError> {
		let context = Context::<T>::new();
		ClientReader::pending_host_consensus_state(&context)
			.map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, ChannelError> {
		if <ClientProcessedTimes<T>>::contains_key(
			client_id.as_bytes(),
			height.encode_vec().map_err(|e| ChannelError::Other {
				description: format!("Encode height failed: {:?}", e),
			})?,
		) {
			let time = <ClientProcessedTimes<T>>::get(
				client_id.as_bytes(),
				height.encode_vec().map_err(|e| ChannelError::Other {
					description: format!("Encode height failed: {:?}", e),
				})?,
			);
			let timestamp = String::from_utf8(time).map_err(|e| ChannelError::Other {
				description: format!("Decode timestamp format String failed: {:?}", e),
			})?;
			let time: Timestamp = serde_json::from_str(&timestamp).map_err(|e| {
				ChannelError::Other { description: format!("Decode timestamp  failed: {:?}", e) }
			})?;
			Ok(time)
		} else {
			Err(ChannelError::ProcessedTimeNotFound { client_id: client_id.clone(), height })
		}
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, ChannelError> {
		if <ClientProcessedHeights<T>>::contains_key(
			client_id.as_bytes(),
			height.encode_vec().map_err(|e| ChannelError::Other {
				description: format!("Encode height failed: {:?}", e),
			})?,
		) {
			let host_height = <ClientProcessedHeights<T>>::get(
				client_id.as_bytes(),
				height.encode_vec().map_err(|e| ChannelError::Other {
					description: format!("Encode height failed: {:?}", e),
				})?,
			);
			let host_height = Height::decode(&mut &host_height[..]).map_err(|e| {
				ChannelError::Other { description: format!("Decode Host height failed: {:?}", e) }
			})?;
			Ok(host_height)
		} else {
			Err(ChannelError::ProcessedHeightNotFound { client_id: client_id.clone(), height })
		}
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ChannelError> {
		Ok(<Pallet<T> as Store>::ChannelCounter::get())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		Duration::from_secs(6)
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
		let packet_commitments_path = CommitmentsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: seq,
		}
		.to_string()
		.as_bytes()
		.to_vec();

		<PacketCommitment<T>>::insert(packet_commitments_path, commitment.into_vec());

		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		let packet_commitments_path = CommitmentsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: seq,
		}
		.to_string()
		.as_bytes()
		.to_vec();

		<PacketCommitment<T>>::remove(&packet_commitments_path);

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		receipt: Receipt,
	) -> Result<(), PacketError> {
		let packet_receipt_path = ReceiptsPath {
			port_id: port_id.clone(),
			channel_id: channel_id.clone(),
			sequence: seq,
		}
		.to_string()
		.as_bytes()
		.to_vec();

		let receipt = match receipt {
			Receipt::Ok => "Ok".as_bytes().to_vec(),
		};

		<PacketReceipt<T>>::insert(packet_receipt_path, receipt);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), PacketError> {
		let acks_path =
			AcksPath { port_id: port_id.clone(), channel_id: channel_id.clone(), sequence: seq }
				.to_string()
				.as_bytes()
				.to_vec();

		<Acknowledgements<T>>::insert(&acks_path, ack_commitment.into_vec());

		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		let acks_path =
			AcksPath { port_id: port_id.clone(), channel_id: channel_id.clone(), sequence: seq }
				.to_string()
				.as_bytes()
				.to_vec();

		<Acknowledgements<T>>::remove(&acks_path);

		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), ChannelError> {
		let connections_path = ConnectionsPath(conn_id).to_string().as_bytes().to_vec();
		let channel_ends_path = ChannelEndsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <ChannelsConnection<T>>::contains_key(&connections_path) {
			let _ = <ChannelsConnection<T>>::try_mutate(
				&connections_path,
				|val| -> Result<(), ChannelError> {
					val.push(channel_ends_path.clone());
					Ok(())
				},
			)
			.map_err(|e| ChannelError::Other {
				description: format!("store connection channels failed: {:?}", e),
			});
		} else {
			<ChannelsConnection<T>>::insert(connections_path, vec![channel_ends_path]);
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Result<(), ChannelError> {
		let channel_end_path = ChannelEndsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();
		let channel_end = channel_end.encode_vec().map_err(|e| ChannelError::Other {
			description: format!("encode channel end failed: {:?}", e),
		})?;
		<Channels<T>>::insert(channel_end_path, channel_end);

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		let seq_sends_path = SeqSendsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		let sequence = u64::from(seq);

		<NextSequenceSend<T>>::insert(seq_sends_path, sequence);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		let seq_recvs_path = SeqRecvsPath(port_id.clone(), channel_id.clone())
			.to_string()
			.as_bytes()
			.to_vec();
		let sequence = u64::from(seq);

		<NextSequenceRecv<T>>::insert(seq_recvs_path, sequence);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_id: PortId,
		channel_id: ChannelId,
		seq: Sequence,
	) -> Result<(), PacketError> {
		let seq_acks_path =
			SeqAcksPath(port_id.clone(), channel_id.clone()).to_string().as_bytes().to_vec();
		let sequence = u64::from(seq);

		<NextSequenceAck<T>>::insert(seq_acks_path, sequence);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		let _ = <ChannelCounter<T>>::try_mutate(|val| -> Result<(), ChannelError> {
			let new = val.checked_add(1).ok_or(ChannelError::Other {
				description: format!("add channel counter overflow"),
			})?;
			*val = new;
			Ok(())
		});
	}
}
