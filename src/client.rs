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
		log::info!("In client: [client_type]");

		if <Clients<T>>::contains_key(client_id.as_bytes()) {
			let data = <Clients<T>>::get(client_id.as_bytes());
			let mut data: &[u8] = &data;
			let data = String::decode(&mut data).unwrap();
			log::info!("In client: [client_type] >> date: {} ", data);
			match ClientType::from_str(&data) {
				Err(_err) => None,
				Ok(val) => {
					log::info!("In client: [client_type] >> client_type : {}", val);
					Some(val)
				},
			}
		} else {
			log::info!("In client : [client_type] >> read client_type is None");

			None
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
		log::info!("In client: [client_state]");

		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			log::info!("In client: [client_state] >> client_state: {:?}", AnyClientState::decode_vec(&*data).unwrap());
			Some(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::info!("In client: [client_state] >> read client_state is None");

			None
		}
	}

	fn consensus_state(&self, client_id: &ClientId, height: Height) -> Option<AnyConsensusState> {
		log::info!("In client: [consensus_state]");

		let height = height.encode_vec().unwrap();
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				return Some(any_consensus_state);
			}
		}
		None
	}
	fn client_counter(&self) -> u64 {
		log::info!("In client: [client_counter]");
		log::info!("In client: [client_counter] >> client_counter: {:?}", <ClientCounter<T>>::get());

		<ClientCounter<T>>::get()
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {
		log::info!("In client: [store_client_type]");
		log::info!("In client: [store_client_type] >> client id: {}", client_id);
		log::info!("In client: [store_client_type] >> client type: {}", client_type.as_str());

		let client_id = client_id.as_bytes().to_vec();
		let client_type = client_type.as_str().encode();
		<Clients<T>>::insert(client_id, client_type);
		Ok(())
	}

	fn increase_client_counter(&mut self) {
		log::info!("In client: [increase_client_counter]");

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
		log::info!("In client: [store_client_state]");
		log::info!("In client: [store_client_state] >> client_id: {}", client_id);
		log::info!("In client: [store_client_state] >> client_state: {:?}", client_state);

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
		log::info!("In client: [store_consensus_state]");
		log::info!("In client: [store_consensus_state] >> client_id: {}", client_id);
		log::info!("In client: [store_consensus_state] >> height: {:?}", height);
		log::info!("In client: [store_consensus_state] >> consensus_state: {:?}", consensus_state);



		let height = height.encode_vec().unwrap();
		let data = consensus_state.encode_vec().unwrap();
		if <ConsensusStates<T>>::contains_key(client_id.as_bytes()) {
			// if consensus_state is no empty use push insert an exist ConsensusStates
			<ConsensusStates<T>>::try_mutate(client_id.as_bytes(),|val| -> Result<(), &'static str> {
				val.push((height, data));
				Ok(())
			}).expect("store consensus state error");

		} else {
			// if consensus state is empty insert a new item.
			<ConsensusStates<T>>::insert(client_id.as_bytes(), vec![(height, data)]);
		}
		Ok(())
	}
}
