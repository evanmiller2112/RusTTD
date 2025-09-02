# RusTTD - Terminal Transport Tycoon

A terminal-based transport tycoon game written in Rust, inspired by Transport Tycoon Deluxe and OpenTTD.

## Features

### Core Gameplay
- **ASCII World Map**: Navigate a procedurally generated world with towns, industries, and terrain
- **Economic Simulation**: Dynamic supply and demand system with realistic pricing
- **Vehicle Management**: Purchase and manage trains, buses, trucks, ships, and aircraft
- **Route Planning**: Create profitable transport routes between cities and industries
- **Company Management**: Track finances, reputation, and company growth

### World System
- **Terrain Types**: Grass, water, mountains, deserts, and forests
- **Cities and Towns**: Growing populations with passenger and cargo demands
- **Industries**: Coal mines, factories, steel mills, farms, and more with production chains
- **Infrastructure**: Build railways, roads, stations, and airports
- **Vehicle Display**: Real-time visualization of vehicles on the map with distinct symbols

### Vehicle Types
- **Trains**: Steam, diesel, and electric engines with passenger and freight cars
- **Road Vehicles**: Buses, small trucks, and large trucks
- **Ships**: Cargo ships and passenger ferries
- **Aircraft**: Small and large planes for long-distance transport

### Economic Features
- **Dynamic Markets**: Supply and demand affects cargo prices
- **Economic Cycles**: Boom, stable, and recession periods
- **Inflation**: Realistic price changes over time
- **Distance-based Pricing**: Longer routes yield higher profits

### AI Competition
- **Difficulty Levels**: Easy, Medium, and Hard AI opponents
- **AI Strategies**: Aggressive, Conservative, Balanced, and Specialist approaches
- **Smart Decision Making**: AI builds profitable routes and expands strategically

### Save System
- **JSON Format**: Human-readable save files
- **Complete State**: Save and load entire game world, economy, and progress
- **Quick Save/Load**: F5 to save, F9 to load during gameplay

## Controls

### Basic Navigation
- **Arrow Keys / WASD**: Move cursor around the map
- **Enter / Space**: Select tile to view information

### Game Actions
- **B**: Open build menu
- **1-5**: Select build option (when menu is open)
- **Enter/Space**: Build selected item or select tile
- **ESC**: Cancel build mode/close menus
- **P**: Pause game (planned feature)

### Save/Load
- **F5**: Quick save game
- **F9**: Quick load game

### Help & Interface
- **?**: Show/hide controls help popup
- **ESC**: Close any open popups/menus

### Exit
- **Q**: Quit game

*Press ? in-game for a detailed controls popup menu*

## Vehicle Visualization

### Map Symbols
- **T** - Trains (colored blue, bold)
- **B** - Buses (colored red, bold)
- **t** - Small Trucks (colored red, bold)
- **T** - Large Trucks (colored red, bold)
- **S** - Ships (colored cyan, bold)
- **A** - Aircraft (colored magenta, bold)

### Vehicle Information
When you position your cursor over a vehicle, the info panel shows:
- Vehicle type and specifications
- Current position and state (Idle/Moving/Loading/etc.)
- Age, reliability, and maintenance status
- Financial performance (profit, deliveries)
- Capacity and current cargo

## Technical Architecture

### Modules
- `game.rs`: Main game loop and state management
- `world.rs`: Map generation, terrain, cities, and industries
- `ui.rs`: Terminal user interface using Ratatui
- `player.rs`: Player company management and statistics
- `vehicle.rs`: Vehicle types, movement, and cargo handling
- `economy.rs`: Market simulation and pricing
- `ai.rs`: AI opponent logic and decision making
- `save.rs`: Game save and load functionality

### Dependencies
- **crossterm**: Cross-platform terminal manipulation
- **ratatui**: Rich terminal user interface framework
- **serde**: Serialization for save/load system
- **rand**: Random number generation for world generation

## Building and Running

```bash
# Clone the repository
cd RustroverProjects/RusTTD

# Build the project
cargo build --release

# Run the game
cargo run --release
```

## Development Status

This is a complete working implementation with:
- ✅ Core game architecture
- ✅ Terminal UI with map navigation
- ✅ World generation with cities and industries  
- ✅ Economic simulation
- ✅ Vehicle and route management systems
- ✅ AI opponent system
- ✅ Save/load functionality
- ✅ Interactive build menu and construction system
- ✅ Railway tracks, roads, stations, and vehicle purchasing
- 🔄 Route assignment and vehicle automation (planned)
- 🔄 More vehicle types and advanced routes (planned)
- 🔄 Campaign mode and scenarios (planned)

## Game Concepts

### Transport Chain Example
1. **Coal Mine** produces coal
2. **Train** transports coal to **Steel Mill**
3. **Steel Mill** converts coal + iron ore to steel
4. **Truck** delivers steel to **Factory**
5. **Factory** produces goods for **Cities**

### Profitability Factors
- **Distance**: Longer routes = higher profits
- **Demand**: High-demand cargo pays more
- **Speed**: Faster delivery = better reputation
- **Efficiency**: Full loads maximize income
- **Economic Conditions**: Boom times increase all prices

### AI Behavior
- **Easy AI**: Slower decision making, smaller starting capital
- **Medium AI**: Balanced approach with moderate capital
- **Hard AI**: Aggressive expansion with large starting funds
- **Strategy Types**:
  - Aggressive: Quick expansion, high-capacity vehicles
  - Conservative: Focuses on profitable existing routes
  - Balanced: Mix of expansion and optimization
  - Specialist: Focuses on specific cargo types

## License

This project is open source and available under the MIT License.