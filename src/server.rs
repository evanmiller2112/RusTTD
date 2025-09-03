use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameRenderData {
    pub world_data: WorldRenderData,
    pub player_data: PlayerRenderData,
    pub ui_state: UIRenderState,
    pub notifications: Vec<String>,
    pub game_time: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldRenderData {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileRenderData>>,
    pub vehicles: Vec<VehicleRenderData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileRenderData {
    pub x: usize,
    pub y: usize,
    pub terrain: crate::world::TerrainType,
    pub content: TileContentRenderData,
    pub ascii_char: char,
    pub style_color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TileContentRenderData {
    Empty,
    Town {
        name: String,
        population: u32,
    },
    Industry {
        industry_type: String,
        production_rate: u32,
        stockpile: HashMap<String, u32>,
    },
    Station {
        name: String,
        station_type: String,
        cargo_waiting: HashMap<String, u32>,
    },
    Track {
        track_type: String,
    },
    Road,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VehicleRenderData {
    pub id: u32,
    pub x: usize,
    pub y: usize,
    pub vehicle_type: String,
    pub state: String,
    pub cargo: HashMap<String, u32>,
    pub ascii_char: char,
    pub style_color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerRenderData {
    pub name: String,
    pub money: i64,
    pub vehicle_count: usize,
    pub reputation: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIRenderState {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub camera_x: usize,
    pub camera_y: usize,
    pub selected_tile_info: Option<String>,
    pub show_build_menu: bool,
    pub show_vehicle_menu: bool,
    pub selected_vehicle_id: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputCommand {
    pub command_type: String,
    pub x: Option<usize>,
    pub y: Option<usize>,
    pub vehicle_id: Option<u32>,
    pub build_action: Option<String>,
    pub vehicle_order: Option<String>,
    pub vehicle_purchase_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStateResponse {
    pub success: bool,
    pub message: Option<String>,
    pub render_data: GameRenderData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
}

impl InputCommand {
    pub fn to_input_event(&self) -> Option<crate::ui::InputEvent> {
        match self.command_type.as_str() {
            "quit" => Some(crate::ui::InputEvent::Quit),
            "move" => {
                if let (Some(x), Some(y)) = (self.x, self.y) {
                    // Convert absolute position to direction - this is simplified
                    // In a real implementation, you'd calculate direction based on current cursor
                    Some(crate::ui::InputEvent::Move(crate::ui::CursorDirection::Up))
                } else {
                    None
                }
            },
            "move_up" => Some(crate::ui::InputEvent::Move(crate::ui::CursorDirection::Up)),
            "move_down" => Some(crate::ui::InputEvent::Move(crate::ui::CursorDirection::Down)),
            "move_left" => Some(crate::ui::InputEvent::Move(crate::ui::CursorDirection::Left)),
            "move_right" => Some(crate::ui::InputEvent::Move(crate::ui::CursorDirection::Right)),
            "select" => Some(crate::ui::InputEvent::Select),
            "save" => Some(crate::ui::InputEvent::Save),
            "load" => Some(crate::ui::InputEvent::Load),
            "pause" => Some(crate::ui::InputEvent::Pause),
            "build_menu" => Some(crate::ui::InputEvent::BuildMenu),
            "show_controls" => Some(crate::ui::InputEvent::ShowControls),
            "build_rail" => Some(crate::ui::InputEvent::BuildAction(crate::ui::BuildAction::BuildRailTrack)),
            "build_station" => Some(crate::ui::InputEvent::BuildAction(crate::ui::BuildAction::BuildTrainStation)),
            "build_road" => Some(crate::ui::InputEvent::BuildAction(crate::ui::BuildAction::BuildRoad)),
            "build_bus_stop" => Some(crate::ui::InputEvent::BuildAction(crate::ui::BuildAction::BuildBusStop)),
            "buy_vehicle" => Some(crate::ui::InputEvent::BuildAction(crate::ui::BuildAction::BuyVehicle)),
            "vehicle_go_to" => Some(crate::ui::InputEvent::VehicleOrder(crate::ui::VehicleOrder::GoToLocation)),
            "vehicle_create_route" => Some(crate::ui::InputEvent::VehicleOrder(crate::ui::VehicleOrder::CreateRoute)),
            "vehicle_start_route" => Some(crate::ui::InputEvent::VehicleOrder(crate::ui::VehicleOrder::StartRoute)),
            "vehicle_stop" => Some(crate::ui::InputEvent::VehicleOrder(crate::ui::VehicleOrder::Stop)),
            "vehicle_depot" => Some(crate::ui::InputEvent::VehicleOrder(crate::ui::VehicleOrder::SendToDepot)),
            "finish_route" => Some(crate::ui::InputEvent::FinishRouteCreation),
            "buy_train" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::Train)),
            "buy_bus" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::Bus)),
            "buy_small_truck" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::SmallTruck)),
            "buy_large_truck" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::LargeTruck)),
            "buy_ship" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::Ship)),
            "buy_small_plane" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::SmallPlane)),
            "buy_large_plane" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::LargePlane)),
            "buy_auto" => Some(crate::ui::InputEvent::VehiclePurchase(crate::ui::VehiclePurchaseType::Auto)),
            _ => None,
        }
    }
}