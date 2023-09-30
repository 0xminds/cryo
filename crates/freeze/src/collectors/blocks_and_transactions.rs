use crate::{
    Blocks, BlocksAndTransactions, ChunkDim, CollectByBlock, CollectError, ColumnData, Datatype,
    Params, Schemas, Transactions,
};
use std::collections::HashMap;

use crate::Source;
use polars::prelude::*;

type Result<T> = ::core::result::Result<T, CollectError>;

#[async_trait::async_trait]
impl CollectByBlock for BlocksAndTransactions {
    /// type of block data responses
    type Response = <Transactions as CollectByBlock>::Response;

    /// container for a dataset partition
    type Columns = BlocksAndTransactionsColumns;

    /// parameters for requesting data by block
    fn block_parameters() -> Vec<ChunkDim> {
        Transactions::block_parameters()
    }

    /// fetch dataset data by block
    async fn extract(request: Params, source: Source, schemas: Schemas) -> Result<Self::Response> {
        Transactions::extract(request, source, schemas).await
    }

    /// transform block data response into column data
    fn transform(response: Self::Response, columns: &mut Self::Columns, schemas: &Schemas) {
        let BlocksAndTransactionsColumns(block_columns, transaction_columns) = columns;
        let (block, _) = response.clone();
        super::blocks::process_block(
            block,
            block_columns,
            schemas.get(&Datatype::Blocks).expect("schema undefined"),
        );
        Transactions::transform(response, transaction_columns, schemas);
    }
}

/// Blocks and Transaction Columns
#[derive(Default)]
pub struct BlocksAndTransactionsColumns(
    <Blocks as CollectByBlock>::Columns,
    <Transactions as CollectByBlock>::Columns,
);

impl ColumnData for BlocksAndTransactionsColumns {
    fn datatypes() -> Vec<Datatype> {
        vec![Datatype::Blocks, Datatype::Transactions]
    }

    fn create_dfs(self, schemas: &Schemas, chain_id: u64) -> Result<HashMap<Datatype, DataFrame>> {
        let BlocksAndTransactionsColumns(block_columns, transaction_columns) = self;
        Ok(vec![
            (Datatype::Blocks, block_columns.create_df(schemas, chain_id)?),
            (Datatype::Transactions, transaction_columns.create_df(schemas, chain_id)?),
        ]
        .into_iter()
        .collect())
    }
}
