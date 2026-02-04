extern crate trpl;

use async_stream::stream;
use trpl::StreamExt;
use std::time::Duration;
use std::pin::pin;

/// Tạo stream:
/// - Emit 1 -> 5
/// - Nghỉ 3 giây
/// - Emit 6 -> 10
fn create_stream() -> impl trpl::Stream<Item = i32> {
    stream! {
        for i in 1..=5 {
            yield i;
        }

        trpl::sleep(Duration::from_secs(3)).await;

        for i in 6..=10 {
            yield i;
        }
    }
}

/// Lắng nghe stream (KHÔNG yêu cầu Unpin)
async fn listen_stream<S>(stream: S)
where
    S: trpl::Stream<Item = i32>,
{
    let mut stream = pin!(stream);

    while let Some(value) = stream.as_mut().next().await {
        println!("The value was: {value}");
    }
}

fn main() {
    trpl::block_on(async {
        let stream = create_stream();
        listen_stream(stream).await;
    });
}
