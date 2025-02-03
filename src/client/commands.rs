use crate::client::RustbustersClient;
use common_utils::{
    ClientToServerMessage, HostCommand, HostEvent, HostMessage, ServerToClientMessage,
};
use crossbeam_channel::Sender;
use log::warn;
use wg_2024::network::NodeId;

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
                self.send_message(
                    dest,
                    HostMessage::FromClient(ClientToServerMessage::RegisterUser {
                        name: "Random".to_string(),
                    }),
                    ws_to_ui_sender,
                );
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
                        "Client {}: Unable to send StatsResponse(...) to controller: {}",
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
            _ => {
                unreachable!("Client {}: Unhandled command: {:?}", self.id, command);
            }
        }
    }
}
