use crate::Config;
use sp_std::marker::PhantomData;

use ibc::{applications::transfer::MODULE_ID_STR, core::ics26_routing::context::RouterBuilder};
use pallet_ics20_transfer::ics20_callback::IbcTransferModule;

/// A struct capturing all the functional dependencies (i.e., context)
/// which the ICS26 module requires to be able to dispatch and process IBC messages.
use crate::module::core::ics26_routing::{Router, SubstrateRouterBuilder};
#[cfg(test)]
use ibc::{
	core::ics04_channel::commitment::PacketCommitment,
	core::ics04_channel::packet::Sequence,
	core::{
		ics02_client::{client_type::ClientType, context::ClientKeeper},
		ics04_channel::channel::ChannelEnd,
		ics24_host::identifier::{ChannelId, ClientId, PortId},
	},
	core::{ics03_connection::connection::ConnectionEnd, ics24_host::identifier::ConnectionId},
	mock::client_state::{client_type as mock_client_type, MockClientState},
	mock::host::HostBlock,
	Height,
};

#[derive(Clone, Debug)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: Router,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let r = SubstrateRouterBuilder::default()
			.add_route(MODULE_ID_STR.parse().unwrap(), IbcTransferModule(PhantomData::<T>)) // register transfer Module
			.unwrap()
			.build();

		Self { _pd: PhantomData::default(), router: r }
	}

	/// Associates a client record to this context.
	/// Given a client id and a height, registers a new client in the context and also associates
	/// to this client a mock client state and a mock consensus state for height `height`. The type
	/// of this client is implicitly assumed to be Mock.
	#[cfg(test)]
	pub fn with_client(self, client_id: &ClientId, height: Height) -> Self {
		self.with_client_parametrized(client_id, height, Some(mock_client_type()), Some(height))
	}

	/// Similar to `with_client`, this function associates a client record to this context, but
	/// additionally permits to parametrize two details of the client. If `client_type` is None,
	/// then the client will have type Mock, otherwise the specified type. If
	/// `consensus_state_height` is None, then the client will be initialized with a consensus
	/// state matching the same height as the client state (`client_state_height`).
	#[cfg(test)]
	pub fn with_client_parametrized(
		mut self,
		client_id: &ClientId,
		client_state_height: Height,
		client_type: Option<ClientType>,
		consensus_state_height: Option<Height>,
	) -> Self {
		use crate::module::core::ics24_host::TENDERMINT_CLIENT_TYPE;
		use ibc::{
			clients::ics07_tendermint::client_state::test_util::get_dummy_tendermint_client_state,
			core::{
				ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
				ics24_host::identifier::ChainId,
			},
			mock::{
				client_state::MOCK_CLIENT_TYPE, consensus_state::MockConsensusState,
				header::MockHeader,
			},
			timestamp::Timestamp,
		};

		let cs_height = consensus_state_height.unwrap_or(client_state_height);

		let client_type = client_type.unwrap_or_else(mock_client_type);
		let (client_state, consensus_state) = if client_type.as_str() == MOCK_CLIENT_TYPE {
			(
				Some(MockClientState::new(MockHeader::new(client_state_height)).into_box()),
				MockConsensusState::new(MockHeader::new(cs_height)).into_box(),
			)
		} else if client_type.as_str() == TENDERMINT_CLIENT_TYPE {
			let light_block = HostBlock::generate_tm_block(
				ChainId::default(),
				cs_height.revision_height(),
				Timestamp::now(),
			);

			let client_state =
				get_dummy_tendermint_client_state(light_block.header().clone()).into_box();

			// Return the tuple.
			(Some(client_state), light_block.into())
		} else {
			panic!("unknown client type")
		};

		self.store_client_type(client_id.clone(), client_type).unwrap();
		self.store_client_state(client_id.clone(), client_state.unwrap()).unwrap();
		self.store_consensus_state(client_id.clone(), cs_height, consensus_state)
			.unwrap();

		self
	}

	#[cfg(test)]
	/// Associates a connection to this context.
	pub fn with_connection(
		mut self,
		connection_id: ConnectionId,
		connection_end: ConnectionEnd,
	) -> Self {
		use ibc::core::ics03_connection::context::ConnectionKeeper;
		let _ = self.store_connection(connection_id, &connection_end);

		self
	}

	#[cfg(test)]
	/// Associates a channel (in an arbitrary state) to this context.
	pub fn with_channel(
		mut self,
		port_id: PortId,
		chan_id: ChannelId,
		channel_end: ChannelEnd,
	) -> Self {
		use ibc::core::ics04_channel::context::ChannelKeeper;
		let _ = self.store_channel(port_id, chan_id, channel_end);

		self
	}

	#[cfg(test)]
	pub fn with_packet_commitment(
		mut self,
		port_id: PortId,
		chan_id: ChannelId,
		seq: Sequence,
		data: PacketCommitment,
	) -> Self {
		use ibc::core::ics04_channel::context::ChannelKeeper;

		let _ = self.store_packet_commitment(port_id, chan_id, seq, data);

		self
	}

	#[cfg(test)]
	pub fn with_ack_sequence(
		mut self,
		port_id: PortId,
		chan_id: ChannelId,
		seq_number: Sequence,
	) -> Self {
		use ibc::core::ics04_channel::context::ChannelKeeper;

		let _ = self.store_next_sequence_ack(port_id, chan_id, seq_number);

		self
	}

	#[cfg(test)]
	pub fn with_send_sequence(
		mut self,
		port_id: PortId,
		chan_id: ChannelId,
		seq_number: Sequence,
	) -> Self {
		use ibc::core::ics04_channel::context::ChannelKeeper;

		let _ = self.store_next_sequence_ack(port_id, chan_id, seq_number);

		self
	}
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}
