pub mod test_util {
	use ibc_proto::{
		ibc::core::commitment::v1::MerkleProof as RawMerkleProof, ics23::CommitmentProof,
	};

	#[allow(dead_code)]
	/// Returns a dummy `RawMerkleProof`, for testing only!
	pub fn get_dummy_merkle_proof() -> RawMerkleProof {
		let parsed = CommitmentProof { proof: None };
		let mproofs: Vec<CommitmentProof> = vec![parsed];
		RawMerkleProof { proofs: mproofs }
	}
}
