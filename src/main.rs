use ggez;
use ggez::event;
use ggez::graphics;
use ggez::{GameResult};

mod pingpong;
mod engine_3d;

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("It's a me Marko!", "M.Š.");
    let (ctx, event_loop) = &mut cb.build()?;
    graphics::set_window_title(ctx, "game");

    //let mut state = pingpong::MainState::new(ctx);
    let mut state = engine_3d::MainState::new(ctx);

    event::run(ctx, event_loop, &mut state)?;
    Ok(())
}
