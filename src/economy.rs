use std::collections::HashMap;
use crate::world::{World, CargoType, TileContent, Town, Industry};

pub struct Economy {
    pub cargo_prices: HashMap<CargoType, f32>,
    pub supply_demand: HashMap<CargoType, SupplyDemand>,
    pub inflation_rate: f32,
    pub economic_state: EconomicState,
    pub month: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SupplyDemand {
    pub supply: u32,
    pub demand: u32,
    pub price_multiplier: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum EconomicState {
    Boom,
    Stable,
    Recession,
}

impl Economy {
    pub fn new() -> Self {
        let mut cargo_prices = HashMap::new();
        cargo_prices.insert(CargoType::Passengers, 5.0);
        cargo_prices.insert(CargoType::Mail, 8.0);
        cargo_prices.insert(CargoType::Coal, 3.0);
        cargo_prices.insert(CargoType::IronOre, 4.0);
        cargo_prices.insert(CargoType::Steel, 12.0);
        cargo_prices.insert(CargoType::Wood, 6.0);
        cargo_prices.insert(CargoType::Oil, 8.0);
        cargo_prices.insert(CargoType::Goods, 15.0);
        cargo_prices.insert(CargoType::Food, 7.0);

        let mut supply_demand = HashMap::new();
        for cargo_type in [
            CargoType::Passengers, CargoType::Mail, CargoType::Coal,
            CargoType::IronOre, CargoType::Steel, CargoType::Wood,
            CargoType::Oil, CargoType::Goods, CargoType::Food,
        ] {
            supply_demand.insert(cargo_type, SupplyDemand {
                supply: 1000,
                demand: 1000,
                price_multiplier: 1.0,
            });
        }

        Self {
            cargo_prices,
            supply_demand,
            inflation_rate: 0.02,
            economic_state: EconomicState::Stable,
            month: 0,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        self.month += 1;
        
        if self.month % 30 == 0 {
            self.update_monthly_economics(world);
        }

        self.update_supply_demand(world);
        self.update_cargo_prices();
        self.update_economic_state();
    }

    fn update_monthly_economics(&mut self, _world: &mut World) {
        let cargo_types: Vec<_> = self.cargo_prices.keys().cloned().collect();
        for cargo_type in cargo_types {
            let base_price = self.cargo_prices[&cargo_type];
            let inflated_price = base_price * (1.0 + self.inflation_rate);
            self.cargo_prices.insert(cargo_type, inflated_price);
        }

        match self.economic_state {
            EconomicState::Boom => {
                for sd in self.supply_demand.values_mut() {
                    sd.demand = (sd.demand as f32 * 1.1) as u32;
                }
            }
            EconomicState::Recession => {
                for sd in self.supply_demand.values_mut() {
                    sd.demand = (sd.demand as f32 * 0.9) as u32;
                }
            }
            EconomicState::Stable => {}
        }
    }

    fn update_supply_demand(&mut self, world: &World) {
        let mut new_supply: HashMap<CargoType, u32> = HashMap::new();
        let mut new_demand: HashMap<CargoType, u32> = HashMap::new();

        for y in 0..world.height {
            for x in 0..world.width {
                if let Some(tile) = world.get_tile(x, y) {
                    match &tile.content {
                        TileContent::Town(town) => {
                            self.process_town_demand_supply(town, &mut new_demand, &mut new_supply);
                        }
                        TileContent::Industry(industry) => {
                            self.process_industry_supply(industry, &mut new_supply);
                        }
                        _ => {}
                    }
                }
            }
        }

        for (cargo_type, supply) in new_supply {
            if let Some(sd) = self.supply_demand.get_mut(&cargo_type) {
                sd.supply = supply;
            }
        }

        for (cargo_type, demand) in new_demand {
            if let Some(sd) = self.supply_demand.get_mut(&cargo_type) {
                sd.demand = demand;
            }
        }
    }

    fn update_cargo_prices(&mut self) {
        for (cargo_type, sd) in &mut self.supply_demand {
            let supply_demand_ratio = if sd.supply > 0 {
                sd.demand as f32 / sd.supply as f32
            } else {
                2.0
            };

            sd.price_multiplier = match supply_demand_ratio {
                x if x > 1.5 => 1.5,
                x if x < 0.5 => 0.5,
                x => x,
            };

            if let Some(base_price) = self.cargo_prices.get(cargo_type) {
                let adjusted_price = base_price * sd.price_multiplier;
                self.cargo_prices.insert(cargo_type.clone(), adjusted_price);
            }
        }
    }

    fn update_economic_state(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if self.month % 120 == 0 {
            self.economic_state = match rng.gen_range(0..10) {
                0..=2 => EconomicState::Boom,
                3..=6 => EconomicState::Stable,
                _ => EconomicState::Recession,
            };
        }
    }

    fn process_town_demand_supply(&self, town: &Town, demand: &mut HashMap<CargoType, u32>, supply: &mut HashMap<CargoType, u32>) {
        let population_factor = (town.population as f32 / 1000.0).max(0.1);
        
        *demand.entry(CargoType::Passengers).or_insert(0) += (population_factor * 50.0) as u32;
        *demand.entry(CargoType::Mail).or_insert(0) += (population_factor * 20.0) as u32;
        *demand.entry(CargoType::Goods).or_insert(0) += (population_factor * 30.0) as u32;
        *demand.entry(CargoType::Food).or_insert(0) += (population_factor * 25.0) as u32;
        
        *supply.entry(CargoType::Passengers).or_insert(0) += (population_factor * 40.0) as u32;
        *supply.entry(CargoType::Mail).or_insert(0) += (population_factor * 15.0) as u32;
    }

    fn process_industry_supply(&self, industry: &Industry, supply: &mut HashMap<CargoType, u32>) {
        for cargo_type in &industry.cargo_output {
            *supply.entry(cargo_type.clone()).or_insert(0) += industry.production_rate;
        }
    }

    pub fn get_cargo_price(&self, cargo_type: &CargoType, distance: f32) -> f32 {
        let base_price = self.cargo_prices.get(cargo_type).unwrap_or(&10.0);
        let distance_multiplier = 1.0 + (distance / 100.0).min(0.5);
        let economic_multiplier = match self.economic_state {
            EconomicState::Boom => 1.2,
            EconomicState::Stable => 1.0,
            EconomicState::Recession => 0.8,
        };
        
        base_price * distance_multiplier * economic_multiplier
    }

    pub fn calculate_delivery_payment(&self, cargo_type: &CargoType, quantity: u32, distance: f32) -> i64 {
        let price_per_unit = self.get_cargo_price(cargo_type, distance);
        (price_per_unit * quantity as f32) as i64
    }

    pub fn get_market_info(&self, cargo_type: &CargoType) -> MarketInfo {
        let sd = self.supply_demand.get(cargo_type)
            .unwrap_or(&SupplyDemand { supply: 0, demand: 0, price_multiplier: 1.0 });
        let price = self.cargo_prices.get(cargo_type).unwrap_or(&0.0);

        MarketInfo {
            cargo_type: cargo_type.clone(),
            current_price: *price,
            supply: sd.supply,
            demand: sd.demand,
            price_trend: self.calculate_price_trend(sd),
        }
    }

    fn calculate_price_trend(&self, sd: &SupplyDemand) -> PriceTrend {
        match sd.price_multiplier {
            x if x > 1.2 => PriceTrend::Rising,
            x if x < 0.8 => PriceTrend::Falling,
            _ => PriceTrend::Stable,
        }
    }

    pub fn get_economic_report(&self) -> EconomicReport {
        EconomicReport {
            economic_state: self.economic_state.clone(),
            inflation_rate: self.inflation_rate,
            month: self.month,
            top_commodities: self.get_top_commodities(),
        }
    }

    fn get_top_commodities(&self) -> Vec<(CargoType, f32)> {
        let mut commodities: Vec<_> = self.cargo_prices.iter()
            .map(|(cargo, price)| (cargo.clone(), *price))
            .collect();
        
        commodities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        commodities.into_iter().take(5).collect()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketInfo {
    pub cargo_type: CargoType,
    pub current_price: f32,
    pub supply: u32,
    pub demand: u32,
    pub price_trend: PriceTrend,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PriceTrend {
    Rising,
    Stable,
    Falling,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EconomicReport {
    pub economic_state: EconomicState,
    pub inflation_rate: f32,
    pub month: u32,
    pub top_commodities: Vec<(CargoType, f32)>,
}