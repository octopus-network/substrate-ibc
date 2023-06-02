use crate::node_template;
use anyhow::Result;
use ibc::core::timestamp::Timestamp;
use std::time::Duration;
use subxt::{OnlineClient, SubstrateConfig};

#[derive(Debug)]
pub struct Height {
	pub reversion_number: u64,
	pub reversion_height: u64,
}

#[derive(Debug)]
pub struct ChainStatus {
	pub height: Height,
	pub timestamp: Timestamp,
}

/// Query the latest height and timestamp the application is at
pub async fn query_application_status(
	rpc_client: OnlineClient<SubstrateConfig>,
) -> Result<ChainStatus> {
	let finalized_header_hash = rpc_client.rpc().finalized_head().await?;
	let finalized_header = rpc_client
		.rpc()
		.block(Some(finalized_header_hash))
		.await?
		.ok_or(anyhow::anyhow!("Block is None"))?;
	let block_number = finalized_header.block.header.number;
	let height = Height { reversion_number: 0, reversion_height: block_number as u64 };
	let timestamp_storage_query = node_template::storage().timestamp().now();
	let result = rpc_client
		.storage()
		.at_latest()
		.await?
		.fetch(&timestamp_storage_query)
		.await?
		.ok_or(anyhow::anyhow!("Timestamp is None"))?;
	let duration = Duration::from_millis(result);
	let timestamp = Timestamp::from_nanoseconds(duration.as_nanos() as u64)
		.map_err(|e| anyhow::anyhow!("get timestmap eror({})", e))?;
	Ok(ChainStatus { height, timestamp })
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_query_application_status() {
		// Create a new API client, configured to talk to Polkadot nodes.
		let api = OnlineClient::<SubstrateConfig>::new().await.unwrap();
		let result = query_application_status(api).await.unwrap();
		println!("{:?}", result);
	}
}
