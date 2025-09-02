use crate::player::Player;
use crate::world::{World, CargoType};
use crate::economy::Economy;
use crate::vehicle::{Vehicle, VehicleType, TrainEngine, TrainCar, TruckType};
use rand::Rng;

pub struct AIPlayer {
    pub player: Player,
    pub difficulty: AIDifficulty,
    pub strategy: AIStrategy,
    pub decision_timer: u32,
    pub last_action: u32,
    pub targets: Vec<AITarget>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AIDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AIStrategy {
    Aggressive,
    Conservative,
    Balanced,
    Specialist { focus: CargoType },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AITarget {
    pub target_type: AITargetType,
    pub priority: u32,
    pub estimated_profit: i64,
    pub location: (usize, usize),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AITargetType {
    BuildRoute { from: (usize, usize), to: (usize, usize), cargo: CargoType },
    ExpandExistingRoute { route_id: u32 },
    BuyVehicle { vehicle_type: VehicleType },
    BuildStation { location: (usize, usize) },
}

impl AIPlayer {
    pub fn new(name: String, difficulty: AIDifficulty, strategy: AIStrategy) -> Self {
        let starting_money = match difficulty {
            AIDifficulty::Easy => 500000,
            AIDifficulty::Medium => 800000,
            AIDifficulty::Hard => 1200000,
        };

        Self {
            player: Player::new(name, starting_money),
            difficulty,
            strategy,
            decision_timer: 0,
            last_action: 0,
            targets: Vec::new(),
        }
    }

    pub fn update(&mut self, world: &mut World, economy: &mut Economy) {
        self.player.update(world, economy);
        self.decision_timer += 1;

        let decision_frequency = match self.difficulty {
            AIDifficulty::Easy => 300,
            AIDifficulty::Medium => 200, 
            AIDifficulty::Hard => 100,
        };

        if self.decision_timer >= decision_frequency {
            self.make_decision(world, economy);
            self.decision_timer = 0;
        }
    }

    fn make_decision(&mut self, world: &mut World, economy: &Economy) {
        self.analyze_opportunities(world, economy);

        if let Some(target) = self.select_best_target() {
            self.execute_target(target, world, economy);
        }
    }

    fn analyze_opportunities(&mut self, world: &World, economy: &Economy) {
        self.targets.clear();

        self.find_profitable_routes(world, economy);
        self.analyze_existing_routes(economy);
        self.consider_vehicle_purchases(world, economy);
        self.identify_station_locations(world, economy);

        self.targets.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    fn find_profitable_routes(&mut self, world: &World, economy: &Economy) {
        let _rng = rand::thread_rng();

        for i in 0..world.towns.len() {
            for j in i + 1..world.towns.len() {
                let from = world.towns[i];
                let to = world.towns[j];
                let distance = self.calculate_distance(from, to);

                if distance > 5.0 && distance < 50.0 {
                    let cargo_types = vec![CargoType::Passengers, CargoType::Mail];
                    
                    for cargo_type in cargo_types {
                        let estimated_profit = self.estimate_route_profit(&cargo_type, distance, economy);
                        
                        if estimated_profit > 50000 {
                            let priority = self.calculate_target_priority(estimated_profit, &cargo_type);
                            
                            self.targets.push(AITarget {
                                target_type: AITargetType::BuildRoute { from, to, cargo: cargo_type },
                                priority,
                                estimated_profit,
                                location: from,
                            });
                        }
                    }
                }
            }
        }

        for i in 0..world.industries.len() {
            for j in 0..world.towns.len() {
                let from = world.industries[i];
                let to = world.towns[j];
                let distance = self.calculate_distance(from, to);

                if distance > 3.0 && distance < 40.0 {
                    let cargo_types = vec![CargoType::Goods, CargoType::Coal, CargoType::Steel];
                    
                    for cargo_type in cargo_types {
                        let estimated_profit = self.estimate_route_profit(&cargo_type, distance, economy);
                        
                        if estimated_profit > 30000 {
                            let priority = self.calculate_target_priority(estimated_profit, &cargo_type);
                            
                            self.targets.push(AITarget {
                                target_type: AITargetType::BuildRoute { from, to, cargo: cargo_type },
                                priority,
                                estimated_profit,
                                location: from,
                            });
                        }
                    }
                }
            }
        }
    }

    fn analyze_existing_routes(&mut self, _economy: &Economy) {
        for route in &self.player.routes {
            if route.profit > 100000 && route.vehicle_ids.len() < 3 {
                let estimated_profit = route.profit * 2;
                let priority = (estimated_profit / 10000) as u32;

                self.targets.push(AITarget {
                    target_type: AITargetType::ExpandExistingRoute { route_id: route.id },
                    priority,
                    estimated_profit,
                    location: route.stations[0],
                });
            }
        }
    }

    fn consider_vehicle_purchases(&mut self, _world: &World, economy: &Economy) {
        if self.player.money < 200000 {
            return;
        }

        let vehicle_types = match &self.strategy {
            AIStrategy::Aggressive => vec![
                VehicleType::Train {
                    engine: TrainEngine::Diesel { power: 2000, reliability: 85 },
                    cars: vec![TrainCar::Passenger { capacity: 100 }, TrainCar::Freight { capacity: 80, cargo_type: None }],
                },
            ],
            AIStrategy::Conservative => vec![
                VehicleType::Road { truck_type: TruckType::Bus { capacity: 40 } },
            ],
            AIStrategy::Balanced => vec![
                VehicleType::Train {
                    engine: TrainEngine::Steam { power: 1500, reliability: 80 },
                    cars: vec![TrainCar::Passenger { capacity: 80 }],
                },
                VehicleType::Road { truck_type: TruckType::LargeTruck { capacity: 60 } },
            ],
            AIStrategy::Specialist { focus } => {
                match focus {
                    CargoType::Passengers => vec![
                        VehicleType::Road { truck_type: TruckType::Bus { capacity: 50 } },
                    ],
                    _ => vec![
                        VehicleType::Road { truck_type: TruckType::LargeTruck { capacity: 80 } },
                    ],
                }
            }
        };

        for vehicle_type in vehicle_types {
            let cost = Vehicle::get_purchase_cost(&vehicle_type);
            if self.player.can_afford(cost) {
                let estimated_profit = self.estimate_vehicle_profit(&vehicle_type, economy);
                let priority = (estimated_profit / cost * 100) as u32;

                self.targets.push(AITarget {
                    target_type: AITargetType::BuyVehicle { vehicle_type },
                    priority,
                    estimated_profit,
                    location: (0, 0),
                });
            }
        }
    }

    fn identify_station_locations(&mut self, world: &World, _economy: &Economy) {
        let mut rng = rand::thread_rng();
        
        for _ in 0..5 {
            let x = rng.gen_range(0..world.width);
            let y = rng.gen_range(0..world.height);
            
            if let Some(tile) = world.get_tile(x, y) {
                if matches!(tile.content, crate::world::TileContent::Empty) {
                    let nearby_value = self.calculate_location_value(x, y, world);
                    
                    if nearby_value > 50000 {
                        self.targets.push(AITarget {
                            target_type: AITargetType::BuildStation { location: (x, y) },
                            priority: (nearby_value / 1000) as u32,
                            estimated_profit: nearby_value,
                            location: (x, y),
                        });
                    }
                }
            }
        }
    }

    fn select_best_target(&mut self) -> Option<AITarget> {
        if self.targets.is_empty() {
            return None;
        }

        let filtered_targets: Vec<_> = self.targets.iter()
            .filter(|target| self.can_afford_target(target))
            .cloned()
            .collect();

        if filtered_targets.is_empty() {
            return None;
        }

        let best_target = match &self.strategy {
            AIStrategy::Aggressive => {
                filtered_targets.into_iter()
                    .max_by_key(|target| {
                        match target.target_type {
                            AITargetType::BuyVehicle { .. } => target.priority + 20,
                            AITargetType::BuildRoute { .. } => target.priority + 15,
                            _ => target.priority,
                        }
                    })
            },
            AIStrategy::Conservative => {
                filtered_targets.into_iter()
                    .max_by_key(|target| {
                        match target.target_type {
                            AITargetType::ExpandExistingRoute { .. } => target.priority + 25,
                            _ => target.priority,
                        }
                    })
            },
            AIStrategy::Balanced => {
                filtered_targets.into_iter()
                    .max_by_key(|target| target.priority)
            },
            AIStrategy::Specialist { focus } => {
                filtered_targets.into_iter()
                    .max_by_key(|target| {
                        match &target.target_type {
                            AITargetType::BuildRoute { cargo, .. } if cargo == focus => target.priority + 30,
                            _ => target.priority,
                        }
                    })
            },
        };

        best_target
    }

    fn execute_target(&mut self, target: AITarget, _world: &mut World, _economy: &Economy) {
        match target.target_type {
            AITargetType::BuildRoute { from, to, cargo } => {
                let route_id = self.player.create_route(
                    format!("AI Route {}", self.player.routes.len()),
                    vec![from, to],
                    vec![cargo],
                );
                
                if let Some(vehicle_id) = self.buy_suitable_vehicle(from, &cargo) {
                    self.player.assign_vehicle_to_route(vehicle_id, route_id);
                }
            },
            AITargetType::ExpandExistingRoute { route_id } => {
                let (start_location, cargo_type) = if let Some(route) = self.player.routes.iter().find(|r| r.id == route_id) {
                    (route.stations[0], route.cargo_types[0])
                } else {
                    return;
                };
                
                if let Some(vehicle_id) = self.buy_suitable_vehicle(start_location, &cargo_type) {
                    self.player.assign_vehicle_to_route(vehicle_id, route_id);
                }
            },
            AITargetType::BuyVehicle { vehicle_type } => {
                let location = if !self.player.stations.is_empty() {
                    self.player.stations[0]
                } else {
                    (0, 0)
                };
                self.player.add_vehicle(vehicle_type, location.0, location.1);
            },
            AITargetType::BuildStation { location } => {
                self.player.stations.push(location);
            },
        }

        self.last_action = self.player.game_time;
    }

    fn buy_suitable_vehicle(&mut self, location: (usize, usize), cargo_type: &CargoType) -> Option<u32> {
        let vehicle_type = match cargo_type {
            CargoType::Passengers => VehicleType::Road { truck_type: TruckType::Bus { capacity: 50 } },
            CargoType::Mail => VehicleType::Road { truck_type: TruckType::SmallTruck { capacity: 30 } },
            _ => VehicleType::Road { truck_type: TruckType::LargeTruck { capacity: 80 } },
        };

        self.player.add_vehicle(vehicle_type, location.0, location.1)
    }

    fn calculate_distance(&self, from: (usize, usize), to: (usize, usize)) -> f32 {
        let dx = to.0 as f32 - from.0 as f32;
        let dy = to.1 as f32 - from.1 as f32;
        (dx * dx + dy * dy).sqrt()
    }

    fn estimate_route_profit(&self, cargo_type: &CargoType, distance: f32, economy: &Economy) -> i64 {
        let price = economy.get_cargo_price(cargo_type, distance);
        let estimated_monthly_cargo = match cargo_type {
            CargoType::Passengers => 200,
            CargoType::Mail => 100,
            _ => 150,
        };
        
        ((price * estimated_monthly_cargo as f32) * 12.0) as i64
    }

    fn estimate_vehicle_profit(&self, vehicle_type: &VehicleType, _economy: &Economy) -> i64 {
        let capacity = match vehicle_type {
            VehicleType::Train { cars, .. } => {
                cars.iter().map(|car| match car {
                    TrainCar::Passenger { capacity } => *capacity,
                    TrainCar::Freight { capacity, .. } => *capacity,
                    TrainCar::Mail { capacity } => *capacity,
                }).sum()
            },
            VehicleType::Road { truck_type } => match truck_type {
                TruckType::SmallTruck { capacity } => *capacity,
                TruckType::LargeTruck { capacity } => *capacity,
                TruckType::Bus { capacity } => *capacity,
            },
            _ => 50,
        };

        (capacity as f32 * 10.0 * 365.0) as i64
    }

    fn calculate_target_priority(&self, estimated_profit: i64, cargo_type: &CargoType) -> u32 {
        let base_priority = (estimated_profit / 1000) as u32;
        
        let strategy_modifier = match (&self.strategy, cargo_type) {
            (AIStrategy::Specialist { focus }, cargo) if focus == cargo => 50,
            (AIStrategy::Aggressive, _) => 20,
            (AIStrategy::Conservative, _) => -10,
            _ => 0,
        };

        (base_priority as i32 + strategy_modifier).max(0) as u32
    }

    fn calculate_location_value(&self, x: usize, y: usize, world: &World) -> i64 {
        let mut value = 0i64;
        let search_radius = 5;

        for dy in -(search_radius as i32)..(search_radius as i32) {
            for dx in -(search_radius as i32)..(search_radius as i32) {
                let check_x = x as i32 + dx;
                let check_y = y as i32 + dy;

                if check_x >= 0 && check_y >= 0 &&
                   (check_x as usize) < world.width && (check_y as usize) < world.height {
                    
                    if let Some(tile) = world.get_tile(check_x as usize, check_y as usize) {
                        match &tile.content {
                            crate::world::TileContent::Town(town) => {
                                value += town.population as i64 * 10;
                            },
                            crate::world::TileContent::Industry(_) => {
                                value += 25000;
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        value
    }

    fn can_afford_target(&self, target: &AITarget) -> bool {
        match &target.target_type {
            AITargetType::BuildRoute { .. } => self.player.money > 100000,
            AITargetType::ExpandExistingRoute { .. } => self.player.money > 150000,
            AITargetType::BuyVehicle { vehicle_type } => {
                self.player.can_afford(Vehicle::get_purchase_cost(vehicle_type))
            },
            AITargetType::BuildStation { .. } => self.player.money > 50000,
        }
    }
}