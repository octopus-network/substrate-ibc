pub mod test_util {
	use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
	use ibc_proto::ibc::core::{
		commitment::v1::MerklePrefix, connection::v1::Counterparty as RawCounterparty,
	};

	pub fn get_dummy_raw_counterparty() -> RawCounterparty {
		RawCounterparty {
			client_id: ClientId::new("07-tendermint", 0).unwrap().to_string(),
			connection_id: ConnectionId::default().to_string(),
			prefix: Some(MerklePrefix { key_prefix: b"ibc".to_vec() }),
		}
	}
}
