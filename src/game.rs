use std::time::{Duration, Instant};

pub struct Game {
    pub world: crate::world::World,
    pub ui: crate::ui::UI,
    pub economy: crate::economy::Economy,
    pub player: crate::player::Player,
    pub ai_players: Vec<crate::ai::AIPlayer>,
    pub running: bool,
    last_update: Instant,
    tick_rate: Duration,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: crate::world::World::new(80, 40),
            ui: crate::ui::UI::new(),
            economy: crate::economy::Economy::new(),
            player: crate::player::Player::new("Player".to_string(), 1000000),
            ai_players: Vec::new(),
            running: true,
            last_update: Instant::now(),
            tick_rate: Duration::from_millis(100),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.setup()?;
        
        while self.running {
            self.handle_input()?;
            
            if self.last_update.elapsed() >= self.tick_rate {
                self.update();
                self.last_update = Instant::now();
            }
            
            self.render()?;
            std::thread::sleep(Duration::from_millis(16));
        }
        
        self.ui.cleanup()?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(input) = self.ui.get_input()? {
            match input {
                crate::ui::InputEvent::Quit => self.running = false,
                crate::ui::InputEvent::Move(direction) => {
                    self.ui.move_cursor(direction);
                }
                crate::ui::InputEvent::Select => {
                    if let Some(build_action) = self.ui.get_build_mode() {
                        self.handle_build_action(build_action);
                    } else if let Some((vehicle_id, order)) = self.ui.get_vehicle_order_mode() {
                        self.handle_vehicle_order_select(vehicle_id, order);
                    } else if let Some((vehicle_id, waypoints)) = self.ui.get_route_creation_mode() {
                        self.handle_route_creation_select(vehicle_id);
                    } else {
                        self.handle_selection();
                    }
                }
                crate::ui::InputEvent::Save => {
                    if let Err(e) = crate::save::save_game(self, "savegame.json") {
                        eprintln!("Failed to save game: {}", e);
                    }
                }
                crate::ui::InputEvent::Load => {
                    match crate::save::load_game("savegame.json") {
                        Ok(loaded_game) => {
                            self.world = loaded_game.world;
                            self.economy = loaded_game.economy;
                            self.player = loaded_game.player;
                            self.ai_players = loaded_game.ai_players;
                        }
                        Err(e) => {
                            eprintln!("Failed to load game: {}", e);
                        }
                    }
                }
                crate::ui::InputEvent::Pause => {
                    // Toggle pause - this would need to be implemented in the game logic
                }
                crate::ui::InputEvent::BuildMenu => {
                    self.ui.show_build_menu = !self.ui.show_build_menu;
                }
                crate::ui::InputEvent::ShowControls => {
                    self.ui.show_controls = !self.ui.show_controls;
                }
                crate::ui::InputEvent::BuildAction(build_action) => {
                    self.ui.set_build_mode(Some(build_action));
                }
                crate::ui::InputEvent::VehicleOrder(vehicle_order) => {
                    if let Some(vehicle_id) = self.ui.get_selected_vehicle_id() {
                        self.handle_vehicle_order(vehicle_id, vehicle_order);
                    }
                }
                crate::ui::InputEvent::FinishRouteCreation => {
                    self.finish_route_creation();
                }
            }
        }
        Ok(())
    }

    fn update(&mut self) {
        self.world.update();
        self.economy.update(&mut self.world);
        self.player.update(&mut self.world, &mut self.economy);
        
        for ai_player in &mut self.ai_players {
            ai_player.update(&mut self.world, &mut self.economy);
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.render(&self.world, &self.economy, &self.player)?;
        Ok(())
    }

    fn handle_selection(&mut self) {
        let cursor_pos = self.ui.get_cursor_position();
        let (x, y) = cursor_pos;
        
        // Check if there's a vehicle at this position
        for vehicle in &self.player.vehicles {
            if vehicle.x == x && vehicle.y == y {
                self.ui.show_vehicle_menu(vehicle.id);
                return;
            }
        }
        
        // If no vehicle, show tile info
        if let Some(tile) = self.world.get_tile(x, y) {
            self.ui.set_selected_tile(Some(tile.clone()));
        }
    }

    fn handle_build_action(&mut self, build_action: crate::ui::BuildAction) {
        let cursor_pos = self.ui.get_cursor_position();
        let (x, y) = cursor_pos;

        let build_result = match build_action {
            crate::ui::BuildAction::BuildRailTrack => self.build_rail_track(x, y),
            crate::ui::BuildAction::BuildTrainStation => self.build_train_station(x, y),
            crate::ui::BuildAction::BuildRoad => self.build_road(x, y),
            crate::ui::BuildAction::BuildBusStop => self.build_bus_stop(x, y),
            crate::ui::BuildAction::BuyVehicle => self.buy_vehicle_at_location(x, y),
        };

        if build_result {
            // Build successful, exit build mode
            self.ui.set_build_mode(None);
        }
        // If build failed, stay in build mode so player can try again
    }

    fn build_rail_track(&mut self, x: usize, y: usize) -> bool {
        let cost = 10000;
        if !self.player.can_afford(cost) {
            return false;
        }

        if let Some(tile) = self.world.get_tile(x, y) {
            // Check if tile is suitable for tracks (not water, not already has infrastructure)
            if matches!(tile.terrain, crate::world::TerrainType::Water) {
                return false;
            }
            if !matches!(tile.content, crate::world::TileContent::Empty) {
                return false;
            }
        } else {
            return false;
        }

        // Build the track
        self.player.spend_money(cost);
        self.world.set_tile_content(x, y, crate::world::TileContent::Track(
            crate::world::TrackType::Straight { horizontal: true }
        ));
        true
    }

    fn build_train_station(&mut self, x: usize, y: usize) -> bool {
        let cost = 50000;
        if !self.player.can_afford(cost) {
            return false;
        }

        if let Some(tile) = self.world.get_tile(x, y) {
            if matches!(tile.terrain, crate::world::TerrainType::Water) {
                return false;
            }
            if !matches!(tile.content, crate::world::TileContent::Empty) {
                return false;
            }
        } else {
            return false;
        }

        self.player.spend_money(cost);
        let station = crate::world::Station {
            name: format!("Station {}", self.world.stations.len() + 1),
            station_type: crate::world::StationType::Train,
            cargo_waiting: std::collections::HashMap::new(),
            connections: Vec::new(),
        };
        
        self.world.set_tile_content(x, y, crate::world::TileContent::Station(station));
        self.world.stations.push((x, y));
        self.player.stations.push((x, y));
        true
    }

    fn build_road(&mut self, x: usize, y: usize) -> bool {
        let cost = 5000;
        if !self.player.can_afford(cost) {
            return false;
        }

        if let Some(tile) = self.world.get_tile(x, y) {
            if matches!(tile.terrain, crate::world::TerrainType::Water) {
                return false;
            }
            if !matches!(tile.content, crate::world::TileContent::Empty) {
                return false;
            }
        } else {
            return false;
        }

        self.player.spend_money(cost);
        self.world.set_tile_content(x, y, crate::world::TileContent::Road);
        true
    }

    fn build_bus_stop(&mut self, x: usize, y: usize) -> bool {
        let cost = 25000;
        if !self.player.can_afford(cost) {
            return false;
        }

        if let Some(tile) = self.world.get_tile(x, y) {
            if matches!(tile.terrain, crate::world::TerrainType::Water) {
                return false;
            }
            if !matches!(tile.content, crate::world::TileContent::Empty) {
                return false;
            }
        } else {
            return false;
        }

        self.player.spend_money(cost);
        let station = crate::world::Station {
            name: format!("Bus Stop {}", self.world.stations.len() + 1),
            station_type: crate::world::StationType::Road,
            cargo_waiting: std::collections::HashMap::new(),
            connections: Vec::new(),
        };
        
        self.world.set_tile_content(x, y, crate::world::TileContent::Station(station));
        self.world.stations.push((x, y));
        self.player.stations.push((x, y));
        true
    }

    fn buy_vehicle_at_location(&mut self, x: usize, y: usize) -> bool {
        // For now, just buy a basic bus
        let vehicle_type = crate::vehicle::VehicleType::Road {
            truck_type: crate::vehicle::TruckType::Bus { capacity: 40 }
        };
        
        if let Some(_vehicle_id) = self.player.add_vehicle(vehicle_type, x, y) {
            true
        } else {
            false // Not enough money
        }
    }

    fn handle_vehicle_order(&mut self, vehicle_id: u32, order: crate::ui::VehicleOrder) {
        match order {
            crate::ui::VehicleOrder::GoToLocation => {
                self.ui.set_vehicle_order_mode(Some((vehicle_id, order)));
            },
            crate::ui::VehicleOrder::CreateRoute => {
                // Start route creation mode
                self.ui.set_route_creation_mode(Some((vehicle_id, Vec::new())));
            },
            crate::ui::VehicleOrder::StartRoute => {
                // For now, start simple movement
                if let Some(vehicle) = self.player.vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                    if !vehicle.route.is_empty() {
                        vehicle.state = crate::vehicle::VehicleState::Moving {
                            from: (vehicle.x, vehicle.y),
                            to: vehicle.route[0],
                            progress: 0.0,
                        };
                    }
                }
            },
            crate::ui::VehicleOrder::Stop => {
                if let Some(vehicle) = self.player.vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                    vehicle.state = crate::vehicle::VehicleState::Idle;
                }
            },
            crate::ui::VehicleOrder::SendToDepot => {
                // For now, just placeholder
                println!("Sending vehicle {} to depot", vehicle_id);
            },
        }
    }

    fn handle_vehicle_order_select(&mut self, vehicle_id: u32, order: crate::ui::VehicleOrder) {
        let cursor_pos = self.ui.get_cursor_position();
        let (x, y) = cursor_pos;

        match order {
            crate::ui::VehicleOrder::GoToLocation => {
                if let Some(vehicle) = self.player.vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                    // Set simple route to destination
                    vehicle.route = vec![(x, y)];
                    vehicle.route_index = 0;
                    vehicle.state = crate::vehicle::VehicleState::Moving {
                        from: (vehicle.x, vehicle.y),
                        to: (x, y),
                        progress: 0.0,
                    };
                }
                // Clear the order mode
                self.ui.set_vehicle_order_mode(None);
            },
            _ => {
                // Other order modes not implemented for select yet
                self.ui.set_vehicle_order_mode(None);
            }
        }
    }

    fn handle_route_creation_select(&mut self, vehicle_id: u32) {
        let cursor_pos = self.ui.get_cursor_position();
        let (x, y) = cursor_pos;

        // Check if this is a station
        if let Some(tile) = self.world.get_tile(x, y) {
            match &tile.content {
                crate::world::TileContent::Station(_) => {
                    // Add waypoint to route
                    if self.ui.add_waypoint_to_route(x, y) {
                        // Waypoint added successfully
                    }
                },
                _ => {
                    // Not a station, ignore
                }
            }
        }
    }

    fn finish_route_creation(&mut self) {
        if let Some((vehicle_id, waypoints)) = self.ui.get_route_creation_mode() {
            if waypoints.len() >= 2 {
                // Assign route to vehicle
                if let Some(vehicle) = self.player.vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                    vehicle.assign_route(waypoints);
                }
            }
            // Clear route creation mode
            self.ui.set_route_creation_mode(None);
        }
    }
}