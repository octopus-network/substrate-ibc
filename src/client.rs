use super::*;
use alloc::string::ToString;
use core::str::FromStr;

use crate::routing::Context;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_state::AnyClientState,
			client_type::ClientType,
			context::{ClientKeeper, ClientReader},
			error::Error as ICS02Error,
		},
		ics24_host::identifier::ClientId,
	},
	timestamp::Timestamp,
	Height,
};

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ICS02Error> {
		log::trace!("in client : [client_type] >> client_id = {:?}", client_id);

		if <Clients<T>>::contains_key(client_id.as_bytes()) {
			let data = <Clients<T>>::get(client_id.as_bytes());
			let mut data: &[u8] = &data;
			let data = Vec::<u8>::decode(&mut data).unwrap();
			let data = String::from_utf8(data).unwrap();
			match ClientType::from_str(&data) {
				Err(_err) => Err(ICS02Error::unknown_client_type(data.to_string())),
				Ok(val) => {
					log::trace!("in client : [client_type] >> client_type : {:?}", val);
					Ok(val)
				},
			}
		} else {
			log::trace!("in client : [client_type] >> read client_type is None");
			Err(ICS02Error::client_not_found(client_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS02Error> {
		log::trace!("in client : [client_state] >> client_id = {:?}", client_id);

		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			log::trace!(
				"in client : [client_state] >> any client_state: {:?}",
				AnyClientState::decode_vec(&*data).unwrap()
			);
			Ok(AnyClientState::decode_vec(&*data).unwrap())
		} else {
			log::trace!("in client : [client_state] >> read any client state is None");
			Err(ICS02Error::client_not_found(client_id.clone()))
		}
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS02Error> {
		log::trace!(
			"in client : [consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let mut values = <ConsensusStates<T>>::get(client_id.as_bytes());
		values.sort_by(|(height_left, _), (height_right, _)| {
			let height_left = Height::decode(&height_left[..]).unwrap();
			let height_right = Height::decode(&height_right[..]).unwrap();
			height_left.cmp(&height_right)
		});

		for item in values.iter() {
			let item_height = Height::decode(&item.0.clone()[..]).unwrap();

			if item_height == height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				log::trace!(
					"in client : [consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(any_consensus_state)
			}
		}

		Err(ICS02Error::consensus_state_not_found(client_id.clone(), height))
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, ICS02Error> {
		log::trace!(
			"in client : [next_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let mut values = <ConsensusStates<T>>::get(client_id.as_bytes());
		values.sort_by(|(height_left, _), (height_right, _)| {
			let height_left = Height::decode(&height_left[..]).unwrap();
			let height_right = Height::decode(&height_right[..]).unwrap();
			height_left.cmp(&height_right)
		});

		for item in values.iter() {
			let item_height = Height::decode(&item.0.clone()[..]).unwrap();

			if item_height < height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				log::trace!(
					"in client : [consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(Some(any_consensus_state))
			}
		}

		Ok(Some(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		)))
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, ICS02Error> {
		log::trace!(
			"in client : [next_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let mut values = <ConsensusStates<T>>::get(client_id.as_bytes());
		values.sort_by(|(height_left, _), (height_right, _)| {
			let height_left = Height::decode(&height_left[..]).unwrap();
			let height_right = Height::decode(&height_right[..]).unwrap();
			height_left.cmp(&height_right)
		});

		for item in values.iter() {
			let item_height = Height::decode(&item.0.clone()[..]).unwrap();

			if item_height > height {
				let any_consensus_state = AnyConsensusState::decode_vec(&*item.1).unwrap();
				log::trace!(
					"in client : [consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(Some(any_consensus_state))
			}
		}

		Ok(Some(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		)))
	}

	fn host_height(&self) -> Height {
		log::trace!("in client : [host_height]");
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height = block_number
			.parse()
			.map_err(|e| panic!("{:?}, caused by {:?} from frame_system::Pallet", e, block_number));
		log::trace!(
			"in channel: [host_height] >> host_height = {:?}",
			Height::new(0, current_height.unwrap())
		);
		Height::new(0, current_height.unwrap())
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, ICS02Error> {
		log::trace!("in client : [consensus_state]");

		// TODO
		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, ICS02Error> {
		log::trace!("in client: [pending_host_consensus_state]");
		// TODO
		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	fn client_counter(&self) -> Result<u64, ICS02Error> {
		log::trace!(
			"in client : [client_counter] >> client_counter: {:?}",
			<ClientCounter<T>>::get()
		);

		Ok(<ClientCounter<T>>::get())
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {
		log::info!(
			"in client : [store_client_type] >> client id = {:?}, client_type = {:?}",
			client_id,
			client_type
		);

		let client_id = client_id.as_bytes().to_vec();
		let client_type = client_type.as_str().encode();
		<Clients<T>>::insert(client_id, client_type);
		Ok(())
	}

	fn increase_client_counter(&mut self) {
		log::info!("in client : [increase_client_counter]");

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
		log::trace!(
			"in client : [store_client_state] >> client_id: {:?}, client_state = {:?}",
			client_id,
			client_state
		);

		let data = client_state.encode_vec().unwrap();
		// store client states key-value
		<ClientStates<T>>::insert(client_id.as_bytes().to_vec(), data);

		// store client states keys
		<ClientStatesKeys<T>>::try_mutate(|val| -> Result<(), &'static str> {
			if let Some(_value) = val.iter().find(|&x| x == client_id.as_bytes()) {
			} else {
				val.push(client_id.as_bytes().to_vec());
			}
			Ok(())
		})
		.expect("store client_state keys error");

		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: AnyConsensusState,
	) -> Result<(), ICS02Error> {
		log::trace!("in client : [store_consensus_state] >> client_id: {:?}, height = {:?}, consensus_state = {:?}",
			client_id, height, consensus_state);

		let height = height.encode_vec().unwrap();
		let data = consensus_state.encode_vec().unwrap();
		if <ConsensusStates<T>>::contains_key(client_id.as_bytes()) {
			// consensus_state is stored after mmr root updated
		} else {
			// if consensus state is empty insert a new item.
			<ConsensusStates<T>>::insert(client_id.as_bytes(), vec![(height, data)]);
		}
		Ok(())
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ICS02Error> {
		log::trace!(
			"in client: [store_update_time] >> client_id: {:?}, height: {:?}, timestamp: {:?}",
			client_id,
			height,
			timestamp
		);

		let encode_timestamp = serde_json::to_string(&timestamp).unwrap().as_bytes().to_vec();
		<ClientProcessedTimes<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().unwrap(),
			encode_timestamp,
		);

		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ICS02Error> {
		log::trace!(
			"in client: [store_update_height] >> client_id: {:?}, height: {:?}, host_height: {:?}",
			client_id,
			height,
			host_height
		);

		<ClientProcessedHeights<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().unwrap(),
			host_height.encode_vec().unwrap(),
		);

		Ok(())
	}
}
