#[cfg(test)]
pub mod test_util {
    use ibc::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};

    use ibc_proto::ibc::core::channel::v1::{
        Channel as RawChannel, Counterparty as RawCounterparty,
    };

    /// Returns a dummy `RawCounterparty`, for testing only!
    /// Can be optionally parametrized with a specific channel identifier.
    pub fn get_dummy_raw_counterparty() -> RawCounterparty {
        RawCounterparty {
            port_id: PortId::default().to_string(),
            channel_id: ChannelId::default().to_string(),
        }
    }

    /// Returns a dummy `RawChannel`, for testing only!
    pub fn get_dummy_raw_channel_end() -> RawChannel {
        RawChannel {
            state: 1,
            ordering: 2,
            counterparty: Some(get_dummy_raw_counterparty()),
            connection_hops: vec![ConnectionId::default().to_string()],
            version: "ics20".to_string(), // The version is not validated.
        }
    }
}