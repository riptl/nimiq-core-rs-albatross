use crate::messages::Object;
use block_albatross::{Block, BlockComponents};
use std::convert::TryFrom;

pub enum TemporaryBlockchainState {
    MacroBlocks,
    MicroBlocks,
}

/// The temporary blockchain receives block announcements from the subscriptions
/// and checks whatever is possible to check at that point.
/// It also builds up the chain to by synced once the current target has been met.
struct TemporaryBlockchain {
    chain: Vec<Block>,
    state: TemporaryBlockchainState,
}

impl TemporaryBlockchain {
    fn on_announcement(&mut self, announcment: Object<BlockComponents>) {
        // When receiving an announcement ignore hashes for now.
        if let Object::Object(block_components) = announcment {
            if let Ok(block) = Block::try_from(block_components) {
                // TODO: Checks
                // Check block intrinsics.
                // Check signatures if possible.

                // Check if there is another block at that same height.
                // If so, check which one is better.
                if let Some(other) = self.block_at(block.block_number()) {
                    if block == other {
                        // The block is already known, all good.
                        return;
                    }

                    if block.view_number() < other.view_number() {
                        // The block has a lower view number, ignore.
                        return;
                    }
                }
                // If not check whether block extends previous block.
                self.chain.push(block);
            } else {
                // TODO: What to do here?
            }
        }
    }

    pub fn block_at(&self, height: u32) -> Option<&Block> {
        let first = self.first_block()?;
        self.chain.get((height - first.block_number()) as usize)
    }

    pub fn first_block(&self) -> Option<&Block> {
        self.chain.first()
    }
}
