use starknet_ff::FieldElement;

pub const ETH_RPC: &str = "http://127.0.0.1:8545";
pub const ETH_PRIV_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
pub const ROLLUP_SEQ_URL: &str = "http://127.0.0.1:9944";
pub const ROLLUP_PRIV_KEY: &str = "";
pub const ETH_CHAIN_ID: &str = "31337";
pub const L1_DEPLOYER_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
pub const L2_DEPLOYER_ADDRESS: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000004";
pub const L1_WAIT_TIME: &str = "15";

pub const FEE_TOKEN_ADDRESS: &str =
    "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
pub const MAX_FEE_OVERRIDE: &str = "0x100000";

// Need to use `from_mont` because this needs to be a constant function call
/// ChainId for Starknet Goerli testnet
pub const SN_GOERLI_CHAIN_ID: FieldElement = FieldElement::from_mont([
    3753493103916128178,
    18446744073709548950,
    18446744073709551615,
    398700013197595345,
]);
pub const SN_OS_PROGRAM_HASH: &str =
    "0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79";
pub const SPEC_VERSION: &str = "0.4.0";
pub const SN_OS_CONFIG_HASH_VERSION: &str = "StarknetOsConfig1";

pub const ANVIL_DEFAULT_PUBLIC_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
pub const ERC20_SIERRA_PATH: &str = "src/contracts/erc20.sierra.json";
pub const ERC20_CASM_PATH: &str = "src/contracts/erc20.casm.json";
pub const LEGACY_BRIDGE_PATH: &str = "src/contracts/legacy_token_bridge.json";
pub const TOKEN_BRIDGE_SIERRA_PATH: &str = "src/contracts/token_bridge.sierra.json";
pub const TOKEN_BRIDGE_CASM_PATH: &str = "src/contracts/token_bridge.casm.json";

pub const APP_CHAIN_ID: &str = "MADARA";
