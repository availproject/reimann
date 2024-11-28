use alloy::{
    network::{Ethereum, EthereumWallet, NetworkWallet, TransactionBuilder},
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use alloy_consensus::TxLegacy;
use alloy_primitives::{Address, Bytes, TxKind, B256};
use alloy_provider::{fillers::FillProvider, WalletProvider};
use alloy_signer::{Signer, SignerSync};
use alloy_transport_http::Http;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures::future::{join, join_all};
use reqwest::Url;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::{
    char::from_digit,
    process::{Command, Stdio},
};

sol!(
    #[sol(rpc)]
    NexusSettler,
    "artifacts/NexusSettler.json"
);

sol!(
    #[sol(rpc)]
    RollupSettler,
    "artifacts/RollupSettler.json"
);

sol!(
    #[sol(rpc)]
    MockERC20,
    "artifacts/MockERC20.json"
);

/// CLI tool for managing DA, rollup, and nexus nodes
#[derive(Parser)]
#[command(
    author = "QEDK",
    version,
    about = "A CLI tool for running Reimann",
    long_about = "This CLI tool provides commands to run and initialize various components of Reimann"
)]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run different components of the system
    #[command(
        about = "Run system components",
        long_about = "Run different components of the system such as DA server, rollup nodes, or nexus nodes. \
                     Each component can be configured with specific parameters."
    )]
    Run {
        #[command(subcommand)]
        component: RunCommands,
    },
    /// Genesis commands
    #[command(
        about = "Genesis file operations",
        long_about = "Commands for managing genesis files, including initialization and configuration."
    )]
    Genesis {
        #[command(subcommand)]
        action: GenesisCommands,
    },
    #[command(
        about = "Test operations",
        long_about = "Commands for testing functionality like transfers"
    )]
    Test {
        #[command(subcommand)]
        action: TestCommands,
    },
}

#[derive(Subcommand)]
enum RunCommands {
    /// Run DA server
    #[command(
        about = "Run the DA server",
        long_about = "Start the DA server using 'cargo run --bin da-server'"
    )]
    Da,

    /// Run SMT server
    #[command(
        about = "Run the SMT server",
        long_about = "Start the Sparse Merkle Tree server"
    )]
    Smt,

    /// Run rollup node
    #[command(
        about = "Run a rollup node",
        long_about = "Start a rollup node with specified configuration. \
                     The node will run with:\n\
                     - 200ms block time\n\
                     - 1B gas limit\n\
                     - 16ms builder interval\n\
                     - HTTP enabled\n\
                     - Custom chain directory based on name"
    )]
    Rollup {
        /// Name of the rollup instance (used for chain directory)
        #[arg(long, help = "Unique name for the rollup instance")]
        name: String,
        /// HTTP port for the nexus node
        #[arg(long, help = "HTTP port for the rollup node", default_value = "8546")]
        port: u16,
        #[arg(long, help = "P2P port for the rollup node", default_value = "30304")]
        p2p_port: u16,
        #[arg(
            long,
            help = "AuthRPC port for the rollup node",
            default_value = "8552"
        )]
        authrpc_port: u16,
    },

    /// Run nexus node
    #[command(
        about = "Run a nexus node",
        long_about = "Start a nexus node with specified configuration. \
                     The node will run with:\n\
                     - 2s block time\n\
                     - 1B gas limit\n\
                     - 166ms builder interval\n\
                     - HTTP enabled\n\
                     - Custom chain directory based on name"
    )]
    Nexus {
        /// Name of the nexus instance
        #[arg(
            long,
            help = "Unique name for the nexus instance",
            default_value = "nexus"
        )]
        name: String,
        /// HTTP port for the nexus node
        #[arg(long, help = "HTTP port for the nexus node", default_value = "8545")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum GenesisCommands {
    /// Initialize genesis files
    #[command(
        about = "Initialize genesis files",
        long_about = "Create genesis files for different chains with specific chain IDs (31337, 31338, 31339)"
    )]
    Init,
}

#[derive(Subcommand)]
enum TestCommands {
    /// Test transfers
    #[command(
        about = "Test transfer transactions",
        long_about = "Send multiple transfer transactions from the default account to itself"
    )]
    Transfers {
        /// Number of transfers to send
        #[arg(long, default_value = "1000")]
        count: u32,
        /// HTTP RPC endpoint
        #[arg(long, default_value = "http://127.0.0.1:8546")]
        rpc: String,
        /// Amount of ETH to send in each transfer
        #[arg(long, default_value = "1")]
        amount: f64,
    },
    Full {
        #[command(subcommand)]
        action: FullCommands,
    },
}

#[derive(Subcommand)]
enum FullCommands {
    #[command(
        about = "Init the initialization test",
        long_about = "Deploy and configure settler contracts across chains"
    )]
    Init,
    #[command(
        about = "Run the initialization test",
        long_about = "Send an intent and execute it across chains"
    )]
    Run,
}

struct ChainConfig {
    chain_id: u64,
    rpc: String,
}

async fn deploy_nexus_settler(rpc: String, wallet: &EthereumWallet) -> Result<Address> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let contract = NexusSettler::deploy(&provider).await?;

    println!("🧱 Deployed NexusSettler at: {}", contract.address());
    Ok(*contract.address())
}

async fn deploy_rollup_settler(rpc: String, wallet: &EthereumWallet) -> Result<Address> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let contract = RollupSettler::deploy(&provider).await?;

    println!("🧱 Deployed RollupSettler at: {}", contract.address());
    Ok(*contract.address())
}

async fn authorize_rollups(
    rpc: String,
    wallet: &EthereumWallet,
    contract_address: Address,
    chain_ids: Vec<u64>,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let contract = NexusSettler::new(contract_address, provider);
    for chain_id in chain_ids.clone() {
        contract
            .createRollup(U256::from(chain_id), Address::random())
            .send()
            .await?
            .register()
            .await?;
    }
    println!(
        "✍️ Authorized chain IDs {:?} on Nexus contract {}",
        chain_ids, contract_address
    );
    Ok(())
}

async fn authorize_rollup(
    rpc: String,
    wallet: &EthereumWallet,
    contract_address: Address,
    chain_id: u64,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let contract = RollupSettler::new(contract_address, provider);
    contract
        .authorizeRollup(U256::from(chain_id))
        .send()
        .await?
        .register()
        .await?;

    println!(
        "✍️ Authorized chain {} on contract {}",
        chain_id, contract_address
    );
    Ok(())
}

async fn save_deployments(
    nexus_settler: Address,
    rollup1_settler: Address,
    rollup2_settler: Address,
    rollup1_erc20: Address,
    rollup2_erc20: Address,
) -> Result<()> {
    let deployments = json!({
        "nexusSettler": {
            "address": format!("{:#x}", nexus_settler),
            "chainId": 31337
        },
        "rollup1Settler": {
            "address": format!("{:#x}", rollup1_settler),
            "chainId": 31338
        },
        "rollup2Settler": {
            "address": format!("{:#x}", rollup2_settler),
            "chainId": 31339,
        },
        "rollup1ERC20": {
            "address": format!("{:#x}", rollup1_erc20),
            "chainId": 31338
        },
        "rollup2ERC20": {
            "address": format!("{:#x}", rollup2_erc20),
            "chainId": 31339
        }
    });

    // Create deployments directory if it doesn't exist
    let deployments_dir = PathBuf::from("chains/deployments");
    fs::create_dir_all(&deployments_dir).context("❌ Failed to create deployments directory")?;

    // Write to run-latest.json
    let file_path = deployments_dir.join("run-latest.json");
    fs::write(
        &file_path,
        serde_json::to_string_pretty(&deployments)
            .context("Failed to serialize deployments JSON")?,
    )
    .with_context(|| format!("❌ Failed to write deployments to {}", file_path.display()))?;

    println!("📄 Deployments saved to {}", file_path.display());
    Ok(())
}

async fn deploy_erc20(rpc: String, wallet: &EthereumWallet) -> Result<Address> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let contract = MockERC20::deploy(&provider).await?;

    println!("🧱 Deployed MockERC20 at: {}", contract.address());
    Ok(*contract.address())
}

async fn mint_tokens(
    rpc: &str,
    wallet: &EthereumWallet,
    token: Address,
    amount: U256,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let address = provider.default_signer_address();
    let erc20 = MockERC20::new(token, provider);
    erc20.mint(address, amount).send().await?.register().await?;

    println!(
        "🪙 Minted ERC20 tokens with address {} to: {}",
        token, address
    );
    Ok(())
}

async fn create_order(
    rpc: &str,
    wallet: &EthereumWallet,
    settler: Address,
    fill_deadline: u32,
    from_token: Address,
    to_token: Address,
    recipient: Address,
    amount_in: U256,
    min_amount_out: U256,
    destination: u64,
) -> Result<u32> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let erc20 = MockERC20::new(from_token, &provider);
    let settler = RollupSettler::new(settler, &provider);
    erc20
        .approve(*settler.address(), amount_in)
        .send()
        .await?
        .with_required_confirmations(1)
        .watch()
        .await?;
    let receipt = settler
        .send(
            fill_deadline,
            from_token,
            to_token,
            recipient,
            amount_in,
            min_amount_out,
            U256::from(destination),
        )
        .send()
        .await?
        .get_receipt()
        .await?;
    let order_hash = &receipt.inner.logs()[0].data().data;
    println!(
        "📤 Created order {} on RollupSettler {} to chain {}",
        order_hash,
        *settler.address(),
        destination
    );
    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:3001/add")
        .body(format!("{}", order_hash))
        .send()
        .await?;
    #[derive(Deserialize)]
    struct Response {
        success: bool,
        root: B256,
        index: u32,
    }
    // parse res into response
    let res = res.json::<Response>().await?;
    Ok(res.index)
}

async fn update_nexus_order_root(
    rpc: &str,
    wallet: &EthereumWallet,
    settler: Address,
    chain_id: u64,
    root: B256,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let settler = NexusSettler::new(settler, &provider);
    settler
        .updateRollupOrderRoot(U256::from(chain_id), root)
        .send()
        .await?
        .with_required_confirmations(1)
        .watch()
        .await?;

    println!(
        "⬆️  Updated order root {} for chain {} on NexusSettler {}",
        root,
        chain_id,
        *settler.address()
    );
    Ok(())
}

async fn update_rollup_order_root(
    rpc: &str,
    wallet: &EthereumWallet,
    settler: Address,
    chain_id: u64,
    root: B256,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let settler = RollupSettler::new(settler, &provider);
    settler
        .updateRollupOrderRoot(U256::from(chain_id), root)
        .send()
        .await?
        .with_required_confirmations(1)
        .watch()
        .await?;

    println!(
        "⬆️  Updated order root {} for chain {} on RollupSettler {}",
        root,
        chain_id,
        *settler.address()
    );
    Ok(())
}

async fn fulfill_order(
    rpc: &str,
    wallet: &EthereumWallet,
    fill_deadline: u32,
    settler: Address,
    from_token: Address,
    to_token: Address,
    amount: U256,
    min_amount: U256,
    source_chain: u64,
    nonce: u32,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<Url>()?);

    let erc20 = MockERC20::new(to_token, &provider);
    let settler = RollupSettler::new(settler, &provider);
    let sender = provider.default_signer_address();
    erc20
        .approve(*settler.address(), amount)
        .send()
        .await?
        .with_required_confirmations(1)
        .watch()
        .await?;
    // query proof from smt-server
    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://127.0.0.1:3001/query/{}", nonce))
        .send()
        .await?;
    #[derive(Deserialize)]
    struct Response {
        success: bool,
        proof: [B256; 32],
        root: B256,
    }
    let res = res.json::<Response>().await?;
    let receipt = settler
        .fulfil(
            fill_deadline,
            from_token,
            to_token,
            sender,
            sender,
            amount,
            min_amount,
            U256::from(source_chain),
            nonce,
            res.proof,
        )
        .send()
        .await?
        .get_receipt()
        .await?;
    let order_hash = &receipt.inner.logs()[0].data().data;

    println!(
        "📥 Fulfilled order {} on RollupSettler {} from chain {}",
        order_hash,
        *settler.address(),
        source_chain
    );
    Ok(())
}

async fn test_full_init() -> Result<()> {
    // Default private key
    let signer: PrivateKeySigner =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse::<PrivateKeySigner>()?;
    let wallet = EthereumWallet::from(signer.clone());

    let chains = [
        ChainConfig {
            chain_id: 31337,
            rpc: "http://127.0.0.1:8545".into(),
        },
        ChainConfig {
            chain_id: 31338,
            rpc: "http://127.0.0.1:8546".into(),
        },
        ChainConfig {
            chain_id: 31339,
            rpc: "http://127.0.0.1:8547".into(),
        },
    ];

    let start = std::time::Instant::now();

    // Deploy NexusSettler to chain 31337
    let nexus_settler = deploy_nexus_settler(chains[0].rpc.clone(), &wallet).await?;

    // Deploy RollupSettlers
    let rollup1_settler = deploy_rollup_settler(chains[1].rpc.clone(), &wallet).await?;
    let rollup2_settler = deploy_rollup_settler(chains[2].rpc.clone(), &wallet).await?;

    let rollup1_erc20 = deploy_erc20(chains[1].rpc.clone(), &wallet).await?;
    let rollup2_erc20 = deploy_erc20(chains[2].rpc.clone(), &wallet).await?;

    // Mint tokens for rollups
    mint_tokens(
        chains[1].rpc.as_str(),
        &wallet,
        rollup1_erc20,
        U256::from(1e28),
    )
    .await?;
    mint_tokens(
        chains[2].rpc.as_str(),
        &wallet,
        rollup2_erc20,
        U256::from(1e28),
    )
    .await?;

    // Authorize rollups on NexusSettler
    authorize_rollups(
        chains[0].rpc.clone(),
        &wallet,
        nexus_settler,
        vec![31338, 31339],
    )
    .await?;

    // Cross-authorize rollups
    authorize_rollup(chains[1].rpc.clone(), &wallet, rollup1_settler, 31339).await?;
    authorize_rollup(chains[2].rpc.clone(), &wallet, rollup2_settler, 31338).await?;

    save_deployments(
        nexus_settler,
        rollup1_settler,
        rollup2_settler,
        rollup1_erc20,
        rollup2_erc20,
    )
    .await?;

    let elapsed = start.elapsed();
    println!(
        "✅ Full initialization completed successfully in {}s",
        elapsed.as_secs_f64()
    );
    Ok(())
}

async fn test_full_run() -> Result<()> {
    // Read deployments
    let deployments_file = fs::read_to_string("chains/deployments/run-latest.json")
        .context("Failed to read deployments file")?;
    let deployments: serde_json::Value =
        serde_json::from_str(&deployments_file).context("Failed to parse deployments file")?;

    let nexus_settler_addr = Address::from_str(
        deployments["nexusSettler"]["address"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid nexus settler address"))?,
    )?;
    let rollup1_settler_addr = Address::from_str(
        deployments["rollup1Settler"]["address"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid rollup1 settler address"))?,
    )?;
    let rollup2_settler_addr = Address::from_str(
        deployments["rollup2Settler"]["address"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid rollup2 settler address"))?,
    )?;
    let rollup1_erc20_addr = Address::from_str(
        deployments["rollup1ERC20"]["address"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid rollup1 ERC20 address"))?,
    )?;
    let rollup2_erc20_addr = Address::from_str(
        deployments["rollup2ERC20"]["address"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid rollup2 ERC20 address"))?,
    )?;

    let signer: PrivateKeySigner =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse::<PrivateKeySigner>()?;
    let wallet = EthereumWallet::from(signer.clone());

    let address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?;
    let amount_wei = U256::from(1e18 as u64);

    let rollup1_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&wallet)
        .on_http("http://127.0.0.1:8546".parse::<reqwest::Url>()?);

    let rollup1_settler = RollupSettler::new(rollup1_settler_addr, rollup1_provider.clone());

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as u32;
    let start = std::time::Instant::now();

    let nonce = create_order(
        "http://127.0.0.1:8546",
        &wallet,
        rollup1_settler_addr,
        timestamp + 14, // 14 seconds because block.timestamp gets a bit off
        rollup1_erc20_addr,
        rollup2_erc20_addr,
        address,
        amount_wei,
        amount_wei,
        31339,
    )
    .await?;

    let order_root = rollup1_settler.orderRoot().call().await?._0;

    // Update roots across chains
    update_nexus_order_root(
        "http://127.0.0.1:8545",
        &wallet,
        nexus_settler_addr,
        31338,
        order_root,
    )
    .await?;

    update_rollup_order_root(
        "http://127.0.0.1:8547",
        &wallet,
        rollup2_settler_addr,
        31338,
        order_root,
    )
    .await?;

    // Fulfill order on rollup2
    fulfill_order(
        "http://127.0.0.1:8547",
        &wallet,
        timestamp + 14,
        rollup2_settler_addr,
        rollup1_erc20_addr,
        rollup2_erc20_addr,
        amount_wei, // 1 token
        amount_wei, // 1 token minimum
        31338,
        nonce, // rollup1 chain id
    )
    .await?;
    let elapsed = start.elapsed();

    println!(
        "✅ Full run test completed successfully in {}s",
        elapsed.as_secs_f64()
    );
    Ok(())
}

fn run_da() -> Result<()> {
    Command::new("cargo")
        .args(["run", "--bin", "da-server"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start DA server")?
        .wait()
        .context("Failed to wait for DA server")?;
    Ok(())
}

fn run_smt() -> Result<()> {
    Command::new("cargo")
        .args(["run", "--bin", "smt-server"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start SMT server")?
        .wait()
        .context("Failed to wait for SMT server")?;
    Ok(())
}

fn run_rollup(name: &str, port: u16, p2p_port: u16, authrpc_port: u16) -> Result<()> {
    Command::new("cargo")
        .args([
            "run",
            "--bin",
            "da-exex",
            "--",
            "node",
            "--chain",
            &format!("chains/genesis/{}.json", name),
            "--dev",
            "--dev.block-time",
            "200ms",
            "--builder.gaslimit",
            "1000000000",
            "--builder.interval",
            "16ms",
            "--builder.extradata",
            "da-exex",
            "--http",
            "--http.port",
            &port.to_string(),
            "--datadir",
            &format!("chains/{}", name),
            "--no-persist-peers",
            "--port",
            &p2p_port.to_string(),
            "--authrpc.port",
            &authrpc_port.to_string(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start rollup node")?
        .wait()
        .context("Failed to wait for rollup node")?;
    Ok(())
}

fn run_nexus(name: &str, port: u16) -> Result<()> {
    Command::new("cargo")
        .args([
            "run",
            "--bin",
            "da-exex",
            "--",
            "node",
            "--chain",
            &format!("chains/genesis/{}.json", name),
            "--dev",
            "--dev.block-time",
            "2s",
            "--builder.gaslimit",
            "1000000000",
            "--builder.interval",
            "166ms",
            "--builder.extradata",
            "da-exex",
            "--http",
            "--http.port",
            &port.to_string(),
            "--datadir",
            &format!("chains/{}", name),
            "--no-persist-peers",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start nexus node")?
        .wait()
        .context("Failed to wait for nexus node")?;
    Ok(())
}

fn create_genesis_files() -> Result<()> {
    let base_genesis: Value = json!({
        "config": {
            "homesteadBlock": 0,
            "eip150Block": 0,
            "eip150Hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "eip155Block": 0,
            "eip158Block": 0,
            "byzantiumBlock": 0,
            "constantinopleBlock": 0,
            "petersburgBlock": 0,
            "istanbulBlock": 0,
            "muirGlacierBlock": 0,
            "berlinBlock": 0,
            "londonBlock": 0,
            "arrowGlacierBlock": 0,
            "grayGlacierBlock": 0,
            "mergeNetsplitBlock": 0,
            "shanghaiTime": 0,
            "terminalTotalDifficulty": "0x",
            "clique": {
                "period": 0,
                "epoch": 30000
            }
        },
        "nonce": "0x",
        "timestamp": "0x",
        "extraData": "0x",
        "gasLimit": "0x3b9aca00",
        "difficulty": "0x1",
        "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "coinbase": "0x0000000000000000000000000000000000000000",
        "number": "0x",
        "gasUsed": "0x",
        "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "baseFeePerGas": "0x7",
        "alloc": {
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266": {
                "balance": "0x200000000000000000000000000000000000000000000000000000000000000"
            },
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8": {
                "balance": "0x200000000000000000000000000000000000000000000000000000000000000"
            },
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC": {
                "balance": "0x200000000000000000000000000000000000000000000000000000000000000"
            },
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906": {
                "balance": "0x200000000000000000000000000000000000000000000000000000000000000"
            }
        }
    });

    // Create chains/genesis directory if it doesn't exist
    let genesis_dir = PathBuf::from("chains/genesis");
    fs::create_dir_all(&genesis_dir).context("❌ Failed to create genesis directory")?;

    // Create genesis files for different chain IDs
    let chain_configs = [("nexus", 31337), ("rollup1", 31338), ("rollup2", 31339)];

    for (name, chain_id) in chain_configs.iter() {
        let mut genesis = base_genesis.clone();
        genesis["config"]["chainId"] = json!(chain_id);

        let file_path = genesis_dir.join(format!("{}.json", name));
        fs::write(
            &file_path,
            serde_json::to_string_pretty(&genesis).context("Failed to serialize genesis JSON")?,
        )
        .with_context(|| format!("❌ Failed to write genesis file for {}", name))?;

        println!(
            "📜 Created genesis file for {} with chain ID {} at {:?}",
            name, chain_id, file_path
        );
    }

    Ok(())
}

async fn test_transfers(count: u32, rpc: &str, amount: f64) -> Result<()> {
    // Default private key for account: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    let signer: PrivateKeySigner =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse::<PrivateKeySigner>()?;
    let wallet = EthereumWallet::from(signer.clone());
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc.parse::<reqwest::Url>()?);

    let address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?;
    let amount_wei = U256::from((amount * 1e18) as u64);

    println!(
        "📩 Starting {} transfers of {} ETH each to {}",
        count, amount, address
    );

    let nonce = provider.get_transaction_count(address).await?;
    println!("ℹ Current nonce: {}", nonce);
    let chain_id = provider.get_chain_id().await?;

    let mut futures = vec![];
    let start = std::time::Instant::now();
    for i in 0..count {
        let tx = TransactionRequest::default()
            .with_to(address)
            .with_nonce(nonce + i as u64)
            .with_chain_id(chain_id)
            .with_value(amount_wei)
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1)
            .with_max_fee_per_gas(20_000_000_000);
        futures.push(provider.send_transaction(tx).await?.register());
    }
    join_all(futures).await;
    let elapsed = start.elapsed();
    println!(
        "✅ All transfer requests completed, time elapsed: {}s",
        elapsed.as_secs_f64()
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { component } => match component {
            RunCommands::Da => run_da()?,
            RunCommands::Smt => run_smt()?,
            RunCommands::Rollup {
                name,
                port,
                p2p_port,
                authrpc_port,
            } => run_rollup(&name, port, p2p_port, authrpc_port)?,
            RunCommands::Nexus { name, port } => run_nexus(&name, port)?,
        },
        Commands::Genesis { action } => match action {
            GenesisCommands::Init => create_genesis_files()?,
        },
        Commands::Test { action } => match action {
            TestCommands::Transfers { count, rpc, amount } => {
                test_transfers(count, &rpc, amount).await?
            }
            TestCommands::Full { action } => match action {
                FullCommands::Init => test_full_init().await?,
                FullCommands::Run => test_full_run().await?,
            },
        },
    }

    Ok(())
}
