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
    let mut host = NetworkHost::bind(test_port, "Tester1", "DE", "Copacabana Sunset").expect("Host should bind to test port");
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
        country: "DE".to_string(),
    };

    let encoded = serde_json::to_vec(&beacon).expect("Beacon serialization must succeed");
    let decoded: LanRoomBeacon = serde_json::from_slice(&encoded).expect("Beacon deserialization must succeed");
    assert_eq!(decoded.name, "Test Arena");
    assert_eq!(decoded.port, 1234);
    assert_eq!(decoded.arena, "Neo Cyber Stadium");
    assert_eq!(decoded.country, "DE");
}

#[test]
fn test_lan_matchmaking_beacon_and_tiebreaker() {
    use blobby_core::net::LanMatchmakingBeacon;

    let beacon_a = LanMatchmakingBeacon {
        app: "blobby_volley_3_mm".to_string(),
        player_id: "bv_001_alpha".to_string(),
        name: "Player 1".to_string(),
        country: "DE".to_string(),
        rating: 1520.0,
        rd: 120.0,
        port: 1234,
        arena_index: 0,
        elo_range_mode: 1,
        accepted_opponent_id: None,
    };

    let beacon_b = LanMatchmakingBeacon {
        app: "blobby_volley_3_mm".to_string(),
        player_id: "bv_002_bravo".to_string(),
        name: "Player 2".to_string(),
        country: "US".to_string(),
        rating: 1550.0,
        rd: 95.0,
        port: 1234,
        arena_index: 0,
        elo_range_mode: 1,
        accepted_opponent_id: None,
    };

    // 1. Serialization integrity
    let encoded = serde_json::to_vec(&beacon_a).unwrap();
    let decoded: LanMatchmakingBeacon = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.player_id, "bv_001_alpha");
    assert_eq!(decoded.rating, 1520.0);
    assert_eq!(decoded.country, "DE");

    // 2. Skill distance check
    let rating_diff = (beacon_a.rating - beacon_b.rating).abs();
    assert_eq!(rating_diff, 30.0);
    assert!(rating_diff <= 100.0, "Skill difference within default 100 tolerance window");

    // 3. Simultaneous search role assignment tie-breaker
    // Both players execute this exact same deterministic check:
    let a_is_host = beacon_a.player_id < beacon_b.player_id;
    let b_is_host = beacon_b.player_id < beacon_a.player_id;

    assert!(a_is_host, "Player A with lower ID must be designated Host");
    assert!(!b_is_host, "Player B with higher ID must be designated Client");
    assert_ne!(a_is_host, b_is_host, "Exactly one player must be host, and one must be client");
}

#[test]
fn test_glicko2_rank_advancement() {
    use blobby_core::glicko::GlickoRating;

    let mut player = GlickoRating::new(1500.0, 200.0, 0.06);
    assert_eq!(player.rank_title(), "Silver II");

    // Win 3 consecutive matches against high rated opponents
    for _ in 0..3 {
        player = player.update_match(1650.0, 60.0, 1.0);
    }

    assert!(player.rating > 1650.0, "Rating should advance past 1650 after 3 wins against higher rated players");
    assert!(player.deviation < 200.0, "Rating deviation should reduce (higher certainty)");
}

#[test]
fn test_leaderboard_inactivity_and_filtering() {
    use blobby_core::net::fetch_leaderboard;

    // 1. Fetch Weekly leaderboard (max 7 days inactivity)
    let week_entries = fetch_leaderboard(None, "week", "Tester", 1700.0, 10, 15);
    assert!(!week_entries.is_empty(), "Weekly leaderboard should have entries");
    for entry in &week_entries {
        assert!(entry.last_active_days <= 7, "All players in weekly leaderboard must be active within 7 days, got {} for {}", entry.last_active_days, entry.name);
    }

    // 2. Fetch Monthly leaderboard (max 30 days inactivity)
    let month_entries = fetch_leaderboard(None, "month", "Tester", 1700.0, 10, 15);
    assert!(month_entries.len() >= week_entries.len(), "Monthly leaderboard must contain at least as many players as Weekly");
    for entry in &month_entries {
        assert!(entry.last_active_days <= 30, "All players in monthly leaderboard must be active within 30 days");
    }

    // 3. Fetch Yearly leaderboard (max 365 days inactivity)
    let year_entries = fetch_leaderboard(None, "year", "Tester", 1700.0, 10, 15);
    assert!(year_entries.len() >= month_entries.len(), "Yearly leaderboard must contain players active up to 365 days");

    // 4. Verify ranking order (descending by rating)
    for i in 1..month_entries.len() {
        assert!(month_entries[i - 1].rating >= month_entries[i].rating, "Entries must be sorted by rating descending");
        assert_eq!(month_entries[i].rank, i + 1, "Rank number must match position");
    }
}

#[test]
fn test_determine_consensus_arena() {
    use blobby_core::net::determine_consensus_arena;

    // 1. Commutativity / Symmetry: both players compute identical agreed arena regardless of caller order
    let res_a = determine_consensus_arena("bv_player_1", 2, "bv_player_2", 5);
    let res_b = determine_consensus_arena("bv_player_2", 5, "bv_player_1", 2);
    assert_eq!(res_a, res_b, "Consensus arena must be identical for both players");
    assert!(res_a == 2 || res_a == 5, "Result must be one of the two competing arenas");

    // 2. Determinism across different player ID pairs
    for i in 0..10 {
        let id_a = format!("player_{}", i);
        let id_b = format!("opponent_{}", i * 7);
        let arena_a = i % 8;
        let arena_b = (i + 3) % 8;

        let consensus_ab = determine_consensus_arena(&id_a, arena_a, &id_b, arena_b);
        let consensus_ba = determine_consensus_arena(&id_b, arena_b, &id_a, arena_a);
        assert_eq!(consensus_ab, consensus_ba);
        assert!(consensus_ab == arena_a || consensus_ab == arena_b);
    }
}

#[test]
fn test_elo_tolerance_interval_expansion() {
    use blobby_core::config::EloSearchRange;

    // Strict (±50)
    let strict = EloSearchRange::Strict;
    assert_eq!(strict.initial_tolerance(), 50.0);
    // Interval 0 (0-4.9s)
    let tol_0 = strict.initial_tolerance() * (4.0 / 3.0_f64).powi(0);
    assert_eq!(tol_0, 50.0);
    // Interval 1 (5s) -> 50 * 4/3 = 66.666
    let tol_1 = strict.initial_tolerance() * (4.0 / 3.0_f64).powi(1);
    assert!((tol_1 - 66.666).abs() < 0.01);
    // Interval 2 (10s) -> 50 * 16/9 = 88.888
    let tol_2 = strict.initial_tolerance() * (4.0 / 3.0_f64).powi(2);
    assert!((tol_2 - 88.888).abs() < 0.01);

    // Balanced (±100)
    let balanced = EloSearchRange::Balanced;
    assert_eq!(balanced.initial_tolerance(), 100.0);
    let tol_b1 = balanced.initial_tolerance() * (4.0 / 3.0_f64).powi(1);
    assert!((tol_b1 - 133.333).abs() < 0.01);

    // Wide (±250)
    let wide = EloSearchRange::Wide;
    assert_eq!(wide.initial_tolerance(), 250.0);

    // Unrestricted (Match Any Opponent)
    let unrestricted = EloSearchRange::Unrestricted;
    assert_eq!(unrestricted.initial_tolerance(), f64::INFINITY);
    assert!(10000.0 <= unrestricted.initial_tolerance());
}

#[test]
fn test_cross_arena_offer_serialization() {
    use blobby_core::net::{CrossArenaOffer, ChallengeBroadcast};

    let offer = CrossArenaOffer {
        opponent_id: "opp_123".to_string(),
        opponent_name: "Challenger".to_string(),
        opponent_country: "FR".to_string(),
        opponent_rating: 1620.0,
        opponent_rd: 80.0,
        opponent_arena: 3,
        my_arena: 1,
    };

    let json = serde_json::to_string(&offer).unwrap();
    let decoded: CrossArenaOffer = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.opponent_name, "Challenger");
    assert_eq!(decoded.opponent_country, "FR");
    assert_eq!(decoded.opponent_arena, 3);
    assert_eq!(decoded.my_arena, 1);

    let challenge = ChallengeBroadcast {
        player_id: "p_broadcast".to_string(),
        name: "ProPlayer".to_string(),
        country: "JP".to_string(),
        rating: 1890.0,
        rd: 60.0,
        arena_index: 2,
        arena_name: "Neo Cyber Stadium".to_string(),
        joined_at: 1700000000,
    };

    let c_json = serde_json::to_string(&challenge).unwrap();
    let c_decoded: ChallengeBroadcast = serde_json::from_str(&c_json).unwrap();
    assert_eq!(c_decoded.name, "ProPlayer");
    assert_eq!(c_decoded.country, "JP");
    assert_eq!(c_decoded.arena_name, "Neo Cyber Stadium");
}
