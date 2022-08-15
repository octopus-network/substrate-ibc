use crate::module::core::ics24_host::Height;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use time::PrimitiveDateTime;

/// Tendermint consensus header
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Header {
	pub signed_header: SignedHeader, // contains the commitment root
	pub validator_set: ValidatorSet, // the validator set that signed Header
	pub trusted_height: Height,      /* the height of a trusted header seen by client less than
	                                  * or equal to Header */
	// TODO(thane): Rename this to trusted_next_validator_set?
	pub trusted_validator_set: ValidatorSet, // the last trusted validator set at trusted height
}

/// Signed block headers
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[non_exhaustive]
pub struct SignedHeader {
	/// Block header
	pub header: block::Header,
	/// Commit containing signatures for the header
	pub commit: block::Commit,
}

/// Validator set contains a vector of validators
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ValidatorSet {
	validators: Vec<Info>,
	proposer: Option<Info>,
	total_voting_power: vote::Power,
}

/// Validator information
// Todo: Remove address and make it into a function that generates it on the fly from pub_key.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Info {
	/// Validator account address
	pub address: account::Id,

	/// Validator public key
	pub pub_key: PublicKey,

	/// Validator voting power
	// Compatibility with genesis.json https://github.com/tendermint/tendermint/issues/5549
	pub power: vote::Power,

	/// Validator name
	pub name: Option<Vec<u8>>,

	/// Validator proposer priority
	pub proposer_priority: ProposerPriority,
}

// Todo: Is there more knowledge/restrictions about proposerPriority?
/// Proposer priority
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ProposerPriority(i64);

mod vote {
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_runtime::RuntimeDebug;

	/// Voting power
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Power(u64);
}

// Note:On the golang side this is generic in the sense that it could everything that implements
// github.com/tendermint/tendermint/crypto.PubKey
// While this is meant to be used with different key-types, it currently only uses a PubKeyEd25519
// version.
// TODO: make this more generic

// Warning: the custom serialization implemented here does not use TryFrom<RawPublicKey>.
//          it should only be used to read/write the priva_validator_key.json.
//          All changes to the serialization should check both the JSON and protobuf conversions.
// Todo: Merge JSON serialization with #[serde(try_from = "RawPublicKey", into = "RawPublicKey)]
/// Public keys allowed in Tendermint protocols
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[non_exhaustive]
pub enum PublicKey {
	/// Ed25519 keys
	Ed25519,

	/// Secp256k1 keys
	Secp256k1,
}

mod account {
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_runtime::RuntimeDebug;

	/// Size of an  account ID in bytes
	pub const LENGTH: usize = 20;

	/// Account IDs
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Id([u8; LENGTH]); // JSON custom serialization for priv_validator_key.json
}

mod block {
	use crate::module::clients::ics07_tendermint::header::account;
	use alloc::vec::Vec;
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_runtime::RuntimeDebug;
	use time::PrimitiveDateTime;

	/// Block height for a particular chain (i.e. number of blocks created since
	/// the chain began)
	///
	/// A height of 0 represents a chain which has not yet produced a block.
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Height(u64);

	mod chain {
		use alloc::vec::Vec;
		use codec::{Decode, Encode};
		use scale_info::TypeInfo;
		use sp_runtime::RuntimeDebug;

		/// Chain identifier (e.g. 'gaia-9000')
		#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
		pub struct Id(Vec<u8>);
	}

	/// `Version` contains the protocol version for the blockchain and the
	/// application.
	///
	/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#version>
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Version {
		/// Block version
		pub block: u64,

		/// App version
		pub app: u64,
	}

	/// Tendermint timestamps
	///
	/// A `Time` value is guaranteed to represent a valid `Timestamp` as defined
	/// by Google's well-known protobuf type [specification]. Conversions and
	/// operations that would result in exceeding `Timestamp`'s validity
	/// range return an error or `None`.
	///
	/// The string serialization format for `Time` is defined as an RFC 3339
	/// compliant string with the optional subsecond fraction part having
	/// up to 9 digits and no trailing zeros, and the UTC offset denoted by Z.
	/// This reproduces the behavior of Go's `time.RFC3339Nano` format.
	///
	/// [specification]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
	// For memory efficiency, the inner member is `PrimitiveDateTime`, with assumed
	// UTC offset. The `assume_utc` method is used to get the operational
	// `OffsetDateTime` value.
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	// pub struct Time(PrimitiveDateTime);
	pub struct Time;

	/// Block identifiers which contain two distinct Merkle roots of the block,
	/// as well as the number of parts in the block.
	///
	/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#blockid>
	///
	/// Default implementation is an empty Id as defined by the Go implementation in
	/// <https://github.com/tendermint/tendermint/blob/1635d1339c73ae6a82e062cd2dc7191b029efa14/types/block.go#L1204>.
	///
	/// If the Hash is empty in BlockId, the BlockId should be empty (encoded to None).
	/// This is implemented outside of this struct. Use the Default trait to check for an empty
	/// BlockId. See: <https://github.com/informalsystems/tendermint-rs/issues/663>
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Id {
		/// The block's main hash is the Merkle root of all the fields in the
		/// block header.
		pub hash: Hash,

		/// Parts header (if available) is used for secure gossipping of the block
		/// during consensus. It is the Merkle root of the complete serialized block
		/// cut into parts.
		///
		/// PartSet is used to split a byteslice of data into parts (pieces) for
		/// transmission. By splitting data into smaller parts and computing a
		/// Merkle root hash on the list, you can verify that a part is
		/// legitimately part of the complete data, and the part can be forwarded
		/// to other peers before all the parts are known. In short, it's a fast
		/// way to propagate a large file over a gossip network.
		///
		/// <https://github.com/tendermint/tendermint/wiki/Block-Structure#partset>
		///
		/// PartSetHeader in protobuf is defined as never nil using the gogoproto
		/// annotations. This does not translate to Rust, but we can indicate this
		/// in the domain type.
		pub part_set_header: PartSetHeader,
	}

	/// Block parts header
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[non_exhaustive]
	pub struct PartSetHeader {
		/// Number of parts in this block
		pub total: u32,

		/// Hash of the parts set header,
		pub hash: Hash,
	}

	/// Output size for the SHA-256 hash function
	pub const SHA256_HASH_SIZE: usize = 32;

	/// Hash digests
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum Hash {
		/// SHA-256 hashes
		Sha256([u8; SHA256_HASH_SIZE]),
		/// Empty hash
		None,
	}

	/// AppHash is usually a SHA256 hash, but in reality it can be any kind of data
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct AppHash(Vec<u8>);

	/// Block `Header` values contain metadata about the block and about the
	/// consensus, as well as commitments to the data in the current block, the
	/// previous block, and the results returned by the application.
	///
	/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#header>
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Header {
		/// Header version
		pub version: Version,

		/// Chain ID
		pub chain_id: chain::Id,

		/// Current block height
		pub height: Height,

		/// Current timestamp
		pub time: Time,

		/// Previous block info
		pub last_block_id: Option<Id>,

		/// Commit from validators from the last block
		pub last_commit_hash: Option<Hash>,

		/// Merkle root of transaction hashes
		pub data_hash: Option<Hash>,

		/// Validators for the current block
		pub validators_hash: Hash,

		/// Validators for the next block
		pub next_validators_hash: Hash,

		/// Consensus params for the current block
		pub consensus_hash: Hash,

		/// State after txs from the previous block
		pub app_hash: AppHash,

		/// Root hash of all results from the txs from the previous block
		pub last_results_hash: Option<Hash>,

		/// Hash of evidence included in the block
		pub evidence_hash: Option<Hash>,

		/// Original proposer of the block
		pub proposer_address: account::Id,
	}

	/// Block round for a particular chain
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Round(u32);

	/// Signatures
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Signature(Vec<u8>);

	/// CommitSig represents a signature of a validator.
	/// It's a part of the Commit and can be used to reconstruct the vote set given the validator
	/// set.
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum CommitSig {
		/// no vote was received from a validator.
		BlockIdFlagAbsent,
		/// voted for the Commit.BlockID.
		BlockIdFlagCommit {
			/// Validator address
			validator_address: account::Id,
			/// Timestamp of vote
			timestamp: Time,
			/// Signature of vote
			signature: Option<Signature>,
		},
		/// voted for nil.
		BlockIdFlagNil {
			/// Validator address
			validator_address: account::Id,
			/// Timestamp of vote
			timestamp: Time,
			/// Signature of vote
			signature: Option<Signature>,
		},
	}

	/// Commit contains the justification (ie. a set of signatures) that a block was committed by a
	/// set of validators.
	/// TODO: Update links below!
	/// <https://github.com/tendermint/tendermint/blob/51dc810d041eaac78320adc6d53ad8b160b06601/types/block.go#L486-L502>
	/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#lastcommit>
	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Commit {
		/// Block height
		pub height: Height,

		/// Round
		pub round: Round,

		/// Block ID
		pub block_id: Id,

		/// Signatures
		pub signatures: Vec<CommitSig>,
	}
}
