use scale_info::prelude::{string::String, vec::Vec};
use sp_std::str::FromStr;

use codec::{Decode, Encode};
use ibc::{
	clients::ics10_grandpa::{
		client_state::ClientState as IbcClientState,
		help::{
			BlockHeader, Commitment, MmrRoot as IbcMmrRoot, SignedCommitment, ValidatorMerkleProof,
			ValidatorSet,
		},
	},
	core::ics24_host::identifier::ChainId as IbcChainId,
};

use crate::ibc_core::ics24_host::Height;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MmrRoot {
	pub block_header: Vec<u8>,
	pub signed_commitment: Vec<u8>,
	pub validator_merkle_proofs: Vec<Vec<u8>>,
	pub mmr_leaf: Vec<u8>,
	pub mmr_leaf_proof: Vec<u8>,
}

impl From<IbcMmrRoot> for MmrRoot {
	fn from(ibc_mmr_root: IbcMmrRoot) -> Self {
		let encode_validator_proofs = ibc_mmr_root
			.validator_merkle_proofs
			.into_iter()
			.map(|validator_proof| ValidatorMerkleProof::encode(&validator_proof))
			.collect();

		Self {
			block_header: BlockHeader::encode(&ibc_mmr_root.block_header),
			signed_commitment: SignedCommitment::encode(&ibc_mmr_root.signed_commitment),
			validator_merkle_proofs: encode_validator_proofs,
			mmr_leaf: ibc_mmr_root.mmr_leaf,
			mmr_leaf_proof: ibc_mmr_root.mmr_leaf_proof,
		}
	}
}

impl From<MmrRoot> for IbcMmrRoot {
	fn from(mmr_root: MmrRoot) -> Self {
		let decode_validator_proofs: Vec<ValidatorMerkleProof> = mmr_root
			.validator_merkle_proofs
			.into_iter()
			.map(|validator_proof| {
				ValidatorMerkleProof::decode(&mut &validator_proof[..]).expect("Never fail")
			})
			.collect();

		Self {
			block_header: BlockHeader::decode(&mut &mmr_root.block_header[..]).expect("Never fail"),
			signed_commitment: SignedCommitment::decode(&mut &mmr_root.signed_commitment[..])
				.expect("Never fail"),
			validator_merkle_proofs: decode_validator_proofs,
			mmr_leaf: mmr_root.mmr_leaf,
			mmr_leaf_proof: mmr_root.mmr_leaf_proof,
		}
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientState {
	pub chain_id: Vec<u8>,
	/// block_number is height?
	pub block_number: u32,
	/// Block height when the client was frozen due to a misbehaviour
	pub frozen_height: Option<Height>,
	pub block_header: Vec<u8>,
	pub latest_commitment: Vec<u8>,
	pub validator_set: Vec<u8>,
}
impl From<IbcClientState> for ClientState {
	fn from(ibc_client_state: IbcClientState) -> Self {
		Self {
			chain_id: ibc_client_state.chain_id.as_str().as_bytes().to_vec(),
			block_number: ibc_client_state.block_number,
			frozen_height: ibc_client_state.frozen_height.map(|val| val.into()),
			block_header: BlockHeader::encode(&ibc_client_state.block_header),
			latest_commitment: Commitment::encode(&ibc_client_state.latest_commitment),
			validator_set: ValidatorSet::encode(&ibc_client_state.validator_set),
		}
	}
}

impl From<ClientState> for IbcClientState {
	fn from(client_state: ClientState) -> Self {
		let chain_id_str = String::from_utf8(client_state.chain_id).expect("Never fail");
		Self {
			chain_id: IbcChainId::from_str(&chain_id_str).expect("Never fail"),
			block_number: client_state.block_number,
			frozen_height: client_state.frozen_height.map(|value| value.into()),
			block_header: BlockHeader::decode(&mut &client_state.block_header[..])
				.expect("Never fail"),
			latest_commitment: Commitment::decode(&mut &client_state.latest_commitment[..])
				.expect("Never fail"),
			validator_set: ValidatorSet::decode(&mut &client_state.validator_set[..])
				.expect("Never fail"),
		}
	}
}
