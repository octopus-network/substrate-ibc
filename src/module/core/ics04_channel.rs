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
		ics23_commitment::commitment::CommitmentRoot,
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

impl<T: Config> ChannelReader for Context<T> {
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [channel_end] port_channel_id:{:?}",port_channel_id);

		let channel_end_path =
			ChannelEndsPath(port_channel_id.0.clone(), port_channel_id.1.clone())
				.to_string()
				.as_bytes()
				.to_vec();

		if <Channels<T>>::contains_key(&channel_end_path) {
			let data = <Channels<T>>::get(&channel_end_path);

			let channel_end = ChannelEnd::decode_vec(&*data).map_err(|_| {
				Ics04Error::channel_not_found(port_channel_id.clone().0, port_channel_id.clone().1)
			})?;

			trace!(target:"runtime::pallet-ibc","in channel : [channel_end] >> channel_end = {:?}", channel_end);
			Ok(channel_end)
		} else {
			Err(Ics04Error::channel_not_found(port_channel_id.0.clone(), port_channel_id.1.clone()))
		}
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [connection_end] connection_id:{:?}",connection_id);

		ConnectionReader::connection_end(self, connection_id).map_err(Ics04Error::ics03_connection)
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [connection_channels]");
		// store key
		let connections_path = ConnectionsPath(conn_id.clone()).to_string().as_bytes().to_vec();

		if <ChannelsConnection<T>>::contains_key(&connections_path) {
			let channel_ends_paths = <ChannelsConnection<T>>::get(&connections_path);

			let mut result = vec![];

			for item in channel_ends_paths.into_iter() {
				let raw_path = String::from_utf8(item).map_err(Ics04Error::invalid_from_utf8)?;
				// decode key
				let path = Path::from_str(&raw_path).map_err(Ics04Error::invalid_path_parser)?;
				trace!(target:"runtime::pallet-ibc", "[get_channels] >> Path: {:?}", path);
				match path {
					Path::ChannelEnds(channel_ends_path) => {
						let ChannelEndsPath(port_id, channel_id) = channel_ends_path;
						result.push((port_id, channel_id));
					},
					_ => unimplemented!(),
				}
			}

			trace!(target:"runtime::pallet-ibc",
				"in channel : [connection_channels] >> Vector<(PortId, ChannelId)> =  {:?}",
				result
			);
			Ok(result)
		} else {
			Err(Ics04Error::connection_not_open(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [client_state] client_id:{:?}",client_id);

		ClientReader::client_state(self, client_id).map_err(Ics04Error::ics02_client)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [client_consensus_state] client_id:{:?},height:{:?}",client_id,height);

		let ret = ClientReader::consensus_state(self, client_id, height)
			.map_err(Ics04Error::ics02_client);

		if let Ok(value) = ret {
			Ok(value)
		} else {
			// TODO(davirain) ics20-transfer deatil with
			Ok(AnyConsensusState::Grandpa(
				ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
			))
		}
	}

	fn get_next_sequence_send(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [get_next_sequence] port_channel_id:{:?}",port_channel_id);

		let seq_sends_path = SeqSendsPath(port_channel_id.0.clone(), port_channel_id.1.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <NextSequenceSend<T>>::contains_key(&seq_sends_path) {
			let sequence = <NextSequenceSend<T>>::get(&seq_sends_path);

			trace!(target:"runtime::pallet-ibc","in channel : [get_next_sequence] >> sequence  = {:?}", sequence);
			Ok(Sequence::from(sequence))
		} else {
			Err(Ics04Error::missing_next_send_seq(port_channel_id.clone()))
		}
	}

	fn get_next_sequence_recv(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [get_next_sequence_recv] port_channel_id:{:?}",port_channel_id);
		let seq_recvs_path = SeqRecvsPath(port_channel_id.0.clone(), port_channel_id.1.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <NextSequenceRecv<T>>::contains_key(&seq_recvs_path) {
			let sequence = <NextSequenceRecv<T>>::get(&seq_recvs_path);

			trace!(target:"runtime::pallet-ibc","in channel : [get_next_sequence_recv] >> sequence = {:?}", sequence);
			Ok(Sequence::from(sequence))
		} else {
			Err(Ics04Error::missing_next_recv_seq(port_channel_id.clone()))
		}
	}

	fn get_next_sequence_ack(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [get_next_sequence_ack] port_channel_id:{:?}",port_channel_id);

		let seq_acks_path = SeqAcksPath(port_channel_id.0.clone(), port_channel_id.1.clone())
			.to_string()
			.as_bytes()
			.to_vec();

		if <NextSequenceAck<T>>::contains_key(&seq_acks_path) {
			let sequence = <NextSequenceAck<T>>::get(&seq_acks_path);

			trace!(target:"runtime::pallet-ibc","in channel : [get_next_sequence_ack] >> sequence = {}", sequence);
			Ok(Sequence::from(sequence))
		} else {
			Err(Ics04Error::missing_next_ack_seq(port_channel_id.clone()))
		}
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<IbcPacketCommitment, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [get_packet_commitment] key:{:?}",key);

		let packet_commitments_path =
			CommitmentsPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		if <PacketCommitment<T>>::contains_key(&packet_commitments_path) {
			let data = <PacketCommitment<T>>::get(&packet_commitments_path);

			let packet_commitment = IbcPacketCommitment::from(data);

			trace!(target:"runtime::pallet-ibc",
				"in channel : [get_packet_commitment] >> packet_commitment = {:?}",
				packet_commitment
			);
			Ok(packet_commitment)
		} else {
			trace!(target:"runtime::pallet-ibc",
				"in channel : [get_packet_commitment] >> read get packet commitment return None"
			);
			Err(Ics04Error::packet_commitment_not_found(key.2))
		}
	}

	fn get_packet_receipt(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<Receipt, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [get_packet_receipt] key:{:?}",key);

		let packet_receipt_path =
			ReceiptsPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		if <PacketReceipt<T>>::contains_key(&packet_receipt_path) {
			let data = <PacketReceipt<T>>::get(&packet_receipt_path);
			let data = String::from_utf8(data).map_err(Ics04Error::invalid_from_utf8)?;
			let data = match data.as_ref() {
				"Ok" => Receipt::Ok,
				_ => unreachable!(),
			};
			trace!(target:"runtime::pallet-ibc","in channel : [get_packet_receipt] >> packet_receipt = {:?}", data);
			Ok(data)
		} else {
			error!(target:"runtime::pallet-ibc","in channel : [get_packet_receipt] >> read get packet receipt not found");
			Err(Ics04Error::packet_receipt_not_found(key.2))
		}
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<IbcAcknowledgementCommitment, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel : [get_packet_acknowledgement] key:{:?}",key);

		let acks_path =
			AcksPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		if <Acknowledgements<T>>::contains_key(&acks_path) {
			let data = <Acknowledgements<T>>::get(&acks_path);

			let acknowledgement = IbcAcknowledgementCommitment::from(data);
			trace!(target:"runtime::pallet-ibc",
				"in channel : [get_packet_acknowledgement] >> packet_acknowledgement = {:?}",
				acknowledgement
			);
			Ok(acknowledgement)
		} else {
			error!(target:"runtime::pallet-ibc",
				"in channel : [get_packet_acknowledgement] >> get acknowledgement not found"
			);
			Err(Ics04Error::packet_acknowledgement_not_found(key.2))
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: Vec<u8>) -> Vec<u8> {
		trace!(target:"runtime::pallet-ibc","in channel: [hash]");

		sp_io::hashing::sha2_256(&value).to_vec()
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		trace!(target:"runtime::pallet-ibc","in channel: [host_height]");

		//todo this can improve
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();

		trace!(target:"runtime::pallet-ibc",
			"in channel: [host_height] >> host_height = {:?}",
			Height::new(REVISION_NUMBER, current_height)
		);
		Height::new(REVISION_NUMBER, current_height).unwrap()
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		trace!(target:"runtime::pallet-ibc","in channel: [host_timestamp]");

		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));
		trace!(target:"runtime::pallet-ibc","in channel: [host_timestamp] >> host_timestamp = {:?}", ts.unwrap());

		ts.unwrap()
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(&self, height: Height) -> Result<AnyConsensusState, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [host_consensus_state] height:{:?}",height);

		// ConnectionReader::host_consensus_state(self,
		// height).map_err(Ics04Error::ics03_connection)
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));

		let ts = ts.unwrap().into_tm_time().unwrap();
		log::trace!(target:"runtime::pallet-ibc","in connection : [host_timestamp] >> host_timestamp = {:?}", ts);

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u32 = block_number.parse().unwrap_or_default();

		trace!(target:"runtime::pallet-ibc",
			"in connection: [host_height] >> host_height = {:?}",current_height

		);

		//TODO: need to build a real consensus state from substrate chain
		let cs = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
			commitment: Commitment::default(),
			state_root: CommitmentRoot::from(vec![1, 2, 3]),
			timestamp: ts,
		};
		trace!(target:"runtime::pallet-ibc","in connection : [host_consensus_state] consensus_state = {:?}", cs);
		Ok(AnyConsensusState::Grandpa(cs))
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [pending_host_consensus_stata]");

		// ClientReader::pending_host_consensus_state(self)
		// 	.map_err(|e| Ics04Error::ics03_connection(ICS03Error::ics02_client(e)))
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));

		let ts = ts.unwrap().into_tm_time().unwrap();
		log::trace!(target:"runtime::pallet-ibc","in connection : [host_timestamp] >> host_timestamp = {:?}", ts);

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u32 = block_number.parse().unwrap_or_default();

		trace!(target:"runtime::pallet-ibc",
			"in connection: [host_height] >> host_height = {:?}",current_height

		);

		//TODO: need to build a real consensus state from substrate chain

		let cs = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
			commitment: Commitment::default(),
			state_root: CommitmentRoot::from(vec![1, 2, 3]),
			timestamp: ts,
		};
		trace!(target:"runtime::pallet-ibc","in connection : [host_consensus_state] consensus_state = {:?}", cs);
		Ok(AnyConsensusState::Grandpa(cs))
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [client_update_time] client_id:{:?},height:{:?}",client_id,height);

		if <ClientProcessedTimes<T>>::contains_key(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
		) {
			let time = <ClientProcessedTimes<T>>::get(
				client_id.as_bytes(),
				height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
			);
			let timestamp = String::from_utf8(time).map_err(Ics04Error::invalid_from_utf8)?;
			let time: Timestamp =
				serde_json::from_str(&timestamp).map_err(Ics04Error::invalid_serde_json_decode)?;
			Ok(time)
		} else {
			error!(target:"runtime::pallet-ibc","in channel: [client_update_time] processed time not found");
			Err(Ics04Error::processed_time_not_found(client_id.clone(), height))
		}
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [client_update_height] client_id:{:?},height:{:?}",client_id,height);

		if <ClientProcessedHeights<T>>::contains_key(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
		) {
			let host_height = <ClientProcessedHeights<T>>::get(
				client_id.as_bytes(),
				height.encode_vec().map_err(|_| Ics04Error::invalid_encode())?,
			);
			let host_height =
				Height::decode(&mut &host_height[..]).map_err(Ics04Error::invalid_decode)?;
			Ok(host_height)
		} else {
			error!(target:"runtime::pallet-ibc","in channel: [client_update_height] processed height not found");
			Err(Ics04Error::processed_height_not_found(client_id.clone(), height))
		}
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, Ics04Error> {
		trace!(target:"runtime::pallet-ibc",
			"in channel: [channel_counter]"
		);
		Ok(<Pallet<T> as Store>::ChannelCounter::get())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		trace!(target:"runtime::pallet-ibc","in channel: [max_expected_time_per_block]");

		Duration::from_secs(6)
	}
}

impl<T: Config> ChannelKeeper for Context<T> {
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		commitment: IbcPacketCommitment,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_packet_commitment]. key={:?}", key);

		let packet_commitments_path =
			CommitmentsPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		// insert packet commitment key-value
		<PacketCommitment<T>>::insert(packet_commitments_path, commitment.into_vec());

		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [delete_packet_commitment]. key={:?}", key);

		let packet_commitments_path =
			CommitmentsPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		// delete packet commitment
		<PacketCommitment<T>>::remove(&packet_commitments_path);

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_packet_receipt]");

		let packet_receipt_path =
			ReceiptsPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
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
		key: (PortId, ChannelId, Sequence),
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_packet_acknowledgement]");

		let acks_path =
			AcksPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		// store packet acknowledgement key-value
		<Acknowledgements<T>>::insert(&acks_path, ack_commitment.into_vec());

		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [delete_packet_acknowledgement]");

		let acks_path =
			AcksPath { port_id: key.0.clone(), channel_id: key.1.clone(), sequence: key.2 }
				.to_string()
				.as_bytes()
				.to_vec();

		// remove acknowledgements
		<Acknowledgements<T>>::remove(&acks_path);

		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_connection_channels] conn_id:{:?},port_channel_id:{:?}",conn_id,port_channel_id);

		// store key
		let connections_path = ConnectionsPath(conn_id).to_string().as_bytes().to_vec();

		// store value
		let channel_ends_path =
			ChannelEndsPath(port_channel_id.0.clone(), port_channel_id.1.clone())
				.to_string()
				.as_bytes()
				.to_vec();

		if <ChannelsConnection<T>>::contains_key(&connections_path) {
			trace!(target:"runtime::pallet-ibc","in channel: [store_connection_channels] >> insert port_channel_id");
			// if connection_id exist
			let ret = <ChannelsConnection<T>>::try_mutate(
				&connections_path,
				|val| -> Result<(), Ics04Error> {
					val.push(channel_ends_path.clone());
					Ok(())
				},
			)
			.map_err(|_| Ics04Error::invalid_store_channels_connection());
		} else {
			// if connection_id no exist
			trace!(target:"runtime::pallet-ibc","in channel: [store_connection_channels] >> init ChannelsConnection");
			<ChannelsConnection<T>>::insert(connections_path, vec![channel_ends_path]);
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_channel] channel_end:{:?},port_channel_id:{:?}",channel_end,port_channel_id);

		let channel_end_path = ChannelEndsPath(port_channel_id.0.clone(), port_channel_id.1)
			.to_string()
			.as_bytes()
			.to_vec();
		let channel_end = channel_end.encode_vec().map_err(|_| Ics04Error::invalid_encode())?;

		// store channels key-value
		<Channels<T>>::insert(channel_end_path, channel_end);

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_next_sequence_send] port_channel_id:{:?},seq:{:?}",port_channel_id,seq);

		let seq_sends_path = SeqSendsPath(port_channel_id.0.clone(), port_channel_id.1)
			.to_string()
			.as_bytes()
			.to_vec();

		let sequence = u64::from(seq);

		<NextSequenceSend<T>>::insert(seq_sends_path, sequence);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_next_sequence_recv] port_channel_id:{:?},seq:{:?}",port_channel_id,seq);

		let seq_recvs_path = SeqRecvsPath(port_channel_id.0.clone(), port_channel_id.1)
			.to_string()
			.as_bytes()
			.to_vec();
		let sequence = u64::from(seq);

		<NextSequenceRecv<T>>::insert(seq_recvs_path, sequence);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Ics04Error> {
		trace!(target:"runtime::pallet-ibc","in channel: [store_next_sequence_ack] port_channel_id:{:?},seq:{:?}",port_channel_id,seq);

		let seq_acks_path = SeqAcksPath(port_channel_id.0.clone(), port_channel_id.1)
			.to_string()
			.as_bytes()
			.to_vec();
		let sequence = u64::from(seq);

		<NextSequenceAck<T>>::insert(seq_acks_path, sequence);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		trace!(target:"runtime::pallet-ibc","in channel: [increase_channel_counter]");

		let ret = <ChannelCounter<T>>::try_mutate(|val| -> Result<(), Ics04Error> {
			let new = val.checked_add(1).ok_or_else(Ics04Error::ivalid_increase_channel_counter)?;
			*val = new;
			Ok(())
		});
	}
}
