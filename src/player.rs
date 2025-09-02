use crate::world::{CargoType, World};
use crate::economy::Economy;
use crate::vehicle::{Vehicle, VehicleType};

pub struct Player {
    pub name: String,
    pub money: i64,
    pub vehicles: Vec<Vehicle>,
    pub stations: Vec<(usize, usize)>,
    pub routes: Vec<Route>,
    pub reputation: f32,
    pub game_time: u32,
}

#[derive(Clone, Debug)]
pub struct Route {
    pub id: u32,
    pub name: String,
    pub stations: Vec<(usize, usize)>,
    pub vehicle_ids: Vec<u32>,
    pub cargo_types: Vec<CargoType>,
    pub profit: i64,
}

impl Player {
    pub fn new(name: String, starting_money: i64) -> Self {
        Self {
            name,
            money: starting_money,
            vehicles: Vec::new(),
            stations: Vec::new(),
            routes: Vec::new(),
            reputation: 50.0,
            game_time: 0,
        }
    }

    pub fn update(&mut self, world: &mut World, economy: &mut Economy) {
        self.game_time += 1;
        
        for vehicle in &mut self.vehicles {
            vehicle.update(world, economy);
            
            if vehicle.has_delivered_cargo() {
                let profit = vehicle.calculate_delivery_profit();
                self.money += profit;
                
                if let Some(route) = self.routes.iter_mut().find(|r| r.vehicle_ids.contains(&vehicle.id)) {
                    route.profit += profit;
                }
            }
        }

        for vehicle in &self.vehicles {
            self.money -= vehicle.get_running_costs() as i64;
        }

        if self.game_time % 30 == 0 {
            self.update_reputation();
        }
    }

    pub fn can_afford(&self, _amount: i64) -> bool {
        true // Infinite money for now
    }

    pub fn spend_money(&mut self, _amount: i64) -> bool {
        // Don't actually spend money - infinite money mode
        true
    }

    pub fn add_vehicle(&mut self, vehicle_type: VehicleType, x: usize, y: usize) -> Option<u32> {
        let cost = Vehicle::get_purchase_cost(&vehicle_type);
        
        if self.spend_money(cost) {
            let vehicle_id = self.vehicles.len() as u32;
            let vehicle = Vehicle::new(vehicle_id, vehicle_type, x, y);
            self.vehicles.push(vehicle);
            Some(vehicle_id)
        } else {
            None
        }
    }

    pub fn create_route(&mut self, name: String, stations: Vec<(usize, usize)>, cargo_types: Vec<CargoType>) -> u32 {
        let route_id = self.routes.len() as u32;
        let route = Route {
            id: route_id,
            name,
            stations,
            vehicle_ids: Vec::new(),
            cargo_types,
            profit: 0,
        };
        
        self.routes.push(route);
        route_id
    }

    pub fn assign_vehicle_to_route(&mut self, vehicle_id: u32, route_id: u32) -> bool {
        if let Some(route) = self.routes.iter_mut().find(|r| r.id == route_id) {
            if !route.vehicle_ids.contains(&vehicle_id) {
                route.vehicle_ids.push(vehicle_id);
                
                if let Some(vehicle) = self.vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                    vehicle.assign_route(route.stations.clone());
                }
                return true;
            }
        }
        false
    }

    pub fn get_total_vehicle_value(&self) -> i64 {
        self.vehicles.iter().map(|v| v.get_current_value()).sum()
    }

    pub fn get_monthly_profit(&self) -> i64 {
        self.routes.iter().map(|r| r.profit).sum()
    }

    pub fn get_monthly_expenses(&self) -> i64 {
        self.vehicles.iter().map(|v| v.get_running_costs() as i64 * 30).sum()
    }

    fn update_reputation(&mut self) {
        let on_time_deliveries = self.vehicles.iter()
            .filter(|v| v.is_on_time())
            .count() as f32;
        
        let total_vehicles = self.vehicles.len() as f32;
        
        if total_vehicles > 0.0 {
            let on_time_ratio = on_time_deliveries / total_vehicles;
            
            if on_time_ratio > 0.8 {
                self.reputation = (self.reputation + 1.0).min(100.0);
            } else if on_time_ratio < 0.5 {
                self.reputation = (self.reputation - 1.0).max(0.0);
            }
        }
    }

    pub fn get_company_stats(&self) -> CompanyStats {
        CompanyStats {
            name: self.name.clone(),
            money: self.money,
            vehicle_count: self.vehicles.len(),
            station_count: self.stations.len(),
            route_count: self.routes.len(),
            reputation: self.reputation,
            total_value: self.money + self.get_total_vehicle_value(),
            monthly_profit: self.get_monthly_profit(),
            monthly_expenses: self.get_monthly_expenses(),
        }
    }
}

pub struct CompanyStats {
    pub name: String,
    pub money: i64,
    pub vehicle_count: usize,
    pub station_count: usize,
    pub route_count: usize,
    pub reputation: f32,
    pub total_value: i64,
    pub monthly_profit: i64,
    pub monthly_expenses: i64,
}