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
The UI is implemented using a simple WebServer. The assets for the UI must be inserted in the `static` folder of Network Initializer.

The Server is started by default on `localhost:8000`, but it can be changed in the `Rocket.toml` file in the root of the Network Initializer project.
```toml
[default]
address="localhost"
port=8080
```

#### Communication between Rust backend and the frontend
- The `rocket` server uses SSE (Server-Sent Events) to send updates to the client autonomously.
- The client uses the APIs provided by the server to interact with the network.

