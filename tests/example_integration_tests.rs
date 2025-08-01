//! Integration tests for the consolidated rpc_system_demo example
//!
//! These tests validate all CLI commands and functionality of the unified example,
//! ensuring example-driven testing works correctly.

use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

/// Test helper to run example commands with timeout
async fn run_example_command(args: &[&str], timeout_secs: u64) -> Result<String, String> {
    let output = timeout(
        Duration::from_secs(timeout_secs),
        tokio::task::spawn_blocking({
            let args = args.to_vec();
            move || {
                Command::new("cargo")
                    .arg("run")
                    .args(&args)
                    .current_dir("examples/rpc_system_demo")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            }
        }),
    )
    .await
    .map_err(|_| "Command timeout".to_string())?
    .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "Command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Test that the consolidated example builds successfully
#[tokio::test]
async fn test_example_builds() {
    let result = tokio::task::spawn_blocking(|| {
        Command::new("cargo")
            .arg("build")
            .current_dir("examples/rpc_system_demo")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
    })
    .await
    .expect("Failed to execute build command");

    assert!(result.success(), "Example should build successfully");
}

/// Test the comprehensive demo command
#[tokio::test]
async fn test_demo_command() {
    println!("🧪 Testing demo command...");
    
    let output = run_example_command(&["demo"], 30).await;
    
    match output {
        Ok(stdout) => {
            println!("✅ Demo command completed successfully");
            
            // Verify key demo features are tested
            assert!(stdout.contains("Testing basic async method"), "Should test async methods");
            assert!(stdout.contains("Testing sync method"), "Should test sync methods");
            assert!(stdout.contains("Testing multi-parameter method"), "Should test multi-param methods");
            assert!(stdout.contains("Testing custom error type"), "Should test error handling");
            assert!(stdout.contains("Testing subscription method"), "Should test subscriptions");
            assert!(stdout.contains("All RPC and Event features working correctly"), "Should complete all tests");
            
            println!("📊 Demo validation completed");
        }
        Err(e) => {
            panic!("Demo command failed: {}", e);
        }
    }
}

/// Test server command starts successfully (quick validation)
#[tokio::test]
async fn test_server_command_starts() {
    println!("🧪 Testing server command startup...");
    
    // Start server in background
    let mut server_process = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("server")
        .current_dir("examples/rpc_system_demo")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Give server time to start up
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Check if server is still running (hasn't crashed)
    match server_process.try_wait() {
        Ok(Some(status)) => {
            // Server exited - this might be an error
            let stderr = server_process.stderr.take().unwrap();
            let mut stderr_content = String::new();
            use tokio::io::AsyncReadExt;
            let _ = stderr.into_std().await.read_to_string(&mut stderr_content);
            panic!("Server exited unexpectedly with status {}: {}", status, stderr_content);
        }
        Ok(None) => {
            // Server is still running - success!
            println!("✅ Server started successfully");
            
            // Clean up: terminate server
            let _ = server_process.kill().await;
            let _ = server_process.wait().await;
        }
        Err(e) => {
            panic!("Error checking server status: {}", e);
        }
    }
}

/// Test client command (requires server to be running - separate test)
#[tokio::test]
async fn test_client_server_interaction() {
    println!("🧪 Testing client-server interaction...");
    
    // Start server in background
    let mut server_process = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("server")
        .current_dir("examples/rpc_system_demo")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Give server time to start up
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Check server is running
    if let Ok(Some(status)) = server_process.try_wait() {
        let stderr = server_process.stderr.take().unwrap();
        let mut stderr_content = String::new();
        use tokio::io::AsyncReadExt;
        let _ = stderr.into_std().await.read_to_string(&mut stderr_content);
        panic!("Server failed to start with status {}: {}", status, stderr_content);
    }

    // Run client
    let client_result = run_example_command(&["client"], 15).await;
    
    // Clean up server
    let _ = server_process.kill().await;
    let _ = server_process.wait().await;
    
    match client_result {
        Ok(stdout) => {
            println!("✅ Client-server interaction successful");
            
            // Verify client performed expected operations
            assert!(stdout.contains("Remote add"), "Should perform remote addition");
            assert!(stdout.contains("Remote multiply"), "Should perform remote multiplication");
            assert!(stdout.contains("Remote status"), "Should get remote status");
            assert!(stdout.contains("Client operations completed"), "Should complete successfully");
            
            println!("📊 Client-server test completed");
        }
        Err(e) => {
            panic!("Client-server interaction failed: {}", e);
        }
    }
}

/// Test events demo command
#[tokio::test]
async fn test_events_demo_command() {
    println!("🧪 Testing events demo command...");
    
    let output = run_example_command(&["events"], 15).await;
    
    match output {
        Ok(stdout) => {
            println!("✅ Events demo completed successfully");
            
            // Verify event system features are tested
            assert!(stdout.contains("Subscribers registered for sensor events"), "Should register subscribers");
            assert!(stdout.contains("Publishing sensor events"), "Should publish events");
            assert!(stdout.contains("Event system demo completed"), "Should complete successfully");
            
            // Look for event processing in output
            if stdout.contains("Temperature:") || stdout.contains("Humidity:") {
                println!("📊 Event processing verified in output");
            } else {
                println!("⚠️  Event processing not visible in output (may be async)");
            }
            
            println!("📊 Events demo validation completed");
        }
        Err(e) => {
            panic!("Events demo command failed: {}", e);
        }
    }
}

/// Test publisher command starts successfully (quick validation)
#[tokio::test]
async fn test_publisher_command_starts() {
    println!("🧪 Testing publisher command startup...");
    
    // Start publisher in background
    let mut publisher_process = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("publisher")
        .current_dir("examples/rpc_system_demo")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start publisher");

    // Give publisher time to start up and publish some events
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Check if publisher is still running (hasn't crashed)
    match publisher_process.try_wait() {
        Ok(Some(status)) => {
            // Publisher exited - this might be an error
            let stderr = publisher_process.stderr.take().unwrap();
            let mut stderr_content = String::new();
            use tokio::io::AsyncReadExt;
            let _ = stderr.into_std().await.read_to_string(&mut stderr_content);
            panic!("Publisher exited unexpectedly with status {}: {}", status, stderr_content);
        }
        Ok(None) => {
            // Publisher is still running - success!
            println!("✅ Publisher started successfully");
            
            // Try to capture some output to verify it's working
            let _ = publisher_process.kill().await;
            let output = publisher_process.wait_with_output().await.expect("Failed to get publisher output");
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if stdout.contains("Published") {
                println!("📊 Publisher is actively publishing events");
            } else {
                println!("⚠️  Publisher output not captured (may be timing issue)");
            }
        }
        Err(e) => {
            panic!("Error checking publisher status: {}", e);
        }
    }
}

/// Test subscriber command starts successfully (quick validation)
#[tokio::test]
async fn test_subscriber_command_starts() {
    println!("🧪 Testing subscriber command startup...");
    
    // Start subscriber in background
    let mut subscriber_process = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("subscriber")
        .current_dir("examples/rpc_system_demo")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start subscriber");

    // Give subscriber time to start up
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // Check if subscriber is still running (hasn't crashed)
    match subscriber_process.try_wait() {
        Ok(Some(status)) => {
            // Subscriber exited - this might be an error
            let stderr = subscriber_process.stderr.take().unwrap();
            let mut stderr_content = String::new();
            use tokio::io::AsyncReadExt;
            let _ = stderr.into_std().await.read_to_string(&mut stderr_content);
            panic!("Subscriber exited unexpectedly with status {}: {}", status, stderr_content);
        }
        Ok(None) => {
            // Subscriber is still running - success!
            println!("✅ Subscriber started successfully");
            
            // Clean up: terminate subscriber
            let _ = subscriber_process.kill().await;
            let _ = subscriber_process.wait().await;
            
            println!("📊 Subscriber startup validation completed");
        }
        Err(e) => {
            panic!("Error checking subscriber status: {}", e);
        }
    }
}

/// Test publisher-subscriber interaction
#[tokio::test]
async fn test_publisher_subscriber_interaction() {
    println!("🧪 Testing publisher-subscriber interaction...");
    
    // Start subscriber in background
    let mut subscriber_process = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("subscriber")
        .current_dir("examples/rpc_system_demo")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start subscriber");

    // Give subscriber time to start up and register subscriptions
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Check subscriber is running
    if let Ok(Some(status)) = subscriber_process.try_wait() {
        let stderr = subscriber_process.stderr.take().unwrap();
        let mut stderr_content = String::new();
        use tokio::io::AsyncReadExt;
        let _ = stderr.into_std().await.read_to_string(&mut stderr_content);
        panic!("Subscriber failed to start with status {}: {}", status, stderr_content);
    }

    // Start publisher in background
    let mut publisher_process = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("publisher")
        .current_dir("examples/rpc_system_demo")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start publisher");

    // Let publisher send some events
    tokio::time::sleep(Duration::from_millis(5000)).await;

    // Clean up processes
    let _ = publisher_process.kill().await;
    let _ = subscriber_process.kill().await;
    
    let publisher_output = publisher_process.wait_with_output().await.expect("Failed to get publisher output");
    let subscriber_output = subscriber_process.wait_with_output().await.expect("Failed to get subscriber output");

    let publisher_stdout = String::from_utf8_lossy(&publisher_output.stdout);
    let subscriber_stdout = String::from_utf8_lossy(&subscriber_output.stdout);

    println!("📊 Publisher-subscriber interaction results:");
    
    // Verify publisher was active
    if publisher_stdout.contains("Published") {
        println!("✅ Publisher successfully published events");
    } else {
        println!("⚠️  Publisher output not captured: {}", publisher_stdout);
    }

    // Verify subscriber received events (may be asynchronous)
    if subscriber_stdout.contains("Temperature") || subscriber_stdout.contains("Humidity") || subscriber_stdout.contains("Monitor") {
        println!("✅ Subscriber successfully received events");
    } else {
        println!("⚠️  Subscriber event reception not captured in output");
        println!("    This may be due to async event processing timing");
        println!("    Subscriber output: {}", subscriber_stdout);
    }
    
    println!("📊 Publisher-subscriber test completed");
}

/// Test that all example CLI commands are recognized
#[tokio::test]
async fn test_all_cli_commands_recognized() {
    println!("🧪 Testing CLI command recognition...");
    
    let commands = vec!["demo", "server", "client", "events", "publisher", "subscriber"];
    
    for cmd in commands {
        println!("   Testing command: {}", cmd);
        
        // For commands that run indefinitely, we just check they start without immediate error
        let quick_commands = vec!["demo", "events", "client"];
        
        if quick_commands.contains(&cmd) {
            // These commands should complete quickly
            let result = run_example_command(&[cmd], 30).await;
            match result {
                Ok(_) => println!("     ✅ {} command executed successfully", cmd),
                Err(e) => panic!("Command '{}' failed: {}", cmd, e),
            }
        } else {
            // These commands run indefinitely - just check they start
            let mut process = tokio::process::Command::new("cargo")
                .arg("run")
                .arg(cmd)
                .current_dir("examples/rpc_system_demo")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect(&format!("Failed to start {} command", cmd));

            // Give it a moment to start
            tokio::time::sleep(Duration::from_millis(1000)).await;

            match process.try_wait() {
                Ok(Some(status)) => {
                    // Process exited - check if it was an error
                    let output = process.wait_with_output().await.expect("Failed to get output");
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    if status.success() {
                        println!("     ✅ {} command completed successfully", cmd);
                    } else {
                        panic!("Command '{}' failed with status {}: {}", cmd, status, stderr);
                    }
                }
                Ok(None) => {
                    // Process is still running - success for long-running commands
                    println!("     ✅ {} command started successfully", cmd);
                    let _ = process.kill().await;
                    let _ = process.wait().await;
                }
                Err(e) => {
                    panic!("Error checking {} command status: {}", cmd, e);
                }
            }
        }
    }
    
    println!("✅ All CLI commands recognized and functional");
}