use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::game::Game;
use crate::world::{World, Tile, TerrainType, TileContent, Town, Industry, IndustryType, Station, StationType, TrackType, CargoType};
use crate::player::{Player, Route};
use crate::vehicle::{Vehicle, VehicleType, VehicleState};
use crate::economy::{Economy, EconomicState};
use crate::ai::{AIPlayer, AIDifficulty, AIStrategy};

#[derive(Serialize, Deserialize)]
pub struct GameSave {
    pub world: WorldSave,
    pub player: PlayerSave,
    pub ai_players: Vec<AIPlayerSave>,
    pub economy: EconomySave,
    pub game_time: u32,
}

#[derive(Serialize, Deserialize)]
pub struct WorldSave {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileSave>>,
    pub towns: Vec<(usize, usize)>,
    pub industries: Vec<(usize, usize)>,
    pub stations: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize)]
pub struct TileSave {
    pub terrain: TerrainType,
    pub content: TileContentSave,
    pub height: u8,
}

#[derive(Serialize, Deserialize)]
pub enum TileContentSave {
    Empty,
    Town(TownSave),
    Industry(IndustrySave),
    Station(StationSave),
    Track(TrackTypeSave),
    Road,
}

#[derive(Serialize, Deserialize)]
pub struct TownSave {
    pub name: String,
    pub population: u32,
    pub growth_rate: f32,
}

#[derive(Serialize, Deserialize)]
pub struct IndustrySave {
    pub industry_type: IndustryType,
    pub production_rate: u32,
    pub cargo_input: Vec<CargoType>,
    pub cargo_output: Vec<CargoType>,
}

#[derive(Serialize, Deserialize)]
pub struct StationSave {
    pub name: String,
    pub station_type: StationType,
    pub connections: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize)]
pub enum TrackTypeSave {
    Straight { horizontal: bool },
    Curve { from_dir: u8, to_dir: u8 },
    Junction,
}


#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub name: String,
    pub money: i64,
    pub vehicles: Vec<VehicleSave>,
    pub stations: Vec<(usize, usize)>,
    pub routes: Vec<RouteSave>,
    pub reputation: f32,
    pub game_time: u32,
}

#[derive(Serialize, Deserialize)]
pub struct RouteSave {
    pub id: u32,
    pub name: String,
    pub stations: Vec<(usize, usize)>,
    pub vehicle_ids: Vec<u32>,
    pub cargo_types: Vec<CargoType>,
    pub profit: i64,
}

#[derive(Serialize, Deserialize)]
pub struct VehicleSave {
    pub id: u32,
    pub vehicle_type: VehicleType,
    pub x: usize,
    pub y: usize,
    pub state: VehicleStateSave,
    pub route: Vec<(usize, usize)>,
    pub route_index: usize,
    pub current_path: Vec<(usize, usize)>,
    pub path_index: usize,
    pub age: u32,
    pub reliability: u8,
    pub speed: u32,
    pub last_service: u32,
    pub profit: i64,
    pub on_time_deliveries: u32,
    pub total_deliveries: u32,
}

#[derive(Serialize, Deserialize)]
pub enum VehicleStateSave {
    Idle,
    Moving { from: (usize, usize), to: (usize, usize), progress: f32 },
    Loading,
    Unloading,
    Broken,
}

#[derive(Serialize, Deserialize)]
pub struct EconomySave {
    pub inflation_rate: f32,
    pub economic_state: EconomicState,
    pub month: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AIPlayerSave {
    pub player: PlayerSave,
    pub difficulty: AIDifficulty,
    pub strategy: AIStrategy,
    pub decision_timer: u32,
    pub last_action: u32,
}

impl GameSave {
    pub fn from_game(game: &Game) -> Self {
        Self {
            world: WorldSave::from_world(&game.world),
            player: PlayerSave::from_player(&game.player),
            ai_players: game.ai_players.iter().map(AIPlayerSave::from_ai_player).collect(),
            economy: EconomySave::from_economy(&game.economy),
            game_time: game.player.game_time,
        }
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(filename, json)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !Path::new(filename).exists() {
            return Err("Save file not found".into());
        }
        
        let json = fs::read_to_string(filename)?;
        let save: GameSave = serde_json::from_str(&json)?;
        Ok(save)
    }

    pub fn to_game(self) -> Game {
        let mut game = Game::new();
        game.world = self.world.to_world();
        game.player = self.player.to_player();
        game.ai_players = self.ai_players.into_iter().map(|ai| ai.to_ai_player()).collect();
        game.economy = self.economy.to_economy();
        game
    }
}

impl WorldSave {
    fn from_world(world: &World) -> Self {
        Self {
            width: world.width,
            height: world.height,
            tiles: world.tiles.iter().map(|row| {
                row.iter().map(TileSave::from_tile).collect()
            }).collect(),
            towns: world.towns.clone(),
            industries: world.industries.clone(),
            stations: world.stations.clone(),
        }
    }

    fn to_world(self) -> World {
        let mut world = World::new(self.width, self.height);
        world.tiles = self.tiles.into_iter().map(|row| {
            row.into_iter().map(|tile| tile.to_tile()).collect()
        }).collect();
        world.towns = self.towns;
        world.industries = self.industries;
        world.stations = self.stations;
        world
    }
}

impl TileSave {
    fn from_tile(tile: &Tile) -> Self {
        Self {
            terrain: tile.terrain.clone(),
            content: TileContentSave::from_tile_content(&tile.content),
            height: tile.height,
        }
    }

    fn to_tile(self) -> Tile {
        Tile {
            terrain: self.terrain,
            content: self.content.to_tile_content(),
            height: self.height,
        }
    }
}

impl TileContentSave {
    fn from_tile_content(content: &TileContent) -> Self {
        match content {
            TileContent::Empty => TileContentSave::Empty,
            TileContent::Town(town) => TileContentSave::Town(TownSave::from_town(town)),
            TileContent::Industry(industry) => TileContentSave::Industry(IndustrySave::from_industry(industry)),
            TileContent::Station(station) => TileContentSave::Station(StationSave::from_station(station)),
            TileContent::Track(track) => TileContentSave::Track(TrackTypeSave::from_track_type(track)),
            TileContent::Road => TileContentSave::Road,
        }
    }

    fn to_tile_content(self) -> TileContent {
        match self {
            TileContentSave::Empty => TileContent::Empty,
            TileContentSave::Town(town) => TileContent::Town(town.to_town()),
            TileContentSave::Industry(industry) => TileContent::Industry(industry.to_industry()),
            TileContentSave::Station(station) => TileContent::Station(station.to_station()),
            TileContentSave::Track(track) => TileContent::Track(track.to_track_type()),
            TileContentSave::Road => TileContent::Road,
        }
    }
}

impl TownSave {
    fn from_town(town: &Town) -> Self {
        Self {
            name: town.name.clone(),
            population: town.population,
            growth_rate: town.growth_rate,
        }
    }

    fn to_town(self) -> Town {
        Town {
            name: self.name,
            population: self.population,
            growth_rate: self.growth_rate,
            cargo_demand: std::collections::HashMap::new(),
            cargo_supply: std::collections::HashMap::new(),
        }
    }
}

impl IndustrySave {
    fn from_industry(industry: &Industry) -> Self {
        Self {
            industry_type: industry.industry_type.clone(),
            production_rate: industry.production_rate,
            cargo_input: industry.cargo_input.clone(),
            cargo_output: industry.cargo_output.clone(),
        }
    }

    fn to_industry(self) -> Industry {
        Industry {
            industry_type: self.industry_type,
            production_rate: self.production_rate,
            cargo_input: self.cargo_input,
            cargo_output: self.cargo_output,
            stockpile: std::collections::HashMap::new(),
        }
    }
}

impl StationSave {
    fn from_station(station: &Station) -> Self {
        Self {
            name: station.name.clone(),
            station_type: station.station_type.clone(),
            connections: station.connections.clone(),
        }
    }

    fn to_station(self) -> Station {
        Station {
            name: self.name,
            station_type: self.station_type,
            cargo_waiting: std::collections::HashMap::new(),
            connections: self.connections,
        }
    }
}

impl TrackTypeSave {
    fn from_track_type(track: &TrackType) -> Self {
        match track {
            TrackType::Straight { horizontal } => TrackTypeSave::Straight { horizontal: *horizontal },
            TrackType::Curve { from_dir: _, to_dir: _ } => TrackTypeSave::Curve {
                from_dir: 0,
                to_dir: 1,
            },
            TrackType::Junction => TrackTypeSave::Junction,
        }
    }

    fn to_track_type(self) -> TrackType {
        use crate::world::Direction;
        match self {
            TrackTypeSave::Straight { horizontal } => TrackType::Straight { horizontal },
            TrackTypeSave::Curve { from_dir: _, to_dir: _ } => TrackType::Curve {
                from_dir: Direction::North,
                to_dir: Direction::East,
            },
            TrackTypeSave::Junction => TrackType::Junction,
        }
    }
}


impl PlayerSave {
    fn from_player(player: &Player) -> Self {
        Self {
            name: player.name.clone(),
            money: player.money,
            vehicles: player.vehicles.iter().map(VehicleSave::from_vehicle).collect(),
            stations: player.stations.clone(),
            routes: player.routes.iter().map(RouteSave::from_route).collect(),
            reputation: player.reputation,
            game_time: player.game_time,
        }
    }

    fn to_player(self) -> Player {
        let mut player = Player::new(self.name, self.money);
        player.vehicles = self.vehicles.into_iter().map(|v| v.to_vehicle()).collect();
        player.stations = self.stations;
        player.routes = self.routes.into_iter().map(|r| r.to_route()).collect();
        player.reputation = self.reputation;
        player.game_time = self.game_time;
        player
    }
}

impl RouteSave {
    fn from_route(route: &Route) -> Self {
        Self {
            id: route.id,
            name: route.name.clone(),
            stations: route.stations.clone(),
            vehicle_ids: route.vehicle_ids.clone(),
            cargo_types: route.cargo_types.clone(),
            profit: route.profit,
        }
    }

    fn to_route(self) -> Route {
        Route {
            id: self.id,
            name: self.name,
            stations: self.stations,
            vehicle_ids: self.vehicle_ids,
            cargo_types: self.cargo_types,
            profit: self.profit,
        }
    }
}

impl VehicleSave {
    fn from_vehicle(vehicle: &Vehicle) -> Self {
        Self {
            id: vehicle.id,
            vehicle_type: vehicle.vehicle_type.clone(),
            x: vehicle.x,
            y: vehicle.y,
            state: VehicleStateSave::from_vehicle_state(&vehicle.state),
            route: vehicle.route.clone(),
            route_index: vehicle.route_index,
            current_path: vehicle.current_path.clone(),
            path_index: vehicle.path_index,
            age: vehicle.age,
            reliability: vehicle.reliability,
            speed: vehicle.speed,
            last_service: vehicle.last_service,
            profit: vehicle.profit,
            on_time_deliveries: vehicle.on_time_deliveries,
            total_deliveries: vehicle.total_deliveries,
        }
    }

    fn to_vehicle(self) -> Vehicle {
        Vehicle {
            id: self.id,
            vehicle_type: self.vehicle_type,
            x: self.x,
            y: self.y,
            state: self.state.to_vehicle_state(),
            cargo: std::collections::HashMap::new(),
            route: self.route,
            route_index: self.route_index,
            current_path: self.current_path,
            path_index: self.path_index,
            age: self.age,
            reliability: self.reliability,
            speed: self.speed,
            last_service: self.last_service,
            profit: self.profit,
            on_time_deliveries: self.on_time_deliveries,
            total_deliveries: self.total_deliveries,
        }
    }
}

impl VehicleStateSave {
    fn from_vehicle_state(state: &VehicleState) -> Self {
        match state {
            VehicleState::Idle => VehicleStateSave::Idle,
            VehicleState::Moving { from, to, progress } => VehicleStateSave::Moving {
                from: *from,
                to: *to,
                progress: *progress,
            },
            VehicleState::Loading => VehicleStateSave::Loading,
            VehicleState::Unloading => VehicleStateSave::Unloading,
            VehicleState::Broken => VehicleStateSave::Broken,
        }
    }

    fn to_vehicle_state(self) -> VehicleState {
        match self {
            VehicleStateSave::Idle => VehicleState::Idle,
            VehicleStateSave::Moving { from, to, progress } => VehicleState::Moving { from, to, progress },
            VehicleStateSave::Loading => VehicleState::Loading,
            VehicleStateSave::Unloading => VehicleState::Unloading,
            VehicleStateSave::Broken => VehicleState::Broken,
        }
    }
}

impl EconomySave {
    fn from_economy(economy: &Economy) -> Self {
        Self {
            inflation_rate: economy.inflation_rate,
            economic_state: economy.economic_state.clone(),
            month: economy.month,
        }
    }

    fn to_economy(self) -> Economy {
        let mut economy = Economy::new();
        economy.inflation_rate = self.inflation_rate;
        economy.economic_state = self.economic_state;
        economy.month = self.month;
        economy
    }
}

impl AIPlayerSave {
    fn from_ai_player(ai_player: &AIPlayer) -> Self {
        Self {
            player: PlayerSave::from_player(&ai_player.player),
            difficulty: ai_player.difficulty.clone(),
            strategy: ai_player.strategy.clone(),
            decision_timer: ai_player.decision_timer,
            last_action: ai_player.last_action,
        }
    }

    fn to_ai_player(self) -> AIPlayer {
        AIPlayer {
            player: self.player.to_player(),
            difficulty: self.difficulty,
            strategy: self.strategy,
            decision_timer: self.decision_timer,
            last_action: self.last_action,
            targets: Vec::new(),
        }
    }
}

pub fn save_game(game: &Game, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let save = GameSave::from_game(game);
    save.save_to_file(filename)
}

pub fn load_game(filename: &str) -> Result<Game, Box<dyn std::error::Error>> {
    let save = GameSave::load_from_file(filename)?;
    Ok(save.to_game())
}