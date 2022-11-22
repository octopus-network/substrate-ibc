use crate::{
	context::Context,
	host::TENDERMINT_CLIENT_TYPE,
	prelude::{format, String, ToString},
	ClientCounter, ClientProcessedHeights, ClientProcessedTimes, ClientStates, Clients, Config,
	ConsensusStates, REVISION_NUMBER,
};
use sp_std::{boxed::Box, vec::Vec};

#[cfg(test)]
use crate::host::MOCK_CLIENT_TYPE;
use frame_support::traits::UnixTime;
#[cfg(test)]
use ibc::mock::{
	client_state::MockClientState, consensus_state::MockConsensusState, header::MockHeader,
};
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
			let data = String::from_utf8(data)
				.map_err(|e| Ics02Error::other(format!("Decode ClientType Failed: {:?}", e)))?;
			match data.as_str() {
				TENDERMINT_CLIENT_TYPE => Ok(ClientType::new(TENDERMINT_CLIENT_TYPE.into())),
				#[cfg(test)]
				MOCK_CLIENT_TYPE => Ok(ClientType::new(MOCK_CLIENT_TYPE.into())),
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
				TENDERMINT_CLIENT_TYPE => {
					let result: Ics07ClientState =
						Protobuf::<Any>::decode_vec(&data).map_err(|e| {
							Ics02Error::other(format!("Decode Ics07ClientState failed: {:?}", e))
						})?;

					Ok(Box::new(result))
				},
				#[cfg(test)]
				MOCK_CLIENT_TYPE => {
					let result: MockClientState =
						Protobuf::<Any>::decode_vec(&data).map_err(|e| {
							Ics02Error::other(format!("Deocode Ics10ClientState failed: {:?}", e))
						})?;
					Ok(Box::new(result))
				},
				unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
			}
		} else {
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics02Error> {
		if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
			return Ok(client_state.into_box())
		}
		#[cfg(test)]
		if let Ok(client_state) = MockClientState::try_from(client_state.clone()) {
			return Ok(client_state.into_box())
		}
		Err(Ics02Error::unknown_client_state_type(client_state.type_url))
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
				TENDERMINT_CLIENT_TYPE => {
					let result: Ics07ConsensusState =
						Protobuf::<Any>::decode_vec(&data).map_err(|e| {
							Ics02Error::other(format!("Decode Ics07ConsensusState failed: {:?}", e))
						})?;
					Ok(Box::new(result))
				},
				#[cfg(test)]
				MOCK_CLIENT_TYPE => {
					let result: MockConsensusState =
						Protobuf::<Any>::decode_vec(&data).map_err(|e| {
							Ics02Error::other(format!("Decode MockConsensusState failed: {:?}", e))
						})?;
					Ok(Box::new(result))
				},
				unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
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
				let value = String::from_utf8(value).map_err(|e| {
					Ics02Error::other(format!(
						"hex-encoded string should always be valid UTF-8: {:?}",
						e
					))
				})?;
				let client_consensus_state_path = value.rsplit_once('/').ok_or(
					Ics02Error::other(format!("split client consensus state path failed")),
				)?;
				let (epoch, height) = client_consensus_state_path
					.1
					.split_once('-')
					.ok_or(Ics02Error::other(format!("split height failed")))?;
				let epoch = epoch.parse::<u64>().map_err(|e| {
					Ics02Error::other(format!("parse epoch number failed: {:?}", e))
				})?;
				let height = height
					.parse::<u64>()
					.map_err(|e| Ics02Error::other(format!("parse height failed: {:?}", e)))?;
				Height::new(epoch, height)
					.map_err(|e| Ics02Error::other(format!("contruct IBC height failed: {:?}", e)))
			})
			.collect::<Result<Vec<Height>, Ics02Error>>()?;

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h > height {
				let data = <ConsensusStates<T>>::get(&client_consensus_state_path);
				match self.client_type(client_id)?.as_str() {
					TENDERMINT_CLIENT_TYPE => {
						let result: Ics07ConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| {
								Ics02Error::other(format!(
									"Decode Ics07ConsensusState failed: {:?}",
									e
								))
							})?;
						return Ok(Some(Box::new(result)))
					},
					#[cfg(test)]
					MOCK_CLIENT_TYPE => {
						let result: MockConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| {
								Ics02Error::other(format!(
									"Decode MockConsensusState failed: {:?}",
									e
								))
							})?;
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
				let value = String::from_utf8(value).map_err(|e| {
					Ics02Error::other(format!(
						"hex-encoded string should always be valid UTF-8: {:?}",
						e
					))
				})?;
				let client_consensus_state_path = value.rsplit_once('/').ok_or(
					Ics02Error::other(format!("split client consensus state path failed")),
				)?;
				let (epoch, height) = client_consensus_state_path
					.1
					.split_once('-')
					.ok_or(Ics02Error::other(format!("split height failed")))?;
				let epoch = epoch.parse::<u64>().map_err(|e| {
					Ics02Error::other(format!("parse epoch number failed: {:?}", e))
				})?;
				let height = height
					.parse::<u64>()
					.map_err(|e| Ics02Error::other(format!("parse height failed: {:?}", e)))?;
				Height::new(epoch, height)
					.map_err(|e| Ics02Error::other(format!("contruct IBC height failed: {:?}", e)))
			})
			.collect::<Result<Vec<Height>, Ics02Error>>()?;

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h < height {
				let data = <ConsensusStates<T>>::get(&client_consensus_state_path);
				match self.client_type(client_id)?.as_str() {
					TENDERMINT_CLIENT_TYPE => {
						let result: Ics07ConsensusState = ibc_proto::protobuf::Protobuf::<
							ibc_proto::google::protobuf::Any,
						>::decode_vec(&data)
						.map_err(|e| {
							Ics02Error::other(format!("Decode Ics07ConsensusState failed: {:?}", e))
						})?;
						return Ok(Some(Box::new(result)))
					},
					#[cfg(test)]
					MOCK_CLIENT_TYPE => {
						let result: MockConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| {
								Ics02Error::other(format!(
									"Decode MockConsensusState failed: {:?}",
									e
								))
							})?;
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

	fn host_timestamp(&self) -> Timestamp {
		#[cfg(not(test))]
		{
			let nanoseconds = <T as Config>::TimeProvider::now().as_nanos();
			return Timestamp::from_nanoseconds(nanoseconds as u64).unwrap()
		}
		#[cfg(test)]
		{
			Timestamp::now()
		}
	}

	fn host_consensus_state(&self, _height: Height) -> Result<Box<dyn ConsensusState>, Ics02Error> {
		#[cfg(not(test))]
		{
			Err(Ics02Error::implementation_specific())
		}
		#[cfg(test)]
		{
			let mock_header =
				MockHeader { height: self.host_height(), timestamp: Default::default() };
			Ok(Box::new(MockConsensusState::new(mock_header)))
		}
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics02Error> {
		#[cfg(not(test))]
		{
			Err(Ics02Error::implementation_specific())
		}
		#[cfg(test)]
		{
			let mock_header =
				MockHeader { height: self.host_height(), timestamp: Default::default() };
			Ok(Box::new(MockConsensusState::new(mock_header)))
		}
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

		let data = client_state
			.encode_vec()
			.map_err(|e| Ics02Error::other(format!("Encode ClientState Failed: {:?}", e)))?;

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
			.map_err(|e| Ics02Error::other(format!("Encode ConsensusStates failed: {:?}", e)))?;

		<ConsensusStates<T>>::insert(client_consensus_state_path, consensus_state);

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		let _ = <ClientCounter<T>>::try_mutate(|val| -> Result<(), Ics02Error> {
			let new = val
				.checked_add(1)
				.ok_or(Ics02Error::other(format!("increase client coubter overflow")))?;
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
			.map_err(|e| Ics02Error::other(format!("Encode timestamp failed: {:?}", e)))?
			.as_bytes()
			.to_vec();

		<ClientProcessedTimes<T>>::insert(
			client_id.as_bytes(),
			height
				.encode_vec()
				.map_err(|e| Ics02Error::other(format!("Encode Height failed: {:?}", e)))?,
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
			height
				.encode_vec()
				.map_err(|e| Ics02Error::other(format!("Encode height failed: {:?}", e)))?,
			host_height
				.encode_vec()
				.map_err(|e| Ics02Error::other(format!("Encode Host height failed: {:?}", e)))?,
		);

		Ok(())
	}
}
