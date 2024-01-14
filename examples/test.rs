use tracing::{info, instrument};
use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt, Layer};
use tracing_usdt::UstdTracingLayer;

#[instrument]
pub fn main() {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter::LevelFilter::WARN))
        .with(UstdTracingLayer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut i = 0;
    loop {
        if i % 2 == 0 {
            even(i, i % 3 == 0);
        } else {
            odd();
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
        i += 1;
    }
}

#[instrument(ret)]
fn even(mut arg0: i32, arg1: bool) -> i32 {
    info!("Even called");

    if arg1 {
        arg0 += 1;
        arg0
    } else {
        0
    }
}

#[instrument]
fn odd() {
    info!("Odd called");
}
