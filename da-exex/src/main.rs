use futures::{Future, TryStreamExt};
use alloy_rlp::Encodable;
use reth::revm::primitives::{bytes::BytesMut};
use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use reth_node_api::FullNodeComponents;
use reth_node_ethereum::EthereumNode;
use reth_tracing::tracing::info;
use std::{
    collections::VecDeque, pin::Pin, task::{ready, Context, Poll}, time::Duration
};
use futures_util::{FutureExt};
use std::sync::Arc;
use bytes::Bytes;

struct DAExEx<Node: FullNodeComponents> {
    /// The context of the ExEx
    ctx: ExExContext<Node>,
    /// Execution outcome of the chain
    api_client: Arc<reqwest::Client>,
    api_url: String,
    pending_request: Option<Pin<Box<dyn Future<Output = reqwest::Result<reqwest::Response>> + Send>>>,
    data_queue: VecDeque<Bytes>,
}

impl<Node: FullNodeComponents> DAExEx<Node> {
    /// Create a new instance of the ExEx
    fn new(ctx: ExExContext<Node>) -> Self {
        dotenvy::dotenv().unwrap();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build().unwrap();
        Self { ctx, api_client: Arc::new(client), api_url: dotenvy::var("DA_SERVER_URL").unwrap(), pending_request: None, data_queue: VecDeque::new() }
    }

        /// Try to start a new request if there isn't one pending
        fn try_start_request(&mut self) {
            if self.pending_request.is_none() && !self.data_queue.is_empty() {
                if let Some(data) = self.data_queue.pop_front() {
                    let future = Box::pin(
                        self.api_client
                            .post(&self.api_url)
                            .body(data)
                            .send()
                    );
                    self.pending_request = Some(future);
                }
            }
        }
}

impl<Node: FullNodeComponents + Unpin> Future for DAExEx<Node> {
    type Output = eyre::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let Some(request) = this.pending_request.as_mut() {
            match request.as_mut().poll(cx) {
                Poll::Ready(Ok(_)) => {
                    info!("Request sent successfully");
                    this.pending_request = None;
                    // Try to start the next request if there's data in the queue
                    this.try_start_request();
                }
                Poll::Ready(Err(e)) => {
                    info!("Request failed: {:?}", e);
                    // On error, put the data back in the queue to retry
                    // You might want to add retry limits and error handling here
                    this.pending_request = None;
                    this.try_start_request();
                }
                Poll::Pending => {}
            }
        }

        while let Some(notification) = ready!(this.ctx.notifications.try_next().poll_unpin(cx))? {
            match &notification {
                ExExNotification::ChainCommitted { new } => {
                    info!(committed_chain = ?new.range(), "Received commit");
                }
                ExExNotification::ChainReorged { old, new } => {
                    info!(from_chain = ?old.range(), to_chain = ?new.range(), "Received reorg");
                }
                ExExNotification::ChainReverted { old } => {
                    info!(reverted_chain = ?old.range(), "Received revert");
                }
            };

            if let Some(committed_chain) = notification.committed_chain() {
                // extend the state with the new chain
                let transactions = committed_chain
                    .blocks()
                    .values()
                    .flat_map(|block| block.body.transactions()).collect::<Vec<_>>();
                if !transactions.is_empty() {
                    let mut bytes_arr: BytesMut = BytesMut::new();
                    for tx in transactions {
                        let mut bytes = BytesMut::new();
                        tx.encode(&mut bytes);
                        bytes_arr.extend_from_slice(&bytes[..]);
                    }
                    // log bytes_arr using info!()
                    info!(bytes_arr = ?bytes_arr, "Bytes array");
                    let bytes = Bytes::from(bytes_arr);
                    this.data_queue.push_back(bytes);
                    // Try to start a new request if possible
                    this.try_start_request();
                }
                info!(first_block = %committed_chain.execution_outcome().first_block, "Processed committed blocks");
                this.ctx
                    .events
                    .send(ExExEvent::FinishedHeight(committed_chain.tip().num_hash()))?;
            }
        }

        Poll::Ready(Ok(()))
    }
}

fn main() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let handle = builder
            .node(EthereumNode::default())
            .install_exex("DA", |ctx| async move { Ok(DAExEx::new(ctx)) })
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
}
