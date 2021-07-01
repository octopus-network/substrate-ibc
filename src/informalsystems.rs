use super::*;
use ibc::application::ics20_fungible_token_transfer::context::Ics20Context;
use ibc::ics02_client::client_consensus::AnyConsensusState;
use ibc::ics02_client::client_state::AnyClientState;
use ibc::ics02_client::client_type::ClientType;
use ibc::ics02_client::context::{ClientKeeper, ClientReader};
use ibc::ics02_client::error::Error as ICS02Error;
use ibc::ics03_connection::connection::ConnectionEnd;
use ibc::ics03_connection::context::{ConnectionKeeper, ConnectionReader};
use ibc::ics03_connection::error::Error as ICS03Error;
use ibc::ics04_channel::channel::ChannelEnd;
use ibc::ics04_channel::context::{ChannelKeeper, ChannelReader};
use ibc::ics04_channel::error::Error;
use ibc::ics04_channel::packet::{Receipt, Sequence};
use ibc::ics05_port::capabilities::Capability;
use ibc::ics05_port::context::PortReader;
use ibc::ics23_commitment::commitment::CommitmentPrefix;
use ibc::ics24_host::identifier::ChannelId;
use ibc::ics24_host::identifier::PortId;
use ibc::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::ics26_routing::context::Ics26Context;
use ibc::timestamp::Timestamp;
use ibc::Height;
use tendermint_proto::Protobuf;
use std::str::FromStr;
use alloc::format;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Any {
	pub type_url: String,
	pub value: Vec<u8>,
}

#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub tmp: u8,
}

impl<T: Config> Ics26Context for Context<T> {}

impl<T: Config> Ics20Context for Context<T> {}

impl<T: Config> ChannelKeeper for Context<T> {
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		timestamp: Timestamp,
		heigh: Height,
		data: Vec<u8>,
	) -> Result<(), Error> {
		log::info!("in store packet commitment");

		let input = format!("{:?},{:?},{:?}", timestamp, heigh, data);
		let seq = u64::from(key.2);
		let seq = seq.encode();
		<Pallet<T> as Store>::PacketCommitmentV2::insert((key.0.as_bytes(), key.1.as_bytes(), seq), ChannelReader::hash(self, input).as_bytes());
		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Error> {
		log::info!("delete packet commitment");

		let seq = u64::from(key.2);
		let seq = seq.encode();
		<Pallet<T> as Store>::PacketCommitmentV2::remove((key.0.as_bytes(), key.1.as_bytes(), seq));

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), Error> {
		log::info!("in store packet receipt");

		let receipt = match receipt {
			Receipt::Ok => "Ok".encode(),
		};

		let seq = u64::from(key.2);
		let seq = seq.encode();

		<Pallet<T> as Store>::PacketReceiptV2::insert((key.0.as_bytes(), key.1.as_bytes(), seq), receipt);
		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack: Vec<u8>,
	) -> Result<(), Error> {
		log::info!("in store packet acknowledgement");

		let input = format!("{:?}", ack);
		let seq = u64::from(key.2);
		let data = seq.encode();
		<Pallet<T> as Store>::AcknowledgementsV2::insert((key.0.as_bytes(), key.1.as_bytes(), data), ChannelReader::hash(self, input).as_bytes());
		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Error> {
		log::info!("in delete packet acknowledgement");

		let seq = u64::from(key.2);
		let data = seq.encode();
		<Pallet<T> as Store>::AcknowledgementsV2::remove((key.0.as_bytes(), key.1.as_bytes(), data));
		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		_conn_id: ConnectionId,
		_port_channel_id: &(PortId, ChannelId),
	) -> Result<(), Error> {
		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), Error> {
		log::info!("in store channel");

		let data = channel_end.encode_vec().unwrap();
		<Pallet<T> as Store>::ChannelsV2::insert((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()), data);
		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Error> {
		log::info!("in store next sequence send");

		let seq = u64::from(seq);
		let data = seq.encode();
		<Pallet<T> as Store>::NextSequenceSendV2::insert((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()), data);
		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Error> {
		log::info!("in store next sequence recv");

		let seq = u64::from(seq);
		let data = seq.encode();
		<Pallet<T> as Store>::NextSequenceRecvV2::insert((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()), data);
		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Error> {
		log::info!("in store next sequence ack");

		let seq = u64::from(seq);
		let data = seq.encode();
		<Pallet<T> as Store>::NextSequenceAckV2::insert((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()), data);
		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		log::info!("in increase channel counter");

		match <Pallet<T> as Store>::ChannelCounterV2::get() {
			None => {},
			Some(old) => {
				let new  = old.checked_add(1).unwrap();
				<Pallet<T> as Store>::ChannelCounterV2::put(new)
			},
		}
	}
}

impl<T: Config> ChannelReader for Context<T> {
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Option<ChannelEnd> {
		log::info!("in channel_end");

		if <Pallet<T> as Store>::ChannelsV2::contains_key((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes())) {
			let data = <Pallet<T> as Store>::ChannelsV2::get((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()));
			Some(ChannelEnd::decode_vec(&*data).unwrap())
		} else {
			log::info!("read channel_end return None");

			None
		}
	}

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(&self, connection_id: &ConnectionId) -> Option<ConnectionEnd> {
		log::info!("in connection end");

		ConnectionReader::connection_end(self, connection_id)
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

	fn authenticated_capability(&self, _port_id: &PortId) -> Result<Capability, Error> {
		unimplemented!()
	}

	fn get_next_sequence_send(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		log::info!("in get_next_sequence");

		if <Pallet<T> as Store>::NextSequenceSendV2::contains_key((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes())) {
			let data = <Pallet<T> as Store>::NextSequenceSendV2::get((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()));
			let mut data : &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			Some(Sequence::from(seq))
		} else {
			log::info!("read get next sequence send return None");

			None
		}
	}

	fn get_next_sequence_recv(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		log::info!("in get next sequence recv");

		if <Pallet<T> as Store>::NextSequenceRecvV2::contains_key((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes())) {
			let data = <Pallet<T> as Store>::NextSequenceRecvV2::get((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()));
			let mut data : &[u8] = &data;
			let seq = u64::decode(&mut data).unwrap();
			Some(Sequence::from(seq))
		} else {
			log::info!("read get next sequence recv return None");

			None
		}
	}

	fn get_next_sequence_ack(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		log::info!("in get next sequence ack");

		if <Pallet<T> as Store>::NextSequenceAckV2::contains_key((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes())) {
			let data = <Pallet<T> as Store>::NextSequenceAckV2::get((port_channel_id.0.as_bytes(), port_channel_id.1.as_bytes()));
			let mut data : &[u8] = &data;
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
		if <Pallet<T> as Store>::PacketCommitmentV2::contains_key((key.0.as_bytes(), key.1.as_bytes(), seq.clone())) {
			let data = <Pallet<T> as Store>::PacketCommitmentV2::get((key.0.as_bytes(), key.1.as_bytes(), seq));
			let mut data : &[u8] = &data;
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
		if <Pallet<T> as Store>::PacketReceiptV2::contains_key((key.0.as_bytes(), key.1.as_bytes(), seq.clone())) {
			let data = <Pallet<T> as Store>::PacketReceiptV2::get((key.0.as_bytes(), key.1.as_bytes(), seq));
			let mut data : &[u8] = &data;
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
		if <Pallet<T> as Store>::AcknowledgementsV2::contains_key((key.0.as_bytes(), key.1.as_bytes(), data.clone())) {
			let data = <Pallet<T> as Store>::AcknowledgementsV2::get((key.0.as_bytes(), key.1.as_bytes(), data));
			let mut data : &[u8] = &data;
			Some(String::decode(&mut data).unwrap())
		} else {
			log::info!("read get acknowledgement return None");

			None
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: String) -> String {
		log::info!("in hash");

		let r =  sp_core::hashing::sha2_256(value.as_bytes());

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

		<Pallet<T> as Store>::ChannelCounterV2::get().unwrap()
	}
}

impl<T: Config> PortReader for Context<T> {
	fn lookup_module_by_port(&self, _port_id: &PortId) -> Option<Capability> {
		None
	}
	fn authenticate(&self, _key: &Capability, _port_id: &PortId) -> bool {
		false
	}
}

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {
		log::info!("in read client_type");

		if <Pallet<T> as Store>::ClientsV2::contains_key(client_id.as_bytes()) {
			let data = <Pallet<T> as Store>::ClientsV2::get(client_id.as_bytes());
			let mut data : &[u8] = &data;
			let data = String::decode(&mut data).unwrap();
			match ClientType::from_str(&data) {
				Err(_err) => None,
				Ok(val) => Some(val),
			}
		} else {
			log::info!("read client type returns None");

			None
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		log::info!("in read client_state");

		if <Pallet<T> as Store>::ClientStatesV2::contains_key(client_id.as_bytes()) {
			let data = <Pallet<T> as Store>::ClientStatesV2::get(client_id.as_bytes());
			Some(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::info!("read client_state returns None");

			None
		}
	}

	fn consensus_state(&self, client_id: &ClientId, height: Height) -> Option<AnyConsensusState> {
		log::info!("in read consensus_state");

		let height = height.encode_vec().unwrap();
		if <Pallet<T> as Store>::ConsensusStatesV2::contains_key((client_id.as_bytes(), &height)) {
			let data = <Pallet<T> as Store>::ConsensusStatesV2::get((client_id.as_bytes(), height));
			Some(AnyConsensusState::decode_vec(&*data).unwrap())
		} else {
			log::info!("read consensus_state returns None");

			None
		}
	}
	fn client_counter(&self) -> u64 {
		log::info!("in read client counter");

		<Pallet<T> as Store>::ClientCounterV2::get().unwrap()
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {
		log::info!("in store_client_type");

		let data = client_type.as_string().encode();
		<Pallet<T> as Store>::ClientsV2::insert(client_id.as_bytes(), data);
		Ok(())
	}

	fn increase_client_counter(&mut self) {
		log::info!("in increase client counter");

		match <Pallet<T> as Store>::ClientCounterV2::get() {
			None => {},
			Some(old) => {
				let new  = old.checked_add(1).unwrap();
				<Pallet<T> as Store>::ClientCounterV2::put(new)
			},
		}
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: AnyClientState,
	) -> Result<(), ICS02Error> {
		log::info!("in store_client_state");

		let data = client_state.encode_vec().unwrap();
		<Pallet<T> as Store>::ClientStatesV2::insert(client_id.as_bytes(), data);
		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: AnyConsensusState,
	) -> Result<(), ICS02Error> {
		log::info!("in store_consensus_state");

		let height = height.encode_vec().unwrap();
		let data = consensus_state.encode_vec().unwrap();
		<Pallet<T> as Store>::ConsensusStatesV2::insert((client_id.as_bytes(), height), data);
		Ok(())
	}
}

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Option<ConnectionEnd> {
		log::info!("in connection_end");

		if <Pallet<T> as Store>::ConnectionsV2::contains_key(conn_id.as_bytes()) {
			let data = <Pallet<T> as Store>::ConnectionsV2::get(conn_id.as_bytes());
			Some(ConnectionEnd::decode_vec(&*data).unwrap())
		} else {
			log::info!("read connection end returns None");

			None
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		log::info!("in client state");

		ClientReader::client_state(self, client_id)
	}

	fn host_current_height(&self) -> Height {
		Height::zero()
	}

	fn host_oldest_height(&self) -> Height {
		Height::zero()
	}

	fn connection_counter(&self) -> u64 {
		log::info!("in connection counter");

		<Pallet<T> as Store>::ConnectionCounterV2::get().unwrap()
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		vec![0].into()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Option<AnyConsensusState> {
		log::info!("in client consensus state");
		
		ClientReader::consensus_state(self, client_id, height)
	}

	fn host_consensus_state(&self, _height: Height) -> Option<AnyConsensusState> {
		None
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn increase_connection_counter(&mut self) {
		log::info!("in increase connection counter");

		match <Pallet<T> as Store>::ConnectionCounterV2::get() {
			None => {},
			Some(old) => {
				let new  = old.checked_add(1).unwrap();
				<Pallet<T> as Store>::ConnectionCounterV2::put(new)
			},
		}
	}

	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), ICS03Error> {
		log::info!("in store_connection");

		let data = connection_end.encode_vec().unwrap();
		<Pallet<T> as Store>::ConnectionsV2::insert(connection_id.as_bytes(), data);
		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), ICS03Error> {
		log::info!("in store connection to client");

		<Pallet<T> as Store>::ConnectionToClientV2::insert(connection_id.as_bytes(), client_id.as_bytes());
		Ok(())
	}
}
