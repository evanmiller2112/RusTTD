use std::collections::HashMap;
use crate::world::{CargoType, World};
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
                    self.start_moving_to_next_station();
                }
            }
            VehicleState::Moving { from: _, to, progress } => {
                *progress += self.speed as f32 / 1000.0;
                
                if *progress >= 1.0 {
                    self.x = to.0;
                    self.y = to.1;
                    self.state = VehicleState::Loading;
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

    fn start_moving_to_next_station(&mut self) {
        if let Some(&next_station) = self.route.get(self.route_index) {
            self.state = VehicleState::Moving {
                from: (self.x, self.y),
                to: next_station,
                progress: 0.0,
            };
        }
    }

    fn load_cargo_at_station(&mut self, world: &mut World) {
        if let Some(tile) = world.get_tile(self.x, self.y) {
            if let crate::world::TileContent::Station(station) = &tile.content {
                let available_capacity = self.get_capacity() - self.cargo.values().sum::<u32>();
                
                for (cargo_type, &amount) in &station.cargo_waiting {
                    let to_load = amount.min(available_capacity);
                    if to_load > 0 {
                        *self.cargo.entry(cargo_type.clone()).or_insert(0) += to_load;
                    }
                }
            }
        }
    }

    fn unload_cargo_at_station(&mut self, _world: &mut World, _economy: &mut Economy) {
        let delivered_cargo: Vec<_> = self.cargo.drain().collect();
        
        if !delivered_cargo.is_empty() {
            self.total_deliveries += 1;
            
            if rand::random::<f32>() > 0.8 {
                self.on_time_deliveries += 1;
            }
            
            let delivery_profit = self.calculate_delivery_profit();
            self.profit += delivery_profit;
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
}