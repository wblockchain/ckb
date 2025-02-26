CREATE TABLE IF NOT EXISTS block(
    id BIGSERIAL PRIMARY KEY,
    block_hash BYTEA NOT NULL,
    block_number BIGINT NOT NULL,
    compact_target BYTEA,
    parent_hash BYTEA,
    nonce BYTEA,
    timestamp BIGINT,
    version BYTEA,
    transactions_root BYTEA,
    epoch BYTEA,
    dao BYTEA,
    proposals_hash BYTEA,
    extra_hash BYTEA,
    extension BYTEA
);

CREATE TABLE IF NOT EXISTS block_association_proposal(
    id BIGSERIAL,
    block_id BIGINT NOT NULL,
    proposal BYTEA NOT NULL
);

CREATE TABLE IF NOT EXISTS block_association_uncle(
    id BIGSERIAL,
    block_id BIGINT NOT NULL,
    uncle_id BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS ckb_transaction(
    id BIGSERIAL PRIMARY KEY,
    tx_hash BYTEA NOT NULL,
    version BYTEA NOT NULL,
    input_count INTEGER NOT NULL,
    output_count INTEGER NOT NULL,
    witnesses BYTEA,
    block_id BIGINT NOT NULL,
    tx_index INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tx_association_header_dep(
    id BIGSERIAL,
    tx_id BIGINT NOT NULL,
    block_id BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS tx_association_cell_dep(
    id BIGSERIAL,
    tx_id BIGINT NOT NULL,
    output_id BIGINT NOT NULL,
    dep_type SMALLINT NOT NULL
);

CREATE TABLE IF NOT EXISTS output(
    id BIGSERIAL PRIMARY KEY,
    tx_id BIGINT NOT NULL,
    output_index INTEGER NOT NULL,
    capacity BIGINT NOT NULL,
    lock_script_id BIGINT,
    type_script_id BIGINT,
    data BYTEA
);

CREATE TABLE IF NOT EXISTS input(
    output_id BIGINT PRIMARY KEY,
    since BYTEA NOT NULL,
    consumed_tx_id BIGINT NOT NULL,
    input_index INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS script(
    id BIGSERIAL PRIMARY KEY,
    code_hash BYTEA NOT NULL,
    hash_type SMALLINT NOT NULL,
    args BYTEA,
    UNIQUE(code_hash, hash_type, args)
);
