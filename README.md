# RusTTD - Transport Tycoon in Rust

A transport tycoon game built in Rust with both terminal and web client support.

## Features

- 🚂 **Realistic train movement** - Trains follow tracks and pathfind between stations
- 📦 **Complete cargo system** - Industries produce cargo, stations handle transfers, vehicles transport goods
- 🏭 **Economic simulation** - Supply and demand, production chains, town growth
- 🎮 **Dual interface** - Play in terminal or web browser
- 🌐 **Client-server architecture** - Real-time multiplayer ready

## Running the Game

### Terminal Version (Original)
```bash
cargo run
```

### Web Version
```bash
# Start the web server
cargo run --bin server

# Open your browser to:
# http://127.0.0.1:3000
```

## Architecture

### Client-Server Design
- **Server**: Runs game logic, handles state, serves web client
- **Terminal Client**: Direct connection to game engine
- **Web Client**: JavaScript client communicating via WebSocket
- **API**: REST endpoints for game state and input

The game demonstrates a successful migration from a terminal-only application to a modern client-server web architecture while maintaining full backward compatibility.