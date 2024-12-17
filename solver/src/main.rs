use alloy::{
    consensus::Transaction,
    eips::BlockNumberOrTag,
    network::EthereumWallet,
    primitives::{Address, B256, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::BlockTransactionsKind,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolCall,
};
use alloy_provider::WalletProvider;
use parking_lot::RwLock;
use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs, sync::Arc};
use tokio::{self, try_join};
use RollupSettler::sendCall;

// Re-export contract types
sol!(
    #[sol(rpc)]
    NexusSettler,
    "../cli/artifacts/NexusSettler.json"
);

sol!(
    #[sol(rpc)]
    RollupSettler,
    "../cli/artifacts/RollupSettler.json"
);

sol!(
    #[sol(rpc)]
    MockERC20,
    "../cli/artifacts/MockERC20.json"
);

const PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

struct SharedState {
    orders: Arc<RwLock<HashMap<B256, Order>>>,
    order_hashes: Arc<RwLock<Vec<B256>>>,
}

#[derive(Debug, Clone)]
struct Order {
    fill_deadline: u32,
    from_token: Address,
    to_token: Address,
    sender: Address,
    recipient: Address,
    amount_in: U256,
    min_amount_out: U256,
    destination: U256,
    nonce: u32,
    order_hash: B256,
}

#[derive(Clone)]
struct ContractAddresses {
    nexus_settler: Address,
    rollup1_settler: Address,
    rollup2_settler: Address,
    rollup1_erc20: Address,
    rollup2_erc20: Address,
}

#[derive(Clone)]
struct Chains {
    nexus: ChainData,
    rollup1: ChainData,
    rollup2: ChainData,
}

#[derive(Clone)]
struct ChainData {
    http_url: String,
}

async fn load_contract_addresses(
) -> Result<ContractAddresses, Box<dyn std::error::Error + Send + Sync>> {
    let deployment_json = fs::read_to_string("chains/deployments/run-latest.json")?;
    let deployment: Value = serde_json::from_str(&deployment_json)?;

    Ok(ContractAddresses {
        nexus_settler: deployment["nexusSettler"]["address"]
            .as_str()
            .unwrap_or("Missing Nexus settler address")
            .parse::<Address>()?,
        rollup1_settler: deployment["rollup1Settler"]["address"]
            .as_str()
            .unwrap_or("Missing Rollup1 settler address")
            .parse::<Address>()?,
        rollup2_settler: deployment["rollup2Settler"]["address"]
            .as_str()
            .unwrap_or("Missing Rollup2 settler address")
            .parse::<Address>()?,
        rollup1_erc20: deployment["rollup1ERC20"]["address"]
            .as_str()
            .unwrap_or("Missing Rollup1 token address")
            .parse::<Address>()?,
        rollup2_erc20: deployment["rollup2ERC20"]["address"]
            .as_str()
            .unwrap_or("Missing Rollup2 token address")
            .parse::<Address>()?,
    })
}

#[derive(Deserialize)]
struct AppendResponse {
    index: u32,
}

async fn read_orders(
    state: Arc<SharedState>,
    chains: Chains,
    wallet: Arc<EthereumWallet>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&wallet)
        .on_http(chains.rollup1.http_url.parse::<Url>()?);
    let client = reqwest::Client::new();
    let mut from = http_provider.get_block_number().await?;
    let mut to = from;
    loop {
        println!("Checking blocks {} to {}", from, to);
        for i in from..(to + 1) {
            let block = http_provider
                .get_block_by_number(BlockNumberOrTag::from(i), BlockTransactionsKind::Full)
                .await?
                .unwrap();
            let mut hashes = block.transactions.hashes();
            // check block for send() transactions
            for tx in block.transactions.txns() {
                let tx = tx.clone();
                let data = tx.inner.input();
                let tx_hash = hashes.next().unwrap();
                if let Ok(decoded) = sendCall::abi_decode(data, false) {
                    println!("Decoded a tx!");
                    let receipt = http_provider
                        .get_transaction_receipt(tx_hash)
                        .await?
                        .unwrap();
                    if !receipt.inner.status() {
                        continue;
                    }
                    let order_hash = receipt.inner.logs()[0].data().data.clone();
                    let response = client
                        .post("http://127.0.0.1:3001/add")
                        .body(format!("{}", order_hash))
                        .send()
                        .await?;
                    let index = response.json::<AppendResponse>().await?;
                    let order = Order {
                        fill_deadline: decoded.fillDeadline,
                        from_token: decoded.fromToken,
                        to_token: decoded.toToken,
                        sender: tx.from,
                        recipient: decoded.recipient,
                        amount_in: decoded.amountIn,
                        min_amount_out: decoded.minAmountOut,
                        destination: decoded.destination,
                        nonce: index.index,
                        order_hash: B256::from_slice(&order_hash),
                    };
                    println!("Decoded order: {:?}", order);
                    {
                        let orders_guard = state.orders.try_write();
                        match orders_guard {
                            Some(mut orders) => {
                                println!("Got first write lock");
                                orders.insert(order.order_hash, order.clone());
                                println!("Inserted into orders");
                            }
                            None => println!("Failed to acquire write lock for orders!"),
                        }
                    }
                    println!("13. About to take second write lock");
                    {
                        let order_hashes_guard = state.order_hashes.try_write();
                        match order_hashes_guard {
                            Some(mut order_hashes) => {
                                println!("14. Got second write lock");
                                order_hashes.push(order.order_hash);
                                println!("15. Pushed to hashes");
                            }
                            None => println!("Failed to acquire write lock for order_hashes!"),
                        }
                    }
                    println!("Found send() transaction: {:?}", order);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        from = to + 1;
        to = http_provider.get_block_number().await?;
    }
}

#[derive(Deserialize)]
struct ProofResponse {
    proof: [B256; 32],
}

async fn fill_orders(
    state: Arc<SharedState>,
    chains: Chains,
    addresses: ContractAddresses,
    wallet: Arc<EthereumWallet>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let nexus_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&wallet)
        .on_http(chains.nexus.http_url.parse::<Url>()?);

    let rollup1_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&wallet)
        .on_http(chains.rollup1.http_url.parse::<Url>()?);

    let rollup2_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&wallet)
        .on_http(chains.rollup2.http_url.parse::<Url>()?);

    let nexus_settler = NexusSettler::new(addresses.nexus_settler, &nexus_provider);

    let rollup1_settler = RollupSettler::new(addresses.rollup1_settler, &rollup1_provider);

    let rollup2_settler = RollupSettler::new(addresses.rollup2_settler, &rollup2_provider);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&wallet)
        .on_http(chains.rollup2.http_url.parse::<Url>()?);

    let erc20 = MockERC20::new(addresses.rollup2_erc20, &provider);
    let settler = RollupSettler::new(addresses.rollup2_settler, &provider);

    println!("Getting orders to process");
    let orders_to_process: Vec<Order> = {
        let order_hashes = state.order_hashes.read();
        let orders = state.orders.read();
        order_hashes
            .iter()
            .filter_map(|hash| orders.get(hash).cloned())
            .collect()
    };
    println!("Got {} orders to process", orders_to_process.len());

    for order in &orders_to_process {
        println!("Filling order: {:?}", order.order_hash);

        erc20
            .mint(wallet.default_signer().address(), order.min_amount_out)
            .send()
            .await?
            .with_required_confirmations(1)
            .watch()
            .await?;
        println!("Approving ERC20 transfer");
        erc20
            .approve(*settler.address(), order.min_amount_out)
            .send()
            .await?
            .with_required_confirmations(1)
            .watch()
            .await?;
        println!("ERC20 approval complete");

        println!("Getting proof from server");
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:3001/query/{}", order.nonce))
            .send()
            .await?;
        println!("Got server response");
        let proof = response.json::<ProofResponse>().await?;
        println!("Parsed proof response");

        let order_root = rollup1_settler.orderRoot().call().await?._0;
        println!("Updating rollup order roots: {}", order_root);
        nexus_settler
            .updateRollupOrderRoot(U256::from(31338), order_root)
            .send()
            .await?
            .with_required_confirmations(1)
            .watch()
            .await?;

        rollup2_settler
            .updateRollupOrderRoot(U256::from(31338), order_root)
            .send()
            .await?
            .with_required_confirmations(1)
            .watch()
            .await?;

        println!("Calling fulfil on settler");
        let receipt = settler
            .fulfil(
                order.fill_deadline,
                order.from_token,
                order.to_token,
                order.sender,
                order.recipient,
                order.amount_in,
                order.min_amount_out,
                U256::from(31338),
                order.nonce,
                proof.proof,
            )
            .send()
            .await?
            .with_required_confirmations(1)
            .get_receipt()
            .await?;
        println!("Fulfil transaction sent: {:?}", receipt);
    }

    if !orders_to_process.is_empty() {
        {
            let orders_guard = state.orders.try_write();
            match orders_guard {
                Some(mut orders) => {
                    orders.clear();
                }
                None => println!("Failed to acquire write lock for orders!"),
            }
        }
        {
            let order_hashes_guard = state.order_hashes.try_write();
            match order_hashes_guard {
                Some(mut order_hashes) => {
                    order_hashes.clear();
                }
                None => println!("Failed to acquire write lock for order_hashes!"),
            }
        }
    }

    Ok(())
}

async fn monitor_transactions(
    state: Arc<SharedState>,
    chains: Chains,
    addresses: ContractAddresses,
    wallet: Arc<EthereumWallet>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        fill_orders(
            Arc::clone(&state),
            chains.clone(),
            addresses.clone(),
            wallet.clone(),
        )
        .await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(SharedState {
        orders: Arc::new(RwLock::new(HashMap::new())),
        order_hashes: Arc::new(RwLock::new(Vec::new())),
    });

    let chains: Chains = Chains {
        nexus: ChainData {
            http_url: "http://localhost:8545".to_string(),
        },
        rollup1: ChainData {
            http_url: "http://localhost:8546".to_string(),
        },
        rollup2: ChainData {
            http_url: "http://localhost:8547".to_string(),
        },
    };
    let addresses = load_contract_addresses().await?;
    let wallet = Arc::new(EthereumWallet::new(
        PRIVATE_KEY.parse::<PrivateKeySigner>()?,
    ));

    println!(
        "🧮 Starting solver with address: {:?}",
        wallet.default_signer().address()
    );

    let read_order_handle = tokio::spawn(read_orders(
        Arc::clone(&state),
        chains.clone(),
        Arc::clone(&wallet),
    ));
    let monitor_transactions_handle = tokio::spawn(monitor_transactions(
        state,
        chains.clone(),
        addresses,
        Arc::clone(&wallet),
    ));
    let (read_order_result, monitor_transactions_result) =
        try_join!(read_order_handle, monitor_transactions_handle)?;

    read_order_result?;
    monitor_transactions_result?;

    Ok(())
}
