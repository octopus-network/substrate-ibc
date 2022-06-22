use crate::{
	context::Context,
	utils::{host_height, LOG_TARGET},
	*,
};
use log::trace;

use ibc::{
	clients::ics10_grandpa::{consensus_state::ConsensusState as GPConsensusState, header::Header},
	core::{
		ics02_client::{client_consensus::AnyConsensusState, client_state::AnyClientState},
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as ICS03Error,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::identifier::{ClientId, ConnectionId},
	},
	Height,
};

/// A context supplying all the necessary read-only dependencies for processing an `ConnectionMsg`.
impl<T: Config> ConnectionReader for Context<T> {
	/// Returns the ConnectionEnd for the given identifier `connection_id`.
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ICS03Error> {
		trace!(
			target: LOG_TARGET,
			"in connection : [connection_end] >> connection_id = {:?}",
			conn_id
		);

		if <Connections<T>>::contains_key(conn_id.as_bytes()) {
			let encode_connection_end = <Connections<T>>::get(conn_id.as_bytes());
			let connection_end = ConnectionEnd::decode_vec(&*encode_connection_end)
				.map_err(ICS03Error::invalid_decode)?;
			trace!(
				target: LOG_TARGET,
				"in connection : [connection_end] >>  connection_end = {:?}",
				connection_end
			);
			Ok(connection_end)
		} else {
			error!(
				target: LOG_TARGET,
				"in connection : [connection_end] >> read connection end returns None"
			);
			Err(ICS03Error::connection_mismatch(conn_id.clone()))
		}
	}

	/// Returns the ClientState for the given identifier `client_id`.
	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS03Error> {
		trace!(target: LOG_TARGET, "in connection : [client_state]");

		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let encode_client_state = <ClientStates<T>>::get(client_id.as_bytes());
			let any_client_state = AnyClientState::decode_vec(&*encode_client_state)
				.map_err(ICS03Error::invalid_decode)?;

			trace!(
				target: LOG_TARGET,
				"in connection : [client_state] >> client_state: {:?}",
				any_client_state
			);
			Ok(any_client_state)
		} else {
			error!(
				target: LOG_TARGET,
				"in connection : [client_state] >> read client_state is None"
			);
			Err(ICS03Error::frozen_client(client_id.clone()))
		}
	}

	/// Returns the current height of the local chain.
	fn host_current_height(&self) -> Height {
		trace!(target: LOG_TARGET, "in connection : [host_current_height]");

		let revision_height = host_height::<T>();

		trace!(
			target: LOG_TARGET,
			"in connection : [host_current_height] >> Host revision_height = {:?}",
			revision_height
		);
		let revision_number = 0; // TODO: in the future
		Height::new(revision_number, revision_height)
	}

	/// Returns the oldest height available on the local chain.
	fn host_oldest_height(&self) -> Height {
		trace!(target: LOG_TARGET, "in connection : [host_oldest_height]");

		let mut temp = frame_system::BlockHash::<T>::iter().collect::<Vec<_>>();
		temp.sort_by(|(a, ..), (b, ..)| a.cmp(b));
		let (block_number, ..) = temp.get(0).cloned().unwrap_or_default();
		let block_number = format!("{:?}", block_number);
		let revision_height = block_number.parse().unwrap_or_default();
		let revision_number = 0; //TODO: may be in the future to fix
		trace!(
			"in connection : [host_oldest_height] >> Host oldest height = {:?}",
			revision_height
		);
		Height::new(revision_number, revision_height)
	}

	/// Returns the prefix that local chain uses in the KV store.
	fn commitment_prefix(&self) -> CommitmentPrefix {
		trace!(target: LOG_TARGET, "in connection : [commitment_prefix]");

		// If this conversion fails it means the runtime was not configured well
		T::CONNECTION_PREFIX
			.to_vec()
			.try_into()
			.map_err(|_| panic!("Connection prefix supplied in pallet runtime config is invalid"))
			.unwrap()
	}

	/// Returns the ConsensusState that the given client stores at a specific height.
	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS03Error> {
		trace!(
			target: LOG_TARGET,
			"in connection : [client_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let height = height.encode_vec().map_err(ICS03Error::invalid_encode)?;
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state =
					AnyConsensusState::decode_vec(&*item.1).map_err(ICS03Error::invalid_decode)?;
				return Ok(any_consensus_state)
			}
		}

		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	/// Returns the ConsensusState of the host (local) chain at a specific height.
	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, ICS03Error> {
		trace!(
			target: LOG_TARGET,
			"in connection : [host_consensus_state] >> height = {:?}",
			_height
		);
		let any_consensus_state =
			AnyConsensusState::Grandpa(GPConsensusState::from(Header::default()));

		trace!(
			target: LOG_TARGET,
			"in connection : [host_consensus_state] >> any_consensus_state = {:?}",
			any_consensus_state
		);
		Ok(any_consensus_state)
	}

	/// Returns a counter on how many connections have been created thus far.
	/// The value of this counter should increase only via method
	/// `ConnectionKeeper::increase_connection_counter`.
	fn connection_counter(&self) -> Result<u64, ICS03Error> {
		trace!(target: LOG_TARGET, "in connection : [connection_counter]");

		Ok(<ConnectionCounter<T>>::get())
	}
}

/// A context supplying all the necessary write-only dependencies (i.e, storage writing for
/// facility) for processing any `ConnectionMsg`.
impl<T: Config> ConnectionKeeper for Context<T> {
	/// Stores the given connection_end at a path associated with the connection_id.
	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), ICS03Error> {
		trace!(
			target: LOG_TARGET,
			"in connection : [store_connection] >> connection_id = {:?}, connection_end = {:?}",
			connection_id,
			connection_end
		);

		let encode_connection_id = connection_id.as_bytes();
		let encode_connection_end =
			connection_end.encode_vec().map_err(ICS03Error::invalid_encode)?;

		<Connections<T>>::insert(encode_connection_id, encode_connection_end);

		Ok(())
	}

	/// Stores the given connection_id at a path associated with the client_id.
	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), ICS03Error> {
		trace!(
			target:LOG_TARGET,
			"in connection : [store_connection_to_client] >> connection_id = {:?}, client_id = {:?}",
			connection_id, client_id
		);

		let encode_connection_id = connection_id.as_bytes().to_vec();
		let encode_client_id = client_id.as_bytes().to_vec();

		<ConnectionClient<T>>::try_mutate::<_, _, ICS03Error, _>(encode_client_id, |value| {
			value.push(encode_connection_id);
			Ok(())
		})
	}

	/// Called upon connection identifier creation (Init or Try process).
	/// Increases the counter which keeps track of how many connections have been
	/// created.
	/// Should never fail.
	fn increase_connection_counter(&mut self) {
		trace!(target: LOG_TARGET, "in connection : [increase_connection_counter]");

		<ConnectionCounter<T>>::try_mutate(|val| -> Result<(), ICS03Error> {
			let new = val
				.checked_add(1)
				.ok_or_else(ICS03Error::invalid_increment_connection_counter)?;
			*val = new;
			Ok(())
		})
		.expect("increase_connection_counter error");
	}
}
