use anyhow::Result;
use ibc_relayer::chain::endpoint::{ChainEndpoint, ChainStatus, HealthCheck};

/// Query the latest height and timestamp the application is at
fn query_application_status() -> Result<ChainStatus> {
	todo!()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_query_application_status() {}
}
