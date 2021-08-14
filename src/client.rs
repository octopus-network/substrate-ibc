use super::*;

use crate::routing::Context;
use ibc::ics02_client::client_consensus::AnyConsensusState;
use ibc::ics02_client::client_state::AnyClientState;
use ibc::ics02_client::client_type::ClientType;
use ibc::ics02_client::context::{ClientKeeper, ClientReader};
use ibc::ics02_client::error::Error as ICS02Error;
use ibc::ics24_host::identifier::ClientId;
use ibc::Height;
use tendermint_proto::Protobuf;

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {
		log::info!("in read client_type");

		if <Clients<T>>::contains_key(client_id.as_bytes()) {
			let data = <Clients<T>>::get(client_id.as_bytes());
			let mut data: &[u8] = &data;
			let data = String::decode(&mut data).unwrap();
			log::info!("read client type data: {} ", data);
			match ClientType::from_str(&data) {
				Err(_err) => None,
				Ok(val) => {
					log::info!("client type is {}", val);
					Some(val)
				},
			}
		} else {
			log::info!("read client type returns None");

			None
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		log::info!("in read client_state");

		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			Some(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::info!("read client_state returns None");

			None
		}
	}

	fn consensus_state(&self, client_id: &ClientId, height: Height) -> Option<AnyConsensusState> {
		log::info!("in read consensus_state");

		let height = height.encode_vec().unwrap();
		if <Pallet<T> as Store>::ConsensusStates::contains_key((client_id.as_bytes(), &height)) {
			let data = <ConsensusStates<T>>::get((client_id.as_bytes(), height));
			Some(AnyConsensusState::decode_vec(&*data).unwrap())
		} else {
			log::info!("read consensus_state returns None");

			None
		}
	}
	fn client_counter(&self) -> u64 {
		log::info!("in read client counter");

		<ClientCounter<T>>::get()
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {
		log::info!("in store_client_type");
		log::info!("client id: {}", client_id);
		log::info!("client type: {}", client_type.as_str());

		let client_id = client_id.as_bytes().to_vec();
		let client_type = client_type.as_str().encode();
		<Clients<T>>::insert(client_id, client_type);
		Ok(())
	}

	fn increase_client_counter(&mut self) {
		log::info!("in increase client counter");

		<ClientCounter<T>>::try_mutate(|val| -> Result<(), &'static str> {
			let new = val.checked_add(1).ok_or("Add client counter error")?;
			*val = new;
			Ok(())
		})
		.expect("increase client counter error");
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: AnyClientState,
	) -> Result<(), ICS02Error> {
		log::info!("in store_client_state");

		let data = client_state.encode_vec().unwrap();
		<ClientStates<T>>::insert(client_id.as_bytes(), data);
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
		<ConsensusStates<T>>::insert((client_id.as_bytes(), height), data);
		Ok(())
	}
}
