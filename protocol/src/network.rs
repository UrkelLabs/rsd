#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Network {
    Mainnet,
    Testnet,
    Regtest,
    Simnet,
}

impl Network {
    pub fn from_bech32_prefix(prefix: &str) -> Option<Network> {
        match prefix {
            "hs" => Some(Network::Mainnet),
            "ts" => Some(Network::Testnet),
            "rs" => Some(Network::Regtest),
            "ss" => Some(Network::Simnet),
            _ => None,
        }
    }

    pub fn magic(&self) -> u32 {
        match *self {
            Network::Mainnet => 3958442712,
            Network::Testnet => 165176447,
            Network::Regtest => 3169940394,
            Network::Simnet => 1195102226,
        }
    }

    pub fn port(&self) -> u32 {
        match *self {
            Network::Mainnet => 12038,
            Network::Testnet => 13038,
            Network::Regtest => 14038,
            Network::Simnet => 15038,
        }
    }

    pub fn halvening_interval(&self) -> u32 {
        match *self {
            Network::Mainnet => 170000,
            Network::Testnet => 170000,
            Network::Regtest => 2500,
            Network::Simnet => 170000,
        }
    }

    pub fn coinbase_maturity(&self) -> u32 {
        match *self {
            Network::Mainnet => 100,
            Network::Testnet => 100,
            Network::Regtest => 2,
            Network::Simnet => 6,
        }
    }
}

//from string
