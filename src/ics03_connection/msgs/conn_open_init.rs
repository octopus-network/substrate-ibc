use serde_derive::{Deserialize, Serialize};
use substrate_subxt::{PairSigner, DefaultNodeRuntime};
use sp_core::H256;
use sp_core::sr25519::Pair;
use codec::{Decode, Encode};

///
/// Message definition `MsgConnectionOpenInit`  (i.e., the `ConnOpenInit` datagram).
///
// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MsgConnectionOpenInit {
    pub connection_id: H256,
    pub client_id: H256,
    pub counterparty: Counterparty,
    pub signer: PairSigner<DefaultNodeRuntime, Pair>,
}

#[derive(Clone, Debug, Encode, Decode, Default, PartialEq, Serialize, Deserialize)]
pub struct Counterparty {
    client_id: H256,
    connection_id: H256,
    prefix: Vec<u8>, // Todo: migrate "CommitmentPrefix" from ibc-rs
}

impl MsgConnectionOpenInit {
    /// Getter: borrow the `connection_id` from this message.
    pub fn connection_id(&self) -> &H256 {
        &self.connection_id
    }

    /// Getter: borrow the `client_id` from this message.
    pub fn client_id(&self) -> &H256 {
        &self.client_id
    }

    /// Getter: borrow the `counterparty` from this message.
    pub fn counterparty(&self) -> &Counterparty {
        &self.counterparty
    }
}
