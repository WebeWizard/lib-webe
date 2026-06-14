#![cfg(feature = "tokio")]

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{ManualClock, at_ms, epoch};
use tokio::sync::Mutex;
use webe_id::async_backpressure::{
    BackpressureError, BackpressureOptions, generate_with_backpressure,
};
use webe_id::{GenerateError, Generator, NodeId};

#[tokio::test]
async fn normal_request_style_generation_completes_without_waiting() {
    let clock = ManualClock::new(at_ms(30));
    let generator = Generator::builder(NodeId::from_u8(21))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();
    let shared = Arc::new(Mutex::new(generator));

    let id = generate_with_backpressure(
        Arc::clone(&shared),
        BackpressureOptions::new(Duration::from_millis(50), Duration::from_millis(1)),
    )
    .await
    .unwrap();

    assert_eq!(id.components().time_millis(), 30);
    assert_eq!(id.components().sequence(), 0);
}

#[tokio::test]
async fn bounded_backpressure_succeeds_after_safe_time_advancement() {
    let clock = ManualClock::new(at_ms(40));
    let generator = Generator::builder(NodeId::from_u8(22))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock.clone()))
        .build()
        .unwrap();
    let shared = Arc::new(Mutex::new(generator));

    {
        let mut generator = shared.lock().await;
        for _ in 0..=u16::MAX {
            let _ = generator.generate().unwrap();
        }
        assert!(matches!(
            generator.generate(),
            Err(GenerateError::SequenceCapacityExhausted { .. })
        ));
    }

    let clock_to_advance = clock.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(3)).await;
        clock_to_advance.advance(Duration::from_millis(1));
    });

    let id = generate_with_backpressure(
        Arc::clone(&shared),
        BackpressureOptions::new(Duration::from_millis(100), Duration::from_millis(1)),
    )
    .await
    .unwrap();

    assert_eq!(id.components().time_millis(), 41);
    assert_eq!(id.components().sequence(), 0);
}

#[tokio::test]
async fn bounded_backpressure_times_out_with_last_capacity_error() {
    let clock = ManualClock::new(at_ms(50));
    let generator = Generator::builder(NodeId::from_u8(23))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();
    let shared = Arc::new(Mutex::new(generator));

    {
        let mut generator = shared.lock().await;
        for _ in 0..=u16::MAX {
            let _ = generator.generate().unwrap();
        }
    }

    let result = generate_with_backpressure(
        Arc::clone(&shared),
        BackpressureOptions::new(Duration::from_millis(5), Duration::from_millis(1)),
    )
    .await;

    assert!(matches!(
        result,
        Err(BackpressureError::TimedOut {
            last_error: GenerateError::SequenceCapacityExhausted { time_millis: 50 },
            ..
        })
    ));
}
