use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

pub mod primitive {
    use ibc::ics02_client::client_type::ClientType as IbcClientType;
    use ibc::ics02_client::height::Height as IbcHeight;
    use ibc::ics24_host::identifier::ClientId as IbcClientId;
    use ibc::ics24_host::identifier::ConnectionId as IbcConnectionId;
    use ibc::ics24_host::identifier::PortId as IbcPortId;
    use ibc::ics24_host::identifier::ChannelId as IbcChannelId;

    use codec::{Decode, Encode};
    use sp_runtime::RuntimeDebug;

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct PortId(pub String);

    impl PortId {
        fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcPortId> for PortId {
        fn from(value : IbcPortId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl PortId {
        pub fn to_ibc_port_id(self) -> IbcPortId {
            IbcPortId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ChannelId(pub String);

    impl ChannelId {
        fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl From<IbcChannelId> for ChannelId {
        fn from(value : IbcChannelId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl ChannelId {
        pub fn to_ibc_channel_id(self) -> IbcChannelId {
            IbcChannelId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct Height {
        /// Previously known as "epoch"
        pub revision_number: u64,

        /// The height of a block
        pub revision_height: u64,
    }

    impl From<IbcHeight> for Height {
        fn from(
            IbcHeight {
                revision_number,
                revision_height,
            }: IbcHeight,
        ) -> Self {
            Height {
                revision_number,
                revision_height,
            }
        }
    }

    impl Height {
        pub fn to_ibc_height(self) -> IbcHeight {
            IbcHeight {
                revision_number: self.revision_number,
                revision_height: self.revision_height,
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub enum ClientType {
        Tendermint,
        Grandpa,
    }

    impl From<IbcClientType> for ClientType {
        fn from(value: IbcClientType) -> Self {
            match value {
                IbcClientType::Tendermint => ClientType::Tendermint,
                IbcClientType::Grandpa => ClientType::Grandpa,
                _ => unreachable!(),
            }
        }
    }


    impl ClientType {
        pub fn to_ibc_client_type(self) -> IbcClientType {
            match self {
                ClientType::Tendermint => IbcClientType::Tendermint,
                ClientType::Grandpa => IbcClientType::Grandpa,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ClientId(pub String);

    impl ClientId {
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcClientId> for ClientId {
        fn from(value: IbcClientId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl ClientId {
        pub fn to_ibc_client_id(self) -> IbcClientId {
            IbcClientId(self.0)
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ConnectionId(String);

    impl ConnectionId {
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcConnectionId> for ConnectionId {
        fn from(value: IbcConnectionId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl ConnectionId {
        pub fn to_ibc_connection_id(self) -> IbcConnectionId {
            IbcConnectionId(self.0)
        }
    }
}
