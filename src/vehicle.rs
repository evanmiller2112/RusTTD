use std::collections::{HashMap, VecDeque, HashSet};
use crate::world::{CargoType, World, TileContent};
use crate::economy::Economy;

// Configuration: Set to false to disable vehicle breakdowns
const BREAKDOWNS_ENABLED: bool = false;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum VehicleType {
    Train { engine: TrainEngine, cars: Vec<TrainCar> },
    Road { truck_type: TruckType },
    Ship { ship_type: ShipType },
    Aircraft { plane_type: PlaneType },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TrainEngine {
    Steam { power: u32, reliability: u8 },
    Diesel { power: u32, reliability: u8 },
    Electric { power: u32, reliability: u8 },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TrainCar {
    Passenger { capacity: u32 },
    Freight { capacity: u32, cargo_type: Option<CargoType> },
    Mail { capacity: u32 },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TruckType {
    SmallTruck { capacity: u32 },
    LargeTruck { capacity: u32 },
    Bus { capacity: u32 },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ShipType {
    CargoShip { capacity: u32 },
    PassengerShip { capacity: u32 },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PlaneType {
    SmallPlane { capacity: u32, range: u32 },
    LargePlane { capacity: u32, range: u32 },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum VehicleState {
    Idle,
    Moving { from: (usize, usize), to: (usize, usize), progress: f32 },
    Loading,
    Unloading,
    Broken,
}

pub struct Vehicle {
    pub id: u32,
    pub vehicle_type: VehicleType,
    pub x: usize,
    pub y: usize,
    pub state: VehicleState,
    pub cargo: HashMap<CargoType, u32>,
    pub route: Vec<(usize, usize)>,
    pub route_index: usize,
    pub current_path: Vec<(usize, usize)>, // Step-by-step path to next station
    pub path_index: usize, // Current position in the path
    pub age: u32,
    pub reliability: u8,
    pub speed: u32,
    pub last_service: u32,
    pub profit: i64,
    pub on_time_deliveries: u32,
    pub total_deliveries: u32,
}

impl Vehicle {
    pub fn new(id: u32, vehicle_type: VehicleType, x: usize, y: usize) -> Self {
        let (speed, reliability) = Self::get_vehicle_stats(&vehicle_type);
        
        Self {
            id,
            vehicle_type,
            x,
            y,
            state: VehicleState::Idle,
            cargo: HashMap::new(),
            route: Vec::new(),
            route_index: 0,
            current_path: Vec::new(),
            path_index: 0,
            age: 0,
            reliability,
            speed,
            last_service: 0,
            profit: 0,
            on_time_deliveries: 0,
            total_deliveries: 0,
        }
    }

    pub fn update(&mut self, world: &mut World, economy: &mut Economy) {
        self.age += 1;
        
        if self.age > 0 && self.age % 365 == 0 {
            self.reliability = self.reliability.saturating_sub(5);
        }

        if BREAKDOWNS_ENABLED && self.age - self.last_service > 180 {
            if rand::random::<u8>() > self.reliability {
                self.state = VehicleState::Broken;
                return;
            }
        }

        match &mut self.state {
            VehicleState::Idle => {
                if !self.route.is_empty() {
                    self.start_moving_to_next_station(world);
                }
            }
            VehicleState::Moving { from: _, to, progress } => {
                *progress += self.speed as f32 / 1000.0;
                
                if *progress >= 1.0 {
                    // Move to the next tile
                    self.x = to.0;
                    self.y = to.1;
                    
                    // Check if we've reached the final destination
                    if self.path_index >= self.current_path.len() - 1 {
                        // Reached the station
                        self.state = VehicleState::Loading;
                        self.current_path.clear();
                        self.path_index = 0;
                    } else {
                        // Move to next tile in path
                        self.path_index += 1;
                        *progress = 0.0;
                        *to = self.current_path[self.path_index];
                    }
                }
            }
            VehicleState::Loading => {
                self.load_cargo_at_station(world);
                self.state = VehicleState::Idle;
                self.route_index = (self.route_index + 1) % self.route.len();
            }
            VehicleState::Unloading => {
                self.unload_cargo_at_station(world, economy);
                self.state = VehicleState::Idle;
            }
            VehicleState::Broken => {
                if rand::random::<u8>() < 10 {
                    self.state = VehicleState::Idle;
                    self.last_service = self.age;
                    self.reliability = (self.reliability + 20).min(100);
                }
            }
        }
    }

    pub fn assign_route(&mut self, stations: Vec<(usize, usize)>) {
        self.route = stations;
        self.route_index = 0;
    }

    pub fn get_capacity(&self) -> u32 {
        match &self.vehicle_type {
            VehicleType::Train { cars, .. } => {
                cars.iter().map(|car| match car {
                    TrainCar::Passenger { capacity } => *capacity,
                    TrainCar::Freight { capacity, .. } => *capacity,
                    TrainCar::Mail { capacity } => *capacity,
                }).sum()
            }
            VehicleType::Road { truck_type } => match truck_type {
                TruckType::SmallTruck { capacity } => *capacity,
                TruckType::LargeTruck { capacity } => *capacity,
                TruckType::Bus { capacity } => *capacity,
            }
            VehicleType::Ship { ship_type } => match ship_type {
                ShipType::CargoShip { capacity } => *capacity,
                ShipType::PassengerShip { capacity } => *capacity,
            }
            VehicleType::Aircraft { plane_type } => match plane_type {
                PlaneType::SmallPlane { capacity, .. } => *capacity,
                PlaneType::LargePlane { capacity, .. } => *capacity,
            }
        }
    }

    pub fn get_purchase_cost(vehicle_type: &VehicleType) -> i64 {
        match vehicle_type {
            VehicleType::Train { engine, cars } => {
                let engine_cost = match engine {
                    TrainEngine::Steam { power, .. } => *power as i64 * 100,
                    TrainEngine::Diesel { power, .. } => *power as i64 * 150,
                    TrainEngine::Electric { power, .. } => *power as i64 * 200,
                };
                let car_cost: i64 = cars.iter().map(|car| match car {
                    TrainCar::Passenger { .. } => 50000,
                    TrainCar::Freight { .. } => 30000,
                    TrainCar::Mail { .. } => 40000,
                }).sum();
                engine_cost + car_cost
            }
            VehicleType::Road { truck_type } => match truck_type {
                TruckType::SmallTruck { .. } => 75000,
                TruckType::LargeTruck { .. } => 150000,
                TruckType::Bus { .. } => 120000,
            }
            VehicleType::Ship { ship_type } => match ship_type {
                ShipType::CargoShip { .. } => 500000,
                ShipType::PassengerShip { .. } => 800000,
            }
            VehicleType::Aircraft { plane_type } => match plane_type {
                PlaneType::SmallPlane { .. } => 2000000,
                PlaneType::LargePlane { .. } => 10000000,
            }
        }
    }

    pub fn get_running_costs(&self) -> u32 {
        let base_cost = match &self.vehicle_type {
            VehicleType::Train { .. } => 1000,
            VehicleType::Road { .. } => 200,
            VehicleType::Ship { .. } => 800,
            VehicleType::Aircraft { .. } => 3000,
        };
        
        let age_multiplier = 1.0 + (self.age as f32 / 365.0) * 0.1;
        let reliability_multiplier = 2.0 - (self.reliability as f32 / 100.0);
        
        (base_cost as f32 * age_multiplier * reliability_multiplier) as u32
    }

    pub fn get_current_value(&self) -> i64 {
        let purchase_cost = Self::get_purchase_cost(&self.vehicle_type);
        let depreciation = (self.age as f32 / 365.0 * 0.15).min(0.8);
        (purchase_cost as f32 * (1.0 - depreciation)) as i64
    }

    pub fn has_delivered_cargo(&self) -> bool {
        self.total_deliveries > 0
    }

    pub fn calculate_delivery_profit(&self) -> i64 {
        let base_profit = self.cargo.values().sum::<u32>() as i64 * 10;
        let distance_bonus = if self.route.len() > 1 { 
            self.calculate_route_distance() as i64 * 5 
        } else { 
            0 
        };
        base_profit + distance_bonus
    }

    pub fn is_on_time(&self) -> bool {
        if self.total_deliveries == 0 {
            true
        } else {
            (self.on_time_deliveries as f32 / self.total_deliveries as f32) > 0.8
        }
    }

    fn get_vehicle_stats(vehicle_type: &VehicleType) -> (u32, u8) {
        match vehicle_type {
            VehicleType::Train { engine, .. } => match engine {
                TrainEngine::Steam { reliability, .. } => (60, *reliability),
                TrainEngine::Diesel { reliability, .. } => (80, *reliability),
                TrainEngine::Electric { reliability, .. } => (100, *reliability),
            }
            VehicleType::Road { truck_type } => match truck_type {
                TruckType::SmallTruck { .. } => (90, 85),
                TruckType::LargeTruck { .. } => (70, 80),
                TruckType::Bus { .. } => (85, 88),
            }
            VehicleType::Ship { .. } => (40, 90),
            VehicleType::Aircraft { plane_type } => match plane_type {
                PlaneType::SmallPlane { .. } => (300, 75),
                PlaneType::LargePlane { .. } => (250, 85),
            }
        }
    }

    fn start_moving_to_next_station(&mut self, world: &World) {
        if let Some(&next_station) = self.route.get(self.route_index) {
            // Find path to the next station
            if let Some(path) = self.find_path_to_station(world, next_station) {
                if path.len() > 1 {
                    // Set up step-by-step movement
                    self.current_path = path;
                    self.path_index = 1; // Start at index 1 (0 is current position)
                    
                    self.state = VehicleState::Moving {
                        from: (self.x, self.y),
                        to: self.current_path[self.path_index],
                        progress: 0.0,
                    };
                } else {
                    // Already at destination
                    self.state = VehicleState::Loading;
                }
            } else {
                // No path found - can't move
                self.state = VehicleState::Idle;
            }
        }
    }

    fn load_cargo_at_station(&mut self, world: &mut World) {
        let available_capacity = self.get_capacity() - self.cargo.values().sum::<u32>();
        
        if available_capacity == 0 {
            return; // Vehicle is full
        }
        
        // Get mutable access to the tile
        if let Some(tile) = world.tiles.get_mut(self.y).and_then(|row| row.get_mut(self.x)) {
            if let crate::world::TileContent::Station(ref mut station) = tile.content {
                let mut remaining_capacity = available_capacity;
                let mut cargo_to_remove = Vec::new();
                
                // Determine what cargo to load
                for (cargo_type, &amount) in &station.cargo_waiting {
                    if amount > 0 && remaining_capacity > 0 {
                        let to_load = amount.min(remaining_capacity);
                        *self.cargo.entry(cargo_type.clone()).or_insert(0) += to_load;
                        cargo_to_remove.push((cargo_type.clone(), to_load));
                        remaining_capacity -= to_load;
                    }
                }
                
                // Remove loaded cargo from station
                for (cargo_type, amount) in cargo_to_remove {
                    if let Some(waiting_amount) = station.cargo_waiting.get_mut(&cargo_type) {
                        *waiting_amount = waiting_amount.saturating_sub(amount);
                        if *waiting_amount == 0 {
                            station.cargo_waiting.remove(&cargo_type);
                        }
                    }
                }
            }
        }
    }

    fn unload_cargo_at_station(&mut self, world: &mut World, _economy: &mut Economy) {
        let delivered_cargo: Vec<_> = self.cargo.drain().collect();
        
        if !delivered_cargo.is_empty() {
            // Try to deliver cargo to nearby towns
            let mut total_delivered = 0;
            
            // Check nearby tiles for towns that might want this cargo
            for dx in -2i32..=2i32 {
                for dy in -2i32..=2i32 {
                    let check_x = (self.x as i32 + dx).max(0).min(world.width as i32 - 1) as usize;
                    let check_y = (self.y as i32 + dy).max(0).min(world.height as i32 - 1) as usize;
                    
                    if let Some(tile) = world.tiles.get_mut(check_y).and_then(|row| row.get_mut(check_x)) {
                        if let crate::world::TileContent::Town(ref mut town) = tile.content {
                            // Deliver all cargo types to town (simplified)
                            for (cargo_type, amount) in &delivered_cargo {
                                *town.cargo_demand.entry(cargo_type.clone()).or_insert(0) = 
                                    town.cargo_demand.get(cargo_type).unwrap_or(&0).saturating_sub(*amount);
                                total_delivered += amount;
                            }
                        }
                    }
                }
            }
            
            if total_delivered > 0 {
                self.total_deliveries += 1;
                
                // Better on-time delivery rate for now
                if rand::random::<f32>() > 0.2 {
                    self.on_time_deliveries += 1;
                }
                
                let delivery_profit = total_delivered as i64 * 50; // $50 per unit delivered
                self.profit += delivery_profit;
            }
        }
    }

    fn calculate_route_distance(&self) -> f32 {
        if self.route.len() < 2 {
            return 0.0;
        }
        
        let mut total_distance = 0.0;
        for i in 0..self.route.len() - 1 {
            let (x1, y1) = self.route[i];
            let (x2, y2) = self.route[i + 1];
            let distance = ((x2 as f32 - x1 as f32).powi(2) + (y2 as f32 - y1 as f32).powi(2)).sqrt();
            total_distance += distance;
        }
        total_distance
    }

    // Pathfinding methods
    fn find_path_to_station(&self, world: &World, target: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        match self.vehicle_type {
            VehicleType::Train { .. } => self.find_train_path(world, (self.x, self.y), target),
            VehicleType::Road { .. } => self.find_road_path(world, (self.x, self.y), target),
            _ => {
                // For ships and planes, use direct path for now
                Some(vec![target])
            }
        }
    }

    fn find_train_path(&self, world: &World, start: (usize, usize), goal: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        // A* pathfinding for trains using only tracks and stations
        let mut open_set = VecDeque::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut visited = HashSet::new();

        open_set.push_back(start);
        g_score.insert(start, 0);

        while let Some(current) = open_set.pop_front() {
            if current == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current_pos = current;
                
                while let Some(&previous) = came_from.get(&current_pos) {
                    path.push(current_pos);
                    current_pos = previous;
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // Check all adjacent tiles
            let neighbors = self.get_train_neighbors(world, current);
            for neighbor in neighbors {
                if visited.contains(&neighbor) {
                    continue;
                }

                let tentative_g_score = g_score.get(&current).unwrap_or(&u32::MAX) + 1;
                
                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g_score);
                    
                    if !open_set.contains(&neighbor) {
                        open_set.push_back(neighbor);
                    }
                }
            }
        }

        None // No path found
    }

    fn find_road_path(&self, world: &World, start: (usize, usize), goal: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        // Simple pathfinding for road vehicles - can use roads and empty terrain
        let mut open_set = VecDeque::new();
        let mut came_from = HashMap::new();
        let mut visited = HashSet::new();

        open_set.push_back(start);

        while let Some(current) = open_set.pop_front() {
            if current == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current_pos = current;
                
                while let Some(&previous) = came_from.get(&current_pos) {
                    path.push(current_pos);
                    current_pos = previous;
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            let neighbors = self.get_road_neighbors(world, current);
            for neighbor in neighbors {
                if !visited.contains(&neighbor) && !came_from.contains_key(&neighbor) {
                    came_from.insert(neighbor, current);
                    open_set.push_back(neighbor);
                }
            }
        }

        None // No path found
    }

    fn get_train_neighbors(&self, world: &World, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let (x, y) = pos;

        // Check all 4 directions
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if let (Some(nx), Some(ny)) = (x.checked_add_signed(dx), y.checked_add_signed(dy)) {
                if let Some(tile) = world.get_tile(nx, ny) {
                    match &tile.content {
                        TileContent::Track(_) | TileContent::Station(_) => {
                            neighbors.push((nx, ny));
                        },
                        _ => {} // Trains can't use other tile types
                    }
                }
            }
        }

        neighbors
    }

    fn get_road_neighbors(&self, world: &World, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let (x, y) = pos;

        // Check all 4 directions  
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if let (Some(nx), Some(ny)) = (x.checked_add_signed(dx), y.checked_add_signed(dy)) {
                if let Some(tile) = world.get_tile(nx, ny) {
                    match &tile.content {
                        TileContent::Road | TileContent::Station(_) | TileContent::Empty => {
                            // Road vehicles can use roads, stations, and empty terrain
                            if !matches!(tile.terrain, crate::world::TerrainType::Water | crate::world::TerrainType::Mountain) {
                                neighbors.push((nx, ny));
                            }
                        },
                        _ => {}
                    }
                }
            }
        }

        neighbors
    }
}