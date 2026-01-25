#[macro_export]
macro_rules! query_all {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_into:path, $error_type:path, $query:expr, $( $args:tt )* ) => {
        {
            use tracing::Instrument as _;
            use futures::TryStreamExt as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Result<Vec<$struct_into>, $error_type> = sqlx::query_as!(
                    $struct_row,
                    $query,
                    $( $args )*
                )
                .fetch($pool)
                .map_ok(|row: $struct_row| row.try_into())
                .try_collect::<Vec<Result<$struct_into, $error_type>>>()
                .await?
                .into_iter()
                .collect::<Result<Vec<$struct_into>, $error_type>>();

                result

            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! query_exists {
    ( $class:expr, $pool:expr, $query:expr, $( $args:tt )*) => {
        {
            use tracing::Instrument as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: bool = sqlx::query!(
                    $query,
                    $( $args )*
                )
                .fetch_optional($pool)
                .await?
                .is_some();

                Ok(result)

            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! execute {
    ( $class:expr, $pool:expr, $error_type:path, $query:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Result<database::QueryResult, $error_type> = sqlx::query!(
                    $query,
                    $( $args ),*
                )
                .execute($pool)
                .await
                .map_err(|e| <_ as Into<$error_type>>::into(e))
                .map(|r| <_ as Into<database::QueryResult>>::into(r));

                result
            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! execute_returning {
    ( $class:expr, $pool:expr, $error_type:path, $query:expr, $t:path, $fn:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            use futures::TryFutureExt as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Result<$t, $error_type> = sqlx::query!(
                    $query,
                    $( $args, )*
                )
                .fetch_one($pool)
                .map_ok($fn)
                .await
                .map_err(Into::into);

                result


            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! execute_returning_uuid {
    ( $class:expr, $pool:expr, $query:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            use futures::TryFutureExt as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Uuid = sqlx::query!(
                    $query,
                    $( $args ),*
                )
                .fetch_one($pool)
                .map_ok(|row| row.id)
                .await?;

                Ok(result)
            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! execute_returning_optional_uuid {
    ( $class:expr, $pool:expr, $query:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            use futures::TryFutureExt as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Option<Uuid> = sqlx::query!(
                    $query,
                    $( $args ),*
                )
                .fetch_optional($pool)
                .map_ok(|row| row.map(|row| row.id))
                .await?;

                Ok(result)
            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

// Experiments
//
//
// This does not work, as the query*! macros expect a string literal for the query, so
// it has to be there at compile time
//
// fn query_all<Row, Out>(
//     classification: &QueryClassification,
//     pool: &Pool,
//     query: &'static str,
//     args: &[&str],
// ) {
//     async {
//         sqlx_query(classification, query, &[]);
//         let result: Result<Vec<Out>, Error> = sqlx::query_as!(Row, query, args)
//             .fetch(pool)
//             .map_ok(|row: Row| row.try_into())
//             .try_collect::<Vec<Result<Out, Error>>>()
//             .await?
//             .into_iter()
//             .collect::<Result<Vec<Out>, Error>>();

//         result
//     }
//     .instrument(tracing::info_span!("packager::sql::query", "query"))
// }

// #[tracing::instrument(skip(classification, pool, query))]
// pub async fn query_many_to_many_single<Record, Row, Rows, Container, E>(
//     classification: &crate::QueryClassification,
//     pool: &crate::Pool,
//     query: &'static str,
// ) -> Result<Option<Container>, Error>
// where
//     Row: TryInto<Record> + Send + Unpin,
//     Rows: From<Vec<Row>>,
//     Container: TryFrom<Rows, Error = E>,
//     Error: From<E>,
// {
//     use futures::TryStreamExt as _;

//     sqlx_query(classification, query, &[]);
//     let result: Vec<Row> = sqlx::query_file_as!(Row, "../sql/simple.sql")
//         .fetch(pool)
//         .try_collect::<Vec<Row>>()
//         .await?;

//     let rows: Rows = result.into();

//     if result.is_empty() {
//         Ok(None)
//     } else {
//         let out: Container = <_ as TryInto<Container>>::try_into(rows)?;
//         Ok(Some(out))
//     }
// }
