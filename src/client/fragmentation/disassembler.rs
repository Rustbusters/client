use crate::client::RustbustersClient;
use common_utils::HostMessage;
use wg_2024::packet::{Fragment, FRAGMENT_DSIZE};

impl RustbustersClient {
    /// Splits a message into multiple fragments of fixed size.
    ///
    /// This function serializes a message and splits it into fragments that can be
    /// transmitted over the network. Each fragment contains:
    /// - fragment_index: Position in the sequence
    /// - total_n_fragments: Total number of fragments
    /// - length: Actual data length in the fragment
    /// - data: Fixed-size array containing the fragment data
    ///
    /// ### Arguments
    /// * `message` - The message to be fragmented
    ///
    /// ### Returns
    /// A vector containing all fragments of the message
    pub(crate) fn disassemble_message(&self, message: &HostMessage) -> Vec<Fragment> {
        let serialized_str = serde_json::to_string(&message).unwrap();
        let bytes = serialized_str.as_bytes();

        // Fragment the data into chunks of FRAGMENT_DSIZE bytes
        let total_size = bytes.len();
        let total_n_fragments = ((total_size + FRAGMENT_DSIZE - 1) / FRAGMENT_DSIZE) as u64;

        let mut fragments = Vec::new();
        for (i, chunk) in bytes.chunks(FRAGMENT_DSIZE).enumerate() {
            let mut data_array = [0u8; FRAGMENT_DSIZE];
            let length = chunk.len();
            data_array[..length].copy_from_slice(chunk);

            let fragment = Fragment {
                fragment_index: i as u64,
                total_n_fragments,
                length: length as u8,
                data: data_array,
            };

            fragments.push(fragment);
        }

        fragments
    }
}
