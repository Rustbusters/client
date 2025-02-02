use crate::RustbustersClient;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

/// Wrapper around f32 to implement Ord for use in BinaryHeap
/// Reverses comparison to create a min-heap instead of max-heap
#[derive(Debug, Copy, Clone, PartialEq)]
struct FloatKey(f32);

impl Eq for FloatKey {}

impl PartialOrd for FloatKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Ord for FloatKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order to make BinaryHeap become a min-heap
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

impl RustbustersClient {
    /// Finds the shortest weighted path to a destination node using Dijkstra's algorithm.
    /// 
    /// # Arguments
    /// * `dst` - Destination node ID
    ///
    /// # Returns
    /// * `Some(Vec<NodeId>)` - Vector of node IDs representing the optimal path if found
    /// * `None` - If no valid path exists
    ///
    /// A valid path must follow network rules:
    /// - Client can only connect to Drones
    /// - Drones can connect to other Drones or Servers
    /// - Server must be the final destination
    pub(crate) fn find_weighted_path(&self, dst: NodeId) -> Option<Vec<NodeId>> {
        let mut distance: HashMap<NodeId, f32> = HashMap::new();
        let mut heap: BinaryHeap<(FloatKey, NodeId)> = BinaryHeap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        // Initialize distances
        distance.insert(self.id, 0.0);
        heap.push((FloatKey(0.0), self.id));

        while let Some((FloatKey(cost), node)) = heap.pop() {
            // If we reached the destination and it's a server, build the path
            if node == dst {
                return if let Some(NodeType::Server) = self.get_node_type(node) {
                    let mut path = Vec::new();
                    let mut current = Some(node);

                    while let Some(c) = current {
                        path.push(c);
                        current = prev.get(&c).copied();
                    }
                    path.reverse();
                    Some(path)
                } else {
                    // If the node is not a server, the destination cannot be reached
                    None
                };
            }

            // Ignore paths longer than the already calculated one
            if let Some(&d) = distance.get(&node) {
                if cost > d {
                    continue;
                }
            }

            // Explore neighbors
            for neighbor in self.topology.neighbors(node) {
                let edge_weight = *self
                    .topology
                    .edge_weight(node, neighbor)
                    .unwrap_or(&f32::INFINITY);
                let next_cost = cost + edge_weight;

                // Path validity rules
                match (self.get_node_type(node), self.get_node_type(neighbor)) {
                    (Some(NodeType::Client), Some(NodeType::Drone))
                    | (Some(NodeType::Drone), Some(NodeType::Drone | NodeType::Server)) => {
                        if next_cost < *distance.get(&neighbor).unwrap_or(&f32::INFINITY) {
                            distance.insert(neighbor, next_cost);
                            prev.insert(neighbor, node);
                            heap.push((FloatKey(next_cost), neighbor));
                        }
                    }
                    _ => continue,
                }
            }
        }

        None
    }

    /// Helper function to get the type of a node from known_nodes
    fn get_node_type(&self, node_id: NodeId) -> Option<NodeType> {
        self.known_nodes.lock().unwrap().get(&node_id).copied()
    }
}
