use ibc::{
    clients::ics10_grandpa::{
        client_state::ClientState as IbcClientState,
        help::{
            BlockHeader, Commitment, MmrRoot as IbcMmrRoot, SignedCommitment,
            ValidatorMerkleProof, ValidatorSet,
        },
    },
};

use ibc::core::ics24_host::identifier::ChainId as IbcChainId;
use ibc::core::ics24_host::error::ValidationError;
use alloc::string::String;
use sp_runtime::RuntimeDebug;
use codec::{Decode, Encode};
use flex_error::{define_error, DisplayOnly, TraceError};
use crate::module::core::ics24_host::Height;
use sp_std::{str::FromStr, vec::Vec};
use scale_info::TypeInfo;

define_error! {
    	#[derive(Debug, PartialEq, Eq)]
		Error {
			InvalidFromUtf8
				[DisplayOnly<alloc::string::FromUtf8Error>]
				| _ | { "invalid from utf8 error" },
			InvalidDecode
				[DisplayOnly<codec::Error>]
				| _ | { "invalid decode error" },
			ParseTimestampFailed
				[DisplayOnly<ibc::timestamp::ParseTimestampError>]
				| _ | { "invalid parse timestamp error" },
			ValidationFailed
				[DisplayOnly<ValidationError>]
				| _ | { "invalid validation error"},
			InvalidChainId
				[DisplayOnly<core::convert::Infallible>]
				|_| { "invalid chain id error" },
		}
	}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MmrRoot {
    pub block_header: Vec<u8>,
    pub signed_commitment: Vec<u8>,
    pub validator_merkle_proofs: Vec<Vec<u8>>,
    pub mmr_leaf: Vec<u8>,
    pub mmr_leaf_proof: Vec<u8>,
}

impl From<IbcMmrRoot> for MmrRoot {
    fn from(val: IbcMmrRoot) -> Self {
        let encode_validator_proofs = val
            .validator_merkle_proofs
            .into_iter()
            .map(|validator_proof| ValidatorMerkleProof::encode(&validator_proof))
            .collect();

        Self {
            block_header: BlockHeader::encode(&val.block_header),
            signed_commitment: SignedCommitment::encode(&val.signed_commitment),
            validator_merkle_proofs: encode_validator_proofs,
            mmr_leaf: val.mmr_leaf,
            mmr_leaf_proof: val.mmr_leaf_proof,
        }
    }
}

impl MmrRoot {
    pub fn to_ibc_mmr_root(self) -> Result<IbcMmrRoot, Error> {
        let decode_validator_proofs: Vec<ValidatorMerkleProof> = self
            .validator_merkle_proofs
            .into_iter()
            .map(|validator_proof| {
                ValidatorMerkleProof::decode(&mut &validator_proof[..]).unwrap() // TODO
            })
            .collect();
        Ok(IbcMmrRoot {
            block_header: BlockHeader::decode(&mut &self.block_header[..])
                .map_err(Error::invalid_decode)?,
            signed_commitment: SignedCommitment::decode(&mut &self.signed_commitment[..])
                .map_err(Error::invalid_decode)?,
            validator_merkle_proofs: decode_validator_proofs,
            mmr_leaf: self.mmr_leaf,
            mmr_leaf_proof: self.mmr_leaf_proof,
        })
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
    fn from(val: IbcClientState) -> Self {
        Self {
            chain_id: val.chain_id.as_str().as_bytes().to_vec(),
            block_number: val.block_number,
            frozen_height: val.frozen_height.map(|val| val.into()),
            block_header: BlockHeader::encode(&val.block_header),
            latest_commitment: Commitment::encode(&val.latest_commitment),
            validator_set: ValidatorSet::encode(&val.validator_set),
        }
    }
}

impl ClientState {
    pub fn to_ibc_client_state(self) -> Result<IbcClientState, Error> {
        let chain_id_str =
            String::from_utf8(self.chain_id).map_err(Error::invalid_from_utf8)?;
        Ok(IbcClientState {
            chain_id: IbcChainId::from_str(&chain_id_str).map_err(Error::invalid_chain_id)?,
            block_number: self.block_number,
            frozen_height: self.frozen_height.map(|value| value.to_ibc_height()),
            block_header: BlockHeader::decode(&mut &self.block_header[..])
                .map_err(Error::invalid_decode)?,
            latest_commitment: Commitment::decode(&mut &self.latest_commitment[..])
                .map_err(Error::invalid_decode)?,
            validator_set: ValidatorSet::decode(&mut &self.validator_set[..])
                .map_err(Error::invalid_decode)?,
        })
    }
}