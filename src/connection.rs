use sp_core::H256;
use sp_runtime::RuntimeDebug;
use codec::{Decode, Encode};

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ConnectionState {
    None,
    Init,
    TryOpen,
    Open,
    Closed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct ConnectionEnd {
    pub state: ConnectionState,
    pub counterparty_connection_id: H256,
    /// The prefix used for state verification on the counterparty chain associated with this connection.
    /// If not specified, a default counterpartyPrefix of "ibc" should be used.
    pub counterparty_prefix: Vec<u8>,
    pub client_id: H256,
    pub counterparty_client_id: H256,
    pub version: Vec<u8>, // TODO: A ConnectionEnd should only store one version.
}
