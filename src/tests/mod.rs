pub mod commands_tests;
pub mod edge_stats_tests;
pub mod fragmentation_tests;
pub mod routing_tests;

use std::collections::HashMap;

use crate::RustbustersClient;
use common_utils::{HostCommand, HostEvent};
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::packet::Packet;

fn create_test_client() -> (
    RustbustersClient,
    Receiver<HostEvent>,
    Sender<HostCommand>,
    Sender<Packet>,
) {
    let (tx_event, rx_event) = unbounded();
    let (tx_command, rx_command) = unbounded();
    let (tx_packet, rx_packet) = unbounded();

    (
        RustbustersClient::new(1, tx_event, rx_command, rx_packet, HashMap::new(), None),
        rx_event,
        tx_command,
        tx_packet,
    )
}
