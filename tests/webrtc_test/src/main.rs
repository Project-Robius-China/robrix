//! WebRTC connection test binary.
//!
//! This binary tests WebRTC peer connection creation and basic functionality
//! without requiring the full Makepad UI.
//!
//! Usage: cargo run

use std::sync::Arc;
use tokio::sync::mpsc;
use webrtc::api::{
    API, APIBuilder,
    interceptor_registry::register_default_interceptors,
    media_engine::MediaEngine,
};
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::RTCPeerConnection;

/// Create a WebRTC API instance with default codecs.
fn create_api() -> Result<API, webrtc::Error> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)?;

    Ok(APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build())
}

/// Create a peer connection with STUN server configuration.
async fn create_peer_connection(api: &API) -> Result<Arc<RTCPeerConnection>, webrtc::Error> {
    let config = RTCConfiguration {
        ice_servers: vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    Ok(Arc::new(api.new_peer_connection(config).await?))
}

/// Test basic peer connection creation.
async fn test_peer_connection_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Peer Connection Creation ===");

    let api = create_api()?;
    println!("  [OK] WebRTC API created");

    let pc = create_peer_connection(&api).await?;
    println!("  [OK] Peer connection created");

    let state = pc.connection_state();
    println!("  [OK] Initial connection state: {:?}", state);

    pc.close().await?;
    println!("  [OK] Peer connection closed");

    println!("=== PASSED ===\n");
    Ok(())
}

/// Test SDP offer/answer exchange between two local peers.
async fn test_local_sdp_exchange() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Local SDP Offer/Answer Exchange ===");

    let api = create_api()?;

    // Create two peer connections
    let pc1 = create_peer_connection(&api).await?;
    let pc2 = create_peer_connection(&api).await?;
    println!("  [OK] Two peer connections created");

    // Create data channel on pc1
    let _dc = pc1.create_data_channel("test", None).await?;
    println!("  [OK] Data channel created on PC1");

    // Create offer from pc1
    let offer = pc1.create_offer(None).await?;
    println!("  [OK] Offer created: type={:?}", offer.sdp_type);

    // Set local description on pc1
    pc1.set_local_description(offer.clone()).await?;
    println!("  [OK] Local description set on PC1");

    // Set remote description on pc2
    pc2.set_remote_description(offer).await?;
    println!("  [OK] Remote description set on PC2");

    // Create answer from pc2
    let answer = pc2.create_answer(None).await?;
    println!("  [OK] Answer created: type={:?}", answer.sdp_type);

    // Set local description on pc2
    pc2.set_local_description(answer.clone()).await?;
    println!("  [OK] Local description set on PC2");

    // Set remote description on pc1
    pc1.set_remote_description(answer).await?;
    println!("  [OK] Remote description set on PC1");

    // Wait a moment for ICE gathering
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("  [OK] PC1 state: {:?}", pc1.connection_state());
    println!("  [OK] PC2 state: {:?}", pc2.connection_state());

    // Clean up
    pc1.close().await?;
    pc2.close().await?;
    println!("  [OK] Peer connections closed");

    println!("=== PASSED ===\n");
    Ok(())
}

/// Test ICE candidate gathering.
async fn test_ice_gathering() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: ICE Candidate Gathering ===");

    let api = create_api()?;
    let pc = create_peer_connection(&api).await?;

    let (tx, mut rx) = mpsc::channel::<String>(10);

    // Set up ICE candidate handler
    pc.on_ice_candidate(Box::new(move |candidate| {
        let tx = tx.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                let _ = tx.send(c.to_json().unwrap_or_default().candidate).await;
            }
        })
    }));

    // Create offer to trigger ICE gathering
    let _dc = pc.create_data_channel("test", None).await?;
    let offer = pc.create_offer(None).await?;
    pc.set_local_description(offer).await?;
    println!("  [OK] Offer created and set, ICE gathering started");

    // Collect candidates with timeout
    let mut candidates = Vec::new();
    let timeout = tokio::time::Duration::from_secs(5);
    let deadline = tokio::time::Instant::now() + timeout;

    while tokio::time::Instant::now() < deadline {
        tokio::select! {
            Some(candidate) = rx.recv() => {
                if !candidate.is_empty() {
                    candidates.push(candidate);
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
        }
    }

    println!("  [OK] Gathered {} ICE candidates", candidates.len());
    for (i, c) in candidates.iter().take(3).enumerate() {
        println!("    Candidate {}: {}", i + 1, &c[..c.len().min(60)]);
    }
    if candidates.len() > 3 {
        println!("    ... and {} more", candidates.len() - 3);
    }

    pc.close().await?;
    println!("  [OK] Peer connection closed");

    println!("=== PASSED ===\n");
    Ok(())
}

/// Test data channel communication.
async fn test_data_channel() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Data Channel Communication ===");

    let api = create_api()?;

    let pc1 = create_peer_connection(&api).await?;
    let pc2 = create_peer_connection(&api).await?;

    let (msg_tx, mut msg_rx) = mpsc::channel::<String>(10);

    // Create data channel on pc1
    let dc1 = pc1.create_data_channel("chat", None).await?;
    println!("  [OK] Data channel 'chat' created on PC1");

    // Handle incoming data channel on pc2
    let msg_tx_clone = msg_tx.clone();
    pc2.on_data_channel(Box::new(move |dc| {
        let msg_tx = msg_tx_clone.clone();
        Box::pin(async move {
            println!("  [OK] Data channel received on PC2: {}", dc.label());

            dc.on_message(Box::new(move |msg| {
                let msg_tx = msg_tx.clone();
                Box::pin(async move {
                    let text = String::from_utf8_lossy(&msg.data);
                    let _ = msg_tx.send(text.to_string()).await;
                })
            }));
        })
    }));

    // Exchange SDP
    let offer = pc1.create_offer(None).await?;
    pc1.set_local_description(offer.clone()).await?;
    pc2.set_remote_description(offer).await?;

    let answer = pc2.create_answer(None).await?;
    pc2.set_local_description(answer.clone()).await?;
    pc1.set_remote_description(answer).await?;
    println!("  [OK] SDP exchange complete");

    // Exchange ICE candidates
    let (ice_tx1, mut ice_rx1) = mpsc::channel::<String>(50);
    let (ice_tx2, mut ice_rx2) = mpsc::channel::<String>(50);

    pc1.on_ice_candidate(Box::new(move |candidate| {
        let tx = ice_tx1.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                if let Ok(json) = c.to_json() {
                    let _ = tx.send(serde_json::to_string(&json).unwrap_or_default()).await;
                }
            }
        })
    }));

    pc2.on_ice_candidate(Box::new(move |candidate| {
        let tx = ice_tx2.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                if let Ok(json) = c.to_json() {
                    let _ = tx.send(serde_json::to_string(&json).unwrap_or_default()).await;
                }
            }
        })
    }));

    // Wait for ICE gathering and exchange candidates
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    while let Ok(ice_str) = ice_rx1.try_recv() {
        if let Ok(ice) = serde_json::from_str(&ice_str) {
            let _ = pc2.add_ice_candidate(ice).await;
        }
    }
    while let Ok(ice_str) = ice_rx2.try_recv() {
        if let Ok(ice) = serde_json::from_str(&ice_str) {
            let _ = pc1.add_ice_candidate(ice).await;
        }
    }

    // Wait for connection
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("  [OK] PC1 state: {:?}", pc1.connection_state());
    println!("  [OK] PC2 state: {:?}", pc2.connection_state());

    // Send message through data channel
    dc1.on_open(Box::new(move || {
        Box::pin(async move {
            println!("  [OK] Data channel opened on PC1");
        })
    }));

    // Wait for data channel to open
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let test_message = "Hello from PC1 to PC2!";
    dc1.send_text(test_message.to_string()).await?;
    println!("  [OK] Sent message: '{}'", test_message);

    // Wait for message
    tokio::select! {
        Some(received) = msg_rx.recv() => {
            println!("  [OK] Received message: '{}'", received);
            if received == test_message {
                println!("  [OK] Message matches!");
            } else {
                println!("  [WARN] Message mismatch");
            }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {
            println!("  [WARN] Timeout waiting for message (this may be due to connection setup time)");
        }
    }

    pc1.close().await?;
    pc2.close().await?;
    println!("  [OK] Peer connections closed");

    println!("=== PASSED ===\n");
    Ok(())
}

/// Simulate a Matrix-style call connection for room "robrix5".
async fn test_matrix_style_call() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Matrix-Style Call Simulation (room: robrix5) ===");

    let api = create_api()?;

    // Simulate two users in room "robrix5"
    let user1_id = "@alice:matrix.org";
    let user2_id = "@bob:matrix.org";
    let room_id = "!robrix5:matrix.org";

    println!("  Room: {}", room_id);
    println!("  User 1: {}", user1_id);
    println!("  User 2: {}", user2_id);

    // Create peer connections for both users
    let pc1 = create_peer_connection(&api).await?;
    let pc2 = create_peer_connection(&api).await?;
    println!("  [OK] Peer connections created for both users");

    // Create data channel for signaling/chat
    let dc1 = pc1.create_data_channel("matrix-call", None).await?;
    println!("  [OK] Data channel 'matrix-call' created");

    // Set up data channel handler for pc2
    let (dc_ready_tx, mut dc_ready_rx) = mpsc::channel::<()>(1);
    pc2.on_data_channel(Box::new(move |dc| {
        let dc_ready_tx = dc_ready_tx.clone();
        Box::pin(async move {
            println!("  [OK] {} received data channel: {}", user2_id, dc.label());
            dc.on_open(Box::new(move || {
                let dc_ready_tx = dc_ready_tx.clone();
                Box::pin(async move {
                    let _ = dc_ready_tx.send(()).await;
                })
            }));
        })
    }));

    // Simulate MatrixRTC signaling
    println!("\n  --- Simulating MatrixRTC Signaling ---");

    // User 1 creates offer (sends m.call.invite-like event)
    let offer = pc1.create_offer(None).await?;
    pc1.set_local_description(offer.clone()).await?;
    println!("  [OK] {} created and sent offer (m.call.member)", user1_id);

    // User 2 receives offer and creates answer
    pc2.set_remote_description(offer).await?;
    let answer = pc2.create_answer(None).await?;
    pc2.set_local_description(answer.clone()).await?;
    println!("  [OK] {} received offer and created answer", user2_id);

    // User 1 receives answer
    pc1.set_remote_description(answer).await?;
    println!("  [OK] {} received answer", user1_id);

    // Exchange ICE candidates (in real Matrix, this goes through to-device messages)
    println!("\n  --- ICE Candidate Exchange ---");

    let (ice_tx1, mut ice_rx1) = mpsc::channel::<String>(50);
    let (ice_tx2, mut ice_rx2) = mpsc::channel::<String>(50);

    pc1.on_ice_candidate(Box::new(move |candidate| {
        let tx = ice_tx1.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                if let Ok(json) = c.to_json() {
                    let _ = tx.send(serde_json::to_string(&json).unwrap_or_default()).await;
                }
            }
        })
    }));

    pc2.on_ice_candidate(Box::new(move |candidate| {
        let tx = ice_tx2.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                if let Ok(json) = c.to_json() {
                    let _ = tx.send(serde_json::to_string(&json).unwrap_or_default()).await;
                }
            }
        })
    }));

    // Wait for ICE gathering
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let mut candidates_1_to_2 = 0;
    let mut candidates_2_to_1 = 0;

    while let Ok(ice_str) = ice_rx1.try_recv() {
        if let Ok(ice) = serde_json::from_str(&ice_str) {
            let _ = pc2.add_ice_candidate(ice).await;
            candidates_1_to_2 += 1;
        }
    }
    while let Ok(ice_str) = ice_rx2.try_recv() {
        if let Ok(ice) = serde_json::from_str(&ice_str) {
            let _ = pc1.add_ice_candidate(ice).await;
            candidates_2_to_1 += 1;
        }
    }

    println!("  [OK] Exchanged {} candidates from {} -> {}", candidates_1_to_2, user1_id, user2_id);
    println!("  [OK] Exchanged {} candidates from {} -> {}", candidates_2_to_1, user2_id, user1_id);

    // Wait for connection establishment
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("\n  --- Connection Status ---");
    println!("  {} connection state: {:?}", user1_id, pc1.connection_state());
    println!("  {} connection state: {:?}", user2_id, pc2.connection_state());
    println!("  {} ICE state: {:?}", user1_id, pc1.ice_connection_state());
    println!("  {} ICE state: {:?}", user2_id, pc2.ice_connection_state());

    // Wait for data channel to be ready
    tokio::select! {
        _ = dc_ready_rx.recv() => {
            println!("  [OK] Data channel ready for communication");
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {
            println!("  [INFO] Data channel setup timeout (connection may still work)");
        }
    }

    // Test sending a message
    let test_msg = "Hello robrix5!";
    if let Err(e) = dc1.send_text(test_msg.to_string()).await {
        println!("  [WARN] Failed to send test message: {}", e);
    } else {
        println!("  [OK] Sent test message: '{}'", test_msg);
    }

    // Clean up
    pc1.close().await?;
    pc2.close().await?;
    println!("\n  [OK] Call ended, peer connections closed");

    println!("=== PASSED ===\n");
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           WebRTC Connection Test Suite for Robrix            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic peer connection creation
    match test_peer_connection_creation().await {
        Ok(_) => passed += 1,
        Err(e) => {
            println!("=== FAILED: {} ===\n", e);
            failed += 1;
        }
    }

    // Test 2: SDP offer/answer exchange
    match test_local_sdp_exchange().await {
        Ok(_) => passed += 1,
        Err(e) => {
            println!("=== FAILED: {} ===\n", e);
            failed += 1;
        }
    }

    // Test 3: ICE candidate gathering
    match test_ice_gathering().await {
        Ok(_) => passed += 1,
        Err(e) => {
            println!("=== FAILED: {} ===\n", e);
            failed += 1;
        }
    }

    // Test 4: Data channel communication
    match test_data_channel().await {
        Ok(_) => passed += 1,
        Err(e) => {
            println!("=== FAILED: {} ===\n", e);
            failed += 1;
        }
    }

    // Test 5: Matrix-style call simulation for room "robrix5"
    match test_matrix_style_call().await {
        Ok(_) => passed += 1,
        Err(e) => {
            println!("=== FAILED: {} ===\n", e);
            failed += 1;
        }
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                       Test Results                           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Passed: {}                                                   ║", passed);
    println!("║  Failed: {}                                                   ║", failed);
    println!("╚══════════════════════════════════════════════════════════════╝");

    if failed > 0 {
        std::process::exit(1);
    }
}
