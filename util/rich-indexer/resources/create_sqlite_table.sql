CREATE TABLE IF NOT EXISTS block(
    id INTEGER PRIMARY KEY,
    block_hash BLOB NOT NULL,
    block_number INTEGER NOT NULL,
    compact_target BLOB,
    parent_hash BLOB,
    nonce BLOB,
    timestamp INTEGER,
    version BLOB,
    transactions_root BLOB,
    epoch BLOB,
    dao BLOB,
    proposals_hash BLOB,
    extra_hash BLOB,
    extension BLOB
);

CREATE TABLE IF NOT EXISTS block_association_proposal(
    id INTEGER PRIMARY KEY,
    block_id INTEGER NOT NULL,
    proposal BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS block_association_uncle(
    id INTEGER PRIMARY KEY,
    block_id INTEGER NOT NULL,
    uncle_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ckb_transaction(
    id INTEGER PRIMARY KEY,
    tx_hash BLOB NOT NULL,
    version BLOB NOT NULL,
    input_count INTEGER NOT NULL,
    output_count INTEGER NOT NULL,
    witnesses BLOB,
    block_id INTEGER NOT NULL,
    tx_index INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tx_association_header_dep(
    id INTEGER PRIMARY KEY,
    tx_id INTEGER NOT NULL,
    block_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tx_association_cell_dep(
    id INTEGER PRIMARY KEY,
    tx_id INTEGER NOT NULL,
    output_id INTEGER NOT NULL,
    dep_type INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS output(
    id INTEGER PRIMARY KEY,
    tx_id INTEGER NOT NULL,
    output_index INTEGER NOT NULL,
    capacity INTEGER NOT NULL,
    lock_script_id INTEGER,
    type_script_id INTEGER,
    data BLOB
);

CREATE TABLE IF NOT EXISTS input(
    output_id INTEGER PRIMARY KEY,
    since BLOB NOT NULL,
    consumed_tx_id INTEGER NOT NULL,
    input_index INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS script(
    id INTEGER PRIMARY KEY,
    code_hash BLOB NOT NULL,
    hash_type INTEGER NOT NULL,
    args BLOB,
    UNIQUE(code_hash, hash_type, args)
);
