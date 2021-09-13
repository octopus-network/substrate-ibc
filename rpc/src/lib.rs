//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use pallet_ibc_runtime_api::IbcApi as IbcRuntimeApi;

#[rpc]
pub trait IbcApi<BlockHash> {
    #[rpc(name = "get_consensus_state_with_height")]
    fn get_consensus_state_with_height(&self, client_id: Vec<u8>,  at: Option<BlockHash>)
        -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

/// A struct that implements the `ConsensusStateWithHeightApi`.
pub struct IbcStorage<C, M> {
    // If you have more generics, no need to SumStorage<C, M, N, P, ...>
    // just use a tuple like SumStorage<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> IbcStorage<C, M> {
    /// Create new `ConsensusStateWithHeightStorage` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}


impl<C, Block> IbcApi<<Block as BlockT>::Hash> for IbcStorage<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: IbcRuntimeApi<Block>,
{
    fn get_consensus_state_with_height(&self, client_id: Vec<u8>, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.consensus_state_with_height(&at, client_id);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
