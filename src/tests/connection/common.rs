pub mod test_util {
	use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
	use ibc_proto::ibc::core::{
		commitment::v1::MerklePrefix, connection::v1::Counterparty as RawCounterparty,
	};

	pub fn get_dummy_raw_counterparty() -> RawCounterparty {
		RawCounterparty {
			client_id: ClientId::default().to_string(),
			connection_id: ConnectionId::default().to_string(),
			prefix: Some(MerklePrefix { key_prefix: b"ibc".to_vec() }),
		}
	}
}
