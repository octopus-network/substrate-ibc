use sp_core::H256;
use crate::grandpa;
use sp_trie::StorageProof;
use crate::channel::ChannelOrder;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Packet {
    pub sequence: u64,
    /// If the latest block height of the destination chain is greater than ```timeout_height```, the packet will not be processed.
    pub timeout_height: u32,
    pub source_port: Vec<u8>,
    pub source_channel: H256,
    pub dest_port: Vec<u8>,
    pub dest_channel: H256,
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum Datagram {
    ClientUpdate {
        client_id: H256,
        header: grandpa::header::Header,
    },
    ClientMisbehaviour {
        identifier: H256,
        evidence: Vec<u8>,
    },
    ConnOpenTry {
        connection_id: H256,
        counterparty_connection_id: H256,
        counterparty_client_id: H256,
        client_id: H256,
        version: Vec<u8>, // Todo: remove this field
        counterparty_version: Vec<u8>,
        proof_init: StorageProof,
        proof_consensus: StorageProof,
        proof_height: u32,
        consensus_height: u32,
    },
    ConnOpenAck {
        connection_id: H256,
        counterparty_connection_id: H256,
        version: u8,
        proof_try: StorageProof,
        proof_consensus: StorageProof,
        proof_height: u32,
        consensus_height: u32,
    },
    ConnOpenConfirm {
        connection_id: H256,
        proof_ack: StorageProof,
        proof_height: u32,
    },
    ChanOpenTry {
        order: ChannelOrder,
        connection_hops: Vec<H256>,
        port_id: Vec<u8>,
        channel_id: H256,
        counterparty_port_id: Vec<u8>,
        counterparty_channel_id: H256,
        channel_version: Vec<u8>,
        counterparty_version: Vec<u8>,
        proof_init: StorageProof,
        proof_height: u32,
    },
    ChanOpenAck {
        port_id: Vec<u8>,
        channel_id: H256,
        version: Vec<u8>,
        proof_try: StorageProof, // Todo: In ibc-rs, proofs contains `object_proof`, `client_proof`, `consensus_proof` and `height`
        proof_height: u32,
    },
    ChanOpenConfirm {
        port_id: Vec<u8>,
        channel_id: H256,
        proof_ack: StorageProof,
        proof_height: u32,
    },
    PacketRecv {
        packet: Packet,
        proof: StorageProof,
        proof_height: u32,
    },
    PacketAcknowledgement {
        packet: Packet,
        acknowledgement: Vec<u8>,
        proof: StorageProof,
        proof_height: u32,
    },
}
