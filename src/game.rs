use std::time::{Duration, Instant};

pub struct Game {
    pub world: crate::world::World,
    pub ui: Option<crate::ui::UI>, // Optional for headless mode
    pub economy: crate::economy::Economy,
    pub player: crate::player::Player,
    pub ai_players: Vec<crate::ai::AIPlayer>,
    pub running: bool,
    last_update: Instant,
    tick_rate: Duration,
    // Server-side UI state
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub camera_x: usize,
    pub camera_y: usize,
    pub selected_tile: Option<crate::world::Tile>,
    pub show_build_menu: bool,
    pub show_vehicle_menu: bool,
    pub selected_vehicle_id: Option<u32>,
    pub build_mode: Option<crate::ui::BuildAction>,
    pub vehicle_order_mode: Option<(u32, crate::ui::VehicleOrder)>,
    pub route_creation_mode: Option<(u32, Vec<(usize, usize)>)>,
    pub notifications: Vec<String>,
    pub notification_timer: u32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: crate::world::World::new(80, 40),
            ui: Some(crate::ui::UI::new()),
            economy: crate::economy::Economy::new(),
            player: crate::player::Player::new("Player".to_string(), 1000000),
            ai_players: Vec::new(),
            running: true,
            last_update: Instant::now(),
            tick_rate: Duration::from_millis(100),
            cursor_x: 0,
            cursor_y: 0,
            camera_x: 0,
            camera_y: 0,
            selected_tile: None,
            show_build_menu: false,
            show_vehicle_menu: false,
            selected_vehicle_id: None,
            build_mode: None,
            vehicle_order_mode: None,
            route_creation_mode: None,
            notifications: Vec::new(),
            notification_timer: 0,
        }
    }

    pub fn new_headless() -> Self {
        Self {
            world: crate::world::World::new(80, 40),
            ui: None, // No UI in headless mode
            economy: crate::economy::Economy::new(),
            player: crate::player::Player::new("Player".to_string(), 1000000),
            ai_players: Vec::new(),
            running: true,
            last_update: Instant::now(),
            tick_rate: Duration::from_millis(100),
            cursor_x: 0,
            cursor_y: 0,
            camera_x: 0,
            camera_y: 0,
            selected_tile: None,
            show_build_menu: false,
            show_vehicle_menu: false,
            selected_vehicle_id: None,
            build_mode: None,
            vehicle_order_mode: None,
            route_creation_mode: None,
            notifications: Vec::new(),
            notification_timer: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut ui) = self.ui {
            ui.setup()?;
        }
        
        while self.running {
            self.handle_input()?;
            
            if self.last_update.elapsed() >= self.tick_rate {
                self.update();
                self.last_update = Instant::now();
            }
            
            self.render()?;
            std::thread::sleep(Duration::from_millis(16));
        }
        
        if let Some(ref mut ui) = self.ui {
            ui.cleanup()?;
        }
        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut ui) = self.ui {
            if let Some(input) = ui.get_input()? {
                let cursor_pos = ui.get_cursor_position();
                self.process_input_event(input, cursor_pos);
            }
        }
        // In headless mode, input comes from server API calls
        Ok(())
    }

    // New method for processing input events from both UI and server
    pub fn process_input_event(&mut self, input: crate::ui::InputEvent, cursor_pos: (usize, usize)) {
        // Update cursor position (for server-side state management)
        self.cursor_x = cursor_pos.0;
        self.cursor_y = cursor_pos.1;
        
        match input {
            crate::ui::InputEvent::Quit => self.running = false,
            crate::ui::InputEvent::Move(direction) => {
                self.move_cursor(direction);
                // Also update UI if present
                if let Some(ref mut ui) = self.ui {
                    ui.move_cursor(direction);
                }
            }
            crate::ui::InputEvent::Select => {
                if let Some(build_action) = self.build_mode {
                    self.handle_build_action(build_action);
                } else if let Some((vehicle_id, order)) = self.vehicle_order_mode {
                    self.handle_vehicle_order_select(vehicle_id, order);
                } else if let Some((vehicle_id, _waypoints)) = &self.route_creation_mode {
                    self.handle_route_creation_select(*vehicle_id);
                } else {
                    self.handle_selection();
                }
            }
            crate::ui::InputEvent::Save => {
                if let Err(e) = crate::save::save_game(self, "savegame.json") {
                    self.add_notification(format!("Failed to save game: {}", e));
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
                        self.add_notification(format!("Failed to load game: {}", e));
                    }
                }
            }
            crate::ui::InputEvent::Pause => {
                // Toggle pause - this would need to be implemented in the game logic
            }
            crate::ui::InputEvent::BuildMenu => {
                self.show_build_menu = !self.show_build_menu;
                if let Some(ref mut ui) = self.ui {
                    ui.show_build_menu = self.show_build_menu;
                }
            }
            crate::ui::InputEvent::ShowControls => {
                if let Some(ref mut ui) = self.ui {
                    ui.show_controls = !ui.show_controls;
                }
            }
            crate::ui::InputEvent::BuildAction(build_action) => {
                self.build_mode = Some(build_action);
                if let Some(ref mut ui) = self.ui {
                    ui.set_build_mode(Some(build_action));
                }
            }
            crate::ui::InputEvent::VehicleOrder(vehicle_order) => {
                if let Some(vehicle_id) = self.selected_vehicle_id {
                    self.handle_vehicle_order(vehicle_id, vehicle_order);
                }
            }
            crate::ui::InputEvent::FinishRouteCreation => {
                self.finish_route_creation();
            }
            crate::ui::InputEvent::VehiclePurchase(vehicle_type) => {
                self.handle_vehicle_purchase(vehicle_type, self.cursor_x, self.cursor_y);
            }
        }
    }

    // Server-side cursor movement
    pub fn move_cursor(&mut self, direction: crate::ui::CursorDirection) {
        match direction {
            crate::ui::CursorDirection::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    if self.cursor_y < self.camera_y {
                        self.camera_y = self.cursor_y;
                    }
                }
            }
            crate::ui::CursorDirection::Down => {
                if self.cursor_y < self.world.height - 1 {
                    self.cursor_y += 1;
                    let view_height = 30; // Default view height
                    if self.cursor_y >= self.camera_y + view_height {
                        self.camera_y = self.cursor_y - view_height + 1;
                    }
                }
            }
            crate::ui::CursorDirection::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    if self.cursor_x < self.camera_x {
                        self.camera_x = self.cursor_x;
                    }
                }
            }
            crate::ui::CursorDirection::Right => {
                if self.cursor_x < self.world.width - 1 {
                    self.cursor_x += 1;
                    let view_width = 60; // Default view width
                    if self.cursor_x >= self.camera_x + view_width {
                        self.camera_x = self.cursor_x - view_width + 1;
                    }
                }
            }
        }
    }

    // Server-side notification management
    pub fn add_notification(&mut self, message: String) {
        self.notifications.push(message.clone());
        self.notification_timer = 300; // Show for 5 seconds at 60 FPS
        
        // Keep only the last 5 notifications
        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
        
        // Also add to UI if present
        if let Some(ref mut ui) = self.ui {
            ui.add_notification(message);
        }
    }

    pub fn update_notifications(&mut self) {
        if self.notification_timer > 0 {
            self.notification_timer -= 1;
            if self.notification_timer == 0 {
                self.notifications.clear();
            }
        }
    }

    pub fn update(&mut self) {
        self.world.update();
        self.economy.update(&mut self.world);
        self.player.update(&mut self.world, &mut self.economy);
        
        for ai_player in &mut self.ai_players {
            ai_player.update(&mut self.world, &mut self.economy);
        }
        
        // Update notifications timer
        self.update_notifications();
    }

    // Create render data for sending to clients
    pub fn get_render_data(&self) -> crate::server::GameRenderData {
        crate::server::GameRenderData {
            world_data: self.create_world_render_data(),
            player_data: self.create_player_render_data(),
            ui_state: self.create_ui_render_state(),
            notifications: self.notifications.clone(),
            game_time: self.player.game_time,
        }
    }

    fn create_world_render_data(&self) -> crate::server::WorldRenderData {
        let mut tiles = Vec::new();
        
        // Only send visible tiles (around camera position)
        let view_width = 60;
        let view_height = 30;
        
        for y in self.camera_y..std::cmp::min(self.camera_y + view_height, self.world.height) {
            let mut row = Vec::new();
            for x in self.camera_x..std::cmp::min(self.camera_x + view_width, self.world.width) {
                if let Some(tile) = self.world.get_tile(x, y) {
                    row.push(crate::server::TileRenderData {
                        x,
                        y,
                        terrain: tile.terrain.clone(),
                        content: self.tile_content_to_render_data(&tile.content),
                        ascii_char: self.world.get_ascii_char_with_vehicles(x, y, &self.player.vehicles),
                        style_color: self.get_tile_style_color(x, y),
                    });
                }
            }
            tiles.push(row);
        }

        let vehicles = self.player.vehicles.iter().map(|v| {
            crate::server::VehicleRenderData {
                id: v.id,
                x: v.x,
                y: v.y,
                vehicle_type: self.vehicle_type_to_string(&v.vehicle_type),
                state: self.vehicle_state_to_string(&v.state),
                cargo: v.cargo.iter().map(|(cargo_type, &amount)| {
                    (format!("{:?}", cargo_type), amount)
                }).collect(),
                ascii_char: crate::world::World::get_vehicle_char(&v.vehicle_type),
                style_color: self.get_vehicle_style_color(&v.vehicle_type),
            }
        }).collect();

        crate::server::WorldRenderData {
            width: self.world.width,
            height: self.world.height,
            tiles,
            vehicles,
        }
    }

    fn create_player_render_data(&self) -> crate::server::PlayerRenderData {
        crate::server::PlayerRenderData {
            name: self.player.name.clone(),
            money: self.player.money,
            vehicle_count: self.player.vehicles.len(),
            reputation: self.player.reputation,
        }
    }

    fn create_ui_render_state(&self) -> crate::server::UIRenderState {
        let selected_tile_info = if let Some(tile) = &self.selected_tile {
            Some(self.format_tile_info(tile))
        } else {
            None
        };

        crate::server::UIRenderState {
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
            camera_x: self.camera_x,
            camera_y: self.camera_y,
            selected_tile_info,
            show_build_menu: self.show_build_menu,
            show_vehicle_menu: self.show_vehicle_menu,
            selected_vehicle_id: self.selected_vehicle_id,
        }
    }

    // Helper methods for data conversion
    fn tile_content_to_render_data(&self, content: &crate::world::TileContent) -> crate::server::TileContentRenderData {
        match content {
            crate::world::TileContent::Empty => crate::server::TileContentRenderData::Empty,
            crate::world::TileContent::Town(town) => crate::server::TileContentRenderData::Town {
                name: town.name.clone(),
                population: town.population,
            },
            crate::world::TileContent::Industry(industry) => crate::server::TileContentRenderData::Industry {
                industry_type: format!("{:?}", industry.industry_type),
                production_rate: industry.production_rate,
                stockpile: industry.stockpile.iter().map(|(cargo_type, &amount)| {
                    (format!("{:?}", cargo_type), amount)
                }).collect(),
            },
            crate::world::TileContent::Station(station) => crate::server::TileContentRenderData::Station {
                name: station.name.clone(),
                station_type: format!("{:?}", station.station_type),
                cargo_waiting: station.cargo_waiting.iter().map(|(cargo_type, &amount)| {
                    (format!("{:?}", cargo_type), amount)
                }).collect(),
            },
            crate::world::TileContent::Track(track_type) => crate::server::TileContentRenderData::Track {
                track_type: match track_type {
                    crate::world::TrackType::Straight { horizontal } => {
                        if *horizontal { "Horizontal".to_string() } else { "Vertical".to_string() }
                    },
                    crate::world::TrackType::Curve { .. } => "Curve".to_string(),
                    crate::world::TrackType::Junction => "Junction".to_string(),
                }
            },
            crate::world::TileContent::Road => crate::server::TileContentRenderData::Road,
        }
    }

    fn vehicle_type_to_string(&self, vehicle_type: &crate::vehicle::VehicleType) -> String {
        match vehicle_type {
            crate::vehicle::VehicleType::Train { .. } => "Train".to_string(),
            crate::vehicle::VehicleType::Road { truck_type } => match truck_type {
                crate::vehicle::TruckType::Bus { .. } => "Bus".to_string(),
                crate::vehicle::TruckType::SmallTruck { .. } => "Small Truck".to_string(),
                crate::vehicle::TruckType::LargeTruck { .. } => "Large Truck".to_string(),
            },
            crate::vehicle::VehicleType::Ship { .. } => "Ship".to_string(),
            crate::vehicle::VehicleType::Aircraft { .. } => "Aircraft".to_string(),
        }
    }

    fn vehicle_state_to_string(&self, state: &crate::vehicle::VehicleState) -> String {
        match state {
            crate::vehicle::VehicleState::Idle => "Idle".to_string(),
            crate::vehicle::VehicleState::Moving { .. } => "Moving".to_string(),
            crate::vehicle::VehicleState::Loading => "Loading".to_string(),
            crate::vehicle::VehicleState::Unloading => "Unloading".to_string(),
            crate::vehicle::VehicleState::Broken => "Broken".to_string(),
        }
    }

    fn get_tile_style_color(&self, x: usize, y: usize) -> String {
        if let Some(tile) = self.world.get_tile(x, y) {
            match &tile.content {
                crate::world::TileContent::Town(_) => "blue".to_string(),
                crate::world::TileContent::Industry(_) => "red".to_string(),
                crate::world::TileContent::Station(_) => "green".to_string(),
                crate::world::TileContent::Track(_) => "yellow".to_string(),
                crate::world::TileContent::Road => "gray".to_string(),
                _ => match tile.terrain {
                    crate::world::TerrainType::Grass => "lightgreen".to_string(),
                    crate::world::TerrainType::Water => "blue".to_string(),
                    crate::world::TerrainType::Mountain => "white".to_string(),
                    crate::world::TerrainType::Desert => "yellow".to_string(),
                    crate::world::TerrainType::Forest => "darkgreen".to_string(),
                }
            }
        } else {
            "black".to_string()
        }
    }

    fn get_vehicle_style_color(&self, vehicle_type: &crate::vehicle::VehicleType) -> String {
        match vehicle_type {
            crate::vehicle::VehicleType::Train { .. } => "darkblue".to_string(),
            crate::vehicle::VehicleType::Road { .. } => "darkred".to_string(),
            crate::vehicle::VehicleType::Ship { .. } => "cyan".to_string(),
            crate::vehicle::VehicleType::Aircraft { .. } => "magenta".to_string(),
        }
    }

    fn format_tile_info(&self, tile: &crate::world::Tile) -> String {
        match &tile.content {
            crate::world::TileContent::Empty => "Empty".to_string(),
            crate::world::TileContent::Town(town) => {
                format!("Town: {}\nPopulation: {}", town.name, town.population)
            },
            crate::world::TileContent::Industry(industry) => {
                format!("Industry: {:?}\nProduction: {}/month", industry.industry_type, industry.production_rate)
            },
            crate::world::TileContent::Station(station) => {
                format!("Station: {}\nType: {:?}", station.name, station.station_type)
            },
            crate::world::TileContent::Track(_) => "Railway Track".to_string(),
            crate::world::TileContent::Road => "Road".to_string(),
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut ui) = self.ui {
            ui.render(&self.world, &self.economy, &self.player)?;
        }
        Ok(())
    }

    fn handle_selection(&mut self) {
        let (x, y) = (self.cursor_x, self.cursor_y);
        
        // Check if there's a vehicle at this position
        for vehicle in &self.player.vehicles {
            if vehicle.x == x && vehicle.y == y {
                self.show_vehicle_menu = true;
                self.selected_vehicle_id = Some(vehicle.id);
                // Also update UI if present
                if let Some(ref mut ui) = self.ui {
                    ui.show_vehicle_menu(vehicle.id);
                }
                return;
            }
        }
        
        // If no vehicle, show tile info
        if let Some(tile) = self.world.get_tile(x, y) {
            self.selected_tile = Some(tile.clone());
            // Also update UI if present
            if let Some(ref mut ui) = self.ui {
                ui.set_selected_tile(Some(tile.clone()));
            }
        }
    }

    fn handle_build_action(&mut self, build_action: crate::ui::BuildAction) {
        let (x, y) = (self.cursor_x, self.cursor_y);

        let build_result = match build_action {
            crate::ui::BuildAction::BuildRailTrack => self.build_rail_track(x, y),
            crate::ui::BuildAction::BuildTrainStation => self.build_train_station(x, y),
            crate::ui::BuildAction::BuildRoad => self.build_road(x, y),
            crate::ui::BuildAction::BuildBusStop => self.build_bus_stop(x, y),
            crate::ui::BuildAction::BuyVehicle => self.buy_vehicle_at_location(x, y),
        };

        if build_result {
            // Build successful, exit build mode
            self.build_mode = None;
            if let Some(ref mut ui) = self.ui {
                ui.set_build_mode(None);
            }
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

    // Server-side route management
    pub fn add_waypoint_to_route(&mut self, x: usize, y: usize) -> bool {
        if let Some((_vehicle_id, ref mut waypoints)) = self.route_creation_mode {
            waypoints.push((x, y));
            // Also update UI if present
            if let Some(ref mut ui) = self.ui {
                ui.add_waypoint_to_route(x, y);
            }
            true
        } else {
            false
        }
    }

    fn handle_vehicle_order(&mut self, vehicle_id: u32, order: crate::ui::VehicleOrder) {
        match order {
            crate::ui::VehicleOrder::GoToLocation => {
                self.vehicle_order_mode = Some((vehicle_id, order));
                if let Some(ref mut ui) = self.ui {
                    ui.set_vehicle_order_mode(Some((vehicle_id, order)));
                }
            },
            crate::ui::VehicleOrder::CreateRoute => {
                // Start route creation mode
                self.route_creation_mode = Some((vehicle_id, Vec::new()));
                if let Some(ref mut ui) = self.ui {
                    ui.set_route_creation_mode(Some((vehicle_id, Vec::new())));
                }
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
                self.add_notification(format!("Sending vehicle {} to depot", vehicle_id));
            },
        }
    }

    fn handle_vehicle_order_select(&mut self, vehicle_id: u32, order: crate::ui::VehicleOrder) {
        let (x, y) = (self.cursor_x, self.cursor_y);

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
                self.vehicle_order_mode = None;
                if let Some(ref mut ui) = self.ui {
                    ui.set_vehicle_order_mode(None);
                }
            },
            _ => {
                // Other order modes not implemented for select yet
                self.vehicle_order_mode = None;
                if let Some(ref mut ui) = self.ui {
                    ui.set_vehicle_order_mode(None);
                }
            }
        }
    }

    fn handle_route_creation_select(&mut self, _vehicle_id: u32) {
        let (x, y) = (self.cursor_x, self.cursor_y);

        // Check if this is a station
        if let Some(tile) = self.world.get_tile(x, y) {
            match &tile.content {
                crate::world::TileContent::Station(_) => {
                    // Add waypoint to route
                    if self.add_waypoint_to_route(x, y) {
                        // Waypoint added successfully - could add audio feedback here
                        self.add_notification(format!("Added waypoint at station ({}, {})", x, y));
                    }
                },
                _ => {
                    // Not a station - provide feedback
                    self.add_notification("Not a station! Move cursor to a station (■) and press SPACE".to_string());
                }
            }
        }
    }

    fn finish_route_creation(&mut self) {
        if let Some((vehicle_id, waypoints)) = self.route_creation_mode.clone() {
            if waypoints.len() >= 2 {
                // Assign route to vehicle
                if let Some(vehicle) = self.player.vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                    vehicle.assign_route(waypoints.clone());
                    self.add_notification(format!("Route created for vehicle {} with {} stations", vehicle_id, waypoints.len()));
                }
            } else {
                self.add_notification(format!("Route needs at least 2 stations! Currently have: {}", waypoints.len()));
            }
            // Clear route creation mode
            self.route_creation_mode = None;
            if let Some(ref mut ui) = self.ui {
                ui.set_route_creation_mode(None);
            }
        }
    }

    fn handle_vehicle_purchase(&mut self, vehicle_type: crate::ui::VehiclePurchaseType, x: usize, y: usize) {
        // Handle auto-selection
        let actual_vehicle_type = if matches!(vehicle_type, crate::ui::VehiclePurchaseType::Auto) {
            let recommended = self.get_recommended_vehicle_type(x, y);
            self.add_notification(format!("Auto-selected {} based on tile at ({}, {})", self.get_vehicle_type_name(recommended), x, y));
            recommended
        } else {
            vehicle_type
        };
        
        // Convert UI enum to actual vehicle type
        let vehicle = self.create_vehicle_from_type(actual_vehicle_type);
        
        if let Some(_vehicle_id) = self.player.add_vehicle(vehicle, x, y) {
            let vehicle_name = self.get_vehicle_type_name(actual_vehicle_type);
            self.add_notification(format!("{} purchased successfully at ({}, {})", vehicle_name, x, y));
        } else {
            self.add_notification("Not enough money to purchase vehicle!".to_string());
        }
    }

    fn get_vehicle_type_name(&self, vehicle_type: crate::ui::VehiclePurchaseType) -> &'static str {
        match vehicle_type {
            crate::ui::VehiclePurchaseType::Train => "Train",
            crate::ui::VehiclePurchaseType::Bus => "Bus", 
            crate::ui::VehiclePurchaseType::SmallTruck => "Small Truck",
            crate::ui::VehiclePurchaseType::LargeTruck => "Large Truck",
            crate::ui::VehiclePurchaseType::Ship => "Ship",
            crate::ui::VehiclePurchaseType::SmallPlane => "Small Plane",
            crate::ui::VehiclePurchaseType::LargePlane => "Large Plane",
            crate::ui::VehiclePurchaseType::Auto => "Auto-selected Vehicle",
        }
    }

    pub fn get_recommended_vehicle_type(&self, x: usize, y: usize) -> crate::ui::VehiclePurchaseType {
        if let Some(tile) = self.world.get_tile(x, y) {
            match &tile.content {
                crate::world::TileContent::Track(_) => crate::ui::VehiclePurchaseType::Train,
                crate::world::TileContent::Road => crate::ui::VehiclePurchaseType::Bus,
                crate::world::TileContent::Station(station) => {
                    match station.station_type {
                        crate::world::StationType::Train => crate::ui::VehiclePurchaseType::Train,
                        crate::world::StationType::Road => crate::ui::VehiclePurchaseType::Bus,
                        crate::world::StationType::Airport => crate::ui::VehiclePurchaseType::SmallPlane,
                        crate::world::StationType::Harbor => crate::ui::VehiclePurchaseType::Ship,
                    }
                },
                _ => {
                    match tile.terrain {
                        crate::world::TerrainType::Water => crate::ui::VehiclePurchaseType::Ship,
                        _ => crate::ui::VehiclePurchaseType::Bus, // Default to bus for general use
                    }
                }
            }
        } else {
            crate::ui::VehiclePurchaseType::Bus // Default fallback
        }
    }

    fn create_vehicle_from_type(&self, purchase_type: crate::ui::VehiclePurchaseType) -> crate::vehicle::VehicleType {
        match purchase_type {
            crate::ui::VehiclePurchaseType::Train => {
                crate::vehicle::VehicleType::Train {
                    engine: crate::vehicle::TrainEngine::Steam { power: 500, reliability: 75 },
                    cars: vec![
                        crate::vehicle::TrainCar::Passenger { capacity: 40 },
                        crate::vehicle::TrainCar::Freight { capacity: 30, cargo_type: None }
                    ]
                }
            },
            crate::ui::VehiclePurchaseType::Bus => {
                crate::vehicle::VehicleType::Road {
                    truck_type: crate::vehicle::TruckType::Bus { capacity: 40 }
                }
            },
            crate::ui::VehiclePurchaseType::SmallTruck => {
                crate::vehicle::VehicleType::Road {
                    truck_type: crate::vehicle::TruckType::SmallTruck { capacity: 20 }
                }
            },
            crate::ui::VehiclePurchaseType::LargeTruck => {
                crate::vehicle::VehicleType::Road {
                    truck_type: crate::vehicle::TruckType::LargeTruck { capacity: 60 }
                }
            },
            crate::ui::VehiclePurchaseType::Ship => {
                crate::vehicle::VehicleType::Ship {
                    ship_type: crate::vehicle::ShipType::CargoShip { capacity: 200 }
                }
            },
            crate::ui::VehiclePurchaseType::SmallPlane => {
                crate::vehicle::VehicleType::Aircraft {
                    plane_type: crate::vehicle::PlaneType::SmallPlane { capacity: 50, range: 1000 }
                }
            },
            crate::ui::VehiclePurchaseType::LargePlane => {
                crate::vehicle::VehicleType::Aircraft {
                    plane_type: crate::vehicle::PlaneType::LargePlane { capacity: 200, range: 5000 }
                }
            },
            crate::ui::VehiclePurchaseType::Auto => {
                // This should not be reached since Auto is handled in handle_vehicle_purchase
                // But provide a fallback
                crate::vehicle::VehicleType::Road {
                    truck_type: crate::vehicle::TruckType::Bus { capacity: 40 }
                }
            },
        }
    }
}