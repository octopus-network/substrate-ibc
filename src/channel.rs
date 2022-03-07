use super::*;
use core::str::FromStr;
use core::time::Duration;

use crate::routing::Context;
use ibc::{
	core::ics02_client::{client_consensus::AnyConsensusState, client_state::AnyClientState},
	core::ics03_connection::connection::ConnectionEnd,
	core::ics03_connection::error::Error as ICS03Error,
	core::ics04_channel::{
		channel::ChannelEnd,
		context::{ChannelKeeper, ChannelReader},
		error::Error as ICS04Error,
		packet::{Receipt, Sequence},
	},
	core::ics05_port::{capabilities::Capability, context::PortReader, error::Error as Ics05Error},
	core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	timestamp::Timestamp,
	Height,
};
use ibc::core::ics02_client::context::ClientReader;
use ibc::core::ics03_connection::context::ConnectionReader;

impl<T: Config> ChannelReader for Context<T> {

	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, ICS04Error> {
		// Todo: Confirm if all the errors are accurate
		log::info!(
			"in channel : [channel_end] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		if <Channels<T>>::contains_key(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()) {
			let data =
				<Channels<T>>::get(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes());
			let channel_end = ChannelEnd::decode_vec(&*data).unwrap();
			log::info!("in channel : [channel_end] >> channel_end = {:?}", channel_end);
			Ok(channel_end)
		} else {
			log::info!("in channel : [channel_end] >> read channel_end return None");
			Err(ICS04Error::channel_not_found(port_channel_id.clone().0, port_channel_id.clone().1))
		}
	}

	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ICS04Error> {
		log::info!("in channel : [connection_end] >> connection_id = {:?}", connection_id);

		if <Connections<T>>::contains_key(connection_id.as_bytes()) {
			let data = <Connections<T>>::get(connection_id.as_bytes());
			let ret = ConnectionEnd::decode_vec(&*data).unwrap();
			log::info!("In channel : [connection_end] >> connection_end = {:?}", ret);
			Ok(ret)
		} else {
			log::info!("in channel : [channel_end] >> read connection end returns None");
			Err(ICS04Error::connection_not_open(connection_id.clone()))
		}
	}

	/// Returns the `ChannelsConnection` for the given identifier `conn_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ICS04Error> {
		log::info!("in channel : [connection_channels] >> connection_id : {:?}", conn_id);

		return if <ChannelsConnection<T>>::contains_key(conn_id.as_bytes()) {
			let port_and_channel_id = <ChannelsConnection<T>>::get(conn_id.as_bytes());

			let mut result = vec![];

			for item in port_and_channel_id.iter() {
				let port_id = String::from_utf8(item.0.clone()).unwrap();
				let port_id = PortId::from_str(port_id.as_str()).unwrap();

				let channel_id = String::from_utf8(item.1.clone()).unwrap();
				let channel_id = ChannelId::from_str(channel_id.as_str()).unwrap();

				result.push((port_id, channel_id));
			}

			log::info!(
				"in channel : [connection_channels] >> Vector<(PortId, ChannelId)> =  {:?}",
				result
			);
			Ok(result)
		} else {
			Err(ICS04Error::connection_not_open(conn_id.clone()))
		};
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS04Error> {
		log::info!("in channel : [client_state] >> client_id = {:?}", client_id);

		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			log::info!(
				"in channel : [client_state] >> Any client state: {:?}",
				AnyClientState::decode_vec(&*data).unwrap()
			);
			Ok(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::info!("In client : [client_state] >> read client_state is None");

			Err(ICS04Error::frozen_client(client_id.clone()))
		}
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS04Error> {
		log::info!(
			"in channel : [client_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let height = height.encode_vec().unwrap();
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				log::info!(
					"in channel: [client_consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(any_consensus_state);
			}
		}
		log::info!(
			"in channel : [client_consensus_state] >> read about client_id consensus_state error"
		);

		// TODO
		// Err(ICS04Error::frozen_client(client_id.clone()))
		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	fn authenticated_capability(&self, port_id: &PortId) -> Result<Capability, ICS04Error> {
		log::info!("in channel : [authenticated_capability] >> port_id: {:?}", port_id);

		match PortReader::lookup_module_by_port(self, port_id) {
			Ok((_, key)) => {
				if !PortReader::authenticate(self, port_id.clone(), &key) {
					Err(ICS04Error::invalid_port_capability())
				} else {
					Ok(key)
				}
			}
			Err(e) if e.detail() == Ics05Error::unknown_port(port_id.clone()).detail() => {
				Err(ICS04Error::no_port_capability(port_id.clone()))
			}
			Err(_) => Err(ICS04Error::implementation_specific()),
		}
	}

	fn get_next_sequence_send(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		log::info!(
			"in channel : [get_next_sequence] >> port_id = {:?}, channel_id =  {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		if <NextSequenceSend<T>>::contains_key(
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		) {
			let data = <NextSequenceSend<T>>::get(
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			);
			let mut data: &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			log::info!("in channel : [get_next_sequence] >> sequence  = {:?}", seq);
			Ok(Sequence::from(seq))
		} else {
			log::info!(
				"in channel : [get_next_sequence] >> read get next sequence send return None"
			);
			Err(ICS04Error::missing_next_send_seq(port_channel_id.clone()))
		}
	}

	fn get_next_sequence_recv(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		log::info!(
			"in channel : [get_next_sequence_recv] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		if <NextSequenceRecv<T>>::contains_key(
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		) {
			let data = <NextSequenceRecv<T>>::get(
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			);
			let mut data: &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			log::info!("in channel : [get_next_sequence_recv] >> sequence = {:?}", seq);
			Ok(Sequence::from(seq))
		} else {
			log::info!(
				"in channel : [get_next_sequence_recv] >> read get next sequence recv return None"
			);
			Err(ICS04Error::missing_next_recv_seq(port_channel_id.clone()))
		}
	}

	fn get_next_sequence_ack(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		log::info!(
			"in channel : [get_next_sequence_ack] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		if <NextSequenceAck<T>>::contains_key(
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		) {
			let data = <NextSequenceAck<T>>::get(
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			);
			let mut data: &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			log::info!("in channel : [get_next_sequence_ack] >> sequence = {:?}", seq);
			Ok(Sequence::from(seq))
		} else {
			log::info!(
				"in channel : [get_next_sequence_ack] >> read get next sequence ack return None"
			);
			Err(ICS04Error::missing_next_ack_seq(port_channel_id.clone()))
		}
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<String, ICS04Error> {
		log::info!("in channel : [get_packet_commitment] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0, key.1, key.2
		);

		let seq = u64::from(key.2);
		let seq = seq.encode();

		if <PacketCommitment<T>>::contains_key((key.0.as_bytes(), key.1.as_bytes(), seq.clone())) {
			let data = <PacketCommitment<T>>::get((key.0.as_bytes(), key.1.as_bytes(), seq));
			let mut data: &[u8] = &data;
			let data = Vec::<u8>::decode(&mut data).unwrap();
			log::info!(
				"in channel : [get_packet_commitment] >> packet_commitment = {:?}, packet_commitment_vec = {:?}",
				String::from_utf8(data.clone()).unwrap(), data.clone()
			);
			Ok(String::from_utf8(data).unwrap())
		} else {
			log::info!(
				"in channel : [get_packet_commitment] >> read get packet commitment return None"
			);
			Err(ICS04Error::packet_commitment_not_found(key.2))
		}
	}

	fn get_packet_receipt(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<Receipt, ICS04Error> {
		log::info!("in channel : [get_packet_receipt] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0, key.1, key.2
		);

		let seq = u64::from(key.2);
		let seq = seq.encode();

		if <PacketReceipt<T>>::contains_key((key.0.as_bytes(), key.1.as_bytes(), seq.clone())) {
			let data = <PacketReceipt<T>>::get((key.0.as_bytes(), key.1.as_bytes(), seq));
			let mut data: &[u8] = &data;
			// let data = String::decode(&mut data).unwrap();
			let data = Vec::<u8>::decode(&mut data).unwrap();
			let data = String::from_utf8(data).unwrap();

			let data = match data.as_ref() {
				"Ok" => Receipt::Ok,
				_ => unreachable!(),
			};
			log::info!("in channel : [get_packet_receipt] >> packet_receipt = {:?}", data);
			Ok(data)
		} else {
			log::info!("in channel : [get_packet_receipt] >> read get packet receipt not found");
			Err(ICS04Error::packet_receipt_not_found(key.2))
		}
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<String, ICS04Error> {
		log::info!("in channel : [get_packet_acknowledgement] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0, key.1, key.2
		);

		let seq = u64::from(key.2);
		let seq = seq.encode();

		if <Acknowledgements<T>>::contains_key((key.0.as_bytes(), key.1.as_bytes(), seq.clone())) {
			let data = <Acknowledgements<T>>::get((key.0.as_bytes(), key.1.as_bytes(), seq));
			let mut data: &[u8] = &data;
			let data = Vec::<u8>::decode(&mut data).unwrap();
			log::info!(
				"in channel : [get_packet_acknowledgement] >> packet_acknowledgement = {:?}",
				String::from_utf8(data.clone()).unwrap()
			);
			Ok(String::from_utf8(data).unwrap())
		} else {
			log::info!(
				"in channel : [get_packet_acknowledgement] >> get acknowledgement not found"
			);
			Err(ICS04Error::packet_acknowledgement_not_found(key.2))
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: String) -> String {
		log::info!("in channel: [hash] >> value = {:?}", value.clone());

		let r = sp_io::hashing::sha2_256(value.as_bytes());

		let mut tmp = String::new();
		for item in r.iter() {
			tmp.push_str(&format!("{:02x}", item));
		}
		log::info!("in channel: [hash] >> result = {:?}", tmp.clone());
		tmp
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		log::info!("in channel: [host_height]");

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height = block_number
			.parse()
			.map_err(|e| panic!("{:?}, caused by {:?} from frame_system::Pallet", e, block_number));
		log::info!(
			"in channel: [host_height] >> host_height = {:?}",
			Height::new(0, current_height.unwrap())
		);
		Height::new(0, current_height.unwrap())
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		log::info!("in channel: [host_timestamp]");

		// use frame_support::traits::UnixTime;
		// let time = T::TimeProvider::now();
		// let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
		// 	.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));
		// log::info!("in channel: [host_timestamp] >> host_timestamp = {:?}", ts.clone().unwrap());
		// ts.unwrap()
		ClientReader::host_timestamp(self)
	}

	/// Returns the `AnyConsensusState` for the given identifier `height`.
	fn host_consensus_state(&self, height: Height) -> Result<AnyConsensusState, ICS04Error> {
		log::info!("in channel: [host_consensus_state] >> height = {:?}", height);

		ConnectionReader::host_consensus_state(self, height).map_err(ICS04Error::ics03_connection)
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, ICS04Error> {
		log::info!("in channel: [pending_host_consensus_stata]");

		ClientReader::pending_host_consensus_state(self)
			.map_err(|e| ICS04Error::ics03_connection(ICS03Error::ics02_client(e)))
	}

	/// Returns the `ClientProcessedTimes` for the given identifier `client_id` & `height`.
	fn client_update_time(&self, client_id: &ClientId, height: Height) -> Result<Timestamp, ICS04Error> {
		log::info!("in channel: [client_update_time] >> client_id = {:?}, height = {:?}", client_id, height);

		if <ClientProcessedTimes<T>>::contains_key(client_id.as_bytes(), height.encode_vec().unwrap()) {
			let time = <ClientProcessedTimes<T>>::get(client_id.as_bytes(), height.encode_vec().unwrap());
			let timestamp = String::from_utf8(time).unwrap();
			let time: Timestamp = serde_json::from_str(&timestamp).unwrap();
			Ok(time)
		} else {
			Err(ICS04Error::processed_time_not_found(client_id.clone(), height))
		}
	}

	fn client_update_height(&self, client_id: &ClientId, height: Height) -> Result<Height, ICS04Error> {
		if <ClientProcessedHeights<T>>::contains_key(client_id.as_bytes(), height.encode_vec().unwrap()) {
			let host_height = <ClientProcessedHeights<T>>::get(client_id.as_bytes(), height.encode_vec().unwrap());
			let host_height = Height::decode(&host_height[..]).unwrap();
			Ok(host_height)
		} else {
			Err(ICS04Error::processed_height_not_found(
				client_id.clone(),
				height,
			))
		}
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ICS04Error> {
		log::info!(
			"in channel: [channel_counter] >> channel_counter = {:?}",
			<Pallet<T> as Store>::ChannelCounter::get()
		);
		Ok(<Pallet<T> as Store>::ChannelCounter::get())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		todo!()
	}
}

impl<T: Config> ChannelKeeper for Context<T> {
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		timestamp: Timestamp,
		heigh: Height,
		data: Vec<u8>,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_packet_commitment] >> port_id = {:?}, channel_id = {:?}, \
		sequence = {:?}, timestamp = {:?}, height  = {:?}, data = {:?}",
			key.0,
			key.1,
			key.2,
			timestamp,
			heigh,
			data
		);

		let input = format!("{:?},{:?},{:?}", timestamp, heigh, data);
		let seq = u64::from(key.2);
		let seq = seq.encode();

		// inser packet commitment key-value
		<PacketCommitment<T>>::insert(
			(key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone()),
			ChannelReader::hash(self, input.clone()).encode(),
		);
		log::info!("in channel: [store_packet_commitment], val = {:?}", ChannelReader::hash(self, input).encode());

		// insert packet commitment keys
		<PacketCommitmentKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			val.push((key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone()));
			Ok(())
		})
		.expect("store packet commitment keys error");
		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [delete_packet_commitment] >> port_id = {:?}, channel_id = {:?}, \
		sequence = {:?}",
			key.0,
			key.1,
			key.2
		);

		let seq = u64::from(key.2);
		let seq = seq.encode();

		// delete packet commitment
		<PacketCommitment<T>>::remove((
			key.0.as_bytes().to_vec(),
			key.1.as_bytes().to_vec(),
			seq.clone(),
		));

		// delete packet commitment keys
		<PacketCommitmentKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let index = val
				.iter()
				.position(|value| {
					value == &(key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone())
				})
				.unwrap();
			let ret = val.remove(index);
			assert_eq!(ret, (key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone()));
			Ok(())
		})
		.expect("delete packet commitment keys error");
		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_packet_receipt] >> port_id = {:?}, channel_id = {:?},\
		 sequence = {:?}, receipt = {:?}",
			key.0,
			key.1,
			key.2,
			receipt
		);

		let receipt = match receipt {
			Receipt::Ok => "Ok".encode(),
		};

		let seq = u64::from(key.2);
		let seq = seq.encode();

		<PacketReceipt<T>>::insert(
			(key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq),
			receipt,
		);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack: Vec<u8>,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_packet_acknowledgement] >> port_id = {:?}, channel_id = {:?},\
		 sequence = {:?}, ack = {:?}",
			key.0,
			key.1,
			key.2,
			ack
		);

		let ack = format!("{:?}", ack);
		let seq = u64::from(key.2);
		let seq = seq.encode();

		log::info!("in channel: [store_packet_acknowledgement], ack hash = {:?}", ChannelReader::hash(self, ack.clone()).encode());
		// store packet acknowledgement key-value
		<Acknowledgements<T>>::insert(
			(key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone()),
			ChannelReader::hash(self, ack).encode(),
		);

		// store packet acknowledgement keys
		<AcknowledgementsKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			val.push((key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq));
			Ok(())
		})
		.expect("store acknowledgement keys error");

		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [delete_packet_acknowledgement] >> port_id = {:?}, channel_id = {:?},\
		 sequence = {:?}",
			key.0,
			key.1,
			key.2
		);

		let seq = u64::from(key.2);
		let seq = seq.encode();

		// remove acknowledgements
		<Acknowledgements<T>>::remove((
			key.0.as_bytes().to_vec(),
			key.1.as_bytes().to_vec(),
			seq.clone(),
		));

		// remove acknowledgement keys
		<AcknowledgementsKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let index = val
				.iter()
				.position(|value| {
					value == &(key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone())
				})
				.unwrap();
			let ret = val.remove(index);
			assert_eq!(&ret, &(key.0.as_bytes().to_vec(), key.1.as_bytes().to_vec(), seq.clone()));
			Ok(())
		})
		.expect("delete packet acknowledgement keys error");

		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_connection_channels] >> connection_id = {:?}, port_id = {:?},\
		 channel_id = {:?}",
			conn_id,
			port_channel_id.0,
			port_channel_id.1
		);

		let conn_id = conn_id.as_bytes().to_vec();

		let port_channel_id =
			(port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.as_bytes().to_vec());

		if <ChannelsConnection<T>>::contains_key(conn_id.clone()) {
			log::info!("in channel: [store_connection_channels] >> insert port_channel_id");
			// if connection_identifier exist
			<ChannelsConnection<T>>::try_mutate(conn_id, |val| -> Result<(), &'static str> {
				val.push(port_channel_id);
				Ok(())
			})
			.expect("store connection channels error");
		} else {
			// if connection_identifier no exist
			log::info!("in channel: [store_connection_channels] >> init ChannelsConnection");
			let temp_connection_channels = vec![port_channel_id];
			<ChannelsConnection<T>>::insert(conn_id, temp_connection_channels);
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_channel] >> port_id = {:?}, channel_id = {:?}, \
			channel_end = {:?}",
			port_channel_id.0,
			port_channel_id.1,
			channel_end
		);

		let channel_end = channel_end.encode_vec().unwrap();

		// store channels key-value
		<Channels<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.as_bytes().to_vec(),
			channel_end,
		);

		// store channels keys
		<ChannelsKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			if let Some(_value) = val.iter().find(|&x| {
				x == &(port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.as_bytes().to_vec())
			}) {
			} else {
				val.push((
					port_channel_id.0.as_bytes().to_vec(),
					port_channel_id.1.as_bytes().to_vec(),
				));
			}

			Ok(())
		})
		.expect("store channels keys error");

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_next_sequence_send] >> port_id = {:?}, channel_id = {:?},\
		 sequence = {:?}",
			port_channel_id.0,
			port_channel_id.1,
			seq
		);

		let seq = u64::from(seq);
		let seq = seq.encode();

		<NextSequenceSend<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.as_bytes().to_vec(),
			seq,
		);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_next_sequence_recv] >> port_id = {:?}, channel_id = {:?}, \
			seq = {:?}",
			port_channel_id.0,
			port_channel_id.1,
			seq
		);

		let seq = u64::from(seq);
		let seq = seq.encode();

		<NextSequenceRecv<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.as_bytes().to_vec(),
			seq,
		);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!(
			"in channel: [store_next_sequence_ack] >> port_id = {:?}, channel_id = {:?},\
		 sequence = {:?}",
			port_channel_id.0,
			port_channel_id.1,
			seq
		);

		let seq = u64::from(seq);
		let seq = seq.encode();

		<NextSequenceAck<T>>::insert(
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.as_bytes().to_vec(),
			seq,
		);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		log::info!("in channel: [increase_channel_counter]");

		<ChannelCounter<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let new = val.checked_add(1).ok_or("Add channel counter error")?;
			*val = new;
			Ok(())
		})
		.expect("increase channel counter error");
	}
}
