use mini_logger::*;

fn init_logger_once() {
    let _ = Builder::new().env_default().output_stdout().try_init();
}

#[test]
fn test_logging_match() {
    init_logger_once();

    debug!("Test message 1");
    info!("Test message 2");

    let logger = Logger::get().expect("Logger not initialized");

    let msg = RecMessage::builder()
        .level(FilterLevel::Info)
        .target("test")
        .module(Some("test"))
        .msg(format_args!("Test message 2"))
        .build();

    assert!(logger.matches(&msg));
}

#[test]
fn test_multithreaded_logging() {
    init_logger_once();

    let mut handles = vec![];

    for i in 0..4 {
        let handle = std::thread::spawn(move || {
            for j in 0..100 {
                debug!("Thread {} message {}", i, j);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let logger = Logger::get().expect("Logger not initialized");

    let msg = RecMessage::builder()
        .level(FilterLevel::Debug)
        .target("thread-test")
        .module(Some("thread-test"))
        .msg(format_args!("Thread 3 message 99"))
        .build();

    assert!(logger.matches(&msg));
}

#[test]
fn test_logging_benchmark() {
    init_logger_once();

    let start = std::time::Instant::now();

    for i in 0..1000 {
        debug!("Benchmark message {}", i);
    }

    let duration = start.elapsed();

    println!("benchmark_logging_overhead: {:?}", duration);
}
