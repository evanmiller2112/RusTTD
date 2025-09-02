use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use crate::world::{Tile, World};
use crate::economy::Economy;
use crate::player::Player;

pub enum InputEvent {
    Quit,
    Move(CursorDirection),
    Select,
    BuildMenu,
    Pause,
    Save,
    Load,
    ShowControls,
    BuildAction(BuildAction),
    VehicleOrder(VehicleOrder),
    FinishRouteCreation,
}

#[derive(Clone, Copy, Debug)]
pub enum BuildAction {
    BuildRailTrack,
    BuildTrainStation,
    BuildRoad,
    BuildBusStop,
    BuyVehicle,
}

#[derive(Clone, Copy, Debug)]
pub enum VehicleOrder {
    GoToLocation,
    CreateRoute,
    StartRoute,
    Stop,
    SendToDepot,
}

#[derive(Clone)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct UI {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    cursor_x: usize,
    cursor_y: usize,
    camera_x: usize,
    camera_y: usize,
    view_width: usize,
    view_height: usize,
    selected_tile: Option<Tile>,
    pub show_build_menu: bool,
    pub show_controls: bool,
    show_vehicle_menu: bool,
    selected_vehicle_id: Option<u32>,
    build_mode: Option<BuildAction>,
    vehicle_order_mode: Option<(u32, VehicleOrder)>,
    route_creation_mode: Option<(u32, Vec<(usize, usize)>)>, // (vehicle_id, waypoints)
    paused: bool,
}

impl UI {
    pub fn new() -> Self {
        Self {
            terminal: None,
            cursor_x: 0,
            cursor_y: 0,
            camera_x: 0,
            camera_y: 0,
            view_width: 60,
            view_height: 30,
            selected_tile: None,
            show_build_menu: false,
            show_controls: false,
            show_vehicle_menu: false,
            selected_vehicle_id: None,
            build_mode: None,
            vehicle_order_mode: None,
            route_creation_mode: None,
            paused: false,
        }
    }

    pub fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        self.terminal = Some(terminal);
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        if let Some(mut terminal) = self.terminal.take() {
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
        }
        Ok(())
    }

    pub fn get_input(&mut self) -> Result<Option<InputEvent>, Box<dyn std::error::Error>> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    return Ok(match key.code {
                        KeyCode::Char('q') => Some(InputEvent::Quit),
                        KeyCode::Up | KeyCode::Char('w') => Some(InputEvent::Move(CursorDirection::Up)),
                        KeyCode::Down | KeyCode::Char('s') => Some(InputEvent::Move(CursorDirection::Down)),
                        KeyCode::Left | KeyCode::Char('a') => Some(InputEvent::Move(CursorDirection::Left)),
                        KeyCode::Right | KeyCode::Char('d') => Some(InputEvent::Move(CursorDirection::Right)),
                        KeyCode::Enter => {
                            if self.route_creation_mode.is_some() {
                                Some(InputEvent::FinishRouteCreation)
                            } else {
                                Some(InputEvent::Select)
                            }
                        },
                        KeyCode::Char(' ') => Some(InputEvent::Select),
                        KeyCode::Char('b') => Some(InputEvent::BuildMenu),
                        KeyCode::Char('p') => Some(InputEvent::Pause),
                        KeyCode::F(5) => Some(InputEvent::Save),
                        KeyCode::F(9) => Some(InputEvent::Load),
                        KeyCode::Char('?') => Some(InputEvent::ShowControls),
                        KeyCode::Esc => {
                            if self.show_controls || self.show_build_menu || self.show_vehicle_menu {
                                self.show_controls = false;
                                self.show_build_menu = false;
                                self.show_vehicle_menu = false;
                                self.build_mode = None;
                                self.vehicle_order_mode = None;
                                self.route_creation_mode = None;
                                None
                            } else if self.build_mode.is_some() || self.vehicle_order_mode.is_some() || self.route_creation_mode.is_some() {
                                self.build_mode = None;
                                self.vehicle_order_mode = None;
                                self.route_creation_mode = None;
                                None
                            } else {
                                None
                            }
                        },
                        // Build menu number keys
                        KeyCode::Char('1') if self.show_build_menu => {
                            self.show_build_menu = false;
                            Some(InputEvent::BuildAction(BuildAction::BuildRailTrack))
                        },
                        KeyCode::Char('2') if self.show_build_menu => {
                            self.show_build_menu = false;
                            Some(InputEvent::BuildAction(BuildAction::BuildTrainStation))
                        },
                        KeyCode::Char('3') if self.show_build_menu => {
                            self.show_build_menu = false;
                            Some(InputEvent::BuildAction(BuildAction::BuildRoad))
                        },
                        KeyCode::Char('4') if self.show_build_menu => {
                            self.show_build_menu = false;
                            Some(InputEvent::BuildAction(BuildAction::BuildBusStop))
                        },
                        KeyCode::Char('5') if self.show_build_menu => {
                            self.show_build_menu = false;
                            Some(InputEvent::BuildAction(BuildAction::BuyVehicle))
                        },
                        // Vehicle menu number keys
                        KeyCode::Char('1') if self.show_vehicle_menu => {
                            self.show_vehicle_menu = false;
                            Some(InputEvent::VehicleOrder(VehicleOrder::GoToLocation))
                        },
                        KeyCode::Char('2') if self.show_vehicle_menu => {
                            self.show_vehicle_menu = false;
                            Some(InputEvent::VehicleOrder(VehicleOrder::CreateRoute))
                        },
                        KeyCode::Char('3') if self.show_vehicle_menu => {
                            self.show_vehicle_menu = false;
                            Some(InputEvent::VehicleOrder(VehicleOrder::StartRoute))
                        },
                        KeyCode::Char('4') if self.show_vehicle_menu => {
                            self.show_vehicle_menu = false;
                            Some(InputEvent::VehicleOrder(VehicleOrder::Stop))
                        },
                        KeyCode::Char('5') if self.show_vehicle_menu => {
                            self.show_vehicle_menu = false;
                            Some(InputEvent::VehicleOrder(VehicleOrder::SendToDepot))
                        },
                        _ => None,
                    });
                }
            }
        }
        Ok(None)
    }

    pub fn move_cursor(&mut self, direction: CursorDirection) {
        match direction {
            CursorDirection::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    if self.cursor_y < self.camera_y {
                        self.camera_y = self.cursor_y;
                    }
                }
            }
            CursorDirection::Down => {
                self.cursor_y += 1;
                if self.cursor_y >= self.camera_y + self.view_height {
                    self.camera_y = self.cursor_y - self.view_height + 1;
                }
            }
            CursorDirection::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    if self.cursor_x < self.camera_x {
                        self.camera_x = self.cursor_x;
                    }
                }
            }
            CursorDirection::Right => {
                self.cursor_x += 1;
                if self.cursor_x >= self.camera_x + self.view_width {
                    self.camera_x = self.cursor_x - self.view_width + 1;
                }
            }
        }
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn get_build_mode(&self) -> Option<BuildAction> {
        self.build_mode
    }

    pub fn set_build_mode(&mut self, mode: Option<BuildAction>) {
        self.build_mode = mode;
    }

    pub fn get_selected_vehicle_id(&self) -> Option<u32> {
        self.selected_vehicle_id
    }

    pub fn set_selected_vehicle(&mut self, vehicle_id: Option<u32>) {
        self.selected_vehicle_id = vehicle_id;
    }

    pub fn show_vehicle_menu(&mut self, vehicle_id: u32) {
        self.selected_vehicle_id = Some(vehicle_id);
        self.show_vehicle_menu = true;
    }

    pub fn get_vehicle_order_mode(&self) -> Option<(u32, VehicleOrder)> {
        self.vehicle_order_mode
    }

    pub fn set_vehicle_order_mode(&mut self, mode: Option<(u32, VehicleOrder)>) {
        self.vehicle_order_mode = mode;
    }

    pub fn get_route_creation_mode(&self) -> Option<(u32, Vec<(usize, usize)>)> {
        self.route_creation_mode.clone()
    }

    pub fn set_route_creation_mode(&mut self, mode: Option<(u32, Vec<(usize, usize)>)>) {
        self.route_creation_mode = mode;
    }

    pub fn add_waypoint_to_route(&mut self, x: usize, y: usize) -> bool {
        if let Some((vehicle_id, ref mut waypoints)) = self.route_creation_mode {
            waypoints.push((x, y));
            true
        } else {
            false
        }
    }

    pub fn set_selected_tile(&mut self, tile: Option<Tile>) {
        self.selected_tile = tile;
    }

    pub fn render(&mut self, world: &World, _economy: &Economy, player: &Player) -> Result<(), Box<dyn std::error::Error>> {
        let cursor_x = self.cursor_x;
        let cursor_y = self.cursor_y;
        let camera_x = self.camera_x;
        let camera_y = self.camera_y;
        let view_width = self.view_width;
        let view_height = self.view_height;
        let selected_tile = self.selected_tile.clone();
        let show_build_menu = self.show_build_menu;
        let show_controls = self.show_controls;
        let show_vehicle_menu = self.show_vehicle_menu;
        let selected_vehicle_id = self.selected_vehicle_id;
        let build_mode = self.build_mode;
        let vehicle_order_mode = self.vehicle_order_mode;
        let route_creation_mode = self.route_creation_mode.clone();
        let paused = self.paused;

        if let Some(ref mut terminal) = self.terminal {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(LayoutDirection::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(5),
                    ])
                    .split(f.size());

                Self::render_header_static(f, chunks[0], player, paused, build_mode, vehicle_order_mode, route_creation_mode);
                Self::render_game_area_static(f, chunks[1], world, &player.vehicles, cursor_x, cursor_y, camera_x, camera_y, view_width, view_height);
                Self::render_info_panel_static(f, chunks[2], cursor_x, cursor_y, &selected_tile, build_mode, &player.vehicles);

                if show_build_menu {
                    Self::render_build_menu_static(f, f.size());
                }
                
                if show_controls {
                    Self::render_controls_popup(f, f.size());
                }

                if show_vehicle_menu {
                    if let Some(vehicle_id) = selected_vehicle_id {
                        Self::render_vehicle_menu_static(f, f.size(), vehicle_id, &player.vehicles);
                    }
                }
            })?;
        }
        Ok(())
    }

    fn render_header_static(f: &mut Frame, area: Rect, player: &Player, paused: bool, build_mode: Option<BuildAction>, vehicle_order_mode: Option<(u32, VehicleOrder)>, route_creation_mode: Option<(u32, Vec<(usize, usize)>)>) {
        let status_text = if let Some(build_action) = build_mode {
            match build_action {
                BuildAction::BuildRailTrack => "BUILD: Railway Track".to_string(),
                BuildAction::BuildTrainStation => "BUILD: Train Station".to_string(),
                BuildAction::BuildRoad => "BUILD: Road".to_string(),
                BuildAction::BuildBusStop => "BUILD: Bus Stop".to_string(),
                BuildAction::BuyVehicle => "BUILD: Buy Vehicle".to_string(),
            }
        } else if let Some((vehicle_id, waypoints)) = route_creation_mode {
            format!("ROUTE: Vehicle {} ({} waypoints) - Click stations, ENTER to finish", vehicle_id, waypoints.len())
        } else if let Some((vehicle_id, order)) = vehicle_order_mode {
            let order_text = match order {
                VehicleOrder::GoToLocation => "Go To Location",
                VehicleOrder::CreateRoute => "Create Route",
                VehicleOrder::StartRoute => "Start Route",
                VehicleOrder::Stop => "Stop",
                VehicleOrder::SendToDepot => "To Depot",
            };
            format!("ORDER: Vehicle {} {}", vehicle_id, order_text)
        } else if paused {
            "PAUSED".to_string()
        } else {
            "".to_string()
        };

        let header = Paragraph::new(format!(
            "RusTTD - {} | Money: ${} | Year: {} | {}",
            player.name,
            player.money,
            1950 + (player.game_time / 365),
            status_text
        ))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
        
        f.render_widget(header, area);
    }

    fn render_game_area_static(f: &mut Frame, area: Rect, world: &World, vehicles: &[crate::vehicle::Vehicle], cursor_x: usize, cursor_y: usize, camera_x: usize, camera_y: usize, _view_width: usize, _view_height: usize) {
        let mut lines = Vec::new();
        
        for y in camera_y..std::cmp::min(camera_y + area.height as usize - 2, world.height) {
            let mut line_spans = Vec::new();
            
            for x in camera_x..std::cmp::min(camera_x + area.width as usize - 2, world.width) {
                let ch = world.get_ascii_char_with_vehicles(x, y, vehicles);
                let style = if x == cursor_x && y == cursor_y {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Self::get_tile_style_with_vehicles_static(world, vehicles, x, y)
                };
                
                line_spans.push(Span::styled(ch.to_string(), style));
            }
            
            lines.push(Line::from(line_spans));
        }

        let map = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("World Map"));
        
        f.render_widget(map, area);
    }

    fn render_info_panel_static(f: &mut Frame, area: Rect, cursor_x: usize, cursor_y: usize, selected_tile: &Option<Tile>, build_mode: Option<BuildAction>, vehicles: &[crate::vehicle::Vehicle]) {
        let cursor_info = if let Some(build_action) = build_mode {
            let (action_text, cost_text) = match build_action {
                BuildAction::BuildRailTrack => ("Building Railway Track", "Cost: $10,000"),
                BuildAction::BuildTrainStation => ("Building Train Station", "Cost: $50,000"),
                BuildAction::BuildRoad => ("Building Road", "Cost: $5,000"),
                BuildAction::BuildBusStop => ("Building Bus Stop", "Cost: $25,000"),
                BuildAction::BuyVehicle => ("Select location for vehicle", "Various costs"),
            };
            format!(
                "BUILD MODE: {}\n{}\nCursor: ({}, {})\nClick to build, ESC to cancel",
                action_text, cost_text, cursor_x, cursor_y
            )
        } else {
            // Check if there's a vehicle at cursor position
            let vehicle_at_cursor = vehicles.iter().find(|v| v.x == cursor_x && v.y == cursor_y);
            
            if let Some(vehicle) = vehicle_at_cursor {
                // Show vehicle info
                Self::format_vehicle_info(vehicle, cursor_x, cursor_y)
            } else if let Some(tile) = selected_tile {
                // Show selected tile info
                format!(
                    "Selected Tile: ({}, {})\nTerrain: {:?}\nContent: {}",
                    cursor_x, cursor_y, tile.terrain, Self::format_tile_content(&tile.content)
                )
            } else {
                // Show cursor position and instructions
                format!("Cursor: ({}, {})\nPress ENTER to select tile\nPress B for build menu\nPress ? for controls", cursor_x, cursor_y)
            }
        };

        let title = if build_mode.is_some() { 
            "Build Mode" 
        } else if vehicles.iter().any(|v| v.x == cursor_x && v.y == cursor_y) {
            "Vehicle Info"
        } else {
            "Tile Info"
        };
        let info = Paragraph::new(cursor_info)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: true });

        f.render_widget(info, area);
    }

    fn format_tile_content(content: &crate::world::TileContent) -> String {
        match content {
            crate::world::TileContent::Empty => "Empty".to_string(),
            crate::world::TileContent::Town(town) => {
                format!("Town: {}\nPopulation: {}\nGrowth: {:.1}%", 
                    town.name, town.population, town.growth_rate)
            },
            crate::world::TileContent::Industry(industry) => {
                format!("Industry: {:?}\nProduction: {}/month\nInputs: {}\nOutputs: {}", 
                    industry.industry_type, 
                    industry.production_rate,
                    Self::format_cargo_list(&industry.cargo_input),
                    Self::format_cargo_list(&industry.cargo_output))
            },
            crate::world::TileContent::Station(station) => {
                format!("Station: {}\nType: {:?}\nConnections: {}", 
                    station.name, station.station_type, station.connections.len())
            },
            crate::world::TileContent::Track(track_type) => {
                match track_type {
                    crate::world::TrackType::Straight { horizontal } => {
                        format!("Railway Track ({})", if *horizontal { "Horizontal" } else { "Vertical" })
                    },
                    crate::world::TrackType::Curve { from_dir: _, to_dir: _ } => {
                        "Railway Track (Curve)".to_string()
                    },
                    crate::world::TrackType::Junction => "Railway Junction".to_string(),
                }
            },
            crate::world::TileContent::Road => "Road".to_string(),
        }
    }

    fn format_cargo_list(cargo_list: &[crate::world::CargoType]) -> String {
        if cargo_list.is_empty() {
            "None".to_string()
        } else if cargo_list.len() == 1 {
            format!("{:?}", cargo_list[0])
        } else if cargo_list.len() <= 3 {
            cargo_list.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join(", ")
        } else {
            format!("{:?} and {} more", cargo_list[0], cargo_list.len() - 1)
        }
    }

    fn format_vehicle_info(vehicle: &crate::vehicle::Vehicle, x: usize, y: usize) -> String {
        let vehicle_type_name = match &vehicle.vehicle_type {
            crate::vehicle::VehicleType::Train { engine, cars } => {
                let engine_type = match engine {
                    crate::vehicle::TrainEngine::Steam { .. } => "Steam Train",
                    crate::vehicle::TrainEngine::Diesel { .. } => "Diesel Train", 
                    crate::vehicle::TrainEngine::Electric { .. } => "Electric Train",
                };
                format!("{} ({} cars)", engine_type, cars.len())
            },
            crate::vehicle::VehicleType::Road { truck_type } => match truck_type {
                crate::vehicle::TruckType::Bus { capacity } => format!("Bus ({})", capacity),
                crate::vehicle::TruckType::SmallTruck { capacity } => format!("Small Truck ({})", capacity),
                crate::vehicle::TruckType::LargeTruck { capacity } => format!("Large Truck ({})", capacity),
            },
            crate::vehicle::VehicleType::Ship { ship_type } => match ship_type {
                crate::vehicle::ShipType::CargoShip { capacity } => format!("Cargo Ship ({})", capacity),
                crate::vehicle::ShipType::PassengerShip { capacity } => format!("Passenger Ship ({})", capacity),
            },
            crate::vehicle::VehicleType::Aircraft { plane_type } => match plane_type {
                crate::vehicle::PlaneType::SmallPlane { capacity, .. } => format!("Small Plane ({})", capacity),
                crate::vehicle::PlaneType::LargePlane { capacity, .. } => format!("Large Plane ({})", capacity),
            },
        };

        let state_text = match &vehicle.state {
            crate::vehicle::VehicleState::Idle => "Idle",
            crate::vehicle::VehicleState::Moving { .. } => "Moving",
            crate::vehicle::VehicleState::Loading => "Loading",
            crate::vehicle::VehicleState::Unloading => "Unloading", 
            crate::vehicle::VehicleState::Broken => "Broken",
        };

        format!(
            "VEHICLE: {}\nPosition: ({}, {})\nState: {}\nAge: {} years\nReliability: {}%\nProfit: ${}\nDeliveries: {}/{}",
            vehicle_type_name,
            x, y,
            state_text,
            vehicle.age / 365,
            vehicle.reliability,
            vehicle.profit,
            vehicle.on_time_deliveries,
            vehicle.total_deliveries
        )
    }

    fn render_build_menu_static(f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect_static(50, 50, area);
        f.render_widget(Clear, popup_area);

        let items = vec![
            ListItem::new("1. Build Railway Track      $10,000"),
            ListItem::new("2. Build Train Station      $50,000"),
            ListItem::new("3. Build Road               $5,000"),
            ListItem::new("4. Build Bus Stop           $25,000"),
            ListItem::new("5. Buy Vehicle              $75,000"),
            ListItem::new(""),
            ListItem::new("ESC. Cancel"),
        ];

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Build Menu"))
            .style(Style::default().fg(Color::White));

        f.render_widget(list, popup_area);
    }

    fn render_controls_popup(f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect_static(60, 70, area);
        f.render_widget(Clear, popup_area);

        let controls_text = vec![
            Line::from("🎮 CONTROLS"),
            Line::from(""),
            Line::from("🗺️  Navigation:"),
            Line::from("   Arrow Keys / WASD    Move cursor"),
            Line::from("   ENTER / Space        Select tile"),
            Line::from(""),
            Line::from("🚂 Game Actions:"),
            Line::from("   B                    Open build menu"),
            Line::from("   1-5 (in menu)        Select build option"),
            Line::from("   P                    Pause game (planned)"),
            Line::from(""),
            Line::from("💾 Save/Load:"),
            Line::from("   F5                   Quick save"),
            Line::from("   F9                   Quick load"),
            Line::from(""),
            Line::from("ℹ️  Information:"),
            Line::from("   ?                    Show/hide this help"),
            Line::from(""),
            Line::from("🚪 Exit:"),
            Line::from("   Q                    Quit game"),
            Line::from("   ESC                  Close this menu"),
            Line::from(""),
            Line::from("Press ESC or ? to close this menu"),
        ];

        let controls = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title(" 🎮 Controls Help "))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        f.render_widget(controls, popup_area);
    }

    fn get_tile_style_static(world: &World, x: usize, y: usize) -> Style {
        if let Some(tile) = world.get_tile(x, y) {
            match &tile.content {
                crate::world::TileContent::Town(_) => Style::default().fg(Color::Blue),
                crate::world::TileContent::Industry(_) => Style::default().fg(Color::Red),
                crate::world::TileContent::Station(_) => Style::default().fg(Color::Green),
                crate::world::TileContent::Track(_) => Style::default().fg(Color::Yellow),
                crate::world::TileContent::Road => Style::default().fg(Color::Gray),
                _ => match tile.terrain {
                    crate::world::TerrainType::Grass => Style::default().fg(Color::Green),
                    crate::world::TerrainType::Water => Style::default().fg(Color::Blue),
                    crate::world::TerrainType::Mountain => Style::default().fg(Color::White),
                    crate::world::TerrainType::Desert => Style::default().fg(Color::Yellow),
                    crate::world::TerrainType::Forest => Style::default().fg(Color::Green),
                }
            }
        } else {
            Style::default()
        }
    }

    fn get_tile_style_with_vehicles_static(world: &World, vehicles: &[crate::vehicle::Vehicle], x: usize, y: usize) -> Style {
        // Check if there's a vehicle at this position first
        for vehicle in vehicles {
            if vehicle.x == x && vehicle.y == y {
                return match &vehicle.vehicle_type {
                    crate::vehicle::VehicleType::Train { .. } => Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD),
                    crate::vehicle::VehicleType::Road { .. } => Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD),
                    crate::vehicle::VehicleType::Ship { .. } => Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD),
                    crate::vehicle::VehicleType::Aircraft { .. } => Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD),
                };
            }
        }
        
        // If no vehicle, use normal tile styling
        Self::get_tile_style_static(world, x, y)
    }

    fn render_vehicle_menu_static(
        f: &mut Frame,
        area: Rect,
        vehicle_id: u32,
        vehicles: &[crate::vehicle::Vehicle],
    ) {
        if let Some(vehicle) = vehicles.iter().find(|v| v.id == vehicle_id) {
            let popup_area = Self::centered_rect_static(50, 60, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Vehicle {} Orders", vehicle.id))
                .border_style(Style::default().fg(Color::Yellow));

            let menu_items = vec![
                "1. Go to Location",
                "2. Create Route",
                "3. Start Route",
                "4. Stop Vehicle",
                "5. Send to Depot",
                "",
                "Press number to select, ESC to cancel"
            ];

            let menu_text: Vec<Line> = menu_items
                .iter()
                .map(|&item| Line::from(Span::raw(item)))
                .collect();

            let paragraph = Paragraph::new(menu_text)
                .block(block)
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left);

            f.render_widget(Clear, popup_area);
            f.render_widget(paragraph, popup_area);
        }
    }

    fn centered_rect_static(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}