use blobby_core::net::{NetworkHost, NetworkClient, ClientPacket, LanRoomBeacon, query_public_endpoint};
use blobby_core::physics::{PhysicWorld, PlayerInput};
use blobby_core::rules::{GameLogic, RuleMode};
use std::thread;
use std::time::Duration;

#[test]
fn test_packet_serialization() {
    let input = PlayerInput {
        left: true,
        right: false,
        up: true,
    };
    let json = serde_json::to_string(&ClientPacket::Input(input)).unwrap();
    let deserialized: ClientPacket = serde_json::from_str(&json).unwrap();
    if let ClientPacket::Input(inp) = deserialized {
        assert_eq!(input.left, inp.left);
        assert_eq!(input.right, inp.right);
        assert_eq!(input.up, inp.up);
    } else {
        panic!("Deserialized packet mismatch");
    }
}

#[test]
fn test_host_client_communication() {
    let test_port = 19283;
    let mut host = NetworkHost::bind(test_port, "Tester1", "Copacabana Sunset").expect("Host should bind to test port");
    let mut client = NetworkClient::new();

    client.connect(&format!("127.0.0.1:{}", test_port));

    // Give OS socket a moment to establish handshake
    thread::sleep(Duration::from_millis(50));
    assert!(host.poll_connection(), "Host should accept incoming connection");
    assert!(host.is_client_connected);
    assert!(client.is_connected);

    // Host sends game state
    let physics = PhysicWorld::new();
    let logic = GameLogic::new(RuleMode::Modern, 15);
    host.send_game_state(&physics, &logic, &[]);

    // Give packet time to transit
    thread::sleep(Duration::from_millis(20));
    client.update();

    assert!(client.latest_state.is_some(), "Client should receive game state");
    let state = client.latest_state.as_ref().unwrap();
    assert_eq!(state.scores, [0, 0]);

    // Client sends input
    client.send_input(PlayerInput { left: true, right: false, up: true });
    thread::sleep(Duration::from_millis(20));
    host.update(1.0);

    assert_eq!(host.client_input.left, true);
    assert_eq!(host.client_input.right, false);
    assert_eq!(host.client_input.up, true);

    // Chat bidirectional test
    client.send_chat("P2", "Nice shot!");
    thread::sleep(Duration::from_millis(20));
    host.update(1.1);
    assert_eq!(host.new_chat_messages.len(), 1);
    assert_eq!(host.new_chat_messages[0], ("P2".to_string(), "Nice shot!".to_string()));

    host.send_chat("Host", "Thanks!");
    thread::sleep(Duration::from_millis(20));
    client.update();
    assert_eq!(client.new_chat_messages.len(), 1);
    assert_eq!(client.new_chat_messages[0], ("Host".to_string(), "Thanks!".to_string()));
}

#[test]
fn test_stun_resolution() {
    // Queries public STUN servers for reflexive NAT endpoint
    if let Some(endpoint) = query_public_endpoint() {
        println!("STUN resolved public endpoint: {}", endpoint);
        assert!(!endpoint.ip().is_unspecified(), "STUN endpoint must have valid IPv4 address");
        assert_ne!(endpoint.port(), 0, "STUN port must be valid non-zero port");
    }
}

#[test]
fn test_lan_discovery_beacon() {
    let beacon = LanRoomBeacon {
        app: "blobby_volley_3".to_string(),
        name: "Test Arena".to_string(),
        host: "PlayerTest".to_string(),
        port: 1234,
        arena: "Neo Cyber Stadium".to_string(),
    };

    let encoded = serde_json::to_vec(&beacon).expect("Beacon serialization must succeed");
    let decoded: LanRoomBeacon = serde_json::from_slice(&encoded).expect("Beacon deserialization must succeed");
    assert_eq!(decoded.name, "Test Arena");
    assert_eq!(decoded.port, 1234);
    assert_eq!(decoded.arena, "Neo Cyber Stadium");
}
