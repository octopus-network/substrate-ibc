use super::impls::{AnyConsensusState, IbcContext};
use crate::{constant::TENDERMINT_CLIENT_TYPE, *};
use ibc::{
	clients::ics07_tendermint::{
		client_state::ClientState as TmClientState,
		consensus_state::ConsensusState as TmConsensusState, CommonContext,
		ValidationContext as TmValidationContext,
	},
	core::{
		ics02_client::{
			client_state::ClientStateCommon, error::ClientError, ClientExecutionContext,
		},
		ics24_host::{
			identifier::ClientId,
			path::{ClientConsensusStatePath, ClientStatePath},
		},
		timestamp::Timestamp,
		ContextError, ValidationContext,
	},
};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};

impl<T: Config> ClientExecutionContext for IbcContext<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment>
	+ From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>,

{
	type ClientValidationContext = Self;

	type AnyClientState = TmClientState;

	type AnyConsensusState = AnyConsensusState;

	/// Called upon successful client creation and update
	fn store_client_state(
		&mut self,
		client_state_path: ClientStatePath,
		client_state: Self::AnyClientState,
	) -> Result<(), ContextError> {
        // update client type
		<ClientTypeById<T>>::insert(client_state_path.0.clone(), client_state.client_type());
		// update host height
		let latest_height = client_state.latest_height();
		<HostHeight<T>>::put(latest_height);
		// update client state
		let encode_data = <ibc::clients::ics07_tendermint::client_state::ClientState as Protobuf<Any>>::encode_vec(&client_state);
		<ClientStates<T>>::insert(client_state_path, encode_data);
        Ok(())
    }

	/// Called upon successful client creation and update
	fn store_consensus_state(
		&mut self,
		consensus_state_path: ClientConsensusStatePath,
		consensus_state: Self::AnyConsensusState,
	) -> Result<(), ContextError> {
		let tm_consensus_state: TmConsensusState = consensus_state.try_into().map_err(|_| {
			ClientError::Other { description: "Consensus state type mismatch".to_string() }
		})?;
		let consensus_state = <ibc::clients::ics07_tendermint::consensus_state::ConsensusState as Protobuf<Any>>::encode_vec(&tm_consensus_state);
		<ConsensusStates<T>>::insert(consensus_state_path, consensus_state);

		Ok(())
	}
}

impl<T: Config> CommonContext for IbcContext<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment>
	+ From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>,

{
	type ConversionError = &'static str;
	type AnyConsensusState = AnyConsensusState;

	fn consensus_state(
		&self,
		client_cons_state_path: &ClientConsensusStatePath,
	) -> Result<Self::AnyConsensusState, ContextError> {
		ValidationContext::consensus_state(self, client_cons_state_path)
	}
}

impl<T: Config> TmValidationContext for IbcContext<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment>
	+ From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>,
{

	fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
		ValidationContext::host_timestamp(self)
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: &ibc::Height,
	) -> Result<Option<Self::AnyConsensusState>, ContextError> {
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
							let result = <ibc::clients::ics07_tendermint::consensus_state::ConsensusState as Protobuf<Any>>::decode_vec(&data).map_err(|e| ClientError::Other {
								description: format!("Decode Ics07ConsensusState failed: {:?}", e),
							})?;

								return Ok(Some(AnyConsensusState::Tendermint(result)));
							},

							unimplemented => {
								return Err(ClientError::Other {
									description: format!("unknow client state type: {}", unimplemented),
								}
								.into())
							},
						}
					}
				}
				Ok(None)
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: &ibc::Height,
	) -> Result<Option<Self::AnyConsensusState>, ContextError> {
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
							let result = <ibc::clients::ics07_tendermint::consensus_state::ConsensusState as Protobuf<Any>>::decode_vec(&data).map_err(|e| ClientError::Other {
								description: format!("Decode Ics07ConsensusState failed: {:?}", e),
							})?;
								return Ok(Some(AnyConsensusState::Tendermint(result)));
							},

							unimplemented => {
								return Err(ClientError::Other {
									description: format!("unknow client state type: {}", unimplemented),
								}
								.into())
							},
						}
					}
				}
				Ok(None)
	}
}
