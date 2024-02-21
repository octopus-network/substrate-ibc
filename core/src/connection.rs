use crate::{
	context::Context, Config, ConnectionClient, ConnectionCounter, Connections, OldHeight,
};
pub use alloc::{
	format,
	string::{String, ToString},
};
use frame_system::pallet_prelude::BlockNumberFor;
use ibc::{
	core::{
		ics02_client::{
			client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::ConnectionError,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::{
			identifier::{ClientId, ConnectionId},
			path::ConnectionsPath,
		},
	},
	Height,
};
use ibc_proto::google::protobuf::Any;
use sp_core::Get;
use sp_std::boxed::Box;

impl<T: Config> ConnectionReader for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ConnectionError> {
		<Connections<T>>::get(ConnectionsPath(conn_id.clone()))
			.ok_or(ConnectionError::ConnectionMismatch { connection_id: conn_id.clone() })
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ConnectionError> {
		ClientReader::client_state(self, client_id).map_err(ConnectionError::Client)
	}

	fn decode_client_state(
		&self,
		client_state: Any,
	) -> Result<Box<dyn ClientState>, ConnectionError> {
		ClientReader::decode_client_state(self, client_state).map_err(ConnectionError::Client)
	}

	fn host_current_height(&self) -> Result<Height, ConnectionError> {
		let current_height = <frame_system::Pallet<T>>::block_number();
		<OldHeight<T>>::put(u64::from(current_height));
		Height::new(T::ChainVersion::get(), u64::from(current_height))
			.map_err(ConnectionError::Client)
	}

	fn host_oldest_height(&self) -> Result<Height, ConnectionError> {
		let height = <OldHeight<T>>::get();
		Height::new(T::ChainVersion::get(), height).map_err(ConnectionError::Client)
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		CommitmentPrefix::try_from(T::IBC_COMMITMENT_PREFIX.to_vec()).unwrap_or_default()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ConnectionError> {
		// Forward method call to the Ics2Client-specific method.
		self.consensus_state(client_id, height).map_err(ConnectionError::Client)
	}

	fn host_consensus_state(
		&self,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ConnectionError> {
		ClientReader::host_consensus_state(self, height).map_err(ConnectionError::Client)
	}

	fn connection_counter(&self) -> Result<u64, ConnectionError> {
		Ok(<ConnectionCounter<T>>::get())
	}

	fn validate_self_client(&self, _counterparty_client_state: Any) -> Result<(), ConnectionError> {
		Ok(())
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: ConnectionEnd,
	) -> Result<(), ConnectionError> {
		<Connections<T>>::insert(ConnectionsPath(connection_id), connection_end);

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: ClientId,
	) -> Result<(), ConnectionError> {
		<ConnectionClient<T>>::insert(client_id, connection_id);

		Ok(())
	}

	fn increase_connection_counter(&mut self) {
		let _ = ConnectionCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}
}
