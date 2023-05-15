use actix_rt;
use actix_web::HttpResponse;
use criterion::{
    async_executor::AsyncExecutor, black_box, criterion_group, criterion_main, Criterion,
};
use server::{create_database_pool, database::Connection, message::*, metadata::*};
use std::future::Future;

pub struct MyRuntime(actix_rt::Runtime);

impl AsyncExecutor for MyRuntime {
    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        self.0.block_on(future)
    }
}

async fn query_metadata(number_entities: i32) -> HttpResponse {
    let metadata_message =
        MetadataMessage::EntitiesMetadataQuery(entities_metadata_query::Payload {
            first: number_entities,
            after: None,
            instance: None,
            modified_after: None,
        });
    metadata_message
        .handle(Connection::Transaction(
            &mut create_database_pool().await.unwrap().begin().await.unwrap(),
        ))
        .await
}

fn criterion_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("query entity metadata", |bencher| {
        bencher
            .to_async(MyRuntime(actix_rt::Runtime::new().unwrap()))
            .iter(|| query_metadata(black_box(10000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
