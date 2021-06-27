use codec::{Decode, Encode};
use finality_grandpa::{voter_set::VoterSet, Error as GrandpaError};
use sp_finality_grandpa::{AuthorityId, AuthoritySignature, SetId};
use sp_runtime::{
	traits::{Block as BlockT, Header as HeaderT, NumberFor},
	RuntimeDebug,
};
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	prelude::*,
};

type Commit<Block> = finality_grandpa::Commit<
	<Block as BlockT>::Hash,
	NumberFor<Block>,
	AuthoritySignature,
	AuthorityId,
>;

#[derive(Encode, Decode)]
pub struct GrandpaJustification<Block: BlockT> {
	round: u64,
	pub commit: Commit<Block>,
	votes_ancestries: Vec<Block::Header>,
}

impl<Block: BlockT> GrandpaJustification<Block> {
	pub fn verify(&self, set_id: SetId, voters: &VoterSet<AuthorityId>) -> Result<(), Error>
	where
		NumberFor<Block>: finality_grandpa::BlockNumberOps,
	{
		Ok(())
	}
}

#[derive(RuntimeDebug)]
pub enum Error {
	/// Invalid authorities set received from the runtime.
	InvalidAuthoritiesSet,
	/// Could not get runtime version.
	VersionInvalid,
	/// Genesis config is invalid.
	GenesisInvalid,
	/// Error decoding header justification.
	JustificationDecode,
	/// Justification for header is correctly encoded, but invalid.
	BadJustification,
	/// Invalid calculated state root on block import.
	InvalidStateRoot,
}

struct AncestryChain<Block: BlockT> {
	ancestry: BTreeMap<Block::Hash, Block::Header>,
}

impl<Block: BlockT> AncestryChain<Block> {
	fn new(ancestry: &[Block::Header]) -> AncestryChain<Block> {
		let ancestry: BTreeMap<_, _> =
			ancestry.iter().cloned().map(|h: Block::Header| (h.hash(), h)).collect();

		AncestryChain { ancestry }
	}
}

impl<Block: BlockT> finality_grandpa::Chain<Block::Hash, NumberFor<Block>> for AncestryChain<Block>
where
	NumberFor<Block>: finality_grandpa::BlockNumberOps,
{
	fn ancestry(
		&self,
		base: Block::Hash,
		block: Block::Hash,
	) -> Result<Vec<Block::Hash>, GrandpaError> {
		let mut route = Vec::new();
		let mut current_hash = block;
		loop {
			if current_hash == base {
				break;
			}
			match self.ancestry.get(&current_hash) {
				Some(current_header) => {
					current_hash = *current_header.parent_hash();
					route.push(current_hash);
				}
				_ => return Err(GrandpaError::NotDescendent),
			}
		}
		route.pop(); // remove the base

		Ok(route)
	}

	fn best_chain_containing(
		&self,
		_block: Block::Hash,
	) -> Option<(Block::Hash, NumberFor<Block>)> {
		None
	}
}
