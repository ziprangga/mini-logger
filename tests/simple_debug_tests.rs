use mini_logger::*;

fn main() {
    println!("Initializing global logger once for all tests...");

    Builder::new().env_default().output_stdout().init();

    // -------------------------------
    // Test logging works (no buffer access)
    // -------------------------------
    {
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

        println!("test_logging_match passed");
    }

    // -------------------------------
    // Test multi-threaded logging stress test
    // -------------------------------
    {
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
            h.join().expect("Thread panicked");
        }

        let logger = Logger::get().expect("Logger not initialized");

        let msg = RecMessage::builder()
            .level(FilterLevel::Debug)
            .target("thread-test")
            .module(Some("thread-test"))
            .msg(format_args!("Thread 3 message 99"))
            .build();

        assert!(logger.matches(&msg));

        println!("test_multithreaded_logging passed");
    }

    // -------------------------------
    // Benchmark logging overhead
    // -------------------------------
    {
        let start = std::time::Instant::now();

        for i in 0..1000 {
            debug!("Benchmark message {}", i);
        }

        let duration = start.elapsed();

        println!(
            "benchmark_logging_overhead: Time to log 1000 messages: {:?}",
            duration
        );
    }

    println!("All tests completed successfully!");
}
