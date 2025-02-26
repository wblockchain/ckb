use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder},
    packed::{CellInput, OutPoint},
    prelude::*,
};

use crate::component::{entry::TxEntry, pool_map::PoolMap, sort_key::AncestorsScoreSortKey};

const DEFAULT_MAX_ANCESTORS_COUNT: usize = 125;

#[test]
fn test_min_fee_and_weight() {
    let result = vec![
        (0, 0, 0, 0),
        (1, 0, 1, 0),
        (500, 10, 1000, 30),
        (10, 500, 30, 1000),
        (500, 10, 1000, 20),
        (u64::MAX, 0, u64::MAX, 0),
        (u64::MAX, 100, u64::MAX, 2000),
        (u64::MAX, u64::MAX, u64::MAX, u64::MAX),
    ]
    .into_iter()
    .map(|(fee, weight, ancestors_fee, ancestors_weight)| {
        let key = AncestorsScoreSortKey {
            fee: Capacity::shannons(fee),
            weight,
            ancestors_fee: Capacity::shannons(ancestors_fee),
            ancestors_weight,
        };
        key.min_fee_and_weight()
    })
    .collect::<Vec<_>>();
    assert_eq!(
        result,
        vec![
            (Capacity::shannons(0), 0),
            (Capacity::shannons(1), 0),
            (Capacity::shannons(1000), 30),
            (Capacity::shannons(10), 500),
            (Capacity::shannons(1000), 20),
            (Capacity::shannons(u64::MAX), 0),
            (Capacity::shannons(u64::MAX), 2000),
            (Capacity::shannons(u64::MAX), u64::MAX),
        ]
    );
}

#[test]
fn test_ancestors_sorted_key_order() {
    let table = vec![
        (0, 0, 0, 0),
        (1, 0, 1, 0),
        (500, 10, 1000, 30),
        (10, 500, 30, 1000),
        (500, 10, 1000, 30),
        (10, 500, 30, 1000),
        (500, 10, 1000, 20),
        (u64::MAX, 0, u64::MAX, 0),
        (u64::MAX, 100, u64::MAX, 2000),
        (u64::MAX, u64::MAX, u64::MAX, u64::MAX),
    ];
    let mut keys = table
        .clone()
        .into_iter()
        .map(
            |(fee, weight, ancestors_fee, ancestors_weight)| AncestorsScoreSortKey {
                fee: Capacity::shannons(fee),
                weight,
                ancestors_fee: Capacity::shannons(ancestors_fee),
                ancestors_weight,
            },
        )
        .collect::<Vec<_>>();
    keys.sort();
    let now = keys
        .into_iter()
        .map(|k| (k.fee, k.weight, k.ancestors_fee, k.ancestors_weight))
        .collect::<Vec<_>>();
    let expect = [0, 3, 5, 9, 2, 4, 6, 8, 1, 7]
        .iter()
        .map(|&i| {
            let key = table[i as usize];
            (
                Capacity::shannons(key.0),
                key.1,
                Capacity::shannons(key.2),
                key.3,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(now, expect);
}

#[test]
fn test_remove_entry() {
    let mut map = PoolMap::new(DEFAULT_MAX_ANCESTORS_COUNT);
    let tx1 = TxEntry::dummy_resolve(
        TransactionBuilder::default().build(),
        100,
        Capacity::shannons(100),
        100,
    );
    let tx2 = TxEntry::dummy_resolve(
        TransactionBuilder::default()
            .input(
                CellInput::new_builder()
                    .previous_output(
                        OutPoint::new_builder()
                            .tx_hash(tx1.transaction().hash())
                            .index(0u32.pack())
                            .build(),
                    )
                    .build(),
            )
            .witness(Bytes::new().pack())
            .build(),
        200,
        Capacity::shannons(200),
        200,
    );
    let tx3 = TxEntry::dummy_resolve(
        TransactionBuilder::default()
            .input(
                CellInput::new_builder()
                    .previous_output(
                        OutPoint::new_builder()
                            .tx_hash(tx2.transaction().hash())
                            .index(0u32.pack())
                            .build(),
                    )
                    .build(),
            )
            .witness(Bytes::new().pack())
            .build(),
        200,
        Capacity::shannons(200),
        200,
    );
    let tx1_id = tx1.proposal_short_id();
    let tx2_id = tx2.proposal_short_id();
    let tx3_id = tx3.proposal_short_id();
    map.add_proposed(tx1).unwrap();
    map.add_proposed(tx2).unwrap();
    map.add_proposed(tx3).unwrap();
    let descendants_set = map.calc_descendants(&tx1_id);
    assert!(descendants_set.contains(&tx2_id));
    assert!(descendants_set.contains(&tx3_id));

    let tx3_entry = map.get(&tx3_id);
    assert!(tx3_entry.is_some());
    let tx3_entry = tx3_entry.unwrap();
    assert_eq!(tx3_entry.ancestors_count, 3);

    map.remove_entry(&tx1_id);
    assert!(!map.contains_key(&tx1_id));
    assert!(map.contains_key(&tx2_id));
    assert!(map.contains_key(&tx3_id));

    let tx3_entry = map.get(&tx3_id).unwrap();
    assert_eq!(tx3_entry.ancestors_count, 2);
    assert_eq!(
        map.calc_ancestors(&tx3_id),
        vec![tx2_id].into_iter().collect()
    );
}

#[test]
fn test_remove_entry_and_descendants() {
    let mut map = PoolMap::new(DEFAULT_MAX_ANCESTORS_COUNT);
    let tx1 = TxEntry::dummy_resolve(
        TransactionBuilder::default().build(),
        100,
        Capacity::shannons(100),
        100,
    );
    let tx2 = TxEntry::dummy_resolve(
        TransactionBuilder::default()
            .input(
                CellInput::new_builder()
                    .previous_output(
                        OutPoint::new_builder()
                            .tx_hash(tx1.transaction().hash())
                            .index(0u32.pack())
                            .build(),
                    )
                    .build(),
            )
            .witness(Bytes::new().pack())
            .build(),
        200,
        Capacity::shannons(200),
        200,
    );
    let tx3 = TxEntry::dummy_resolve(
        TransactionBuilder::default()
            .input(
                CellInput::new_builder()
                    .previous_output(
                        OutPoint::new_builder()
                            .tx_hash(tx2.transaction().hash())
                            .index(0u32.pack())
                            .build(),
                    )
                    .build(),
            )
            .witness(Bytes::new().pack())
            .build(),
        200,
        Capacity::shannons(200),
        200,
    );
    let tx1_id = tx1.proposal_short_id();
    let tx2_id = tx2.proposal_short_id();
    let tx3_id = tx3.proposal_short_id();
    map.add_proposed(tx1).unwrap();
    map.add_proposed(tx2).unwrap();
    map.add_proposed(tx3).unwrap();
    let descendants_set = map.calc_descendants(&tx1_id);
    assert!(descendants_set.contains(&tx2_id));
    assert!(descendants_set.contains(&tx3_id));
    map.remove_entry_and_descendants(&tx2_id);
    assert!(!map.contains_key(&tx2_id));
    assert!(!map.contains_key(&tx3_id));
    let descendants_set = map.calc_descendants(&tx1_id);
    assert!(!descendants_set.contains(&tx2_id));
    assert!(!descendants_set.contains(&tx3_id));
}
