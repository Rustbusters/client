use crate::RustbustersClient;
use common_utils::{ClientToServerMessage, HostMessage};
use wg_2024::network::NodeId;

impl RustbustersClient {
    pub(crate) fn handle_ui_message(&mut self, server_id: NodeId, message: ClientToServerMessage) {
        let message_to_send = HostMessage::FromClient(message);

        self.send_message(server_id, message_to_send);
    }
}
