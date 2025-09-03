use axum::{
    extract::{ws::WebSocket, ws::Message, WebSocketUpgrade, State, Path, Query},
    response::{Html, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{stream::StreamExt, sink::SinkExt};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, error};

use crate::game::Game;
use crate::server::{InputCommand, CommandResponse, GameRenderData};

#[derive(Clone)]
pub struct AppState {
    pub game: Arc<Mutex<Game>>,
    pub tx: broadcast::Sender<GameRenderData>,
}

#[derive(Deserialize)]
pub struct MoveQuery {
    direction: String,
}

#[derive(Deserialize)]
pub struct ActionQuery {
    action: String,
    x: Option<usize>,
    y: Option<usize>,
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create game instance in headless mode
    let game = Arc::new(Mutex::new(Game::new_headless()));
    
    // Create broadcast channel for real-time updates
    let (tx, _) = broadcast::channel(100);
    
    let app_state = AppState {
        game: game.clone(),
        tx: tx.clone(),
    };

    // Spawn game update loop
    let game_clone = game.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            
            // Update game state
            {
                let mut game_guard = game_clone.lock().unwrap();
                game_guard.update();
            }

            // Send update to all connected clients
            let render_data = {
                let game_guard = game_clone.lock().unwrap();
                game_guard.get_render_data()
            };

            if tx_clone.send(render_data).is_err() {
                // No receivers, continue
            }
        }
    });

    let app = Router::new()
        // WebSocket endpoint for real-time updates
        .route("/ws", get(websocket_handler))
        
        // REST API endpoints
        .route("/api/state", get(get_game_state))
        .route("/api/command", post(send_command))
        .route("/api/move", post(move_cursor))
        .route("/api/action", post(perform_action))
        .route("/api/build/:action", post(build_action))
        .route("/api/vehicle/:id/order", post(vehicle_order))
        .route("/api/purchase/:vehicle_type", post(purchase_vehicle))
        
        // Serve static files and main page
        .route("/", get(serve_index))
        .route("/health", get(health_check))
        
        // CORS middleware
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("🚂 RusTTD Web Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // Send initial game state
    let initial_state = {
        let game_guard = state.game.lock().unwrap();
        game_guard.get_render_data()
    };
    
    if socket.send(Message::Text(serde_json::to_string(&initial_state).unwrap())).await.is_err() {
        return;
    }

    // Handle outgoing broadcasts
    let (mut sender, mut receiver) = socket.split();
    
    let tx_task = tokio::spawn(async move {
        while let Ok(render_data) = rx.recv().await {
            let json_str = match serde_json::to_string(&render_data) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to serialize render data: {}", e);
                    continue;
                }
            };
            
            if sender.send(Message::Text(json_str)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let rx_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        if let Ok(command) = serde_json::from_str::<InputCommand>(&text) {
                            if let Some(input_event) = command.to_input_event() {
                                let mut game_guard = state_clone.game.lock().unwrap();
                                let cursor_pos = (
                                    command.x.unwrap_or(game_guard.cursor_x),
                                    command.y.unwrap_or(game_guard.cursor_y)
                                );
                                game_guard.process_input_event(input_event, cursor_pos);
                            }
                        }
                    },
                    Message::Close(_) => break,
                    _ => {}
                }
            } else {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = tx_task => {},
        _ = rx_task => {},
    }
}

async fn get_game_state(State(state): State<AppState>) -> Json<GameRenderData> {
    let game_guard = state.game.lock().unwrap();
    Json(game_guard.get_render_data())
}

async fn send_command(
    State(state): State<AppState>,
    Json(command): Json<InputCommand>,
) -> Json<CommandResponse> {
    if let Some(input_event) = command.to_input_event() {
        let mut game_guard = state.game.lock().unwrap();
        let cursor_pos = (
            command.x.unwrap_or(game_guard.cursor_x),
            command.y.unwrap_or(game_guard.cursor_y)
        );
        game_guard.process_input_event(input_event, cursor_pos);
        
        Json(CommandResponse {
            success: true,
            message: "Command executed successfully".to_string(),
        })
    } else {
        Json(CommandResponse {
            success: false,
            message: "Invalid command".to_string(),
        })
    }
}

async fn move_cursor(
    State(state): State<AppState>,
    Query(params): Query<MoveQuery>,
) -> Json<CommandResponse> {
    let input_event = match params.direction.as_str() {
        "up" => crate::ui::InputEvent::Move(crate::ui::CursorDirection::Up),
        "down" => crate::ui::InputEvent::Move(crate::ui::CursorDirection::Down),
        "left" => crate::ui::InputEvent::Move(crate::ui::CursorDirection::Left),
        "right" => crate::ui::InputEvent::Move(crate::ui::CursorDirection::Right),
        _ => return Json(CommandResponse {
            success: false,
            message: "Invalid direction".to_string(),
        }),
    };

    let mut game_guard = state.game.lock().unwrap();
    let cursor_pos = (game_guard.cursor_x, game_guard.cursor_y);
    game_guard.process_input_event(input_event, cursor_pos);

    Json(CommandResponse {
        success: true,
        message: "Cursor moved".to_string(),
    })
}

async fn perform_action(
    State(state): State<AppState>,
    Query(params): Query<ActionQuery>,
) -> Json<CommandResponse> {
    let input_event = match params.action.as_str() {
        "select" => crate::ui::InputEvent::Select,
        "build_menu" => crate::ui::InputEvent::BuildMenu,
        "save" => crate::ui::InputEvent::Save,
        "load" => crate::ui::InputEvent::Load,
        _ => return Json(CommandResponse {
            success: false,
            message: "Invalid action".to_string(),
        }),
    };

    let mut game_guard = state.game.lock().unwrap();
    let cursor_pos = (
        params.x.unwrap_or(game_guard.cursor_x),
        params.y.unwrap_or(game_guard.cursor_y)
    );
    game_guard.process_input_event(input_event, cursor_pos);

    Json(CommandResponse {
        success: true,
        message: format!("Action '{}' performed", params.action),
    })
}

async fn build_action(
    Path(action): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<ActionQuery>,
) -> Json<CommandResponse> {
    let build_action = match action.as_str() {
        "rail" => crate::ui::BuildAction::BuildRailTrack,
        "station" => crate::ui::BuildAction::BuildTrainStation,
        "road" => crate::ui::BuildAction::BuildRoad,
        "bus_stop" => crate::ui::BuildAction::BuildBusStop,
        "vehicle" => crate::ui::BuildAction::BuyVehicle,
        _ => return Json(CommandResponse {
            success: false,
            message: "Invalid build action".to_string(),
        }),
    };

    let mut game_guard = state.game.lock().unwrap();
    let cursor_pos = (
        params.x.unwrap_or(game_guard.cursor_x),
        params.y.unwrap_or(game_guard.cursor_y)
    );
    game_guard.process_input_event(
        crate::ui::InputEvent::BuildAction(build_action),
        cursor_pos
    );

    Json(CommandResponse {
        success: true,
        message: format!("Build action '{}' performed", action),
    })
}

async fn vehicle_order(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Query(params): Query<ActionQuery>,
) -> Json<CommandResponse> {
    let order = match params.action.as_str() {
        "go_to" => crate::ui::VehicleOrder::GoToLocation,
        "create_route" => crate::ui::VehicleOrder::CreateRoute,
        "start_route" => crate::ui::VehicleOrder::StartRoute,
        "stop" => crate::ui::VehicleOrder::Stop,
        "depot" => crate::ui::VehicleOrder::SendToDepot,
        _ => return Json(CommandResponse {
            success: false,
            message: "Invalid vehicle order".to_string(),
        }),
    };

    let mut game_guard = state.game.lock().unwrap();
    game_guard.selected_vehicle_id = Some(id);
    let cursor_pos = (
        params.x.unwrap_or(game_guard.cursor_x),
        params.y.unwrap_or(game_guard.cursor_y)
    );
    game_guard.process_input_event(crate::ui::InputEvent::VehicleOrder(order), cursor_pos);

    Json(CommandResponse {
        success: true,
        message: format!("Vehicle {} order '{}' sent", id, params.action),
    })
}

async fn purchase_vehicle(
    Path(vehicle_type): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<ActionQuery>,
) -> Json<CommandResponse> {
    let purchase_type = match vehicle_type.as_str() {
        "train" => crate::ui::VehiclePurchaseType::Train,
        "bus" => crate::ui::VehiclePurchaseType::Bus,
        "small_truck" => crate::ui::VehiclePurchaseType::SmallTruck,
        "large_truck" => crate::ui::VehiclePurchaseType::LargeTruck,
        "ship" => crate::ui::VehiclePurchaseType::Ship,
        "small_plane" => crate::ui::VehiclePurchaseType::SmallPlane,
        "large_plane" => crate::ui::VehiclePurchaseType::LargePlane,
        "auto" => crate::ui::VehiclePurchaseType::Auto,
        _ => return Json(CommandResponse {
            success: false,
            message: "Invalid vehicle type".to_string(),
        }),
    };

    let mut game_guard = state.game.lock().unwrap();
    let cursor_pos = (
        params.x.unwrap_or(game_guard.cursor_x),
        params.y.unwrap_or(game_guard.cursor_y)
    );
    game_guard.process_input_event(
        crate::ui::InputEvent::VehiclePurchase(purchase_type),
        cursor_pos
    );

    Json(CommandResponse {
        success: true,
        message: format!("Vehicle '{}' purchase attempted", vehicle_type),
    })
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn health_check() -> &'static str {
    "OK"
}