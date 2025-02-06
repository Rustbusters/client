use crate::client::RustbustersClient;
use common_utils::{
    ClientToServerMessage, HostCommand, HostMessage, MessageBody, MessageContent,
    ServerToClientMessage,
};
use crossbeam_channel::Sender;
use rand::distr::Alphanumeric;
use rand::prelude::SliceRandom;
use rand::{rng, Rng};
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

impl RustbustersClient {
    /// Handles various commands received from the controller
    ///
    /// ### Arguments
    /// * `command` - The command to be executed
    /// * `ws_to_ui_sender` - Channel sender for sending messages back to the UI
    pub(crate) fn handle_command(
        &mut self,
        command: HostCommand,
        ws_to_ui_sender: &Sender<(NodeId, ServerToClientMessage)>,
    ) {
        match command {
            HostCommand::SendRandomMessage(dest) => {
                let random_client_dest = {
                    let nodes = self.known_nodes.lock().unwrap();
                    let mut client_nodes: Vec<_> = nodes
                        .iter()
                        .filter(|(node_id, node_type)| {
                            matches!(node_type, NodeType::Client) && **node_id != self.id
                        })
                        .collect();
                    client_nodes.shuffle(&mut rng());
                    client_nodes.first().map(|(node_id, _)| **node_id).unwrap()
                };

                self.send_message(
                    dest,
                    HostMessage::FromClient(ClientToServerMessage::SendPrivateMessage {
                        recipient_id: random_client_dest,
                        message: MessageBody {
                            sender_id: self.id,
                            timestamp: chrono::Local::now().format("%H:%M").to_string(),
                            content: MessageContent::Text({
                                let mut rng = rng();
                                let length = rng.random_range(5..=20);
                                let random_string: String = (0..length)
                                    .map(|_| rng.sample(Alphanumeric) as char)
                                    .collect();
                                random_string
                            }),
                        },
                    }),
                    ws_to_ui_sender,
                );
            }
            HostCommand::DiscoverNetwork => {
                self.discover_network();
            }
            HostCommand::AddSender(sender_id, sender) => {
                self.packet_send.insert(sender_id, sender);
                self.discover_network();
            }
            HostCommand::RemoveSender(sender_id) => {
                self.packet_send.remove(&sender_id);
                self.topology.remove_edge(self.id, sender_id);
                self.edge_stats.remove(&(self.id, sender_id));
                self.discover_network();
            }
            _ => {
                unreachable!("Client {}: Unhandled command: {:?}", self.id, command);
            }
        }
    }
}
