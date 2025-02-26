#![no_main]
use libfuzzer_sys::fuzz_target;

use ckb_chain_spec::consensus::ConsensusBuilder;
use ckb_script::{TransactionScriptsVerifier, TxVerifyEnv};
use ckb_traits::{CellDataProvider, ExtensionProvider, HeaderProvider};
use ckb_types::{
    bytes::Bytes,
    core::{
        capacity_bytes,
        cell::{CellMetaBuilder, ResolvedTransaction},
        hardfork::{HardForks, CKB2021, CKB2023},
        Capacity, HeaderView, ScriptHashType, TransactionBuilder, TransactionInfo,
    },
    h256,
    packed::{
        self, Byte32, CellInput, CellOutput, CellOutputBuilder, OutPoint, Script,
        TransactionInfoBuilder, TransactionKeyBuilder,
    },
    prelude::*,
};

#[derive(Default, PartialEq, Eq, Clone)]
struct MockDataLoader {}

impl CellDataProvider for MockDataLoader {
    fn get_cell_data(&self, _out_point: &OutPoint) -> Option<Bytes> {
        None
    }

    fn get_cell_data_hash(&self, _out_point: &OutPoint) -> Option<Byte32> {
        None
    }
}

impl HeaderProvider for MockDataLoader {
    fn get_header(&self, _block_hash: &Byte32) -> Option<HeaderView> {
        None
    }
}

impl ExtensionProvider for MockDataLoader {
    fn get_block_extension(&self, _hash: &Byte32) -> Option<packed::Bytes> {
        None
    }
}

fn mock_transaction_info() -> TransactionInfo {
    TransactionInfoBuilder::default()
        .block_number(1u64.pack())
        .block_epoch(0u64.pack())
        .key(
            TransactionKeyBuilder::default()
                .block_hash(Byte32::zero())
                .index(1u32.pack())
                .build(),
        )
        .build()
        .unpack()
}

static PROGRAM_DATA: &[u8] = include_bytes!("../../testdata/spawn_fuzzing");

fn run(data: &[u8]) {
    if data.len() < 8 {
        return;
    }
    let split_offset = data[0] as usize;
    let split_offset = usize::min(split_offset, data.len() - 1);
    let parent_witness = Bytes::copy_from_slice(&data[0..split_offset]);
    let child_witness = Bytes::copy_from_slice(&data[split_offset..]);
    let witnesses = vec![parent_witness.pack(), child_witness.pack()];

    let transaction = TransactionBuilder::default()
        .input(CellInput::new(OutPoint::null(), 0))
        .set_witnesses(witnesses)
        .build();

    let data: Bytes = (Vec::from(PROGRAM_DATA)).into();
    let script = Script::new_builder()
        .hash_type(ScriptHashType::Data2.into())
        .code_hash(CellOutput::calc_data_hash(&data))
        .build();
    let dep_cell = CellMetaBuilder::from_cell_output(
        CellOutput::new_builder()
            .capacity(Capacity::bytes(data.len()).unwrap().pack())
            .build(),
        data,
    )
    .transaction_info(mock_transaction_info())
    .out_point(OutPoint::new(h256!("0x0").pack(), 0))
    .build();

    let input_cell = CellMetaBuilder::from_cell_output(
        CellOutputBuilder::default()
            .capacity(capacity_bytes!(100).pack())
            .lock(script)
            .build(),
        Bytes::new(),
    )
    .transaction_info(mock_transaction_info())
    .build();

    let rtx = ResolvedTransaction {
        transaction,
        resolved_cell_deps: vec![dep_cell],
        resolved_inputs: vec![input_cell],
        resolved_dep_groups: vec![],
    };

    let provider = MockDataLoader {};
    let hardfork_switch = HardForks {
        ckb2021: CKB2021::new_mirana().as_builder().build().unwrap(),
        ckb2023: CKB2023::new_mirana()
            .as_builder()
            .rfc_0049(0)
            .build()
            .unwrap(),
    };
    let consensus = ConsensusBuilder::default()
        .hardfork_switch(hardfork_switch)
        .build();
    let tx_verify_env =
        TxVerifyEnv::new_submit(&HeaderView::new_advanced_builder().epoch(0.pack()).build());
    let verifier = TransactionScriptsVerifier::new(
        rtx.into(),
        provider,
        consensus.into(),
        tx_verify_env.into(),
    );
    let _ = verifier.verify(70_000_000);
}

fuzz_target!(|data: &[u8]| {
    run(data);
});
