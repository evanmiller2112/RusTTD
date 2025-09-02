use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TerrainType {
    Grass,
    Water,
    Mountain,
    Desert,
    Forest,
}

#[derive(Clone, Debug)]
pub enum TileContent {
    Empty,
    Town(Town),
    Industry(Industry),
    Station(Station),
    Track(TrackType),
    Road,
}

#[derive(Clone, Debug)]
pub enum TrackType {
    Straight { horizontal: bool },
    Curve { from_dir: Direction, to_dir: Direction },
    Junction,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Debug)]
pub struct Town {
    pub name: String,
    pub population: u32,
    pub growth_rate: f32,
    pub cargo_demand: HashMap<CargoType, u32>,
    pub cargo_supply: HashMap<CargoType, u32>,
}

#[derive(Clone, Debug)]
pub struct Industry {
    pub industry_type: IndustryType,
    pub production_rate: u32,
    pub cargo_input: Vec<CargoType>,
    pub cargo_output: Vec<CargoType>,
    pub stockpile: HashMap<CargoType, u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum IndustryType {
    CoalMine,
    IronOreMine,
    SteelMill,
    Factory,
    Farm,
    Sawmill,
    OilRig,
    Refinery,
}

#[derive(Clone, Debug)]
pub struct Station {
    pub name: String,
    pub station_type: StationType,
    pub cargo_waiting: HashMap<CargoType, u32>,
    pub connections: Vec<(usize, usize)>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StationType {
    Train,
    Road,
    Airport,
    Harbor,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CargoType {
    Passengers,
    Mail,
    Coal,
    IronOre,
    Steel,
    Wood,
    Oil,
    Goods,
    Food,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub terrain: TerrainType,
    pub content: TileContent,
    pub height: u8,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub towns: Vec<(usize, usize)>,
    pub industries: Vec<(usize, usize)>,
    pub stations: Vec<(usize, usize)>,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let mut world = Self {
            width,
            height,
            tiles: vec![vec![Tile {
                terrain: TerrainType::Grass,
                content: TileContent::Empty,
                height: 0,
            }; width]; height],
            towns: Vec::new(),
            industries: Vec::new(),
            stations: Vec::new(),
        };
        
        world.generate_terrain();
        world.generate_towns();
        world.generate_industries();
        world
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y)?.get(x)
    }

    pub fn set_tile_content(&mut self, x: usize, y: usize, content: TileContent) {
        if let Some(tile) = self.tiles.get_mut(y).and_then(|row| row.get_mut(x)) {
            tile.content = content;
        }
    }

    pub fn update(&mut self) {
        for (x, y) in &self.towns.clone() {
            if let Some(tile) = self.tiles.get_mut(*y).and_then(|row| row.get_mut(*x)) {
                if let TileContent::Town(ref mut town) = tile.content {
                    town.population = (town.population as f32 * (1.0 + town.growth_rate / 100.0)) as u32;
                }
            }
        }

        let industries = self.industries.clone();
        for (x, y) in industries {
            if let Some(tile) = self.tiles.get_mut(y).and_then(|row| row.get_mut(x)) {
                if let TileContent::Industry(ref mut industry) = tile.content {
                    Self::update_industry_production_static(industry);
                }
            }
        }
    }

    fn generate_terrain(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for y in 0..self.height {
            for x in 0..self.width {
                let terrain = match rng.gen_range(0..100) {
                    0..=60 => TerrainType::Grass,
                    61..=70 => TerrainType::Forest,
                    71..=80 => TerrainType::Water,
                    81..=90 => TerrainType::Mountain,
                    _ => TerrainType::Desert,
                };

                self.tiles[y][x].terrain = terrain;
                self.tiles[y][x].height = rng.gen_range(0..10);
            }
        }
    }

    fn generate_towns(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let town_names = vec![
            "Springfield", "Riverside", "Madison", "Georgetown", "Franklin",
            "Clinton", "Chester", "Marion", "Greenwood", "Fairview"
        ];

        for i in 0..5 {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if matches!(self.tiles[y][x].terrain, TerrainType::Grass | TerrainType::Forest) {
                let town = Town {
                    name: town_names[i % town_names.len()].to_string(),
                    population: rng.gen_range(500..5000),
                    growth_rate: rng.gen_range(0.1..2.0),
                    cargo_demand: HashMap::new(),
                    cargo_supply: HashMap::new(),
                };

                self.tiles[y][x].content = TileContent::Town(town);
                self.towns.push((x, y));
            }
        }
    }

    fn generate_industries(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..8 {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if matches!(self.tiles[y][x].content, TileContent::Empty) {
                let industry_type = match rng.gen_range(0..8) {
                    0 => IndustryType::CoalMine,
                    1 => IndustryType::IronOreMine,
                    2 => IndustryType::SteelMill,
                    3 => IndustryType::Factory,
                    4 => IndustryType::Farm,
                    5 => IndustryType::Sawmill,
                    6 => IndustryType::OilRig,
                    _ => IndustryType::Refinery,
                };

                let (input, output) = self.get_industry_cargo(&industry_type);
                
                let industry = Industry {
                    industry_type,
                    production_rate: rng.gen_range(10..100),
                    cargo_input: input,
                    cargo_output: output,
                    stockpile: HashMap::new(),
                };

                self.tiles[y][x].content = TileContent::Industry(industry);
                self.industries.push((x, y));
            }
        }
    }

    fn get_industry_cargo(&self, industry_type: &IndustryType) -> (Vec<CargoType>, Vec<CargoType>) {
        match industry_type {
            IndustryType::CoalMine => (vec![], vec![CargoType::Coal]),
            IndustryType::IronOreMine => (vec![], vec![CargoType::IronOre]),
            IndustryType::SteelMill => (vec![CargoType::Coal, CargoType::IronOre], vec![CargoType::Steel]),
            IndustryType::Factory => (vec![CargoType::Steel], vec![CargoType::Goods]),
            IndustryType::Farm => (vec![], vec![CargoType::Food]),
            IndustryType::Sawmill => (vec![], vec![CargoType::Wood]),
            IndustryType::OilRig => (vec![], vec![CargoType::Oil]),
            IndustryType::Refinery => (vec![CargoType::Oil], vec![CargoType::Goods]),
        }
    }

    fn update_industry_production_static(industry: &mut Industry) {
        if industry.cargo_input.is_empty() {
            for cargo_type in &industry.cargo_output {
                *industry.stockpile.entry(cargo_type.clone()).or_insert(0) += industry.production_rate;
            }
        }
    }

    pub fn get_ascii_char(&self, x: usize, y: usize) -> char {
        if let Some(tile) = self.get_tile(x, y) {
            match &tile.content {
                TileContent::Town(_) => '◉',
                TileContent::Industry(_) => '▓',
                TileContent::Station(_) => '■',
                TileContent::Track(track_type) => match track_type {
                    TrackType::Straight { horizontal: true } => '─',
                    TrackType::Straight { horizontal: false } => '│',
                    TrackType::Curve { .. } => '┐',
                    TrackType::Junction => '┼',
                },
                TileContent::Road => '.',
                TileContent::Empty => match tile.terrain {
                    TerrainType::Grass => ' ',
                    TerrainType::Water => '~',
                    TerrainType::Mountain => '^',
                    TerrainType::Desert => '∙',
                    TerrainType::Forest => '♠',
                }
            }
        } else {
            ' '
        }
    }

    pub fn get_ascii_char_with_vehicles(&self, x: usize, y: usize, vehicles: &[crate::vehicle::Vehicle]) -> char {
        // First check if there's a vehicle at this position
        for vehicle in vehicles {
            if vehicle.x == x && vehicle.y == y {
                return Self::get_vehicle_char(&vehicle.vehicle_type);
            }
        }
        
        // If no vehicle, show the normal tile content
        self.get_ascii_char(x, y)
    }

    pub fn get_vehicle_char(vehicle_type: &crate::vehicle::VehicleType) -> char {
        match vehicle_type {
            crate::vehicle::VehicleType::Train { .. } => 'T',
            crate::vehicle::VehicleType::Road { truck_type } => match truck_type {
                crate::vehicle::TruckType::Bus { .. } => 'B',
                crate::vehicle::TruckType::SmallTruck { .. } => 't',
                crate::vehicle::TruckType::LargeTruck { .. } => 'T',
            },
            crate::vehicle::VehicleType::Ship { .. } => 'S',
            crate::vehicle::VehicleType::Aircraft { .. } => 'A',
        }
    }
}