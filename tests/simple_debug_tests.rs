use simple_debug::*;

fn main() {
    println!("Initializing global logger once for all tests...");
    let global_log = DebugLog::init(None);

    // -------------------------------
    // Test buffer retrieval correctness
    // -------------------------------
    {
        debug_dev!("Test message 1");
        info_dev!("Test message 2");

        let buf = global_log.get_log_from_buffer().expect("Buffer missing");
        assert!(buf.contains("Test message 1"));
        assert!(buf.contains("Test message 2"));

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
                    debug_dev!("Thread {} message {}", i, j);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Thread panicked");
        }

        let buf = global_log.get_log_from_buffer().expect("Buffer missing");
        assert!(buf.contains("Thread 0 message 0"));
        assert!(buf.contains("Thread 3 message 99"));

        println!("test_multithreaded_logging passed");
    }

    // -------------------------------
    // Test benchmark logging overhead
    // -------------------------------
    {
        let start = std::time::Instant::now();

        for i in 0..1000 {
            debug_dev!("Benchmark message {}", i);
        }

        let duration = start.elapsed();
        println!(
            "benchmark_logging_overhead: Time to log 1000 messages: {:?}",
            duration
        );
    }

    println!("All tests completed successfully!");
}
