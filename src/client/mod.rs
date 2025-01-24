mod commands;
mod fragmentation;
mod handlers;
mod packet_sender;
mod routing;
mod ui_connector;

use crate::client::routing::edge_stats::EdgeStats;
use common_utils::{HostCommand, HostEvent, Stats};
use crossbeam_channel::{select_biased, Receiver, Sender};
use log::{error, info};
use petgraph::prelude::GraphMap;
use petgraph::Undirected;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, NodeType, Packet};

const DEFAULT_DISCOVERY_INTERVAL: Duration = Duration::from_secs(20);

pub struct RustbustersClient {
    pub(crate) id: NodeId,
    controller_send: Sender<HostEvent>,
    controller_recv: Receiver<HostCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pub(crate) known_nodes: Arc<Mutex<HashMap<NodeId, NodeType>>>,
    topology: GraphMap<NodeId, f32, Undirected>,
    flood_id_counter: u64,
    session_id_counter: u64,
    // (session_id, fragment_index) -> packet
    pending_sent: HashMap<(u64, u64), Packet>,
    // session_id -> (fragments, num_fragments) (u8 is the number of fragments received) (for reassembly)
    pending_received: HashMap<u64, (Vec<Option<Fragment>>, u64)>,
    stats: Stats,
    edge_stats: HashMap<(NodeId, NodeId), EdgeStats>,
    last_discovery: Instant,
    discovery_interval: Duration,
}

impl RustbustersClient {
    pub fn new(
        id: NodeId,
        controller_send: Sender<HostEvent>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        discovery_interval: Option<Duration>,
    ) -> Self {
        let discovery_interval = discovery_interval.unwrap_or(DEFAULT_DISCOVERY_INTERVAL);
        info!(
            "Client {} spawned successfully with discovery interval {:?}",
            id, discovery_interval
        );

        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            topology: GraphMap::new(),
            flood_id_counter: 73,    // arbitrary value
            session_id_counter: 73, // arbitrary value
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            stats: Stats::new(),
            edge_stats: HashMap::new(),
            last_discovery: Instant::now(),
            discovery_interval,
        }
    }

    fn should_perform_discovery(&self) -> bool {
        self.last_discovery.elapsed() >= self.discovery_interval
    }

    pub fn run(&mut self) {
        // Generate UI crossbeam channels
        let (ui_to_ws_sender, ui_to_ws_receiver) = crossbeam_channel::unbounded();
        let (ws_to_ui_sender, ws_to_ui_receiver) = crossbeam_channel::unbounded();

        self.run_ui(ui_to_ws_sender, ws_to_ui_receiver);

        // Start network discovery
        info!("Client {} started network discovery", self.id);
        self.discover_network();

        loop {
            // Check if we need to perform discovery
            if self.should_perform_discovery() {
                info!("Client {}: Performing periodic network discovery", self.id);
                self.discover_network();
                self.last_discovery = Instant::now();
            }

            select_biased! {
                // Handle UI commands
                recv(ui_to_ws_receiver) -> msg_to_srv => {
                    if let Ok(msg) = msg_to_srv {
                        let server_id = msg.0;
                        let message = msg.1;

                        self.handle_ui_message(server_id, message, &ws_to_ui_sender);
                    } else {
                        error!("Client {} - Error in receiving command", self.id);
                    }
                },
                // Handle SC commands
                recv(self.controller_recv) -> command => {
                    if let Ok(cmd) = command {
                        self.handle_command(cmd, &ws_to_ui_sender);
                    } else {
                        error!("Client {} - Error in receiving command", self.id);
                    }
                },
                // Handle incoming packets
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet, &ws_to_ui_sender);
                    } else {
                        error!("Client {} - Error in receiving packet", self.id);
                    }
                },
                default(Duration::from_millis(100)) => {
                }
            }
        }
    }

    pub(crate) fn send_to_sc(&mut self, event: HostEvent) {
        if self.controller_send.send(event).is_ok() {
            info!("Client {} - Sent NodeEvent to SC", self.id);
        } else {
            error!("Client {} - Error in sending event to SC", self.id);
        }
    }
}
