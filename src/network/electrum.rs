use crate::e::S5Error;

// TODO: policy asset should only be set for ElementsRegtest, fail otherwise
const _LIQUID_POLICY_ASSET_STR: &str =
    "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const LIQUID_TESTNET_POLICY_ASSET_STR: &str =
    "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49";

pub const DEFAULT_TESTNET_NODE: &str = "electrum.bullbitcoin.com:60002";
pub const DEFAULT_LIQUID_TESTNET_NODE: &str = "electrs.sideswap.io:12002";

pub const DEFAULT_MAINNET_NODE: &str = "electrum.bullbitcoin.com:50002";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitcoinNetwork {
    Bitcoin,
    BitcoinTestnet,
    Liquid,
    LiquidTestnet,
    ElementsRegtest,
}

#[derive(Debug, Clone)]
pub enum ElectrumUrl {
    Tls(String, bool), // the bool value indicates if the domain name should be validated
    Plaintext(String),
}

impl ElectrumUrl {
    pub fn build_client(&self) -> Result<electrum_client::Client, S5Error> {
        let builder = electrum_client::ConfigBuilder::new();
        let builder = builder.timeout(Some(9));
        let (url, builder) = match self {
            ElectrumUrl::Tls(url, validate) => {
                (format!("ssl://{}", url), builder.validate_domain(*validate))
            }
            ElectrumUrl::Plaintext(url) => (format!("tcp://{}", url), builder),
        };
        // Ok(_builder.build())
        Ok(electrum_client::Client::from_config(&url, builder.build()).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub network: BitcoinNetwork,
    pub electrum_url: ElectrumUrl,
    pub spv_enabled: bool,
    _policy_asset: Option<elements::issuance::AssetId>,
}

impl NetworkConfig {
    pub fn default() -> Result<Self, S5Error> {
        NetworkConfig::new(
            BitcoinNetwork::BitcoinTestnet,
            DEFAULT_TESTNET_NODE,
            false,
            true,
            false,
            None,
        )
    }
    pub fn default_liquid() -> Result<Self, S5Error> {
        NetworkConfig::new(
            BitcoinNetwork::LiquidTestnet,
            DEFAULT_LIQUID_TESTNET_NODE,
            false,
            true,
            false,
            Some(LIQUID_TESTNET_POLICY_ASSET_STR),
        )
    }
    pub fn new(
        network: BitcoinNetwork,
        electrum_url: &str,
        tls: bool,
        validate_domain: bool,
        spv_enabled: bool,
        policy_asset: Option<&str>,
    ) -> Result<Self, S5Error> {
        let electrum_url = match tls {
            true => ElectrumUrl::Tls(electrum_url.into(), validate_domain),
            false => ElectrumUrl::Plaintext(electrum_url.into()),
        };
        Ok(NetworkConfig {
            network: network,
            electrum_url,
            spv_enabled,
            _policy_asset: match policy_asset {
                Some(policy_asset) => Some(
                    elements::issuance::AssetId::from_slice(&hex::decode(policy_asset).unwrap())
                        .unwrap(),
                ),
                None => None,
            },
        })
    }

    pub fn network(&self) -> BitcoinNetwork {
        self.network
    }

    pub fn electrum_url(&self) -> ElectrumUrl {
        self.electrum_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{key::ec::BlindingKeyPair, swaps::liquid::script::LBtcRevSwapScript};

    use super::*;
    use bitcoin::{Script, ScriptBuf};
    use electrum_client::ElectrumApi;

    #[test]
    fn test_electrum_bitcoin_client() {
        let network_config = NetworkConfig::default().unwrap();
        let electrum_client = network_config.electrum_url.build_client().unwrap();
        assert!(electrum_client.ping().is_ok());
        // let res = electrum_client.server_features().unwrap();
        // println!("chain genesis block: {:#?}", res.genesis_hash);
    }
    #[test]
    #[ignore]
    fn test_electrum_liquid_client() {
        let redeem_script_str = "8201208763a9148514cc9235824c914d94fda549e45d6dec629b9788210223a99c57bfbc2a4bfc9353d49d6fd7312afaec8e8eefb82273d26c34c54589866775037ffe11b1752102869bf2e041d122d67b222d7b2fdc1e2466e726bbcacd35feccdfb0101cec359868ac".to_string();
        let expected_address = "tlq1qqtvg2v6wv2akxa8dpcdrfemgwnr09ragwlqagr57ezc8nzrvvd6x32rtt4s3e2xylcukuz64fm2zu0l4erdr2h98zjv07w4rearycpxqlz2gstkfw7ln";
        let blinding_key = BlindingKeyPair::from_secret_string(
            "bf99362dff7e8f2ec01e081215cab9047779da4547a6f47d67bb1cbb8c96961d".to_string(),
        );
        let script_elements = LBtcRevSwapScript::from_str(&redeem_script_str.clone()).unwrap();
        let script_address = script_elements.to_typed();
        let script_address_bitcoin: ScriptBuf =
            Script::from_bytes(script_address.as_bytes()).to_owned();
        let network_config = NetworkConfig::default_liquid().unwrap();
        let electrum_client = network_config.electrum_url.build_client().unwrap();
        let utxos = electrum_client
            .script_list_unspent(&script_address_bitcoin.to_v0_p2wsh())
            .unwrap();
        // assert!(electrum_client.ping().is_ok());
        // let res = electrum_client.server_features().unwrap();
        println!("UTXOS: {:#?}", utxos);
    }
}