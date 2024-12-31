use crate::RustbustersClient;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

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
        // Invertiamo l'ordine per fare in modo che BinaryHeap diventi un min-heap
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

impl RustbustersClient {
    pub(crate) fn find_weighted_path(&self, dst: NodeId) -> Option<Vec<NodeId>> {
        let mut distance: HashMap<NodeId, f32> = HashMap::new();
        let mut heap: BinaryHeap<(FloatKey, NodeId)> = BinaryHeap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        // Inizializza le distanze
        distance.insert(self.id, 0.0);
        heap.push((FloatKey(0.0), self.id));

        while let Some((FloatKey(cost), node)) = heap.pop() {
            // Se siamo arrivati alla destinazione e il nodo è un server, costruisci il percorso
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
                    // Se il nodo non è un server, non è possibile raggiungere la destinazione
                    None
                };
            }

            // Ignora percorsi più lunghi di quello già calcolato
            if let Some(&d) = distance.get(&node) {
                if cost > d {
                    continue;
                }
            }

            // Esplora i vicini
            for neighbor in self.topology.neighbors(node) {
                let edge_weight = *self
                    .topology
                    .edge_weight(node, neighbor)
                    .unwrap_or(&f32::INFINITY);
                let next_cost = cost + edge_weight;

                // Regole di validità del percorso
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

    fn get_node_type(&self, node_id: NodeId) -> Option<NodeType> {
        self.known_nodes.lock().unwrap().get(&node_id).copied()
    }
}
