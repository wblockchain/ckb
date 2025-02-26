use crate::LonelyBlockHash;
use ckb_logger::debug;
use ckb_store::{ChainDB, ChainStore};
use ckb_types::core::{BlockView, EpochNumber};
use ckb_types::packed;
use ckb_util::{parking_lot::RwLock, shrink_to_fit};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub type ParentHash = packed::Byte32;

const SHRINK_THRESHOLD: usize = 100;
pub const EXPIRED_EPOCH: u64 = 6;

#[derive(Default)]
struct InnerPool {
    // Group by blocks in the pool by the parent hash.
    blocks: HashMap<ParentHash, HashMap<packed::Byte32, LonelyBlockHash>>,
    // The map tells the parent hash when given the hash of a block in the pool.
    //
    // The block is in the orphan pool if and only if the block hash exists as a key in this map.
    parents: HashMap<packed::Byte32, ParentHash>,
    // Leaders are blocks not in the orphan pool but having at least a child in the pool.
    leaders: HashSet<ParentHash>,
}

impl InnerPool {
    fn with_capacity(capacity: usize) -> Self {
        InnerPool {
            blocks: HashMap::with_capacity(capacity),
            parents: HashMap::new(),
            leaders: HashSet::new(),
        }
    }

    fn insert(&mut self, lonely_block: LonelyBlockHash) {
        let hash = lonely_block.hash();
        let parent_hash = lonely_block.parent_hash();
        self.blocks
            .entry(parent_hash.clone())
            .or_default()
            .insert(hash.clone(), lonely_block);
        // Out-of-order insertion needs to be deduplicated
        self.leaders.remove(&hash);
        // It is a possible optimization to make the judgment in advance,
        // because the parent of the block must not be equal to its own hash,
        // so we can judge first, which may reduce one arc clone
        if !self.parents.contains_key(&parent_hash) {
            // Block referenced by `parent_hash` is not in the pool,
            // and it has at least one child, the new inserted block, so add it to leaders.
            self.leaders.insert(parent_hash.clone());
        }
        self.parents.insert(hash, parent_hash);
    }

    pub fn remove_blocks_by_parent(&mut self, parent_hash: &ParentHash) -> Vec<LonelyBlockHash> {
        // try remove leaders first
        if !self.leaders.remove(parent_hash) {
            return Vec::new();
        }

        let mut queue: VecDeque<packed::Byte32> = VecDeque::new();
        queue.push_back(parent_hash.to_owned());

        let mut removed: Vec<LonelyBlockHash> = Vec::new();
        while let Some(parent_hash) = queue.pop_front() {
            if let Some(orphaned) = self.blocks.remove(&parent_hash) {
                let (hashes, blocks): (Vec<_>, Vec<_>) = orphaned.into_iter().unzip();
                for hash in hashes.iter() {
                    self.parents.remove(hash);
                }
                queue.extend(hashes);
                removed.extend(blocks);
            }
        }

        debug!("orphan pool pop chain len: {}", removed.len());
        debug_assert_ne!(
            removed.len(),
            0,
            "orphan pool removed list must not be zero"
        );

        shrink_to_fit!(self.blocks, SHRINK_THRESHOLD);
        shrink_to_fit!(self.parents, SHRINK_THRESHOLD);
        shrink_to_fit!(self.leaders, SHRINK_THRESHOLD);
        removed
    }

    pub fn get_block(&self, hash: &packed::Byte32) -> Option<&LonelyBlockHash> {
        self.parents.get(hash).and_then(|parent_hash| {
            self.blocks
                .get(parent_hash)
                .and_then(|blocks| blocks.get(hash))
        })
    }

    /// cleanup expired blocks(epoch + EXPIRED_EPOCH < tip_epoch)
    pub fn clean_expired_blocks(&mut self, tip_epoch: EpochNumber) -> Vec<LonelyBlockHash> {
        let mut result = vec![];

        for hash in self.leaders.clone().iter() {
            if self.need_clean(hash, tip_epoch) {
                // remove items in orphan pool and return hash to callee(clean header map)
                let descendants = self.remove_blocks_by_parent(hash);
                result.extend(descendants);
            }
        }
        result
    }

    /// get 1st block belongs to that parent and check if it's expired block
    fn need_clean(&self, parent_hash: &packed::Byte32, tip_epoch: EpochNumber) -> bool {
        self.blocks
            .get(parent_hash)
            .and_then(|map| {
                map.iter().next().map(|(_, lonely_block)| {
                    lonely_block.epoch_number() + EXPIRED_EPOCH < tip_epoch
                })
            })
            .unwrap_or_default()
    }
}

// NOTE: Never use `LruCache` as container. We have to ensure synchronizing between
// orphan_block_pool and block_status_map, but `LruCache` would prune old items implicitly.
// RwLock ensures the consistency between maps. Using multiple concurrent maps does not work here.
#[derive(Default)]
pub struct OrphanBlockPool {
    inner: RwLock<InnerPool>,
}

impl OrphanBlockPool {
    pub fn with_capacity(capacity: usize) -> Self {
        OrphanBlockPool {
            inner: RwLock::new(InnerPool::with_capacity(capacity)),
        }
    }

    /// Insert orphaned block, for which we have already requested its parent block
    pub fn insert(&self, lonely_block: LonelyBlockHash) {
        self.inner.write().insert(lonely_block);
    }

    pub fn remove_blocks_by_parent(&self, parent_hash: &ParentHash) -> Vec<LonelyBlockHash> {
        self.inner.write().remove_blocks_by_parent(parent_hash)
    }

    pub fn get_block(&self, store: &ChainDB, hash: &packed::Byte32) -> Option<Arc<BlockView>> {
        let inner = self.inner.read();
        let lonely_block_hash: &LonelyBlockHash = inner.get_block(hash)?;
        store.get_block(&lonely_block_hash.hash()).map(Arc::new)
    }

    pub fn clean_expired_blocks(&self, epoch: EpochNumber) -> Vec<LonelyBlockHash> {
        self.inner.write().clean_expired_blocks(epoch)
    }

    pub fn len(&self) -> usize {
        self.inner.read().parents.len()
    }

    pub fn clone_leaders(&self) -> Vec<ParentHash> {
        self.inner.read().leaders.iter().cloned().collect()
    }

    #[cfg(test)]
    pub(crate) fn leaders_len(&self) -> usize {
        self.inner.read().leaders.len()
    }
}
