# Client
Isaia's client implementation for the Advanced Programming Course 2024-2025 at UNITN

## Network
The client is built for the drone network. It has to be used as a library in the [Network Initializer](http://github.com/Rustbusters/network-initializer) project and in the [Simulation Controller](http://github.com/Rustbusters/simulation-controller) project.

The client implements the functionalities related to the drone network, such as:
- Packet Source Routing
- Packet Handling
- Network Discovery
- Communication with the Simulation Controller

It also implements a UI external and indipendent from the SC.

## The UI
The UI is implemented using a simple WebServer. The assets for the UI must be inserted in the `static/client/tonini` folder of Network Initializer.

The Server is started by default on `localhost:7373` with tiny_http.

#### Communication between Rust backend and the frontend
- The frontend uses tiny_http defined endpoints to communicate with the backend.
- The backend uses the `tungstenite` library to communicate with the frontend via WebSockets.
  > The WSS is started on `localhost:7374` and the frontend connects to it.

