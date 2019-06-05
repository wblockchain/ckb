use ckb_chain_spec::consensus::Consensus;
use ckb_core::extras::EpochExt;
use ckb_core::header::{BlockNumber, Header};
use ckb_script::ScriptConfig;
use ckb_store::ChainStore;
use numext_fixed_hash::H256;
use std::sync::Arc;

pub trait ChainProvider: Sync + Send {
    type Store: ChainStore;

    fn store(&self) -> &Arc<Self::Store>;

    fn script_config(&self) -> &ScriptConfig;

    fn genesis_hash(&self) -> &H256;

    fn get_ancestor(&self, base: &H256, number: BlockNumber) -> Option<Header>;

    fn get_block_epoch(&self, hash: &H256) -> Option<EpochExt>;

    fn next_epoch_ext(&self, last_epoch: &EpochExt, header: &Header) -> Option<EpochExt>;

    fn consensus(&self) -> &Consensus;
}
