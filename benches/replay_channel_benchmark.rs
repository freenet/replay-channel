use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use criterion::async_executor::FuturesExecutor;
use replay_channel::ReplayChannel;  // Replace `your_crate` with the name of your crate

// Async function that encapsulates the send and receive operations
async fn send_and_receive() {
    let channel = ReplayChannel::new();
    let sender = channel.sender();
    let receiver = channel.receiver();

    sender.send(42);  // Assuming send is not async
    let _ = receiver.receive().await;
}

fn replay_channel_benchmark(c: &mut Criterion) {
    c.bench_function(BenchmarkId::new("replay_channel_send_receive", 1), |b| {
        b.to_async(FuturesExecutor).iter(|| send_and_receive());
    });
}

criterion_group!(benches, replay_channel_benchmark);
criterion_main!(benches);
