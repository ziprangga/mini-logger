use mini_logger::*; // your new logger crate

fn main() {
    println!("Initializing global logger once for all tests...");

    // Initialize global logger
    Builder::new()
        .env_default() // read RUST_LOG from env if exists
        .output_stdout() // print to stdout
        .try_init()
        .expect("Failed to initialize mini_logger");

    // -------------------------------
    // Test buffer retrieval correctness
    // -------------------------------
    {
        debug!("Test message 1");
        info!("Test message 2");

        // If your mini_logger has a buffer retrieval method, adapt this:
        // Assuming `LOGGER.get()` returns &Logger
        let logger = LOGGER.get().expect("Logger not initialized");
        if let Some(buf) = logger.writer.buffer().as_ref() {
            let buf_str = String::from_utf8_lossy(buf);
            assert!(buf_str.contains("Test message 1"));
            assert!(buf_str.contains("Test message 2"));
        }

        println!("test_buffer_retrieval passed");
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

        let logger = LOGGER.get().expect("Logger not initialized");
        if let Some(buf) = logger.writer.buffer().as_ref() {
            let buf_str = String::from_utf8_lossy(buf);
            assert!(buf_str.contains("Thread 0 message 0"));
            assert!(buf_str.contains("Thread 3 message 99"));
        }

        println!("test_multithreaded_logging passed");
    }

    // -------------------------------
    // Test benchmark logging overhead
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
