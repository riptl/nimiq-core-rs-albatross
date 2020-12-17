use std::{
    task::{Context, Poll},
    sync::Arc,
    collections::BTreeMap,
    pin::Pin,
};

use futures::stream::{
    BoxStream, Stream,
};
use pin_project::pin_project;

use nimiq_blockchain_albatross::Blockchain;
use nimiq_block_albatross::Block;
use nimiq_network_interface::network::Topic;
use nimiq_primitives::policy;

// mock
#[derive(Clone, Debug, Default)]
pub struct PeerTrackingAndRequestComponent;

impl Stream for PeerTrackingAndRequestComponent {
    type Item = Vec<Block>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}


pub struct BlockTopic;

impl Topic for BlockTopic {
    type Item = Block;

    fn topic(&self) -> String {
        "blocks".to_owned()
    }
}


pub type BlockStream = BoxStream<'static, Block>;


#[derive(Clone, Debug)]
pub struct BlockQueueConfig {
    /// Buffer size limit
    buffer_max: usize,

    /// How many blocks ahead we will buffer.
    window_max: u32,

    /// The min size a gap at the head needs to be, to issue a request for these missing blocks. If `None`, doesn't
    /// request any gaps.
    request_gaps: Option<usize>,
}

impl Default for BlockQueueConfig {
    fn default() -> Self {
        Self {
            buffer_max: 4 * policy::BATCH_LENGTH as usize,
            window_max: 2 * policy::BATCH_LENGTH,
            request_gaps: None,
        }
    }
}


struct Inner {
    /// Configuration for the block queue
    config: BlockQueueConfig,

    /// Reference to the block chain
    blockchain: Arc<Blockchain>,

    /// Buffered blocks - `block_height -> [Block]`. There can be multiple blocks at a height if there are forks.
    ///
    /// # TODO
    ///
    ///  - The inner `Vec` should really be a `SmallVec<[Block; 1]>` or similar.
    ///
    buffer: BTreeMap<u32, Vec<Block>>,
}

impl Inner {
    fn on_block_announced(&mut self, block: Block) {
        let block_height = block.block_number();
        let head_height = self.blockchain.block_number();

        if block_height <= head_height {
            // Fork block
            self.push_block(block);
        }
        else if block_height == head_height + 1 {
            // New head block
            self.push_block(block);
            self.push_buffered();
        }
        else if block_height > head_height + self.config.window_max {
            log::warn!(
                "Discarding block #{} outside of buffer window (max {}).",
                 block_height,
                 head_height + self.config.window_max,
             );
        }
        else if self.buffer.len() >= self.config.buffer_max {
            log::warn!(
                "Discarding block #{}, buffer full (max {})",
                block_height,
                self.buffer.len(),
            )
        }
        else {
            // Block inside buffer window
            self.insert_into_buffer(block);

            if let Some(_n) = self.config.request_gaps {
                // Request missing blocks, if the gap is sufficiently large, or after a timeout?
                todo!()
                //self.request_component.request_missing_blocks()
            }
        }
    }

    fn on_missing_blocks_received(&mut self, _blocks: Vec<Block>) {
        todo!();
    }

    fn push_block(&mut self, block: Block) {
        match self.blockchain.push(block) {
            Ok(result) => log::debug!("Block pushed: {:?}", result),
            Err(e) => log::error!("Failed to push block: {}", e),
        }
    }

    fn push_buffered(&mut self) {
        loop {
            let head_height = self.blockchain.block_number();
            log::trace!("head_height = {}", head_height);

            // Check if queued block can be pushed to block chain
            if let Some(entry) = self.buffer.first_entry() {
                log::trace!("first entry: {:?}", entry);

                if *entry.key() > head_height + 1 {
                    break;
                }

                // Pop block from queue
                let (_, blocks) = entry.remove_entry();

                // If we get a Vec from the BTree, it must not be empty
                assert!(!blocks.is_empty());

                for block in blocks {
                    log::trace!(
                        "Pushing block #{} (currently at #{}, {} blocks left)",
                        block.block_number(),
                        head_height,
                        self.buffer.len(),
                    );
                    self.push_block(block);
                }
            }
            else {
                break;
            }
        }
    }

    fn insert_into_buffer(&mut self, block: Block) {
        self.buffer.entry(block.block_number())
            .or_default()
            .push(block)
    }
}


#[pin_project]
pub struct BlockQueue {
    /// The Peer Tracking and Request Component.
    #[pin]
    request_component: PeerTrackingAndRequestComponent,

    /// The blocks received via gossipsub.
    #[pin]
    block_stream: BlockStream,

    /// The inner state of the block queue.
    inner: Inner,
}

impl BlockQueue {
    pub fn new(config: BlockQueueConfig, blockchain: Arc<Blockchain>, request_component: PeerTrackingAndRequestComponent, block_stream: BlockStream) -> Self {
        let buffer = BTreeMap::new();

        Self {
            request_component,
            block_stream,
            inner: Inner {
                config,
                blockchain,
                buffer,
            }
        }
    }

    /// Returns an iterator over the buffered blocks
    pub fn buffered_blocks(&self) -> impl Iterator<Item=(u32, &[Block])> {
        self.inner.buffer.iter().map(|(block_number, blocks)| (*block_number, blocks.as_ref()))
    }
}

impl Stream for BlockQueue {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // Note: I think it doesn't matter what is done first

        // First, try to get as many blocks from the gossipsub stream as possible
        match this.block_stream.poll_next(cx) {
            Poll::Ready(Some(block)) => {
                this.inner.on_block_announced(block);
                return Poll::Ready(Some(()));
            },

            // If the block_stream is exhausted, we quit as well
            Poll::Ready(None) => return Poll::Ready(None),

            Poll::Pending => {},
        }

        // Then, read all the responses we got for our missing blocks requests
        match this.request_component.poll_next(cx) {
            Poll::Ready(Some(blocks)) => {
                this.inner.on_missing_blocks_received(blocks);
                return Poll::Ready(Some(()))
            },
            Poll::Ready(None) => panic!("The request_component stream is exhausted"),
            Poll::Pending => {},
        }

        Poll::Pending
    }
}
