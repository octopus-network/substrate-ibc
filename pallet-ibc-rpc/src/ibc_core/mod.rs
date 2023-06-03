use crate::node_template;
use anyhow::Result;

use std::time::Duration;
use subxt::{OnlineClient, SubstrateConfig};

#[derive(Debug)]
pub struct Height {
	pub reversion_number: u64,
	pub reversion_height: u64,
}

#[derive(Debug)]
pub struct Timestamp(pub u64);

#[derive(Debug)]
pub struct ChainStatus {
	pub height: Height,
	pub timestamp: Timestamp,
}

/// Query the latest height and timestamp the application is at
pub async fn query_application_status_with_substrate(rpc_url: &str) -> Result<ChainStatus> {
	// Create a new API client, configured to talk to Polkadot nodes.
	let api = OnlineClient::<SubstrateConfig>::from_url(rpc_url).await?;
	query_application_status(&api).await
}

/// Query the latest height and timestamp the application is at
pub async fn query_application_status(
	rpc_client: &OnlineClient<SubstrateConfig>,
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
	Ok(ChainStatus { height, timestamp: Timestamp(result) })
}

#[cfg(test)]
mod tests {
	use super::*;
	use ibc::core::timestamp::Timestamp;

	#[tokio::test]
	async fn test_query_application_status() {
		// Create a new API client, configured to talk to Polkadot nodes.
		let api = OnlineClient::<SubstrateConfig>::new().await.unwrap();
		let result = query_application_status(&api).await.unwrap();
		println!("height = {:?}", result.height);
		let duration = Duration::from_millis(result.timestamp.0);
		let timestamp = Timestamp::from_nanoseconds(duration.as_nanos() as u64)
			.map_err(|e| anyhow::anyhow!("get timestmap eror({})", e))
			.unwrap();
		println!("timestamp = {:?}", timestamp);
	}

	#[tokio::test]
	async fn test_query_application_status_with_substrate() {
		let result = query_application_status_with_substrate("ws://127.0.0.1:9944").await.unwrap();
		println!("height = {:?}", result.height);
		let duration = Duration::from_millis(result.timestamp.0);
		let timestamp = Timestamp::from_nanoseconds(duration.as_nanos() as u64)
			.map_err(|e| anyhow::anyhow!("get timestmap eror({})", e))
			.unwrap();
		println!("timestamp = {:?}", timestamp);
	}
}
