pub mod ibc_core;

#[subxt::subxt(runtime_metadata_path = "metadata/metadata.scale")]
pub mod node_template {}

#[cfg(test)]
mod tests {
	use super::*;
}
