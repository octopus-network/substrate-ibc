use crate::context::Context;
use crate::Config;
use core::marker::PhantomData;
use derive_more::{From, TryInto};
use ibc::clients::tendermint::client_state::ClientState as TmClientState;
use ibc::clients::tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::tendermint::types::{
	TENDERMINT_CLIENT_STATE_TYPE_URL, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::types::path::ClientConsensusStatePath;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::derive::{ClientState, ConsensusState};
use ibc::primitives::proto::{Any, Protobuf};

impl<T: Config> ibc::clients::tendermint::context::CommonContext for Context<T>
where
	u64: std::convert::From<<T as pallet_timestamp::Config>::Moment>,
	u64: std::convert::From<<<<T as frame_system::Config>::Block as frame_support::sp_runtime::traits::Block>::Header as frame_support::sp_runtime::traits::Header>::Number>
{
	type ConversionError = &'static str;
	type AnyConsensusState = AnyConsensusState;

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
		ValidationContext::host_timestamp(self)
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Result<Height, ContextError> {
		ValidationContext::host_height(self)
	}

	/// Retrieve the consensus state for the given client ID at the specified
	/// height.
	///
	/// Returns an error if no such state exists.
	fn consensus_state(
		&self,
		client_cons_state_path: &ClientConsensusStatePath,
	) -> Result<Self::AnyConsensusState, ContextError> {
		ValidationContext::consensus_state(self, client_cons_state_path)
	}

	/// Returns all the heights at which a consensus state is stored
	fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
		todo!()
	}
}

impl<T: Config> ibc::clients::tendermint::context::ValidationContext for Context<T>
where
	u64: std::convert::From<<T as pallet_timestamp::Config>::Moment>,
	u64: std::convert::From<<<<T as frame_system::Config>::Block as frame_support::sp_runtime::traits::Block>::Header as frame_support::sp_runtime::traits::Header>::Number>
{
	/// Search for the lowest consensus state higher than `height`.
	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Self::AnyConsensusState>, ContextError> {
		todo!()
	}

	/// Search for the highest consensus state lower than `height`.
	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: &Height,
	) -> Result<Option<Self::AnyConsensusState>, ContextError> {
		todo!()
	}
}

#[derive(Debug, Clone, From, PartialEq, ClientState)]
#[validation(Context<T: Config>)]
#[execution(Context<T: Config>)]
pub enum AnyClientState {
	Tendermint(TmClientState),
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
	type Error = ClientError;

	fn try_from(raw: Any) -> Result<Self, Self::Error> {
		if raw.type_url == TENDERMINT_CLIENT_STATE_TYPE_URL {
			Ok(TmClientState::try_from(raw)?.into())
		} else {
			Err(ClientError::Other { description: "failed to deserialize message".to_string() })
		}
	}
}

impl From<AnyClientState> for Any {
	fn from(host_client_state: AnyClientState) -> Self {
		match host_client_state {
			AnyClientState::Tendermint(cs) => cs.into(),
		}
	}
}

#[derive(Debug, Clone, From, TryInto, PartialEq, ConsensusState)]
pub enum AnyConsensusState {
	Tendermint(TmConsensusState),
}

impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
	type Error = ClientError;

	fn try_from(raw: Any) -> Result<Self, Self::Error> {
		if raw.type_url == TENDERMINT_CONSENSUS_STATE_TYPE_URL {
			Ok(TmConsensusState::try_from(raw)?.into())
		} else {
			Err(ClientError::Other { description: "failed to deserialize message".to_string() })
		}
	}
}

impl From<AnyConsensusState> for Any {
	fn from(host_consensus_state: AnyConsensusState) -> Self {
		match host_consensus_state {
			AnyConsensusState::Tendermint(cs) => cs.into(),
		}
	}
}
