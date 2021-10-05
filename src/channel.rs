use super::*;

use crate::routing::Context;
use ibc::ics02_client::client_consensus::AnyConsensusState;
use ibc::ics02_client::client_state::AnyClientState;
use ibc::ics02_client::context::ClientReader;
use ibc::ics02_client::error::Error as ICS02Error;
use ibc::ics03_connection::connection::ConnectionEnd;
use ibc::ics03_connection::context::ConnectionReader;
use ibc::ics03_connection::error::Error as ICS03Error;
use ibc::ics04_channel::channel::ChannelEnd;
use ibc::ics04_channel::context::{ChannelKeeper, ChannelReader};
use ibc::ics04_channel::error::Error as ICS04Error;
use ibc::ics04_channel::packet::{Receipt, Sequence};
use ibc::ics05_port::capabilities::Capability;
use ibc::ics05_port::error::Error as Ics05Error;
use ibc::ics24_host::identifier::ChannelId;
use ibc::ics24_host::identifier::PortId;
use ibc::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::timestamp::Timestamp;
use ibc::Height;
use tendermint_proto::Protobuf;
use ibc::ics05_port::context::PortReader;

impl<T: Config> ChannelReader for Context<T> {
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd,ICS04Error > {
		log::info!("in channel: [channel_end]");

		if <Channels<T>>::contains_key((
			port_channel_id.0.as_bytes(),
			port_channel_id.1.as_bytes(),
		)) {
			let data = <Channels<T>>::get((
				port_channel_id.0.as_bytes(),
				port_channel_id.1.as_bytes(),
			));
			let channel_end = ChannelEnd::decode_vec(&*data).unwrap();
			log::info!("in channel: [channel_end] >> channel_end : {:?}", channel_end.clone());
			Ok(channel_end)
		} else {
			log::info!("read channel_end return None");

			todo!()
		}
	}

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ICS04Error> {
		log::info!("in channel: [connection_end]");

		if <Connections<T>>::contains_key(connection_id.as_bytes()) {
			let data = <Connections<T>>::get(connection_id.as_bytes());
			let ret = ConnectionEnd::decode_vec(&*data).unwrap();
			log::info!("In connection: [connection_end] >>  {:?}", ret.clone());
			Ok(ret)
		} else {
			log::info!("read connection end returns None");

			todo!()
		}
	}

	fn connection_channels(&self, conn_id: &ConnectionId) -> Result<Vec<(PortId, ChannelId)>, ICS04Error> {
		log::info!("in channel: [connection_channels]");
		log::info!("in channel: [connection_channels] >> connection_id : {:?}", conn_id);

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

			log::info!("in channel: [connection_channels] >> result: {:?}", result);
			Ok(result)
		} else {
			todo!()
		}
	}

	/// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
	/// proof verification.
	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS04Error> {
		log::info!("in channel: [client_state]");

		// ClientReader::client_state(self, client_id)
		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			log::info!("In client: [client_state] >> client_state: {:?}", AnyClientState::decode_vec(&*data).unwrap());
			Ok(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::info!("In client: [client_state] >> read client_state is None");

			todo!()
		}
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS04Error> {
		log::info!("in channel: [client_consensus_state]");

		let height = height.encode_vec().unwrap();
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				return Ok(any_consensus_state);
			}
		}
		todo!()
		// ClientReader::consensus_state(self, client_id, height)
	}

	fn authenticated_capability(&self, port_id: &PortId) -> Result<Capability, ICS04Error> {
		log::info!("in channel: [authenticated_capability]");

		match PortReader::lookup_module_by_port(self, port_id) {
			Ok(key) => {
				if !PortReader::authenticate(self, &key, port_id) {
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

	fn get_next_sequence_send(&self, port_channel_id: &(PortId, ChannelId)) -> Result<Sequence, ICS04Error> {
		log::info!("in channel: [get_next_sequence]");

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
			Ok(Sequence::from(seq))
		} else {
			log::info!("read get next sequence send return None");

			todo!()
		}
	}

	fn get_next_sequence_recv(&self, port_channel_id: &(PortId, ChannelId)) -> Result<Sequence, ICS04Error> {
		log::info!("in channel: [get_next_sequence_recv]");

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
			Ok(Sequence::from(seq))
		} else {
			log::info!("read get next sequence recv return None");

			todo!()
		}
	}

	fn get_next_sequence_ack(&self, port_channel_id: &(PortId, ChannelId)) -> Result<Sequence, ICS04Error> {
		log::info!("in channel: [get_next_sequence_ack]");

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
			Ok(Sequence::from(seq))
		} else {
			log::info!("read get next sequence ack return None");

			todo!()
		}
	}

		fn get_packet_commitment(&self, key: &(PortId, ChannelId, Sequence)) -> Result<String, ICS04Error> {
		log::info!("in channel: [get_packet_commitment]");

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
			Ok(String::decode(&mut data).unwrap())
		} else {
			log::info!("read get packet commitment return None");

			todo!()
		}
	}

	fn get_packet_receipt(&self, key: &(PortId, ChannelId, Sequence)) -> Result<Receipt, ICS04Error> {
		log::info!("in channel: [get_packet_receipt]");

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
			Ok(data)
		} else {
			log::info!("read get packet receipt return None");

			todo!()
		}
	}

	fn get_packet_acknowledgement(&self, key: &(PortId, ChannelId, Sequence)) -> Result<String, ICS04Error> {
		log::info!("in channel: [get_packet_acknowledgement]");

		let seq = u64::from(key.2);
		let seq = seq.encode();

		if <Acknowledgements<T>>::contains_key((
			key.0.as_bytes(),
			key.1.as_bytes(),
			seq.clone(),
		)) {
			let data = <Acknowledgements<T>>::get((
				key.0.as_bytes(),
				key.1.as_bytes(),
				seq,
			));
			let mut data: &[u8] = &data;
			Ok(String::decode(&mut data).unwrap())
		} else {
			log::info!("read get acknowledgement return None");

			todo!()
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: String) -> String {
		log::info!("in channel: [hash]");

		let r = sp_core::hashing::sha2_256(value.as_bytes());

		let mut tmp = String::new();
		for item in r.iter() {
			tmp.push_str(&format!("{:02x}", item));
		}
		tmp
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		log::info!("in channel: [host_height]");

		Height::zero()
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		log::info!("in channel: [host_timestamp]");

		Timestamp::now()
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ICS04Error> {
		log::info!("in channel : [channel_counter]");

		Ok(<Pallet<T> as Store>::ChannelCounter::get())
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
		log::info!("in channel: [store_packet_commitment]");

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
		log::info!("in channel: [delete_packet_commitment]");

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
		log::info!("in channel: [store_packet_receipt]");

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
		log::info!("in channel: [store_packet_acknowledgement]");

		let ack = format!("{:?}", ack);
		let seq = u64::from(key.2);
		let seq = seq.encode();

		<Acknowledgements<T>>::insert(
			(key.0.as_bytes(), key.1.as_bytes(), seq),
			ChannelReader::hash(self, ack).as_bytes(),
		);
		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		log::info!("in channel: [delete_packet_acknowledgement]");

		let seq = u64::from(key.2);
		let seq = seq.encode();

		<Acknowledgements<T>>::remove((key.0.as_bytes(), key.1.as_bytes(), seq));

		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), ICS04Error> {
		log::info!("in channel: [store_connection_channels]");

		let conn_id = conn_id.as_bytes().to_vec();

		let port_channel_id = (port_channel_id.0.as_bytes().to_vec(), port_channel_id.1.as_bytes().to_vec());

		if <ChannelsConnection<T>>::contains_key(conn_id.clone())  {
			log::info!("in channel: [store_connection_channels] >> insert port_channel_id");
			// if connection_identifier exist
			<ChannelsConnection<T>>::try_mutate(conn_id, |val| -> Result<(), &'static str> {
				val.push(port_channel_id);
				Ok(())
			}).expect("store connection channels error");
		} else {
			// if connection_identifier no exist
			log::info!("in channel: [store_connection_channels] >> init ChannelsConnection");
			let mut temp_connection_channels = vec![];
			temp_connection_channels.push(port_channel_id);
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
		log::info!("in channel: [store_channel]");

		let channel_end = channel_end.encode_vec().unwrap();

		<Channels<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			channel_end,
		);
		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!("in channel: [store_next_sequence_send]");

		let seq = u64::from(seq);
		let seq = seq.encode();

		<NextSequenceSend<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			seq,
		);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!("in channel: [store_next_sequence_recv]");

		let seq = u64::from(seq);
		let seq = seq.encode();

		<NextSequenceRecv<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			seq,
		);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		log::info!("in channel: [store_next_sequence_ack]");

		let seq = u64::from(seq);
		let seq = seq.encode();

		<NextSequenceAck<T>>::insert(
			(port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()),
			seq,
		);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		log::info!("In channel: [increase_channel_counter]");

		<ChannelCounter<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let new = val.checked_add(1).ok_or("Add channel counter error")?;
			*val = new;
			Ok(())
		})
			.expect("increase channel counter error");

	}
}
