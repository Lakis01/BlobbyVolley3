// Blobby Volley 3 - Online & LAN Network Multiplayer Engine
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::io::{BufRead, BufReader, Write};
use serde::{Deserialize, Serialize};
use crate::physics::{PhysicWorld, PlayerInput, MatchEvent};
use crate::rules::GameLogic;

pub const DEFAULT_PORT: u16 = 1234;

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

pub struct NetworkHost {
    listener: TcpListener,
    client_stream: Option<TcpStream>,
    reader: Option<BufReader<TcpStream>>,
    pub client_input: PlayerInput,
    pub is_client_connected: bool,
    pub new_chat_messages: Vec<(String, String)>,
}

impl NetworkHost {
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        listener.set_nonblocking(true)?;

        Ok(Self {
            listener,
            client_stream: None,
            reader: None,
            client_input: PlayerInput::default(),
            is_client_connected: false,
            new_chat_messages: Vec::new(),
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

    pub fn update(&mut self) {
        if !self.is_client_connected {
            self.poll_connection();
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
                if stream.write_all(json.as_bytes()).is_err() {
                    self.is_client_connected = false;
                }
            }
        }
    }
}

pub struct NetworkClient {
    stream: Option<TcpStream>,
    reader: Option<BufReader<TcpStream>>,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub error_message: Option<String>,
    pub latest_state: Option<NetGameState>,
    pub new_chat_messages: Vec<(String, String)>,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            reader: None,
            is_connected: false,
            is_connecting: false,
            error_message: None,
            latest_state: None,
            new_chat_messages: Vec::new(),
        }
    }

    pub fn connect(&mut self, addr: &str) {
        self.is_connecting = true;
        self.error_message = None;

        match TcpStream::connect(addr) {
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

/// Queries the official Blobby Volley master server (http://blobbyvolley.de/server.php)
pub fn query_master_server() -> Vec<ServerEntry> {
    use std::io::Read;
    use std::time::Duration;
    use std::net::ToSocketAddrs;

    let mut list = Vec::new();

    // Query official legacy master list with strict connect & read timeouts (600ms max)
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

    // Always include local LAN / Loopback option for instant testing & matchmaking
    list.push(ServerEntry {
        name: "Localhost Remastered Room".to_string(),
        host: "127.0.0.1".to_string(),
        port: DEFAULT_PORT,
        server_type: "LAN / P2P Room".to_string(),
        ping_ms: Some(1),
    });

    list
}
