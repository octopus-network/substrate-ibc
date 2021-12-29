use super::*;

use crate::routing::Context;
use ibc::{
	ics02_client::{client_consensus::AnyConsensusState, client_state::AnyClientState},
	ics03_connection::{
		connection::ConnectionEnd,
		context::{ConnectionKeeper, ConnectionReader},
		error::Error as ICS03Error,
	},
	ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
	ics23_commitment::commitment::{CommitmentPrefix, CommitmentRoot},
	ics24_host::identifier::{ClientId, ConnectionId},
	Height,
};
use tendermint_proto::Protobuf;

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ICS03Error> {
		log::info!("in connection : [connection_end]");

		if <Connections<T>>::contains_key(conn_id.as_bytes()) {
			let data = <Connections<T>>::get(conn_id.as_bytes());
			let ret = ConnectionEnd::decode_vec(&*data).unwrap();
			log::info!("In connection: [connection_end] >>  {:?}", ret);
			Ok(ret)
		} else {
			log::info!("read connection end returns None");
			Err(ICS03Error::connection_mismatch(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS03Error> {
		log::info!("in connection : [client_state]");

		// ClientReader::client_state(self, client_id)
		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			log::info!(
				"In client: [client_state] >> client_state: {:?}",
				AnyClientState::decode_vec(&*data).unwrap()
			);
			Ok(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::info!("In client: [client_state] >> read client_state is None");

			todo!()
		}
	}

	fn host_current_height(&self) -> Height {
		log::info!("in connection: [host_current_height]");

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		// let current_height = block_number;

		<OldHeight<T>>::put(current_height);

		Height::new(0, current_height)
	}

	fn host_oldest_height(&self) -> Height {
		log::info!("In connection: [host_oldest_height]");

		let height = <OldHeight<T>>::get();

		Height::new(0, height)
	}

	fn connection_counter(&self) -> Result<u64, ICS03Error> {
		log::info!("in connection: [connection_counter]");

		Ok(<ConnectionCounter<T>>::get())
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		log::info!("in connection: [commitment_prefix]");

		"ibc".as_bytes().to_vec().into()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS03Error> {
		log::info!("in connection: [client_consensus_state]");

		// ClientReader::consensus_state(self, client_id, height)
		let height = height.encode_vec().unwrap();
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				return Ok(any_consensus_state)
			}
		}
		todo!()
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, ICS03Error> {
		log::info!("in connection: [host_consensus_state]");
		Ok(AnyConsensusState::Grandpa(GPConsensusState::new(CommitmentRoot::from(vec![1, 2, 3]))))
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn increase_connection_counter(&mut self) {
		log::info!("In connection: [increase_connection_counter]");

		<ConnectionCounter<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let new = val.checked_add(1).ok_or("Add connection counter error")?;
			*val = new;
			Ok(())
		})
		.expect("increase connection counter error");
	}

	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), ICS03Error> {
		log::info!("in connection: [store_connection]");
		log::info!(
			"in connection: [store_connection] >> connection_id: {:?}, connection_end: {:?}",
			connection_id,
			connection_end.clone()
		);

		let data = connection_end.encode_vec().unwrap();
		<Connections<T>>::insert(connection_id.as_bytes().to_vec(), data);

		<ConnectionsKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			val.push(connection_id.as_bytes().to_vec());
			Ok(())
		}).expect("store connections keys error");

		let temp = ConnectionReader::connection_end(self, &connection_id);
		log::info!("in connection: [store_connection] >> read store after: {:?}", temp);
		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), ICS03Error> {
		log::info!("in connection: [store_connection_to_client]");

		<ConnectionClient<T>>::insert(client_id.as_bytes().to_vec(), connection_id.as_bytes().to_vec());
		Ok(())
	}
}
