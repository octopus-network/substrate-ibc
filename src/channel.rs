use super::*;

use crate::routing::Context;
use ibc::ics02_client::client_consensus::AnyConsensusState;
use ibc::ics02_client::client_state::AnyClientState;
use ibc::ics02_client::context::ClientReader;
use ibc::ics03_connection::connection::ConnectionEnd;
use ibc::ics03_connection::context::ConnectionReader;
use ibc::ics04_channel::channel::ChannelEnd;
use ibc::ics04_channel::context::{ChannelKeeper, ChannelReader};
use ibc::ics04_channel::error::Error as ICS04Error;
use ibc::ics04_channel::packet::{Receipt, Sequence};
use ibc::ics05_port::capabilities::Capability;
use ibc::ics24_host::identifier::ChannelId;
use ibc::ics24_host::identifier::PortId;
use ibc::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::timestamp::Timestamp;
use ibc::Height;
use tendermint_proto::Protobuf;
use ibc::ics05_port::context::PortReader;

impl<T: Config> ChannelReader for Context<T> {
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Option<ChannelEnd> {
		log::info!("in channel_end");

		if <Channels<T>>::contains_key((
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		)) {
			let data = <Channels<T>>::get((
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			));
			Some(ChannelEnd::decode_vec(&*data).unwrap())
		} else {
			log::info!("read channel_end return None");

			None
		}
	}

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(&self, connection_id: &ConnectionId) -> Option<ConnectionEnd> {
		log::info!("in [channel] : connection end");

		let ret = ConnectionReader::connection_end(self, connection_id);

		log::info!("in connection end: {:?}", ret);

		ret
	}

	fn connection_channels(&self, _cid: &ConnectionId) -> Option<Vec<(PortId, ChannelId)>> {
		None
	}

	/// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
	/// proof verification.
	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		log::info!("in client state");

		ClientReader::client_state(self, client_id)
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Option<AnyConsensusState> {
		log::info!("in client consensus state");

		ClientReader::consensus_state(self, client_id, height)
	}

	fn authenticated_capability(&self, port_id: &PortId) -> Result<Capability, ICS04Error> {
		let cap = PortReader::lookup_module_by_port(self, port_id);
		match cap {
			Some(key) => {
				if !PortReader::authenticate(self, &key, port_id) {
					Err(ICS04Error::invalid_port_capability())
				} else {
					Ok(key)
				}
			}
			None => Err(ICS04Error::no_port_capability(port_id.clone())),
		}
	}

	fn get_next_sequence_send(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		log::info!("in get_next_sequence");

		if <NextSequenceSend<T>>::contains_key((
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		)) {
			let data = <NextSequenceSend<T>>::get((
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			));
			let mut data: &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			Some(Sequence::from(seq))
		} else {
			log::info!("read get next sequence send return None");

			None
		}
	}

	fn get_next_sequence_recv(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		log::info!("in get next sequence recv");

		if <NextSequenceRecv<T>>::contains_key((
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		)) {
			let data = <NextSequenceRecv<T>>::get((
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			));
			let mut data: &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			Some(Sequence::from(seq))
		} else {
			log::info!("read get next sequence recv return None");

			None
		}
	}

	fn get_next_sequence_ack(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		log::info!("in get next sequence ack");

		if <NextSequenceAck<T>>::contains_key((
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		)) {
			let data = <NextSequenceAck<T>>::get((
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			));
			let mut data: &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			Some(Sequence::from(seq))
		} else {
			log::info!("read get next sequence ack return None");

			None
		}
	}

	fn get_packet_commitment(&self, key: &(PortId, ChannelId, Sequence)) -> Option<String> {
		log::info!("in get packet commitment");

		let seq = u64::from(key.2);
		let seq = seq.encode();

		if <PacketCommitment<T>>::contains_key((
			key.0.as_bytes(),
			key.1.as_bytes(),
			seq.clone(),
		)) {
			let data = <PacketCommitment<T>>::get((
				key.0.as_bytes(),
				key.1.as_bytes(),
				seq,
			));
			let mut data: &[u8] = &data;
			Some(String::decode(&mut data).unwrap())
		} else {
			log::info!("read get packet commitment return None");

			None
		}
	}

	fn get_packet_receipt(&self, key: &(PortId, ChannelId, Sequence)) -> Option<Receipt> {
		log::info!("in get packet receipt");

		let seq = u64::from(key.2);
		let seq = seq.encode();

		if <PacketReceipt<T>>::contains_key((
			key.0.as_bytes(),
			key.1.as_bytes(),
			seq.clone(),
		)) {
			let data =
				<PacketReceipt<T>>::get((key.0.as_bytes(), key.1.as_bytes(), seq));
			let mut data: &[u8] = &data;
			let data = String::decode(&mut data).unwrap();

			let data = match data.as_ref() {
				"Ok" => Receipt::Ok,
				_ => unreachable!(),
			};
			Some(data)
		} else {
			log::info!("read get packet receipt return None");

			None
		}
	}

	fn get_packet_acknowledgement(&self, key: &(PortId, ChannelId, Sequence)) -> Option<String> {
		log::info!("in get packet acknowledgement");

		let seq = u64::from(key.2);
		let data = seq.encode();

		if <Acknowledgements<T>>::contains_key((
			key.0.as_bytes(),
			key.1.as_bytes(),
			data.clone(),
		)) {
			let data = <Acknowledgements<T>>::get((
				key.0.as_bytes(),
				key.1.as_bytes(),
				data,
			));
			let mut data: &[u8] = &data;
			Some(String::decode(&mut data).unwrap())
		} else {
			log::info!("read get acknowledgement return None");

			None
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: String) -> String {
		log::info!("in hash");

		let r = sp_core::hashing::sha2_256(value.as_bytes());

		let mut tmp = String::new();
		for item in r.iter() {
			tmp.push_str(&format!("{:02x}", item));
		}
		tmp
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		Height::zero()
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		Timestamp::now()
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> u64 {
		log::info!("in channel counter");

		<Pallet<T> as Store>::ChannelCounter::get()
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
		log::info!("in store packet commitment");

		let input = format!("{:?},{:?},{:?}", timestamp, heigh, data);
		let seq = u64::from(key.2);
		let seq = seq.encode();

		<PacketCommitment<T>>::insert(
			(key.0.as_bytes(), key.1.as_bytes(), seq),
			ChannelReader::hash(self, input).as_bytes(),
		);
		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		log::info!("delete packet commitment");

		let seq = u64::from(key.2);
		let seq = seq.encode();
		<PacketCommitment<T>>::remove((key.0.as_bytes(), key.1.as_bytes(), seq));

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), ICS04Error> {
		log::info!("in store packet receipt");

		let receipt = match receipt {
			Receipt::Ok => "Ok".encode(),
		};

		let seq = u64::from(key.2);
		let seq = seq.encode();

		<PacketReceipt<T>>::insert(
			(key.0.as_bytes(), key.1.as_bytes(), seq),
			receipt,
		);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack: Vec<u8>,
	) -> Result<(), ICS04Error> {
		log::info!("in store packet acknowledgement");

		let input = format!("{:?}", ack);
		let seq = u64::from(key.2);
		let data = seq.encode();

		<Acknowledgements<T>>::insert(
			(key.0.as_bytes(), key.1.as_bytes(), data),
			ChannelReader::hash(self, input).as_bytes(),
		);
		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		log::info!("in delete packet acknowledgement");

		let seq = u64::from(key.2);
		let data = seq.encode();

		<Acknowledgements<T>>::remove((key.0.as_bytes(), key.1.as_bytes(), data));

		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), ICS04Error> {
		log::info!("in store connection_channels");

		let conn_id = conn_id.as_bytes().to_vec();

		let port_channel_id = (port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.as_bytes().to_vec());

		if <ChannelsConnection<T>>::contains_key(conn_id.clone())  {
			// if connection_identifier exist
			<ChannelsConnection<T>>::try_mutate(conn_id, |val| -> Result<(), &'static str> {
				val.push(port_channel_id);
				Ok(())
			}).expect("store connection channels error");
		} else {
			// if connection_identifier no exist
			<ChannelsConnection<T>>::insert(conn_id, Vec::<(Vec<u8>, Vec<u8>)>::new());
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), ICS04Error> {
		log::info!("in store channel");

		let data = channel_end.encode_vec().unwrap();

		<Channels<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			data,
		);
		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!("in store next sequence send");

		let seq = u64::from(seq);
		let data = seq.encode();

		<NextSequenceSend<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			data,
		);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!("in store next sequence recv");

		let seq = u64::from(seq);
		let data = seq.encode();

		<NextSequenceRecv<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			data,
		);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!("in store next sequence ack");

		let seq = u64::from(seq);
		let data = seq.encode();

		<NextSequenceAck<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			data,
		);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		log::info!("In client: [increase_channel_counter]");

		<ChannelCounter<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let new = val.checked_add(1).ok_or("Add channel counter error")?;
			*val = new;
			Ok(())
		})
			.expect("increase channel counter error");
	}
}
