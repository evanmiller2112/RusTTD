use rust_ttd::game::Game;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = Game::new();
    game.run()
}
