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
		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Error> {
		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), Error> {
		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack: Vec<u8>,
	) -> Result<(), Error> {
		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), Error> {
		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), Error> {
		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), Error> {
		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Error> {
		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Error> {
		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), Error> {
		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		unimplemented!()
	}
}

impl<T: Config> ChannelReader for Context<T> {
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Option<ChannelEnd> {
		None
	}

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(&self, connection_id: &ConnectionId) -> Option<ConnectionEnd> {
		None
	}

	fn connection_channels(&self, cid: &ConnectionId) -> Option<Vec<(PortId, ChannelId)>> {
		None
	}

	/// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
	/// proof verification.
	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		None
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Option<AnyConsensusState> {
		None
	}

	fn authenticated_capability(&self, port_id: &PortId) -> Result<Capability, Error> {
		unimplemented!()
	}

	fn get_next_sequence_send(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		None
	}

	fn get_next_sequence_recv(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		None
	}

	fn get_next_sequence_ack(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
		None
	}

	fn get_packet_commitment(&self, key: &(PortId, ChannelId, Sequence)) -> Option<String> {
		None
	}

	fn get_packet_receipt(&self, key: &(PortId, ChannelId, Sequence)) -> Option<Receipt> {
		None
	}

	fn get_packet_acknowledgement(&self, key: &(PortId, ChannelId, Sequence)) -> Option<String> {
		None
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: String) -> String {
		String::new()
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
		0
	}
}

impl<T: Config> PortReader for Context<T> {
	fn lookup_module_by_port(&self, port_id: &PortId) -> Option<Capability> {
		None
	}
	fn authenticate(&self, key: &Capability, port_id: &PortId) -> bool {
		false
	}
}

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {

		log::info!("in read client_type");

		Some(ClientType::Tendermint)
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
		0
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {

		log::info!("in store_client_type");

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		unimplemented!()
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
		None
	}

	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		None
	}

	fn host_current_height(&self) -> Height {
		Height::zero()
	}

	fn host_oldest_height(&self) -> Height {
		Height::zero()
	}

	fn connection_counter(&self) -> u64 {
		0
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		vec![0].into()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Option<AnyConsensusState> {
		None
	}

	fn host_consensus_state(&self, height: Height) -> Option<AnyConsensusState> {
		None
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn increase_connection_counter(&mut self) {
		unimplemented!()
		// ConnectionId::from_str("todo").unwrap()
	}

	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), ICS03Error> {
		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), ICS03Error> {
		Ok(())
	}
}
