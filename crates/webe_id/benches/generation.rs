use std::collections::HashSet;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use webe_id::time::Clock;
use webe_id::{Generator, NodeId, WebeId};

const GENERATION_COUNT: usize = 100_000;
const LATENCY_SAMPLE_COUNT: usize = 10_000;
const CONCURRENT_WORKERS: usize = 4;

#[derive(Debug)]
struct StepClock {
    next_millis: AtomicU64,
    step_millis: u64,
}

impl StepClock {
    fn new(start_millis: u64, step_millis: u64) -> Self {
        Self {
            next_millis: AtomicU64::new(start_millis),
            step_millis,
        }
    }
}

impl Clock for StepClock {
    fn now(&self) -> SystemTime {
        let millis = self
            .next_millis
            .fetch_add(self.step_millis, Ordering::Relaxed);
        SystemTime::UNIX_EPOCH + Duration::from_millis(millis)
    }
}

fn main() {
    println!("webe_id reporting benchmark");
    println!("package_version: {}", env!("CARGO_PKG_VERSION"));
    println!("os: {}", std::env::consts::OS);
    println!("arch: {}", std::env::consts::ARCH);
    println!("tokio_feature: {}", cfg!(feature = "tokio"));
    println!("logical_cpus: {}", logical_cpus());
    println!("toolchain: {}", rustc_version());
    println!();

    run_single_generation();
    run_generation_latency();
    run_concurrent_generation();
    run_decomposition();
    run_conversion();
}

fn run_single_generation() {
    let mut generator = benchmark_generator(1);
    let started = Instant::now();
    let mut successes = 0_usize;

    for _ in 0..GENERATION_COUNT {
        if generator.generate().is_ok() {
            successes += 1;
        }
    }

    let elapsed = started.elapsed();
    println!(
        "single_generation: count={successes} elapsed={elapsed:?} throughput_per_sec={:.2}",
        throughput(successes, elapsed)
    );
}

fn run_generation_latency() {
    let mut generator = benchmark_generator(10_000_000);
    let mut samples = Vec::with_capacity(LATENCY_SAMPLE_COUNT);

    for _ in 0..LATENCY_SAMPLE_COUNT {
        let started = Instant::now();
        if generator.generate().is_ok() {
            samples.push(started.elapsed());
        }
    }

    println!(
        "generation_latency: samples={} p95={:?}",
        samples.len(),
        percentile_95(&mut samples)
    );
}

fn run_concurrent_generation() {
    let generator = benchmark_generator(20_000_000);
    let shared = Arc::new(Mutex::new(generator));
    let per_worker = GENERATION_COUNT / CONCURRENT_WORKERS;
    let started = Instant::now();

    let handles = (0..CONCURRENT_WORKERS)
        .map(|_| {
            let shared = Arc::clone(&shared);
            thread::spawn(move || {
                let mut ids = Vec::with_capacity(per_worker);
                for _ in 0..per_worker {
                    match shared.lock() {
                        Ok(mut generator) => {
                            if let Ok(id) = generator.generate() {
                                ids.push(id);
                            }
                        }
                        Err(_) => break,
                    }
                }
                ids
            })
        })
        .collect::<Vec<_>>();

    let mut ids = Vec::with_capacity(GENERATION_COUNT);
    for handle in handles {
        if let Ok(mut worker_ids) = handle.join() {
            ids.append(&mut worker_ids);
        }
    }

    let elapsed = started.elapsed();
    let unique = ids.iter().copied().collect::<HashSet<WebeId>>().len();
    let duplicates = ids.len().saturating_sub(unique);
    let duplicate_rate = if ids.is_empty() {
        0.0
    } else {
        duplicates as f64 / ids.len() as f64
    };

    println!(
        "concurrent_generation: workers={CONCURRENT_WORKERS} count={} elapsed={elapsed:?} throughput_per_sec={:.2} duplicates={duplicates} duplicate_rate={duplicate_rate:.8}",
        ids.len(),
        throughput(ids.len(), elapsed)
    );
}

fn run_decomposition() {
    let ids = generated_ids(30_000_000, GENERATION_COUNT);
    let started = Instant::now();
    let mut checksum = 0_u64;

    for id in &ids {
        let components = id.components();
        checksum ^= components.time_millis();
        checksum ^= u64::from(components.node_id().value());
        checksum ^= u64::from(components.sequence());
    }

    let elapsed = started.elapsed();
    println!(
        "decomposition: count={} elapsed={elapsed:?} throughput_per_sec={:.2} checksum={checksum}",
        ids.len(),
        throughput(ids.len(), elapsed)
    );
}

fn run_conversion() {
    let ids = generated_ids(40_000_000, GENERATION_COUNT);
    let started = Instant::now();
    let mut checksum = 0_usize;

    for id in &ids {
        checksum ^= id.to_be_bytes()[0] as usize;
        checksum ^= id.to_decimal_string().len();
        checksum ^= id.to_hex_string().len();
    }

    let elapsed = started.elapsed();
    println!(
        "conversion: count={} elapsed={elapsed:?} throughput_per_sec={:.2} checksum={checksum}",
        ids.len(),
        throughput(ids.len(), elapsed)
    );
}

fn generated_ids(start_millis: u64, count: usize) -> Vec<WebeId> {
    let mut generator = benchmark_generator(start_millis);
    let mut ids = Vec::with_capacity(count);

    for _ in 0..count {
        if let Ok(id) = generator.generate() {
            ids.push(id);
        }
    }

    ids
}

fn benchmark_generator(start_millis: u64) -> Generator {
    let clock = Arc::new(StepClock::new(start_millis, 1));
    match Generator::builder(NodeId::from_u8(1))
        .with_epoch(SystemTime::UNIX_EPOCH)
        .with_clock(clock)
        .build()
    {
        Ok(generator) => generator,
        Err(error) => panic!("benchmark generator setup failed: {error}"),
    }
}

fn throughput(count: usize, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64();
    if seconds == 0.0 {
        0.0
    } else {
        count as f64 / seconds
    }
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    if samples.is_empty() {
        return Duration::ZERO;
    }

    samples.sort_unstable();
    let index = ((samples.len() - 1) * 95) / 100;
    samples[index]
}

fn logical_cpus() -> String {
    match thread::available_parallelism() {
        Ok(count) => count.get().to_string(),
        Err(error) => format!("unavailable ({error})"),
    }
}

fn rustc_version() -> String {
    match Command::new("rustc").arg("--version").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }
        Ok(output) => format!("rustc --version exited with {}", output.status),
        Err(error) => format!("unavailable ({error})"),
    }
}
