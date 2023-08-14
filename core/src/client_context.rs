use std::fmt::Debug;

use crate::Config;
use ibc::{
	clients::ics07_tendermint::{
		client_state::ClientState as TmClientState,
		consensus_state::ConsensusState as TmConsensusState, CommonContext,
		ValidationContext as TmValidationContext,
	},
	core::{
		ics02_client::{error::ClientError, ClientExecutionContext},
		ics24_host::{
			identifier::ClientId,
			path::{ClientConsensusStatePath, ClientStatePath, Path},
		},
		timestamp::Timestamp,
		ContextError, ValidationContext,
	},
};

use super::impls::{AnyConsensusState, IbcContext};

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
		// self.client_state_store
		// 	.set(client_state_path, client_state)
		// 	.map(|_| ())
		// 	.map_err(|_| ClientError::Other {
		// 		description: "Client state store error".to_string(),
		// 	})?;
		// Ok(())
		todo!()
	}

	/// Called upon successful client creation and update
	fn store_consensus_state(
		&mut self,
		consensus_state_path: ClientConsensusStatePath,
		consensus_state: Self::AnyConsensusState,
	) -> Result<(), ContextError> {
		// let tm_consensus_state: TmConsensusState = consensus_state.try_into().map_err(|_| {
		// 	ClientError::Other { description: "Consensus state type mismatch".to_string() }
		// })?;
		// self.consensus_state_store
		// 	.set(consensus_state_path, tm_consensus_state)
		// 	.map_err(|_| ClientError::Other {
		// 		description: "Consensus state store error".to_string(),
		// 	})?;
		// Ok(())
		todo!()
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
		// let path = format!("clients/{client_id}/consensusStates").try_into().unwrap(); // safety - path must be valid since ClientId and height are valid Identifiers

		// let keys = self.store.get_keys(&path);
		// let found_path = keys.into_iter().find_map(|path| {
		// 	if let Ok(Path::ClientConsensusState(path)) = Path::try_from(path) {
		// 		if height > &ibc::Height::new(path.epoch, path.height).unwrap() {
		// 			return Some(path);
		// 		}
		// 	}
		// 	None
		// });

		// if let Some(path) = found_path {
		// 	let consensus_state = self.consensus_state_store.get(Height::Pending, &path).ok_or(
		// 		ClientError::ConsensusStateNotFound {
		// 			client_id: client_id.clone(),
		// 			height: *height,
		// 		},
		// 	)?;

		// 	Ok(Some(consensus_state.into()))
		// } else {
		// 	Ok(None)
		// }
		todo!()
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: &ibc::Height,
	) -> Result<Option<Self::AnyConsensusState>, ContextError> {
		// let path = format!("clients/{client_id}/consensusStates").try_into().unwrap(); // safety - path must be valid since ClientId and height are valid Identifiers

		// let keys = self.store.get_keys(&path);
		// let pos = keys.iter().position(|path| {
		// 	if let Ok(Path::ClientConsensusState(path)) = Path::try_from(path.clone()) {
		// 		height >= &ibc::Height::new(path.epoch, path.height).unwrap()
		// 	} else {
		// 		false
		// 	}
		// });

		// if let Some(pos) = pos {
		// 	if pos > 0 {
		// 		let prev_path = match Path::try_from(keys[pos - 1].clone()) {
		// 			Ok(Path::ClientConsensusState(p)) => p,
		// 			_ => unreachable!(), // safety - path retrieved from store
		// 		};
		// 		let consensus_state = self
		// 			.consensus_state_store
		// 			.get(Height::Pending, &prev_path)
		// 			.ok_or(ClientError::ConsensusStateNotFound {
		// 			client_id: client_id.clone(),
		// 			height: *height,
		// 		})?;
		// 		return Ok(Some(consensus_state.into()));
		// 	}
		// }
		// Ok(None)
		todo!()
	}
}
