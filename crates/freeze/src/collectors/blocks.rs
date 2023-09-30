use crate::{
    conversions::{ToVecHex, ToVecU8},
    dataframes::SortableDataFrame,
    store, with_series, with_series_binary, with_series_option_u256, Blocks, CollectByBlock,
    CollectByTransaction, CollectError, ColumnData, ColumnEncoding, ColumnType, Datatype, Params,
    Schemas, Source, Table, U256Type,
};
use ethers::prelude::*;
use polars::prelude::*;
use std::collections::HashMap;

type Result<T> = ::core::result::Result<T, CollectError>;

#[async_trait::async_trait]
impl CollectByBlock for Blocks {
    type Response = Block<TxHash>;

    type Columns = BlockColumns;

    async fn extract(request: Params, source: Source, _schemas: Schemas) -> Result<Self::Response> {
        let block = source
            .fetcher
            .get_block(request.block_number())
            .await?
            .ok_or(CollectError::CollectError("block not found".to_string()))?;
        Ok(block)
    }

    fn transform(response: Self::Response, columns: &mut Self::Columns, schemas: &Schemas) {
        let schema = schemas.get(&Datatype::Blocks).expect("schema missing");
        process_block(response, columns, schema)
    }
}

#[async_trait::async_trait]
impl CollectByTransaction for Blocks {
    type Response = Block<TxHash>;

    type Columns = BlockColumns;

    async fn extract(request: Params, source: Source, _schemas: Schemas) -> Result<Self::Response> {
        let transaction = source
            .fetcher
            .get_transaction(request.ethers_transaction_hash())
            .await?
            .ok_or(CollectError::CollectError("transaction not found".to_string()))?;
        let block = source
            .fetcher
            .get_block_by_hash(transaction.block_hash.expect("no block hash found"))
            .await?
            .ok_or(CollectError::CollectError("block not found".to_string()))?;
        Ok(block)
    }

    fn transform(response: Self::Response, columns: &mut Self::Columns, schemas: &Schemas) {
        let schema = schemas.get(&Datatype::Blocks).expect("schema missing");
        process_block(response, columns, schema)
    }
}

/// columns for transactions
#[cryo_to_df::to_df(Datatype::Blocks)]
#[derive(Default)]
pub struct BlockColumns {
    n_rows: u64,
    hash: Vec<Vec<u8>>,
    parent_hash: Vec<Vec<u8>>,
    author: Vec<Vec<u8>>,
    state_root: Vec<Vec<u8>>,
    transactions_root: Vec<Vec<u8>>,
    receipts_root: Vec<Vec<u8>>,
    block_number: Vec<Option<u32>>,
    gas_used: Vec<u32>,
    extra_data: Vec<Vec<u8>>,
    logs_bloom: Vec<Option<Vec<u8>>>,
    timestamp: Vec<u32>,
    total_difficulty: Vec<Option<U256>>,
    size: Vec<Option<u32>>,
    base_fee_per_gas: Vec<Option<u64>>,
}

/// process block into columns
pub fn process_block<TX>(block: Block<TX>, columns: &mut BlockColumns, schema: &Table) {
    columns.n_rows += 1;

    store!(schema, columns, hash, block.hash.map(|x| x.0.to_vec()).expect("block hash required"));
    store!(schema, columns, parent_hash, block.parent_hash.0.to_vec());
    store!(schema, columns, author, block.author.map(|x| x.0.to_vec()).expect("author required"));
    store!(schema, columns, state_root, block.state_root.0.to_vec());
    store!(schema, columns, transactions_root, block.transactions_root.0.to_vec());
    store!(schema, columns, receipts_root, block.receipts_root.0.to_vec());
    store!(schema, columns, block_number, block.number.map(|x| x.as_u32()));
    store!(schema, columns, gas_used, block.gas_used.as_u32());
    store!(schema, columns, extra_data, block.extra_data.to_vec());
    store!(schema, columns, logs_bloom, block.logs_bloom.map(|x| x.0.to_vec()));
    store!(schema, columns, timestamp, block.timestamp.as_u32());
    store!(schema, columns, total_difficulty, block.total_difficulty);
    store!(schema, columns, base_fee_per_gas, block.base_fee_per_gas.map(|x| x.as_u64()));
    store!(schema, columns, size, block.size.map(|x| x.as_u32()));
}
