use crate::{
	context::Context,
	module::core::ics24_host::TENDERMINT_TYPE,
	prelude::{format, String, ToString},
	ClientCounter, ClientProcessedHeights, ClientProcessedTimes, ClientStates, Clients, Config,
	ConsensusStates, REVISION_NUMBER,
};
use sp_std::{boxed::Box, vec::Vec};

use ibc::{
	clients::ics07_tendermint::{
		client_state::ClientState as Ics07ClientState,
		consensus_state::ConsensusState as Ics07ConsensusState,
	},
	core::{
		ics02_client::{
			client_state::ClientState,
			client_type::ClientType,
			consensus_state::ConsensusState,
			context::{ClientKeeper, ClientReader},
			error::Error as Ics02Error,
		},
		ics24_host::{
			identifier::ClientId,
			path::{ClientConsensusStatePath, ClientStatePath, ClientTypePath},
		},
	},
	timestamp::Timestamp,
	Height,
};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Ics02Error> {
		let client_type_path = ClientTypePath(client_id.clone()).to_string().as_bytes().to_vec();
		if <Clients<T>>::contains_key(client_type_path.clone()) {
			let data = <Clients<T>>::get(client_type_path);
			let data =
				String::from_utf8(data).map_err(|_| Ics02Error::implementation_specific())?;
			match data.as_str() {
				"07-tendermint" => Ok(ClientType::new(TENDERMINT_TYPE)),
				unimplemented =>
					return Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
			}
		} else {
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics02Error> {
		let client_state_path = ClientStatePath(client_id.clone()).to_string().as_bytes().to_vec();
		if <ClientStates<T>>::contains_key(&client_state_path) {
			let data = <ClientStates<T>>::get(&client_state_path);
			match self.client_type(client_id)?.as_str() {
				"07-tendermint" => {
					// TODO(davirain): need to make sure whether this is written correctly.
					let result: Ics07ClientState = Protobuf::<Any>::decode_vec(&data)
						.map_err(|_| Ics02Error::implementation_specific())?;
					return Ok(Box::new(result))
				},
				unimplemented =>
					return Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
			}
		} else {
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics02Error> {
		if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
			Ok(client_state.into_box())
		} else {
			Err(Ics02Error::unknown_client_state_type(client_state.type_url))
		}
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Box<dyn ConsensusState>, Ics02Error> {
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		if <ConsensusStates<T>>::contains_key(client_consensus_state_path.clone()) {
			let data = <ConsensusStates<T>>::get(client_consensus_state_path);
			match self.client_type(client_id)?.as_str() {
				"07-terdermint" => {
					// TODO(davirain): need to make sure whether this is written correctly.
					let result: Ics07ConsensusState = Protobuf::<Any>::decode_vec(&data)
						.map_err(|_| Ics02Error::implementation_specific())?;
					return Ok(Box::new(result))
				},
				unimplemented =>
					return Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
			}
		} else {
			Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
		}
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<Box<dyn ConsensusState>>, Ics02Error> {
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		let client_consensus_state_key =
			<ConsensusStates<T>>::iter_keys().collect::<Vec<Vec<u8>>>();
		let mut heights = client_consensus_state_key
			.into_iter()
			.map(|value| {
				let value = String::from_utf8(value)
					.expect("hex-encoded string should always be valid UTF-8");
				let client_consensus_state_path = value.rsplit_once('/').expect("Never failed");
				let (epoch, height) =
					client_consensus_state_path.1.split_once('-').expect("never Failed");
				let epoch = epoch.parse::<u64>().expect("Never failed");
				let height = height.parse::<u64>().expect("Never failed");
				Height::new(epoch, height).expect("Never failed")
			})
			.collect::<Vec<Height>>();

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h > height {
				let data = <ConsensusStates<T>>::get(&client_consensus_state_path);
				match self.client_type(client_id)?.as_str() {
					"07-terdermint" => {
						// TODO(davirain): need to make sure whether this is written correctly.
						let result: Ics07ConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|_| Ics02Error::implementation_specific())?;
						return Ok(Some(Box::new(result)))
					},
					_ => {},
				}
			}
		}
		Ok(None)
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<Box<dyn ConsensusState>>, Ics02Error> {
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		let client_consensus_state_key =
			<ConsensusStates<T>>::iter_keys().collect::<Vec<Vec<u8>>>();
		let mut heights = client_consensus_state_key
			.into_iter()
			.map(|value| {
				let value = String::from_utf8(value)
					.expect("hex-encoded string should always be valid UTF-8");
				let client_consensus_state_path = value.rsplit_once('/').expect("Never failed");
				let (epoch, height) =
					client_consensus_state_path.1.split_once('-').expect("never Failed");
				let epoch = epoch.parse::<u64>().expect("Never failed");
				let height = height.parse::<u64>().expect("Never failed");
				Height::new(epoch, height).expect("Never failed")
			})
			.collect::<Vec<Height>>();

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h < height {
				let data = <ConsensusStates<T>>::get(&client_consensus_state_path);
				match self.client_type(client_id)?.as_str() {
					"07-tendermint" => {
						// TODO(davirain): need to make sure whether this is written correctly.
						let result: Ics07ConsensusState = ibc_proto::protobuf::Protobuf::<
							ibc_proto::google::protobuf::Any,
						>::decode_vec(&data)
						.map_err(|_| Ics02Error::implementation_specific())?;
						return Ok(Some(Box::new(result)))
					},

					_ => {},
				}
			}
		}
		Ok(None)
	}

	fn host_height(&self) -> Height {
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		Height::new(REVISION_NUMBER, current_height).unwrap()
	}

	fn host_consensus_state(&self, _height: Height) -> Result<Box<dyn ConsensusState>, Ics02Error> {
		Err(Ics02Error::implementation_specific())
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics02Error> {
		Err(Ics02Error::implementation_specific())
	}

	fn client_counter(&self) -> Result<u64, Ics02Error> {
		Ok(<ClientCounter<T>>::get())
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), Ics02Error> {
		let client_type_path = ClientTypePath(client_id).to_string().as_bytes().to_vec();
		let client_type = client_type.as_str().as_bytes().to_vec();
		<Clients<T>>::insert(client_type_path, client_type);
		Ok(())
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: Box<dyn ClientState>,
	) -> Result<(), Ics02Error> {
		let client_state_path = ClientStatePath(client_id).to_string().as_bytes().to_vec();

		let data = client_state.encode_vec().map_err(|_| Ics02Error::implementation_specific())?;

		<ClientStates<T>>::insert(client_state_path, data);
		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: Box<dyn ConsensusState>,
	) -> Result<(), Ics02Error> {
		let client_consensus_state_path = ClientConsensusStatePath {
			client_id,
			epoch: height.revision_number(),
			height: height.revision_height(),
		}
		.to_string()
		.as_bytes()
		.to_vec();

		let consensus_state = consensus_state
			.encode_vec()
			.map_err(|_| Ics02Error::implementation_specific())?;

		<ConsensusStates<T>>::insert(client_consensus_state_path, consensus_state);

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		let _ = <ClientCounter<T>>::try_mutate(|val| -> Result<(), Ics02Error> {
			let new = val.checked_add(1).expect("increase client coubter overflow");
			*val = new;
			Ok(())
		});
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), Ics02Error> {
		let encode_timestamp = serde_json::to_string(&timestamp)
			.map_err(|_| Ics02Error::implementation_specific())?
			.as_bytes()
			.to_vec();

		<ClientProcessedTimes<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics02Error::implementation_specific())?,
			encode_timestamp,
		);

		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), Ics02Error> {
		<ClientProcessedHeights<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().map_err(|_| Ics02Error::implementation_specific())?,
			host_height.encode_vec().map_err(|_| Ics02Error::implementation_specific())?,
		);

		Ok(())
	}
}
