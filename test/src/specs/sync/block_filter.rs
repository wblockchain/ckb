use crate::utils::wait_until;
use crate::{Net, Node, Spec};
use ckb_hash::blake2b_256;
use ckb_logger::info;
use ckb_network::SupportProtocols;
use ckb_types::{bytes::Bytes, core::BlockNumber, packed, prelude::*};
use std::time::Duration;

const CHECK_POINT_INTERVAL: BlockNumber = 2000;
const HASHES_BATCH_SIZE: BlockNumber = 2000;
const FILTERS_BATCH_SIZE: BlockNumber = 1000;

pub struct GetBlockFilterCheckPoints;

impl Spec for GetBlockFilterCheckPoints {
    crate::setup!(num_nodes: 1);

    fn run(&self, nodes: &mut Vec<Node>) {
        let node = nodes.pop().unwrap();
        let points_num = 2;
        let total = CHECK_POINT_INTERVAL * points_num + 1;
        node.mine(total);
        wait_block_filter(&node, total);

        let mut net = Net::new(
            self.name(),
            node.consensus(),
            vec![SupportProtocols::Filter],
        );
        net.connect(&node);
        let start_number: u64 = 0;
        let request = {
            let content = packed::GetBlockFilterCheckPoints::new_builder()
                .start_number(start_number.pack())
                .build();
            packed::BlockFilterMessage::new_builder()
                .set(content)
                .build()
        };

        info!("Send get block filter check points request to node");
        net.send(&node, SupportProtocols::Filter, request.as_bytes());

        let (_, _, data) = net.receive_timeout(&node, Duration::from_secs(10)).unwrap();
        match packed::BlockFilterMessageReader::from_slice(&data) {
            Ok(msg) => match msg.to_enum() {
                packed::BlockFilterMessageUnionReader::BlockFilterCheckPoints(reader) => {
                    let resp_start_number: u64 = reader.start_number().unpack();
                    assert_eq!(start_number, resp_start_number);
                    info!("start_number matched");

                    let hashes: Vec<packed::Byte32> = reader
                        .block_filter_hashes()
                        .iter()
                        .map(|item| item.to_entity())
                        .collect();
                    assert_eq!(
                        hashes.len(),
                        (points_num + 1) as usize,
                        "hashes length not match"
                    );
                    for i in 0..=points_num {
                        let number = i * CHECK_POINT_INTERVAL;
                        let header = node.get_header_by_number(number);
                        let block_filter: packed::Bytes =
                            node.get_block_filter(header.hash()).data.into();
                        let expected_hash = if i == 0 {
                            blake2b_256(
                                [&[0u8; 32], block_filter.calc_raw_data_hash().as_slice()].concat(),
                            )
                        } else {
                            let parent_block_filter = node.get_block_filter(header.parent_hash());
                            blake2b_256(
                                [
                                    parent_block_filter.hash.0.as_slice(),
                                    block_filter.calc_raw_data_hash().as_slice(),
                                ]
                                .concat(),
                            )
                        };
                        assert_eq!(
                            &expected_hash,
                            hashes[i as usize].as_slice(),
                            "block number: {number}"
                        );
                    }
                    info!("block_filter_hashes matched");
                }
                _ => panic!("unexpected message"),
            },
            _ => panic!("unexpected message"),
        }
    }
}

pub struct GetBlockFilterHashes;

impl Spec for GetBlockFilterHashes {
    crate::setup!(num_nodes: 1);

    fn run(&self, nodes: &mut Vec<Node>) {
        let node = nodes.pop().unwrap();
        node.mine(2001);
        wait_block_filter(&node, 2001);

        let mut net = Net::new(
            self.name(),
            node.consensus(),
            vec![SupportProtocols::Filter],
        );
        net.connect(&node);
        let start_number: u64 = 2;
        let request = {
            let content = packed::GetBlockFilterHashes::new_builder()
                .start_number(start_number.pack())
                .build();
            packed::BlockFilterMessage::new_builder()
                .set(content)
                .build()
        };

        info!("Send get block filter check points request to node");
        net.send(&node, SupportProtocols::Filter, request.as_bytes());

        let (_, _, data) = net.receive_timeout(&node, Duration::from_secs(10)).unwrap();
        match packed::BlockFilterMessageReader::from_slice(&data) {
            Ok(msg) => match msg.to_enum() {
                packed::BlockFilterMessageUnionReader::BlockFilterHashes(reader) => {
                    let resp_start_number: u64 = reader.start_number().unpack();
                    assert_eq!(start_number, resp_start_number);
                    info!("start_number matched");

                    let parent_block_filter_hash = reader.parent_block_filter_hash().to_entity();
                    {
                        let header = node.get_header_by_number(start_number - 1);
                        let block_filter: packed::Bytes =
                            node.get_block_filter(header.hash()).data.into();
                        let parent_block_filter = node.get_block_filter(header.parent_hash());
                        let expected_parent_hash = blake2b_256(
                            [
                                parent_block_filter.hash.0.as_slice(),
                                block_filter.calc_raw_data_hash().as_slice(),
                            ]
                            .concat(),
                        );
                        assert_eq!(&expected_parent_hash, parent_block_filter_hash.as_slice());
                    }
                    info!("parent_block_filter_hash matched");

                    let hashes: Vec<packed::Byte32> = reader
                        .block_filter_hashes()
                        .iter()
                        .map(|item| item.to_entity())
                        .collect();
                    assert_eq!(
                        hashes.len(),
                        HASHES_BATCH_SIZE as usize,
                        "hashes length not match"
                    );
                    for i in 0..HASHES_BATCH_SIZE {
                        let number = start_number + i;
                        let header = node.get_header_by_number(number);
                        let block_filter: packed::Bytes =
                            node.get_block_filter(header.hash()).data.into();
                        let parent_block_filter = node.get_block_filter(header.parent_hash());
                        let expected_hash = blake2b_256(
                            [
                                parent_block_filter.hash.0.as_slice(),
                                block_filter.calc_raw_data_hash().as_slice(),
                            ]
                            .concat(),
                        );
                        assert_eq!(
                            &expected_hash,
                            hashes[i as usize].as_slice(),
                            "block number: {number}"
                        );
                    }
                    info!("block_filter_hashes matched");
                }
                _ => panic!("unexpected message"),
            },
            _ => panic!("unexpected message"),
        }
    }
}

pub struct GetBlockFilters;

impl Spec for GetBlockFilters {
    crate::setup!(num_nodes: 1);

    fn run(&self, nodes: &mut Vec<Node>) {
        let node = nodes.pop().unwrap();
        node.mine(2001);
        wait_block_filter(&node, 2001);

        let mut net = Net::new(
            self.name(),
            node.consensus(),
            vec![SupportProtocols::Filter],
        );
        net.connect(&node);
        let start_number: u64 = 42;
        let request = {
            let content = packed::GetBlockFilters::new_builder()
                .start_number(start_number.pack())
                .build();
            packed::BlockFilterMessage::new_builder()
                .set(content)
                .build()
        };

        info!("Send get block filter check points request to node");
        net.send(&node, SupportProtocols::Filter, request.as_bytes());

        let (_, _, data) = net.receive_timeout(&node, Duration::from_secs(10)).unwrap();
        match packed::BlockFilterMessageReader::from_slice(&data) {
            Ok(msg) => match msg.to_enum() {
                packed::BlockFilterMessageUnionReader::BlockFilters(reader) => {
                    let resp_start_number: u64 = reader.start_number().unpack();
                    assert_eq!(start_number, resp_start_number);
                    info!("start_number matched");

                    let block_hashes: Vec<packed::Byte32> = reader
                        .block_hashes()
                        .iter()
                        .map(|item| item.to_entity())
                        .collect();
                    let filters: Vec<Bytes> = reader
                        .filters()
                        .iter()
                        .map(|item| item.to_entity().unpack())
                        .collect();

                    assert_eq!(
                        block_hashes.len(),
                        FILTERS_BATCH_SIZE as usize,
                        "block hashes length not match"
                    );
                    assert_eq!(
                        filters.len(),
                        FILTERS_BATCH_SIZE as usize,
                        "filters length not match"
                    );
                    for i in 0..FILTERS_BATCH_SIZE {
                        let number = start_number + i;
                        let header = node.get_header_by_number(number);
                        let block_filter = node.get_block_filter(header.hash()).data.into_bytes();
                        assert_eq!(
                            header.hash(),
                            block_hashes[i as usize],
                            "block hash not match, block number: {number}"
                        );
                        assert_eq!(
                            block_filter, filters[i as usize],
                            "block filter not match, block number: {number}"
                        );
                    }
                    info!("block hashes/filters matched");
                }
                _ => panic!("unexpected message"),
            },
            _ => panic!("unexpected message"),
        }
    }
}

pub struct GetBlockFiltersNotReachBatch;

impl Spec for GetBlockFiltersNotReachBatch {
    crate::setup!(num_nodes: 1);

    fn run(&self, nodes: &mut Vec<Node>) {
        let node = nodes.pop().unwrap();
        let total = 2001;
        node.mine(total);
        wait_block_filter(&node, total);

        let mut net = Net::new(
            self.name(),
            node.consensus(),
            vec![SupportProtocols::Filter],
        );
        net.connect(&node);
        let filters_count = 123;
        let start_number: u64 = total - filters_count + 1;
        info!("start_number: {start_number}");
        let request = {
            let content = packed::GetBlockFilters::new_builder()
                .start_number(start_number.pack())
                .build();
            packed::BlockFilterMessage::new_builder()
                .set(content)
                .build()
        };

        info!("Send get block filter check points request to node");
        net.send(&node, SupportProtocols::Filter, request.as_bytes());

        let (_, _, data) = net.receive_timeout(&node, Duration::from_secs(10)).unwrap();
        match packed::BlockFilterMessageReader::from_slice(&data) {
            Ok(msg) => match msg.to_enum() {
                packed::BlockFilterMessageUnionReader::BlockFilters(reader) => {
                    let resp_start_number: u64 = reader.start_number().unpack();
                    assert_eq!(start_number, resp_start_number);
                    info!("start_number matched");

                    let block_hashes: Vec<packed::Byte32> = reader
                        .block_hashes()
                        .iter()
                        .map(|item| item.to_entity())
                        .collect();
                    let filters: Vec<Bytes> = reader
                        .filters()
                        .iter()
                        .map(|item| item.to_entity().unpack())
                        .collect();

                    assert_eq!(
                        block_hashes.len(),
                        filters_count as usize,
                        "block hashes length not match"
                    );
                    assert_eq!(
                        filters.len(),
                        filters_count as usize,
                        "filters length not match"
                    );
                    for i in 0..filters_count {
                        let number = start_number + i;
                        let header = node.get_header_by_number(number);
                        let block_filter = node.get_block_filter(header.hash()).data.into_bytes();
                        assert_eq!(
                            header.hash(),
                            block_hashes[i as usize],
                            "block hash not match, block number: {number}"
                        );
                        assert_eq!(
                            block_filter, filters[i as usize],
                            "block filter not match, block number: {number}"
                        );
                    }
                    info!("block hashes/filters matched");
                }
                _ => panic!("unexpected message"),
            },
            _ => panic!("unexpected message"),
        }
    }
}

fn wait_block_filter(node: &Node, total: BlockNumber) {
    let tip_header = node.get_header_by_number(node.get_tip_block_number());
    info!("sync block filter for tip hash: {}", tip_header.hash());
    assert_eq!(tip_header.number(), total);
    let success = wait_until(5, || {
        node.rpc_client()
            .get_block_filter(tip_header.hash())
            .is_some()
    });
    assert!(
        success,
        "the last block(number={}) filter is missing",
        tip_header.number()
    );
}
