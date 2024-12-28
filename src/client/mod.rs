mod assembler;
mod commands;
mod disassembler;
mod handlers;
mod packet_sender;
mod routing;

use common_utils::client_to_server::MessageToServer;
use common_utils::server_to_client::MessageToClient;
use common_utils::{HostCommand, HostEvent, Stats};
use crossbeam_channel::{select_biased, Receiver, Sender};
use log::{error, info};
use petgraph::prelude::GraphMap;
use petgraph::Undirected;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, NodeType, Packet};

pub struct RustbustersClient {
    pub(crate) id: NodeId,
    controller_send: Sender<HostEvent<MessageToServer, MessageToClient>>,
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
}

impl RustbustersClient {
    pub fn new(
        id: NodeId,
        controller_send: Sender<HostEvent<MessageToServer, MessageToClient>>,
        controller_recv: Receiver<HostCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        info!("Client {} spawned succesfully", id);
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            topology: GraphMap::new(),
            flood_id_counter: 73,    // arbitrary value
            session_id_counter: 173, // arbitrary value
            pending_sent: HashMap::new(),
            pending_received: HashMap::new(),
            stats: Stats::new(),
        }
    }

    pub fn run(&mut self) {
        self.run_ui();

        // Start network discovery
        info!("Client {} started network discovery", self.id);
        self.discover_network();

        loop {
            select_biased! {
                // Handle SC commands
                recv(self.controller_recv) -> command => {
                    if let Ok(cmd) = command {
                        self.handle_command(cmd);
                    } else {
                        error!("Client {} - Error in receiving command", self.id);
                    }
                },
                // Handle incoming packets
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                    } else {
                        error!("Client {} - Error in receiving packet", self.id);
                    }
                },
                default(Duration::from_millis(100)) => {
                }
            }
        }
    }

    pub(crate) fn send_to_sc(&mut self, event: HostEvent<MessageToServer, MessageToClient>) {
        if self.controller_send.send(event).is_ok() {
            info!("Client {} - Sent NodeEvent to SC", self.id);
        } else {
            error!("Client {} - Error in sending event to SC", self.id);
        }
    }
}
