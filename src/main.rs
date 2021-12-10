use jwrs::spinup;
use std::sync::mpsc::channel;

use rand::prelude::*;
use std::{thread, time};
use tracing_subscriber::EnvFilter;

use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_subscriber::prelude::*;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    price: u64,
    quantity: u64,
}

#[tokio::main]
async fn main() {
    // Create a jaeger exporter pipeline for a `trace_demo` service.
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("rubicon-stream-server")
        //.install_batch(opentelemetry::runtime::Tokio)
        .install_simple()
        .expect("Error initializing Jaeger exporter");

    // Create a layer with the configured tracer
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(otel_layer); // really we don't want to be storing logs in memory, because we us ALOT of it if we're going fast.
    subscriber.init();


    let (tx, rx) = channel::<Order>();
    let name = "rand_server";
    spinup(rx, format!("127.0.0.1:{}", 8080), name).await;

    let k = time::Duration::from_micros(1*1000*1000);
    let mut rng = thread_rng();

    loop {
        let tmp = Order {
            price: rng.next_u64(),
            quantity: rng.next_u64(),
        };


        tx.send(tmp);
        thread::sleep(k);
    }
}
