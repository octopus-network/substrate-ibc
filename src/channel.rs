use sp_core::H256;
use sp_runtime::RuntimeDebug;
use codec::{Decode, Encode};

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChannelState {
    None,
    Init,
    TryOpen,
    Open,
    Closed,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self::None
    }
}

// Todo: In ibc-rs, `ChannelOrder` is type i32
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum ChannelOrder {
    Ordered,
    Unordered,
}

impl Default for ChannelOrder {
    fn default() -> Self {
        Self::Ordered
    }
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct ChannelEnd {
    pub state: ChannelState,
    pub ordering: ChannelOrder,
    pub counterparty_port_id: Vec<u8>,
    pub counterparty_channel_id: H256,
    pub connection_hops: Vec<H256>,
    pub version: Vec<u8>,
}
