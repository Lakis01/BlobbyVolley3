// Blobby Volley 3 - Online & LAN Network Multiplayer Engine (Zero-Cost P2P + STUN + Serverless)
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddrV4, Ipv4Addr, ToSocketAddrs};
use std::io::{BufRead, BufReader, Write, Read};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::physics::{PhysicWorld, PlayerInput, MatchEvent};
use crate::rules::GameLogic;

pub const DEFAULT_PORT: u16 = 1234;
pub const LAN_DISCOVERY_PORT: u16 = 1235;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    Input(PlayerInput),
    Chat { sender: String, text: String },
    Ping(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    GameState(NetGameState),
    Chat { sender: String, text: String },
    Pong(u64),
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
    pub arena_name: String,
    pub port: u16,
    pub local_ip: String,
    pub public_endpoint: Option<String>,
    broadcast_socket: Option<UdpSocket>,
    last_broadcast_time: f32,
}

impl NetworkHost {
    pub fn bind(port: u16, host_name: &str, arena_name: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        listener.set_nonblocking(true)?;

        let local_ip = get_local_ip();

        // Query public STUN endpoint for zero-cost internet P2P discovery
        let public_endpoint = query_public_endpoint().map(|ep| format!("{}:{}", ep.ip(), port));

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
            arena_name: arena_name.to_string(),
            port,
            local_ip,
            public_endpoint,
            broadcast_socket,
            last_broadcast_time: -10.0,
        })
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
                return true;
            }
        }
        false
    }

    pub fn update(&mut self, current_time: f32) {
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
                    }
                }
                line.clear();
            }
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
            .map(|e| match e {
                MatchEvent::BallHitBlob { player, intensity } => NetEvent::HitBlob {
                    player: player.index(),
                    intensity: *intensity,
                },
                MatchEvent::BallHitGround { side } => NetEvent::HitGround {
                    side: side.index(),
                },
                MatchEvent::BallHitWall { .. } => NetEvent::HitWall,
                MatchEvent::BallHitNet { .. } | MatchEvent::BallHitNetTop => NetEvent::HitNet,
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
    pub new_chat_messages: Vec<(String, String)>,
    #[allow(dead_code)]
    pub ping_ms: u32,
    pub error_message: Option<String>,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            reader: None,
            is_connected: false,
            is_connecting: false,
            latest_state: None,
            new_chat_messages: Vec::new(),
            ping_ms: 0,
            error_message: None,
        }
    }

    pub fn connect(&mut self, target_addr: &str) {
        self.is_connecting = true;
        self.error_message = None;

        let addr = if target_addr.contains(':') {
            target_addr.to_string()
        } else {
            format!("{}:{}", target_addr, DEFAULT_PORT)
        };

        match TcpStream::connect_timeout(
            &addr.parse().unwrap_or_else(|_| "127.0.0.1:1234".parse().unwrap()),
            Duration::from_millis(3000),
        ) {
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

    pub fn update(&mut self) {
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
                            self.latest_state = Some(state);
                        }
                        ServerPacket::Chat { sender, text } => {
                            self.new_chat_messages.push((sender, text));
                        }
                        ServerPacket::Pong(_) => {}
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
                        rooms.push(ServerEntry {
                            name: format!("{} ({})", beacon.name, beacon.arena),
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

/// Queries available servers: LAN auto-discovered rooms, Cloudflare Worker rooms, and legacy BV2 servers
pub fn query_master_server() -> Vec<ServerEntry> {
    let mut list = Vec::new();

    // 1. Poll LAN Discovery Listener for active rooms on the local network
    let mut lan_listener = LanDiscoveryListener::new();
    let lan_rooms = lan_listener.poll_rooms();
    for room in lan_rooms {
        if !list.iter().any(|s: &ServerEntry| s.host == room.host && s.port == room.port) {
            list.push(room);
        }
    }

    // 2. Query official legacy master list with strict connect & read timeouts (600ms max)
    if let Ok(mut addrs) = ("blobbyvolley.de", 80).to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(600)) {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(600)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(600)));
                let req = "GET /server.php HTTP/1.1\r\nHost: blobbyvolley.de\r\nConnection: close\r\n\r\n";
                if stream.write_all(req.as_bytes()).is_ok() {
                    let mut response = String::new();
                    if stream.read_to_string(&mut response).is_ok() {
                        list.extend(parse_server_xml(&response));
                    }
                }
            }
        }
    }

    // 3. Always include local loopback option for testing
    list.push(ServerEntry {
        name: "Localhost Remastered Room".to_string(),
        host: "127.0.0.1".to_string(),
        port: DEFAULT_PORT,
        server_type: "LAN / Loopback".to_string(),
        ping_ms: Some(1),
    });

    list
}
