use mini_logger::*;

pub fn run() {
    debug!("======================");
    info!("this from another module");

    let mut handles = Vec::new();

    for i in 0..4 {
        let handle = std::thread::spawn(move || {
            let msg = format!(
                "this from another module with multiple thread, thread {}",
                i
            );
            debug!("{}", msg);
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}
