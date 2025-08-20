//! Test to verify that the logging initialization fix works correctly
//! This test specifically addresses the issue where multiple calls to
//! init_logging() would cause panics due to env_logger being initialized multiple times.

use netabase::init_logging;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

#[test]
fn test_multiple_init_logging_calls() {
    // Test that we can call init_logging multiple times without panicking
    init_logging();
    init_logging();
    init_logging();

    // This should complete without any panics
    println!("✓ Multiple init_logging calls succeeded");
}

#[test]
fn test_concurrent_init_logging_calls() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn multiple threads that all try to initialize logging
    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // Each thread calls init_logging
            init_logging();
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }

    // Verify all threads completed
    assert_eq!(counter.load(Ordering::SeqCst), 10);
    println!("✓ Concurrent init_logging calls succeeded");
}

#[test]
fn test_logging_after_init() {
    // Initialize logging
    init_logging();

    // Test that we can use logging macros after initialization
    log::info!("This is a test log message");
    log::debug!("This is a debug message");
    log::warn!("This is a warning message");

    // Call init_logging again to ensure it doesn't panic
    init_logging();

    // Test logging again
    log::info!("Logging still works after second init call");

    println!("✓ Logging functionality works correctly");
}

#[test]
fn test_database_test_pattern() {
    // Simulate the pattern used in database tests
    for i in 0..5 {
        init_logging();
        log::info!("Test iteration {}", i);

        // Simulate some work that database tests do
        let temp_dir = netabase::get_test_temp_dir(Some(i), None);
        assert!(
            temp_dir
                .to_string_lossy()
                .contains(&format!("netabase_test_{}", i))
        );
    }

    println!("✓ Database test pattern simulation succeeded");
}
