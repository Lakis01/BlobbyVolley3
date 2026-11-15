// Blobby Volley 3 - Online & LAN Network Multiplayer Engine (Zero-Cost P2P + STUN + Serverless)
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddrV4, Ipv4Addr, ToSocketAddrs};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::physics::{PhysicWorld, PlayerInput, MatchEvent};
use crate::rules::GameLogic;
use crate::config::EloSearchRange;

pub const DEFAULT_PORT: u16 = 1234;
pub const LAN_DISCOVERY_PORT: u16 = 1235;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    Input(PlayerInput),
    Chat { sender: String, text: String },
    Ping(u64),
    Hello { name: String, country: String, rating: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchConfigData {
    pub score_to_win: i32,
    pub rule_mode_index: usize,
    pub arena_index: usize,
    pub host_theme_index: usize,
    pub client_theme_index: usize,
    pub host_name: String,
    pub host_country: String,
    pub host_rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    GameState(NetGameState),
    Chat { sender: String, text: String },
    Pong(u64),
    MatchConfig {
        score_to_win: i32,
        rule_mode_index: usize,
        arena_index: usize,
        host_theme_index: usize,
        client_theme_index: usize,
        host_name: String,
        host_country: String,
        host_rating: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetGameState {
    pub ball_pos: (f32, f32),
    pub ball_vel: (f32, f32),
    pub ball_rotation: f32,
    pub p1_pos: (f32, f32),
    pub p1_vel: (f32, f32),
    pub p1_squash: f32,
    pub p2_pos: (f32, f32),
    pub p2_vel: (f32, f32),
    pub p2_squash: f32,
    pub scores: [i32; 2],
    pub serving_player: usize,
    pub is_game_running: bool,
    pub is_ball_valid: bool,
    pub events: Vec<NetEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetEvent {
    HitBlob { player: usize, intensity: f32 },
    HitGround { side: usize },
    HitWall,
    HitNet,
    ScorePoint,
    Whistle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanRoomBeacon {
    pub app: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub arena: String,
    #[serde(default)]
    pub country: String,
}

pub struct NetworkHost {
    listener: TcpListener,
    client_stream: Option<TcpStream>,
    reader: Option<BufReader<TcpStream>>,
    pub client_input: PlayerInput,
    pub is_client_connected: bool,
    pub new_chat_messages: Vec<(String, String)>,
    pub room_name: String,
    pub host_name: String,
    pub country: String,
    pub arena_name: String,
    pub port: u16,
    pub local_ip: String,
    pub public_endpoint: Option<String>,
    pub match_config: Option<MatchConfigData>,
    pub client_hello: Option<(String, String, f64)>,
    stun_rx: Option<mpsc::Receiver<Option<String>>>,
    broadcast_socket: Option<UdpSocket>,
    last_broadcast_time: f32,
}

impl NetworkHost {
    pub fn bind(port: u16, host_name: &str, country: &str, arena_name: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        listener.set_nonblocking(true)?;

        let local_ip = get_local_ip();

        // Query public STUN endpoint in a background thread so bind() never stalls the UI
        let (stun_tx, stun_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let ep = query_public_endpoint().map(|ep| format!("{}:{}", ep.ip(), port));
            let _ = stun_tx.send(ep);
        });

        // Setup LAN UDP broadcast transmitter
        let broadcast_socket = UdpSocket::bind("0.0.0.0:0").ok().and_then(|sock| {
            let _ = sock.set_broadcast(true);
            let _ = sock.set_nonblocking(true);
            Some(sock)
        });

        Ok(Self {
            listener,
            client_stream: None,
            reader: None,
            client_input: PlayerInput::default(),
            is_client_connected: false,
            new_chat_messages: Vec::new(),
            room_name: format!("{}'s Match", host_name),
            host_name: host_name.to_string(),
            country: country.to_string(),
            arena_name: arena_name.to_string(),
            port,
            local_ip,
            public_endpoint: None,
            match_config: None,
            client_hello: None,
            stun_rx: Some(stun_rx),
            broadcast_socket,
            last_broadcast_time: -10.0,
        })
    }

    #[allow(dead_code)]
    pub fn set_match_config(&mut self, config: MatchConfigData) {
        self.match_config = Some(config);
        if self.is_client_connected {
            self.send_match_config();
        }
    }

    pub fn send_match_config(&mut self) {
        if !self.is_client_connected {
            return;
        }
        if let Some(cfg) = self.match_config.clone() {
            self.send_packet(&ServerPacket::MatchConfig {
                score_to_win: cfg.score_to_win,
                rule_mode_index: cfg.rule_mode_index,
                arena_index: cfg.arena_index,
                host_theme_index: cfg.host_theme_index,
                client_theme_index: cfg.client_theme_index,
                host_name: cfg.host_name,
                host_country: cfg.host_country,
                host_rating: cfg.host_rating,
            });
        }
    }

    pub fn poll_connection(&mut self) -> bool {
        if self.is_client_connected {
            return true;
        }

        if let Ok((stream, _addr)) = self.listener.accept() {
            let _ = stream.set_nodelay(true);
            let _ = stream.set_nonblocking(true);

            if let Ok(stream_clone) = stream.try_clone() {
                self.reader = Some(BufReader::new(stream_clone));
                self.client_stream = Some(stream);
                self.is_client_connected = true;
                self.send_match_config();
                return true;
            }
        }
        false
    }

    pub fn update(&mut self, current_time: f32) {
        if let Some(ref rx) = self.stun_rx {
            match rx.try_recv() {
                Ok(ep) => {
                    self.public_endpoint = ep;
                    self.stun_rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.stun_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        if !self.is_client_connected {
            self.poll_connection();

            // Broadcast LAN discovery beacon every 1.5 seconds
            if current_time - self.last_broadcast_time >= 1.5 {
                self.last_broadcast_time = current_time;
                if let Some(ref sock) = self.broadcast_socket {
                    let beacon = LanRoomBeacon {
                        app: "blobby_volley_3".to_string(),
                        name: self.room_name.clone(),
                        host: self.host_name.clone(),
                        port: self.port,
                        arena: self.arena_name.clone(),
                        country: self.country.clone(),
                    };
                    if let Ok(json) = serde_json::to_vec(&beacon) {
                        let _ = sock.send_to(&json, format!("255.255.255.255:{}", LAN_DISCOVERY_PORT));
                    }
                }
            }
            return;
        }

        // Read all available client packets
        let mut pongs_to_send = Vec::new();
        if let Some(ref mut reader) = self.reader {
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    // Client disconnected
                    self.is_client_connected = false;
                    self.client_stream = None;
                    break;
                }

                if let Ok(packet) = serde_json::from_str::<ClientPacket>(line.trim()) {
                    match packet {
                        ClientPacket::Input(input) => {
                            self.client_input = input;
                        }
                        ClientPacket::Chat { sender, text } => {
                            self.new_chat_messages.push((sender, text));
                        }
                        ClientPacket::Ping(t) => {
                            pongs_to_send.push(t);
                        }
                        ClientPacket::Hello { name, country, rating } => {
                            self.client_hello = Some((name, country, rating));
                        }
                    }
                }
                line.clear();
            }
        }

        if !self.is_client_connected {
            self.reader = None;
        }

        for t in pongs_to_send {
            self.send_packet(&ServerPacket::Pong(t));
        }
    }

    pub fn send_game_state(&mut self, physics: &PhysicWorld, logic: &GameLogic, events: &[MatchEvent]) {
        if !self.is_client_connected {
            return;
        }

        let net_events: Vec<NetEvent> = events
            .iter()
            .filter_map(|e| match e {
                MatchEvent::BallHitBlob { player, intensity } => Some(NetEvent::HitBlob {
                    player: player.index(),
                    intensity: *intensity,
                }),
                MatchEvent::BallHitGround { side } => Some(NetEvent::HitGround {
                    side: side.index(),
                }),
                MatchEvent::BallHitWall { .. } => Some(NetEvent::HitWall),
                MatchEvent::BallHitNet { .. } | MatchEvent::BallHitNetTop => Some(NetEvent::HitNet),
                MatchEvent::BallCrossedNet { .. } => None,
            })
            .collect();

        let state = NetGameState {
            ball_pos: (physics.ball.pos.x, physics.ball.pos.y),
            ball_vel: (physics.ball.vel.x, physics.ball.vel.y),
            ball_rotation: physics.ball.rotation,
            p1_pos: (physics.blobs[0].pos.x, physics.blobs[0].pos.y),
            p1_vel: (physics.blobs[0].vel.x, physics.blobs[0].vel.y),
            p1_squash: physics.blobs[0].squash_factor,
            p2_pos: (physics.blobs[1].pos.x, physics.blobs[1].pos.y),
            p2_vel: (physics.blobs[1].vel.x, physics.blobs[1].vel.y),
            p2_squash: physics.blobs[1].squash_factor,
            scores: logic.scores,
            serving_player: logic.serving_player.index(),
            is_game_running: logic.is_game_running,
            is_ball_valid: logic.is_ball_valid,
            events: net_events,
        };

        self.send_packet(&ServerPacket::GameState(state));
    }

    pub fn send_chat(&mut self, sender: &str, text: &str) {
        self.send_packet(&ServerPacket::Chat {
            sender: sender.to_string(),
            text: text.to_string(),
        });
    }

    fn send_packet(&mut self, packet: &ServerPacket) {
        if let Some(ref mut stream) = self.client_stream {
            if let Ok(mut json) = serde_json::to_string(packet) {
                json.push('\n');
                let _ = stream.write_all(json.as_bytes());
            }
        }
    }
}

pub struct NetworkClient {
    stream: Option<TcpStream>,
    reader: Option<BufReader<TcpStream>>,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub latest_state: Option<NetGameState>,
    pub pending_events: Vec<NetEvent>,
    pub received_match_config: Option<MatchConfigData>,
    pub new_chat_messages: Vec<(String, String)>,
    #[allow(dead_code)]
    pub ping_ms: u32,
    pub error_message: Option<String>,
    connect_rx: Option<mpsc::Receiver<Result<TcpStream, String>>>,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            reader: None,
            is_connected: false,
            is_connecting: false,
            latest_state: None,
            pending_events: Vec::new(),
            received_match_config: None,
            new_chat_messages: Vec::new(),
            ping_ms: 0,
            error_message: None,
            connect_rx: None,
        }
    }

    #[allow(dead_code)]
    pub fn take_events(&mut self) -> Vec<NetEvent> {
        std::mem::take(&mut self.pending_events)
    }

    #[allow(dead_code)]
    pub fn connect(&mut self, target_addr: &str) {
        self.is_connecting = true;
        self.error_message = None;
        self.connect_rx = None;
        self.pending_events.clear();
        self.received_match_config = None;

        let addr = if target_addr.contains(':') {
            target_addr.to_string()
        } else {
            format!("{}:{}", target_addr, DEFAULT_PORT)
        };

        let resolved = addr
            .to_socket_addrs()
            .ok()
            .and_then(|mut it| it.next())
            .unwrap_or_else(|| "127.0.0.1:1234".parse().unwrap());

        match TcpStream::connect_timeout(&resolved, Duration::from_millis(3000)) {
            Ok(stream) => {
                let _ = stream.set_nodelay(true);
                let _ = stream.set_nonblocking(true);

                if let Ok(stream_clone) = stream.try_clone() {
                    self.reader = Some(BufReader::new(stream_clone));
                    self.stream = Some(stream);
                    self.is_connected = true;
                    self.is_connecting = false;
                }
            }
            Err(e) => {
                self.is_connecting = false;
                self.is_connected = false;
                self.error_message = Some(format!("Connection failed: {}", e));
            }
        }
    }

    #[allow(dead_code)]
    pub fn connect_async(&mut self, target_addr: &str) {
        self.is_connecting = true;
        self.is_connected = false;
        self.error_message = None;
        self.stream = None;
        self.reader = None;
        self.pending_events.clear();
        self.received_match_config = None;

        let addr = if target_addr.contains(':') {
            target_addr.to_string()
        } else {
            format!("{}:{}", target_addr, DEFAULT_PORT)
        };

        let (tx, rx) = mpsc::channel();
        self.connect_rx = Some(rx);
        std::thread::spawn(move || {
            let res = match addr.to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(sa) = addrs.next() {
                        TcpStream::connect_timeout(&sa, Duration::from_millis(3000))
                            .map_err(|e| format!("Connection failed: {}", e))
                    } else {
                        Err("Could not resolve host address".to_string())
                    }
                }
                Err(e) => Err(format!("Invalid address: {}", e)),
            };
            let _ = tx.send(res);
        });
    }

    pub fn update(&mut self) {
        if let Some(ref rx) = self.connect_rx {
            match rx.try_recv() {
                Ok(Ok(stream)) => {
                    let _ = stream.set_nodelay(true);
                    let _ = stream.set_nonblocking(true);
                    if let Ok(stream_clone) = stream.try_clone() {
                        self.reader = Some(BufReader::new(stream_clone));
                        self.stream = Some(stream);
                        self.is_connected = true;
                        self.is_connecting = false;
                    } else {
                        self.is_connecting = false;
                        self.is_connected = false;
                        self.error_message = Some("Failed to clone client socket".to_string());
                    }
                    self.connect_rx = None;
                }
                Ok(Err(e)) => {
                    self.is_connecting = false;
                    self.is_connected = false;
                    self.error_message = Some(e);
                    self.connect_rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.is_connecting = false;
                    self.is_connected = false;
                    self.error_message = Some("Connection worker terminated".to_string());
                    self.connect_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        if !self.is_connected {
            return;
        }

        if let Some(ref mut reader) = self.reader {
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    self.is_connected = false;
                    self.error_message = Some("Disconnected from host".to_string());
                    break;
                }

                if let Ok(packet) = serde_json::from_str::<ServerPacket>(line.trim()) {
                    match packet {
                        ServerPacket::GameState(state) => {
                            self.pending_events.extend(state.events.iter().cloned());
                            self.latest_state = Some(state);
                        }
                        ServerPacket::Chat { sender, text } => {
                            self.new_chat_messages.push((sender, text));
                        }
                        ServerPacket::Pong(_) => {}
                        ServerPacket::MatchConfig {
                            score_to_win,
                            rule_mode_index,
                            arena_index,
                            host_theme_index,
                            client_theme_index,
                            host_name,
                            host_country,
                            host_rating,
                        } => {
                            self.received_match_config = Some(MatchConfigData {
                                score_to_win,
                                rule_mode_index,
                                arena_index,
                                host_theme_index,
                                client_theme_index,
                                host_name,
                                host_country,
                                host_rating,
                            });
                        }
                    }
                }
                line.clear();
            }
        }
    }

    pub fn send_input(&mut self, input: PlayerInput) {
        self.send_packet(&ClientPacket::Input(input));
    }

    pub fn send_chat(&mut self, sender: &str, text: &str) {
        self.send_packet(&ClientPacket::Chat {
            sender: sender.to_string(),
            text: text.to_string(),
        });
    }

    #[allow(dead_code)]
    pub fn send_hello(&mut self, name: &str, country: &str, rating: f64) {
        self.send_packet(&ClientPacket::Hello {
            name: name.to_string(),
            country: country.to_string(),
            rating,
        });
    }

    fn send_packet(&mut self, packet: &ClientPacket) {
        if let Some(ref mut stream) = self.stream {
            if let Ok(mut json) = serde_json::to_string(packet) {
                json.push('\n');
                if stream.write_all(json.as_bytes()).is_err() {
                    self.is_connected = false;
                    self.error_message = Some("Lost connection to host".to_string());
                }
            }
        }
    }
}

/// Detects local LAN IP address
pub fn get_local_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// STUN Client (RFC 5389 / RFC 3489)
/// Queries public STUN servers to discover reflexive external IP and port for zero-cost P2P matches
pub fn query_public_endpoint() -> Option<SocketAddrV4> {
    let stun_servers = [
        "stun.l.google.com:19302",
        "stun1.l.google.com:19302",
        "stun.cloudflare.com:3478",
    ];

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    let _ = socket.set_read_timeout(Some(Duration::from_millis(800)));

    for server_str in stun_servers {
        if let Ok(mut addrs) = server_str.to_socket_addrs() {
            if let Some(server_addr) = addrs.next() {
                // STUN Binding Request packet (20 bytes)
                let mut packet = [0u8; 20];
                packet[0] = 0x00;
                packet[1] = 0x01; // Binding Request
                packet[4] = 0x21;
                packet[5] = 0x12;
                packet[6] = 0xA4;
                packet[7] = 0x42; // Magic Cookie
                for i in 8..20 {
                    packet[i] = (quad_rand::rand() & 0xFF) as u8;
                }

                if socket.send_to(&packet, server_addr).is_ok() {
                    let mut buf = [0u8; 512];
                    if let Ok((len, _)) = socket.recv_from(&mut buf) {
                        if len >= 20 {
                            let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
                            let magic = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
                            if msg_type == 0x0101 && magic == 0x2112A442 {
                                let mut pos = 20;
                                while pos + 4 <= len {
                                    let attr_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
                                    let attr_len = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]) as usize;
                                    pos += 4;
                                    if pos + attr_len > len {
                                        break;
                                    }
                                    let attr_val = &buf[pos..pos + attr_len];
                                    if attr_type == 0x0020 && attr_len >= 8 {
                                        // XOR-MAPPED-ADDRESS
                                        if attr_val[1] == 0x01 {
                                            let xport = u16::from_be_bytes([attr_val[2], attr_val[3]]);
                                            let port = xport ^ 0x2112;
                                            let xip = u32::from_be_bytes([attr_val[4], attr_val[5], attr_val[6], attr_val[7]]);
                                            let ip = xip ^ 0x2112A442;
                                            return Some(SocketAddrV4::new(Ipv4Addr::from(ip), port));
                                        }
                                    } else if attr_type == 0x0001 && attr_len >= 8 {
                                        // MAPPED-ADDRESS
                                        if attr_val[1] == 0x01 {
                                            let port = u16::from_be_bytes([attr_val[2], attr_val[3]]);
                                            let ip = u32::from_be_bytes([attr_val[4], attr_val[5], attr_val[6], attr_val[7]]);
                                            return Some(SocketAddrV4::new(Ipv4Addr::from(ip), port));
                                        }
                                    }
                                    pos += attr_len;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// LAN Discovery Listener: Listens on UDP port 1235 for automatic local network room beacons
pub struct LanDiscoveryListener {
    socket: Option<UdpSocket>,
}

impl LanDiscoveryListener {
    pub fn new() -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", LAN_DISCOVERY_PORT)).ok().and_then(|s| {
            let _ = s.set_nonblocking(true);
            Some(s)
        });
        Self { socket }
    }

    pub fn poll_rooms(&mut self) -> Vec<ServerEntry> {
        let mut rooms = Vec::new();
        if let Some(ref socket) = self.socket {
            let mut buf = [0u8; 1024];
            while let Ok((len, src_addr)) = socket.recv_from(&mut buf) {
                if let Ok(beacon) = serde_json::from_slice::<LanRoomBeacon>(&buf[..len]) {
                    if beacon.app == "blobby_volley_3" {
                        let display_name = if beacon.country.is_empty() {
                            format!("{} ({})", beacon.name, beacon.arena)
                        } else {
                            format!("[{}] {} ({})", beacon.country, beacon.name, beacon.arena)
                        };
                        rooms.push(ServerEntry {
                            name: display_name,
                            host: src_addr.ip().to_string(),
                            port: beacon.port,
                            server_type: "LAN Room (Auto-Discovered)".to_string(),
                            ping_ms: Some(1),
                        });
                    }
                }
            }
        }
        rooms
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub server_type: String,
    pub ping_ms: Option<u32>,
}

/// Parses server entries from Blobby Volley 2 XML response
#[allow(dead_code)]
pub fn parse_server_xml(xml: &str) -> Vec<ServerEntry> {
    let mut list = Vec::new();
    let mut current_host = String::new();
    for line in xml.lines() {
        let trimmed = line.trim();
        if trimmed.contains("host=\"") {
            if let Some(start) = trimmed.find("host=\"") {
                let rest = &trimmed[start + 6..];
                if let Some(end) = rest.find('"') {
                    current_host = rest[..end].to_string();
                }
            }
        } else if trimmed.contains("port=\"") && !current_host.is_empty() {
            if let Some(start) = trimmed.find("port=\"") {
                let rest = &trimmed[start + 6..];
                if let Some(end) = rest.find('"') {
                    if let Ok(port) = rest[..end].parse::<u16>() {
                        let is_official = current_host.contains("blobbyvolley.de");
                        let s_name = if is_official {
                            format!("Official Dedicated ({})", current_host)
                        } else {
                            format!("Community Dedicated ({})", current_host)
                        };
                        list.push(ServerEntry {
                            name: s_name,
                            host: current_host.clone(),
                            port,
                            server_type: "Dedicated Server".to_string(),
                            ping_ms: None,
                        });
                    }
                }
            }
            current_host.clear();
        }
    }
    list
}

/// Queries available servers: LAN auto-discovered rooms, optional coordinator registry rooms, and local loopback
#[allow(dead_code)]
pub fn query_master_server() -> Vec<ServerEntry> {
    query_master_server_with_registry(None)
}

/// Queries available servers with an optional HTTP/HTTPS coordinator registry URL
pub fn query_master_server_with_registry(coordinator_url: Option<&str>) -> Vec<ServerEntry> {
    let mut list = Vec::new();

    // 1. Poll LAN Discovery Listener for active rooms on the local network
    let mut lan_listener = LanDiscoveryListener::new();
    let lan_rooms = lan_listener.poll_rooms();
    for room in lan_rooms {
        if !list.iter().any(|s: &ServerEntry| s.host == room.host && s.port == room.port) {
            list.push(room);
        }
    }

    // 2. Optional remote registry /rooms query via ureq (HTTP & HTTPS)
    if let Some(raw_url) = coordinator_url {
        if let Some(base) = normalize_base_url(raw_url) {
            let endpoint = format!("{}/rooms", base);
            let agent = build_http_agent();
            if let Ok(resp) = agent.get(&endpoint).call() {
                if let Ok(text) = resp.into_body().read_to_string() {
                    let parsed_rooms = serde_json::from_str::<Vec<ServerEntry>>(&text).ok().or_else(|| {
                        serde_json::from_str::<serde_json::Value>(&text)
                            .ok()
                            .and_then(|val| val.get("rooms").cloned())
                            .and_then(|rooms| serde_json::from_value::<Vec<ServerEntry>>(rooms).ok())
                    });
                    if let Some(rooms) = parsed_rooms {
                        for room in rooms {
                            if !list.iter().any(|s: &ServerEntry| s.host == room.host && s.port == room.port) {
                                list.push(room);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Always include local loopback option for testing
    if !list.iter().any(|s: &ServerEntry| s.host == "127.0.0.1" && s.port == DEFAULT_PORT) {
        list.push(ServerEntry {
            name: "Localhost Remastered Room".to_string(),
            host: "127.0.0.1".to_string(),
            port: DEFAULT_PORT,
            server_type: "LAN / Loopback".to_string(),
            ping_ms: Some(1),
        });
    }

    list
}

/// Deterministic consensus algorithm to resolve simultaneous arena switch race condition.
/// Both clients computing this will ALWAYS resolve to the exact same arena index.
pub fn determine_consensus_arena(id_a: &str, arena_a: usize, id_b: &str, arena_b: usize) -> usize {
    let (first_id, first_arena, second_id, second_arena) = if id_a <= id_b {
        (id_a, arena_a, id_b, arena_b)
    } else {
        (id_b, arena_b, id_a, arena_a)
    };
    let mut sum: u32 = 0;
    for b in first_id.bytes().chain(second_id.bytes()) {
        sum = sum.wrapping_mul(31).wrapping_add(b as u32);
    }
    if sum % 2 == 0 {
        first_arena
    } else {
        second_arena
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossArenaOffer {
    pub opponent_id: String,
    pub opponent_name: String,
    #[serde(default)]
    pub opponent_country: String,
    pub opponent_rating: f64,
    pub opponent_rd: f64,
    pub opponent_arena: usize,
    pub my_arena: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChallengeBroadcast {
    pub player_id: String,
    pub name: String,
    #[serde(default)]
    pub country: String,
    pub rating: f64,
    pub rd: f64,
    pub arena_index: usize,
    pub arena_name: String,
    #[serde(default)]
    pub joined_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanMatchmakingBeacon {
    pub app: String,
    pub player_id: String,
    pub name: String,
    #[serde(default)]
    pub country: String,
    pub rating: f64,
    pub rd: f64,
    pub port: u16,
    pub arena_index: usize,
    #[serde(default = "crate::config::default_elo_range")]
    pub elo_range_mode: usize,
    #[serde(default)]
    pub accepted_opponent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchmakingStatus {
    Searching {
        elapsed_sec: f32,
        current_tolerance: f64,
        elo_range_mode: usize,
        current_arena: usize,
    },
    CrossArenaOfferReceived(CrossArenaOffer),
    MatchedAsHost {
        opponent_name: String,
        opponent_country: String,
        opponent_rating: f64,
        opponent_rd: f64,
        agreed_arena: usize,
    },
    MatchedAsClient {
        target_addr: String,
        opponent_name: String,
        opponent_country: String,
        opponent_rating: f64,
        opponent_rd: f64,
        agreed_arena: usize,
    },
}

pub struct QuickMatchAgent {
    pub player_id: String,
    pub player_name: String,
    pub country_code: String,
    pub rating: f64,
    pub rd: f64,
    pub arena_index: usize,
    pub elo_range: EloSearchRange,
    pub search_start_time: f32,
    pub last_beacon_time: f32,
    pub coordinator_url: Option<String>,
    pub last_http_poll_time: f32,
    pub last_challenges_poll_time: f32,
    pub active_offer: Option<CrossArenaOffer>,
    pub accepted_cross_id: Option<String>,
    pub active_challenges: Vec<ChallengeBroadcast>,
    socket: Option<UdpSocket>,
    http_poll_rx: Option<mpsc::Receiver<Option<MatchmakingStatus>>>,
    challenges_rx: Option<mpsc::Receiver<Vec<ChallengeBroadcast>>>,
}

impl QuickMatchAgent {
    pub fn new(
        player_name: &str,
        country_code: &str,
        rating: f64,
        rd: f64,
        arena_index: usize,
        elo_range_idx: usize,
        coordinator_url: Option<String>,
        start_time: f32,
    ) -> Self {
        let rand_id = quad_rand::rand();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;
        let player_id = format!("bv_{:08x}_{:04x}", rand_id, (timestamp & 0xFFFF));

        // Try binding discovery ports 1235..=1240
        let mut bound_socket = None;
        for p in 1235..=1240 {
            if let Ok(s) = UdpSocket::bind(format!("0.0.0.0:{}", p)) {
                let _ = s.set_broadcast(true);
                let _ = s.set_nonblocking(true);
                bound_socket = Some(s);
                break;
            }
        }
        if bound_socket.is_none() {
            if let Ok(s) = UdpSocket::bind("0.0.0.0:0") {
                let _ = s.set_broadcast(true);
                let _ = s.set_nonblocking(true);
                bound_socket = Some(s);
            }
        }

        Self {
            player_id,
            player_name: player_name.to_string(),
            country_code: country_code.to_string(),
            rating,
            rd,
            arena_index,
            elo_range: EloSearchRange::from_index(elo_range_idx),
            search_start_time: start_time,
            last_beacon_time: -10.0,
            coordinator_url,
            last_http_poll_time: -10.0,
            last_challenges_poll_time: -10.0,
            active_offer: None,
            accepted_cross_id: None,
            active_challenges: Vec::new(),
            socket: bound_socket,
            http_poll_rx: None,
            challenges_rx: None,
        }
    }

    pub fn cycle_elo_range(&mut self) -> EloSearchRange {
        self.elo_range = self.elo_range.next();
        self.elo_range
    }

    pub fn cycle_arena(&mut self, next: bool) -> usize {
        let total = 8;
        if next {
            self.arena_index = (self.arena_index + 1) % total;
        } else {
            self.arena_index = (self.arena_index + total - 1) % total;
        }
        self.arena_index
    }

    pub fn accept_cross_arena(&mut self, opponent_id: &str) {
        self.accepted_cross_id = Some(opponent_id.to_string());
    }

    pub fn decline_cross_arena(&mut self) {
        self.active_offer = None;
    }

    pub fn poll(&mut self, current_time: f32) -> MatchmakingStatus {
        let elapsed = (current_time - self.search_start_time).max(0.0) as f64;
        let intervals = (elapsed / 5.0).floor() as i32;

        let tolerance = if self.elo_range == EloSearchRange::Unrestricted {
            f64::INFINITY
        } else {
            self.elo_range.initial_tolerance() * (4.0 / 3.0_f64).powi(intervals)
        };

        // 1. Broadcast LAN matchmaking beacon every 0.6 seconds
        if current_time - self.last_beacon_time >= 0.6 {
            self.last_beacon_time = current_time;
            if let Some(ref sock) = self.socket {
                let beacon = LanMatchmakingBeacon {
                    app: "blobby_volley_3_mm".to_string(),
                    player_id: self.player_id.clone(),
                    name: self.player_name.clone(),
                    country: self.country_code.clone(),
                    rating: self.rating,
                    rd: self.rd,
                    port: DEFAULT_PORT,
                    arena_index: self.arena_index,
                    elo_range_mode: self.elo_range.to_index(),
                    accepted_opponent_id: self.accepted_cross_id.clone(),
                };
                if let Ok(payload) = serde_json::to_vec(&beacon) {
                    let _ = sock.send_to(&payload, format!("255.255.255.255:{}", LAN_DISCOVERY_PORT));
                    for p in 1235..=1240 {
                        let _ = sock.send_to(&payload, format!("127.0.0.1:{}", p));
                    }
                }
            }
        }

        // 2. Poll incoming UDP packets
        if let Some(ref sock) = self.socket {
            let mut buf = [0u8; 1024];
            while let Ok((len, src_addr)) = sock.recv_from(&mut buf) {
                if let Ok(beacon) = serde_json::from_slice::<LanMatchmakingBeacon>(&buf[..len]) {
                    if beacon.app == "blobby_volley_3_mm" && beacon.player_id != self.player_id {
                        let opp_range = EloSearchRange::from_index(beacon.elo_range_mode);
                        let opp_tol = if opp_range == EloSearchRange::Unrestricted {
                            f64::INFINITY
                        } else {
                            opp_range.initial_tolerance() * (4.0 / 3.0_f64).powi(intervals)
                        };
                        let effective_tol = tolerance.max(opp_tol);
                        let diff = (self.rating - beacon.rating).abs();

                        if diff <= effective_tol {
                            if self.arena_index == beacon.arena_index {
                                return self.create_match(&beacon, self.arena_index, src_addr.ip().to_string());
                            } else {
                                if self.accepted_cross_id.as_deref() == Some(&beacon.player_id)
                                    && beacon.accepted_opponent_id.as_deref() == Some(&self.player_id)
                                {
                                    let agreed_arena = determine_consensus_arena(
                                        &self.player_id,
                                        self.arena_index,
                                        &beacon.player_id,
                                        beacon.arena_index,
                                    );
                                    return self.create_match(&beacon, agreed_arena, src_addr.ip().to_string());
                                } else if self.accepted_cross_id.as_deref() == Some(&beacon.player_id) {
                                    // Already accepted, waiting for opponent
                                } else {
                                    let offer = CrossArenaOffer {
                                        opponent_id: beacon.player_id.clone(),
                                        opponent_name: beacon.name.clone(),
                                        opponent_country: beacon.country.clone(),
                                        opponent_rating: beacon.rating,
                                        opponent_rd: beacon.rd,
                                        opponent_arena: beacon.arena_index,
                                        my_arena: self.arena_index,
                                    };
                                    self.active_offer = Some(offer.clone());
                                    return MatchmakingStatus::CrossArenaOfferReceived(offer);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Poll HTTP/HTTPS serverless coordinator asynchronously via background mpsc channels
        if let Some(ref rx) = self.http_poll_rx {
            match rx.try_recv() {
                Ok(Some(status)) => {
                    self.http_poll_rx = None;
                    match status {
                        MatchmakingStatus::CrossArenaOfferReceived(ref offer) => {
                            self.active_offer = Some(offer.clone());
                            return status;
                        }
                        MatchmakingStatus::MatchedAsHost { .. } | MatchmakingStatus::MatchedAsClient { .. } => {
                            return status;
                        }
                        _ => {}
                    }
                }
                Ok(None) | Err(mpsc::TryRecvError::Disconnected) => {
                    self.http_poll_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        if let Some(ref rx) = self.challenges_rx {
            match rx.try_recv() {
                Ok(list) => {
                    self.active_challenges = list;
                    self.challenges_rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.challenges_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        if let Some(ref url) = self.coordinator_url {
            if current_time - self.last_http_poll_time >= 1.5 && self.http_poll_rx.is_none() {
                self.last_http_poll_time = current_time;
                let url_c = url.clone();
                let pid_c = self.player_id.clone();
                let name_c = self.player_name.clone();
                let country_c = self.country_code.clone();
                let rating_c = self.rating;
                let rd_c = self.rd;
                let arena_c = self.arena_index;
                let range_c = self.elo_range.to_index();
                let accepted_c = self.accepted_cross_id.clone();
                let (tx, rx) = mpsc::channel();
                self.http_poll_rx = Some(rx);
                std::thread::spawn(move || {
                    let res = poll_http_coordinator(
                        &url_c,
                        &pid_c,
                        &name_c,
                        &country_c,
                        rating_c,
                        rd_c,
                        arena_c,
                        range_c,
                        accepted_c.as_deref(),
                    );
                    let _ = tx.send(res);
                });
            }

            if current_time - self.last_challenges_poll_time >= 3.0 && self.challenges_rx.is_none() {
                self.last_challenges_poll_time = current_time;
                let url_c = url.clone();
                let (tx, rx) = mpsc::channel();
                self.challenges_rx = Some(rx);
                std::thread::spawn(move || {
                    let list = fetch_challenges(&url_c);
                    let _ = tx.send(list);
                });
            }
        }

        if let Some(ref offer) = self.active_offer {
            return MatchmakingStatus::CrossArenaOfferReceived(offer.clone());
        }

        MatchmakingStatus::Searching {
            elapsed_sec: elapsed as f32,
            current_tolerance: tolerance,
            elo_range_mode: self.elo_range.to_index(),
            current_arena: self.arena_index,
        }
    }

    fn create_match(
        &self,
        beacon: &LanMatchmakingBeacon,
        agreed_arena: usize,
        src_ip: String,
    ) -> MatchmakingStatus {
        if self.player_id < beacon.player_id {
            MatchmakingStatus::MatchedAsHost {
                opponent_name: beacon.name.clone(),
                opponent_country: beacon.country.clone(),
                opponent_rating: beacon.rating,
                opponent_rd: beacon.rd,
                agreed_arena,
            }
        } else {
            let target_addr = format!("{}:{}", src_ip, beacon.port);
            MatchmakingStatus::MatchedAsClient {
                target_addr,
                opponent_name: beacon.name.clone(),
                opponent_country: beacon.country.clone(),
                opponent_rating: beacon.rating,
                opponent_rd: beacon.rd,
                agreed_arena,
            }
        }
    }

    pub fn cancel(&self) {
        if let Some(ref url) = self.coordinator_url {
            let url_c = url.clone();
            let pid_c = self.player_id.clone();
            std::thread::spawn(move || {
                cancel_http_coordinator(&url_c, &pid_c);
            });
        }
    }
}

fn normalize_base_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Some(trimmed.to_string())
    } else {
        Some(format!("http://{}", trimmed))
    }
}

fn build_http_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_millis(1000)))
        .timeout_global(Some(Duration::from_millis(1500)))
        .build();
    ureq::Agent::new_with_config(config)
}

fn poll_http_coordinator(
    base_url: &str,
    player_id: &str,
    name: &str,
    country: &str,
    rating: f64,
    rd: f64,
    arena_index: usize,
    elo_range_mode: usize,
    accepted_cross_id: Option<&str>,
) -> Option<MatchmakingStatus> {
    let base = normalize_base_url(base_url)?;
    let endpoint = format!("{}/matchmaking/queue", base);
    let payload = serde_json::json!({
        "player_id": player_id,
        "name": name,
        "country": country,
        "rating": rating,
        "rd": rd,
        "arena_index": arena_index,
        "elo_range_mode": elo_range_mode,
        "accepted_cross_arena_id": accepted_cross_id,
    });
    let body = serde_json::to_string(&payload).ok()?;

    let agent = build_http_agent();
    let resp = agent
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .send(&body)
        .ok()?;
    let json_body = resp.into_body().read_to_string().ok()?;

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_body) {
        let status = parsed.get("status").and_then(|v| v.as_str());
        if status == Some("matched") {
            if let Some(m) = parsed.get("match") {
                let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("client");
                let opp_name = m.get("opponent_name").and_then(|v| v.as_str()).unwrap_or("Opponent").to_string();
                let opp_country = m.get("opponent_country").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let opp_r = m.get("opponent_rating").and_then(|v| v.as_f64()).unwrap_or(1500.0);
                let opp_rd = m.get("opponent_rd").and_then(|v| v.as_f64()).unwrap_or(350.0);
                let agreed_arena = m.get("agreed_arena").and_then(|v| v.as_u64()).unwrap_or(arena_index as u64) as usize;
                let connect_to = m.get("connect_to").and_then(|v| v.as_str()).unwrap_or("127.0.0.1:1234").to_string();

                if role == "host" {
                    return Some(MatchmakingStatus::MatchedAsHost {
                        opponent_name: opp_name,
                        opponent_country: opp_country,
                        opponent_rating: opp_r,
                        opponent_rd: opp_rd,
                        agreed_arena,
                    });
                } else {
                    return Some(MatchmakingStatus::MatchedAsClient {
                        target_addr: connect_to,
                        opponent_name: opp_name,
                        opponent_country: opp_country,
                        opponent_rating: opp_r,
                        opponent_rd: opp_rd,
                        agreed_arena,
                    });
                }
            }
        } else if status == Some("cross_arena_offer") {
            if let Some(o) = parsed.get("offer") {
                if let Ok(offer) = serde_json::from_value::<CrossArenaOffer>(o.clone()) {
                    return Some(MatchmakingStatus::CrossArenaOfferReceived(offer));
                }
            }
        }
    }
    None
}

pub fn fetch_challenges(base_url: &str) -> Vec<ChallengeBroadcast> {
    if let Some(base) = normalize_base_url(base_url) {
        let endpoint = format!("{}/challenges", base);
        let agent = build_http_agent();
        if let Ok(resp) = agent.get(&endpoint).call() {
            if let Ok(json) = resp.into_body().read_to_string() {
                if let Ok(list) = serde_json::from_str::<Vec<ChallengeBroadcast>>(&json) {
                    return list;
                }
            }
        }
    }
    Vec::new()
}

fn cancel_http_coordinator(base_url: &str, player_id: &str) {
    if let Some(base) = normalize_base_url(base_url) {
        let endpoint = format!("{}/matchmaking/cancel", base);
        let agent = build_http_agent();
        let _ = agent.delete(&endpoint).query("player_id", player_id).call();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub player_id: String,
    pub name: String,
    pub rating: f64,
    pub rank_title: String,
    pub matches_won: u32,
    pub matches_played: u32,
    pub win_rate: u32,
    pub last_active_days: u32,
}

pub fn submit_leaderboard_rating(
    coordinator_url: Option<&str>,
    player_id: &str,
    name: &str,
    rating: f64,
    rd: f64,
    wins: u32,
    total: u32,
) {
    if let Some(raw_url) = coordinator_url {
        if let Some(base) = normalize_base_url(raw_url) {
            let endpoint = format!("{}/leaderboard/submit", base);
            let payload = serde_json::json!({
                "player_id": player_id,
                "name": name,
                "rating": rating,
                "rd": rd,
                "matches_won": wins,
                "matches_played": total,
            });
            if let Ok(body) = serde_json::to_string(&payload) {
                let agent = build_http_agent();
                let _ = agent
                    .post(&endpoint)
                    .header("Content-Type", "application/json")
                    .send(&body);
            }
        }
    }
}

pub fn fetch_leaderboard(
    coordinator_url: Option<&str>,
    period: &str,
    my_name: &str,
    my_rating: f64,
    my_wins: u32,
    my_played: u32,
) -> Vec<LeaderboardEntry> {
    // 1. Try querying remote coordinator if configured
    if let Some(raw_url) = coordinator_url {
        if let Some(base) = normalize_base_url(raw_url) {
            let endpoint = format!("{}/leaderboard", base);
            let agent = build_http_agent();
            if let Ok(resp) = agent.get(&endpoint).query("period", period).call() {
                if let Ok(json) = resp.into_body().read_to_string() {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
                        if let Some(list) = val.get("leaderboard").and_then(|v| v.as_array()) {
                            let mut entries = Vec::new();
                            for item in list {
                                if let Ok(entry) = serde_json::from_value::<LeaderboardEntry>(item.clone()) {
                                    entries.push(entry);
                                }
                            }
                            if !entries.is_empty() {
                                return entries;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Offline Fallback Leaderboard: Calibrated competitive roster with inactivity filter
    let max_inactivity = match period {
        "week" => 7,
        "month" => 30,
        "year" => 365,
        _ => 90,
    };

    let get_rank_title = |r: f64| -> &'static str {
        if r < 1100.0 { "Bronze I" }
        else if r < 1250.0 { "Bronze II" }
        else if r < 1400.0 { "Silver I" }
        else if r < 1550.0 { "Silver II" }
        else if r < 1700.0 { "Gold I" }
        else if r < 1850.0 { "Gold II" }
        else if r < 2000.0 { "Platinum I" }
        else if r < 2150.0 { "Platinum II" }
        else if r < 2300.0 { "Diamond" }
        else { "Grandmaster" }
    };

    let mut raw_players = vec![
        ("SpikeMaster", 2240.0, 52, 58, 0),
        ("BlobbyPro", 2090.0, 41, 49, 1),
        ("CyberVolley", 1980.0, 36, 45, 3),
        ("AlpineJumper", 1850.0, 29, 40, 5),
        ("NetGuardian", 1780.0, 22, 32, 12),
        ("CopacabanaAce", 1720.0, 18, 28, 18),
        ("MagmaSmash", 1650.0, 15, 25, 25),
        ("RetroBlob", 1590.0, 12, 22, 45),
        ("ZenServer", 1520.0, 9, 18, 70),
    ];

    // Filter by inactivity threshold
    raw_players.retain(|&(_, _, _, _, days)| days <= max_inactivity);

    // Insert current player profile
    let my_winrate = if my_played > 0 { (my_wins * 100) / my_played } else { 0 };
    let mut entries = Vec::new();
    for (name, r, w, t, days) in raw_players {
        let wr = if t > 0 { (w * 100) / t } else { 0 };
        entries.push(LeaderboardEntry {
            rank: 0,
            player_id: format!("p_{}", name.to_lowercase()),
            name: name.to_string(),
            rating: r,
            rank_title: get_rank_title(r).to_string(),
            matches_won: w,
            matches_played: t,
            win_rate: wr,
            last_active_days: days,
        });
    }

    // Add current user
    entries.push(LeaderboardEntry {
        rank: 0,
        player_id: "local_player".to_string(),
        name: format!("{} (You)", my_name),
        rating: my_rating,
        rank_title: get_rank_title(my_rating).to_string(),
        matches_won: my_wins,
        matches_played: my_played,
        win_rate: my_winrate,
        last_active_days: 0,
    });

    // Sort descending by rating
    entries.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));

    // Assign rank indices
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.rank = i + 1;
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::PlayerSide;
    use crate::rules::RuleMode;

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(
            normalize_base_url("https://relay.example.workers.dev/"),
            Some("https://relay.example.workers.dev".to_string())
        );
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8787///"),
            Some("http://127.0.0.1:8787".to_string())
        );
        assert_eq!(
            normalize_base_url("localhost:8787/"),
            Some("http://localhost:8787".to_string())
        );
        assert_eq!(normalize_base_url("   "), None);
    }

    #[test]
    fn test_match_config_hello_and_pending_events() {
        let port = 19284;
        let mut host = NetworkHost::bind(port, "HostPlayer", "DE", "Copacabana Sunset")
            .expect("bind test port");
        host.set_match_config(MatchConfigData {
            score_to_win: 21,
            rule_mode_index: 1,
            arena_index: 3,
            host_theme_index: 2,
            client_theme_index: 4,
            host_name: "HostPlayer".to_string(),
            host_country: "DE".to_string(),
            host_rating: 1640.0,
        });

        let mut client = NetworkClient::new();
        client.connect_async(&format!("127.0.0.1:{}", port));

        for _ in 0..40 {
            host.poll_connection();
            client.update();
            if host.is_client_connected && client.is_connected && client.received_match_config.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(15));
        }

        assert!(host.is_client_connected);
        assert!(client.is_connected);
        let cfg = client.received_match_config.as_ref().expect("Client should receive MatchConfig");
        assert_eq!(cfg.score_to_win, 21);
        assert_eq!(cfg.arena_index, 3);
        assert_eq!(cfg.host_name, "HostPlayer");

        client.send_hello("GuestPlayer", "FR", 1580.0);
        for _ in 0..20 {
            host.update(0.1);
            if host.client_hello.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            host.client_hello,
            Some(("GuestPlayer".to_string(), "FR".to_string(), 1580.0))
        );

        let physics = PhysicWorld::new();
        let logic = GameLogic::new(RuleMode::Modern, 21);
        host.send_game_state(&physics, &logic, &[MatchEvent::BallHitWall { side: PlayerSide::Left }]);
        host.send_game_state(&physics, &logic, &[MatchEvent::BallHitNetTop]);

        for _ in 0..20 {
            client.update();
            if client.pending_events.len() >= 2 {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let drained = client.take_events();
        assert_eq!(drained.len(), 2);
        assert!(client.take_events().is_empty());
    }
}
