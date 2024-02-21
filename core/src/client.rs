use crate::{
	context::Context, ClientCounter, ClientProcessedHeights, ClientProcessedTimes, ClientStates,
	Clients, Config, ConsensusStates, MOCK_CLIENT_TYPE, TENDERMINT_CLIENT_TYPE,
};
pub use alloc::{
	format,
	string::{String, ToString},
};
use frame_system::pallet_prelude::BlockNumberFor;
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
			error::ClientError,
		},
		ics24_host::{
			identifier::ClientId,
			path::{ClientConsensusStatePath, ClientStatePath, ClientTypePath},
		},
	},
	mock::{client_state::MockClientState, consensus_state::MockConsensusState},
	timestamp::Timestamp,
	Height,
};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use sp_core::Get;
use sp_std::{boxed::Box, vec::Vec};

impl<T: Config> ClientReader for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ClientError> {
		let data = <Clients<T>>::get(ClientTypePath(client_id.clone()))
			.ok_or(ClientError::ClientNotFound { client_id: client_id.clone() })?;
		match data.as_str() {
			TENDERMINT_CLIENT_TYPE => Ok(ClientType::new(TENDERMINT_CLIENT_TYPE.into())),
			MOCK_CLIENT_TYPE => Ok(ClientType::new(MOCK_CLIENT_TYPE.into())),
			unimplemented => {
				return Err(ClientError::UnknownClientStateType {
					client_state_type: unimplemented.to_string(),
				})
			},
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ClientError> {
		let data = <ClientStates<T>>::get(ClientStatePath(client_id.clone()))
			.ok_or(ClientError::ClientNotFound { client_id: client_id.clone() })?;
		match self.client_type(client_id)?.as_str() {
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ClientState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ClientState failed: {:?}", e),
					})?;

				Ok(Box::new(result))
			},
			MOCK_CLIENT_TYPE => {
				let result: MockClientState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Deocode Ics10ClientState failed: {:?}", e),
					})?;
				Ok(Box::new(result))
			},
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}),
		}
	}

	fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ClientError> {
		if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
			return Ok(client_state.into_box());
		}
		#[cfg(test)]
		if let Ok(client_state) = MockClientState::try_from(client_state.clone()) {
			return Ok(client_state.into_box());
		}
		Err(ClientError::UnknownClientStateType { client_state_type: client_state.type_url })
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Box<dyn ConsensusState>, ClientError> {
		let data = <ConsensusStates<T>>::get(ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number(),
			height: height.revision_height(),
		})
		.ok_or(ClientError::ConsensusStateNotFound {
			client_id: client_id.clone(),
			height: *height,
		})?;
		match self.client_type(client_id)?.as_str() {
			TENDERMINT_CLIENT_TYPE => {
				let result: Ics07ConsensusState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode Ics07ConsensusState failed: {:?}", e),
					})?;
				Ok(Box::new(result))
			},
			MOCK_CLIENT_TYPE => {
				let result: MockConsensusState =
					Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
						description: format!("Decode MockConsensusState failed: {:?}", e),
					})?;
				Ok(Box::new(result))
			},
			unimplemented => Err(ClientError::UnknownClientStateType {
				client_state_type: unimplemented.to_string(),
			}),
		}
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
		let mut heights = <ConsensusStates<T>>::iter_keys()
			.map(|key| {
				let height = Height::new(key.epoch, key.height);
				height
			})
			.collect::<Result<Vec<Height>, ClientError>>()?;

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h > *height {
				let data = <ConsensusStates<T>>::get(ClientConsensusStatePath {
					client_id: client_id.clone(),
					epoch: h.revision_number(),
					height: h.revision_height(),
				})
				.ok_or(ClientError::ConsensusStateNotFound {
					client_id: client_id.clone(),
					height: h,
				})?;
				match self.client_type(client_id)?.as_str() {
					TENDERMINT_CLIENT_TYPE => {
						let result: Ics07ConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| ClientError::Other {
								description: format!("Decode Ics07ConsensusState failed: {:?}", e),
							})?;
						return Ok(Some(Box::new(result)));
					},
					MOCK_CLIENT_TYPE => {
						let result: MockConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| ClientError::Other {
								description: format!("Decode MockConsensusState failed: {:?}", e),
							})?;
						return Ok(Some(Box::new(result)));
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
		height: &Height,
	) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
		let mut heights = <ConsensusStates<T>>::iter_keys()
			.map(|key| {
				let height = Height::new(key.epoch, key.height);
				height
			})
			.collect::<Result<Vec<Height>, ClientError>>()?;

		heights.sort_by(|a, b| b.cmp(a));

		// Search for previous state.
		for h in heights {
			if h < *height {
				let data = <ConsensusStates<T>>::get(ClientConsensusStatePath {
					client_id: client_id.clone(),
					epoch: h.revision_number(),
					height: h.revision_height(),
				})
				.ok_or(ClientError::ConsensusStateNotFound {
					client_id: client_id.clone(),
					height: h,
				})?;
				match self.client_type(client_id)?.as_str() {
					TENDERMINT_CLIENT_TYPE => {
						let result: Ics07ConsensusState = ibc_proto::protobuf::Protobuf::<
							ibc_proto::google::protobuf::Any,
						>::decode_vec(&data)
						.map_err(|e| ClientError::Other {
							description: format!("Decode Ics07ConsensusState failed: {:?}", e),
						})?;
						return Ok(Some(Box::new(result)));
					},
					MOCK_CLIENT_TYPE => {
						let result: MockConsensusState = Protobuf::<Any>::decode_vec(&data)
							.map_err(|e| ClientError::Other {
								description: format!("Decode MockConsensusState failed: {:?}", e),
							})?;
						return Ok(Some(Box::new(result)));
					},
					_ => {},
				}
			}
		}
		Ok(None)
	}

	fn host_height(&self) -> Result<Height, ClientError> {
		let block_height = <frame_system::Pallet<T>>::block_number();
		Height::new(T::ChainVersion::get(), u64::from(block_height)).map_err(|e| {
			ClientError::Other { description: format!("contruct Ibc Height error: {}", e) }
		})
	}

	fn host_timestamp(&self) -> Result<Timestamp, ClientError> {
		#[cfg(not(test))]
		{
			use core::time::Duration;
			let current_time = <pallet_timestamp::Pallet<T>>::get();
			let duration = Duration::from_millis(current_time.into());
			Timestamp::from_nanoseconds(duration.as_nanos() as u64).map_err(|e| {
				ClientError::Other { description: format!("get host time stamp error: {}", e) }
					.into()
			})
		}
		#[cfg(test)]
		{
			Ok(Timestamp::now())
		}
	}

	fn host_consensus_state(
		&self,
		_height: &Height,
	) -> Result<Box<dyn ConsensusState>, ClientError> {
		#[cfg(not(test))]
		{
			Err(ClientError::ImplementationSpecific)
		}
		#[cfg(test)]
		{
			use ibc::mock::header::MockHeader;
			let mock_header =
				MockHeader { height: self.host_height()?, timestamp: Default::default() };
			Ok(Box::new(MockConsensusState::new(mock_header)))
		}
	}

	fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ClientError> {
		#[cfg(not(test))]
		{
			Err(ClientError::ImplementationSpecific)
		}
		#[cfg(test)]
		{
			use ibc::mock::header::MockHeader;
			let mock_header =
				MockHeader { height: self.host_height()?, timestamp: Default::default() };
			Ok(Box::new(MockConsensusState::new(mock_header)))
		}
	}

	fn client_counter(&self) -> Result<u64, ClientError> {
		Ok(<ClientCounter<T>>::get())
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ClientError> {
		<Clients<T>>::insert(ClientTypePath(client_id), client_type);

		Ok(())
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: Box<dyn ClientState>,
	) -> Result<(), ClientError> {
		let data = client_state.encode_vec().map_err(|e| ClientError::Other {
			description: format!("Encode ClientState Failed: {:?}", e),
		})?;

		<ClientStates<T>>::insert(ClientStatePath(client_id), data);
		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: Box<dyn ConsensusState>,
	) -> Result<(), ClientError> {
		let consensus_state = consensus_state.encode_vec().map_err(|e| ClientError::Other {
			description: format!("Encode ConsensusStates failed: {:?}", e),
		})?;

		<ConsensusStates<T>>::insert(
			ClientConsensusStatePath {
				client_id,
				epoch: height.revision_number(),
				height: height.revision_height(),
			},
			consensus_state,
		);

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		let _ = ClientCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ClientError> {
		<ClientProcessedTimes<T>>::insert(client_id, height, timestamp.nanoseconds());

		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ClientError> {
		<ClientProcessedHeights<T>>::insert(client_id, height, host_height);

		Ok(())
	}
}
