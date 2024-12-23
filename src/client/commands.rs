use crate::client::RustbustersClient;
use common_utils::{HostCommand, HostEvent};
use log::warn;

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
        }
    }
}
