use crate::client::messages::Message;
use crate::client::stats::Stats;
use crate::client::RustbustersClient;
use crossbeam_channel::Sender;
use log::warn;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

#[derive(Debug, Clone)]
pub enum HostCommand {
    SendRandomMessage(NodeId),
    DiscoverNetwork,
    StatsRequest,
    AddSender(NodeId, Sender<Packet>),
    RemoveSender(NodeId),
}

#[derive(Debug, Clone)]
pub enum HostEvent {
    MessageSent(Message),
    MessageReceived(Message),
    StatsResponse(Stats),
    ControllerShortcut(Packet),
}

impl RustbustersClient {
    pub(crate) fn handle_command(&mut self, command: HostCommand) {
        match command {
            HostCommand::SendRandomMessage(dest) => {
                self.send_random_message(dest);
            }
            HostCommand::DiscoverNetwork => {
                self.discover_network();
            }
            HostCommand::StatsRequest => {
                if let Err(err) = self
                    .controller_send
                    .send(HostEvent::StatsResponse(self.stats.clone()))
                {
                    warn!(
                        "Node {}: Unable to send StatsResponse(...) to controller: {}",
                        self.id, err
                    );
                }
            }
            HostCommand::AddSender(sender_id, sender) => {
                self.packet_send.insert(sender_id, sender);
                self.discover_network();
            }
            HostCommand::RemoveSender(sender_id) => {
                self.packet_send.remove(&sender_id);
                self.discover_network();
            }
        }
    }
}
