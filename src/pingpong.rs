use ggez;
use ggez::event;
use ggez::graphics;
use ggez::input::keyboard::{self, KeyCode};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use rand::{self, thread_rng, Rng};

#[allow(dead_code)]

const RACKET_PADDING: f32 = 10.0;
const RACKET_HEIGHT: f32 = 100.0;
const RACKET_HEIGHT_HALF: f32 = RACKET_HEIGHT / 2.0;
const RACKET_WIDTH: f32 = 20.0;
const RACKET_WIDTH_HALF: f32 = RACKET_WIDTH / 2.0;
const BALL_SIZE: f32 = 30.0;
const BALL_SIZE_HALF: f32 = BALL_SIZE / 2.0;
const PLAYER_SPEED: f32 = 500.0;
const BALL_VEL: f32 = 300.0;
const MIDDLE_LINE_W: f32 = 5.0;
const BALL_ACC: f32 = BALL_VEL * 0.1;
const RACKET: graphics::Rect = graphics::Rect::new(
    -RACKET_WIDTH_HALF,
    -RACKET_HEIGHT_HALF,
    RACKET_WIDTH,
    RACKET_HEIGHT,
);
const BALL: graphics::Rect =
    graphics::Rect::new(-BALL_SIZE_HALF, -BALL_SIZE_HALF, BALL_SIZE, BALL_SIZE);

fn clamp(value: &mut f32, low: f32, high: f32) {
    if *value < low {
        *value = low;
    } else if *value > high {
        *value = high;
    }
}

pub struct MainState {
    player_1_pos: na::Point2<f32>,
    player_2_pos: na::Point2<f32>,
    ball_pos: na::Point2<f32>,
    ball_vel: na::Vector2<f32>,
    player_1_score: i32,
    player_2_score: i32,
    ball_is_inside_racket: bool,
    racket_mesh: GameResult<graphics::Mesh>,
    ball_mesh: GameResult<graphics::Mesh>,
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        move_racket(&mut self.player_1_pos, self.ball_pos, KeyCode::W, 1.0, ctx);
        move_racket(&mut self.player_1_pos, self.ball_pos, KeyCode::S, -1.0, ctx);

        move_racket(&mut self.player_2_pos, self.ball_pos, KeyCode::Up, 1.0, ctx);
        move_racket(
            &mut self.player_2_pos,
            self.ball_pos,
            KeyCode::Down,
            -1.0,
            ctx,
        );

        let (player_1_scored, player_2_scored) = move_ball(
            &mut self.ball_pos,
            &mut self.ball_vel,
            self.player_1_pos,
            self.player_2_pos,
            &mut self.ball_is_inside_racket,
            ctx,
        );
        if player_1_scored {
            self.player_1_score += 1;
        }
        if player_2_scored {
            self.player_2_score += 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        let mut draw_param = graphics::DrawParam::default();
        let racket_mesh = match self.racket_mesh.as_ref() {
            Ok(mesh) => mesh,
            Err(_e) => return Ok(()),
        };
        draw_param.dest = self.player_1_pos.into();
        graphics::draw(ctx, racket_mesh, draw_param)?;
        draw_param.dest = self.player_2_pos.into();
        graphics::draw(ctx, racket_mesh, draw_param)?;

        let (screen_w, screen_h) = graphics::drawable_size(ctx);
        let (screen_w_half, _screen_h_half) = (screen_w * 0.5, screen_h * 0.5);
        let middle_line = graphics::Rect::new(-MIDDLE_LINE_W * 0.5, 0.0, MIDDLE_LINE_W, screen_h);
        let middle_line_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            middle_line,
            graphics::WHITE,
        )?;
        draw_param.dest = [screen_w_half, 0.0].into();
        graphics::draw(ctx, &middle_line_mesh, draw_param)?;

        let ball_mesh = match self.ball_mesh.as_ref() {
            Ok(mesh) => mesh,
            Err(_e) => return Ok(()),
        };
        draw_param.dest = self.ball_pos.into();
        graphics::draw(ctx, ball_mesh, draw_param)?;

        let score_text = graphics::Text::new(format!(
            "{}      {}",
            self.player_1_score, self.player_2_score
        ));
        let (score_text_w, _score_text_h) = score_text.dimensions(ctx);
        let score_pos =
            na::Point2::new(screen_w_half, 20.0) - na::Vector2::new(score_text_w as f32 * 0.5, 0.0);
        draw_param.dest = score_pos.into();
        graphics::draw(ctx, &score_text, draw_param)?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn move_racket(
    pos: &mut na::Point2<f32>,
    ball_pos: na::Point2<f32>,
    keycode: KeyCode,
    y_dir: f32,
    ctx: &mut Context,
)
{
    let dt = ggez::timer::delta(ctx).as_secs_f32();
    let screen_h = graphics::drawable_size(ctx).1;

    let intersects_player = ball_pos.x - BALL_SIZE_HALF < pos.x + RACKET_WIDTH_HALF
        && ball_pos.x + BALL_SIZE_HALF > pos.x - RACKET_WIDTH_HALF
        && ball_pos.y - BALL_SIZE_HALF < pos.y + RACKET_HEIGHT_HALF
        && ball_pos.y + BALL_SIZE_HALF > pos.y - RACKET_HEIGHT_HALF;

    if !intersects_player && keyboard::is_key_pressed(ctx, keycode) {
        pos.y -= y_dir * PLAYER_SPEED * dt;
    }

    clamp(
        &mut pos.y,
        RACKET_HEIGHT_HALF,
        screen_h - RACKET_HEIGHT_HALF,
    );
}

fn move_ball
(
    pos: &mut na::Point2<f32>,
    vel: &mut na::Vector2<f32>,
    pos1: na::Point2<f32>,
    pos2: na::Point2<f32>,
    ball_was_inside_racket: &mut bool,
    ctx: &mut Context,
) -> (bool, bool)
{
    let dt = ggez::timer::delta(ctx).as_secs_f32();
    let (screen_w, screen_h) = graphics::drawable_size(ctx);
    *pos += (*vel) * dt;

    if pos.x < 0.0 {
        pos.x = screen_w * 0.5;
        pos.y = screen_h * 0.5;
        randomize_vec(vel, BALL_VEL, BALL_VEL);
        return (false, true);
    } else if pos.x > screen_w {
        pos.x = screen_w * 0.5;
        pos.y = screen_h * 0.5;
        randomize_vec(vel, BALL_VEL, BALL_VEL);
        return (true, false);
    }

    if pos.y <= BALL_SIZE_HALF {
        vel.y = vel.y.abs();
    } else if pos.y >= screen_h - BALL_SIZE_HALF {
        vel.y = -vel.y.abs();
    }

    let intersects_player_1 = pos.x - BALL_SIZE_HALF < pos1.x + RACKET_WIDTH_HALF
        && pos.x + BALL_SIZE_HALF > pos1.x - RACKET_WIDTH_HALF
        && pos.y - BALL_SIZE_HALF < pos1.y + RACKET_HEIGHT_HALF
        && pos.y + BALL_SIZE_HALF > pos1.y - RACKET_HEIGHT_HALF;
    let intersects_player_2 = pos.x - BALL_SIZE_HALF < pos2.x + RACKET_WIDTH_HALF
        && pos.x + BALL_SIZE_HALF > pos2.x - RACKET_WIDTH_HALF
        && pos.y - BALL_SIZE_HALF < pos2.y + RACKET_HEIGHT_HALF
        && pos.y + BALL_SIZE_HALF > pos2.y - RACKET_HEIGHT_HALF;

    if !*ball_was_inside_racket {
        if intersects_player_1 {
            let intersects_player_1_front = pos.y - BALL_SIZE_HALF > pos1.y - RACKET_HEIGHT_HALF
                && pos.y + BALL_SIZE_HALF < pos1.y + RACKET_HEIGHT_HALF
                || pos.x - BALL_SIZE_HALF > pos1.x;
            if intersects_player_1_front {
                vel.x = vel.x.abs();
            } else {
                vel.y = vel.y * -1.0;
            }
        }

        if intersects_player_2 {
            let intersects_player_2_front = pos.y - BALL_SIZE_HALF > pos2.y - RACKET_HEIGHT_HALF
                && pos.y + BALL_SIZE_HALF < pos2.y + RACKET_HEIGHT_HALF
                || pos.x + BALL_SIZE_HALF < pos2.x;
            if intersects_player_2_front {
                vel.x = -vel.x.abs();
            } else {
                vel.y = vel.y * -1.0;
            }
        }
    }

    if !intersects_player_1 && !intersects_player_2 {
        *ball_was_inside_racket = false;
    } else {
        *ball_was_inside_racket = true;

        if vel.x > 0.0 {
            vel.x += BALL_ACC;
        } else {
            vel.x -= BALL_ACC;
        }
        if vel.y > 0.0 {
            vel.y += BALL_ACC;
        } else {
            vel.y -= BALL_ACC;
        }
    }

    (false, false)
}

fn randomize_vec(vec: &mut na::Vector2<f32>, x: f32, y: f32)
{
    let mut rng = thread_rng();
    vec.x = match rng.gen_bool(0.5) {
        true => x,
        false => -x,
    };

    vec.y = match rng.gen_bool(0.5) {
        true => y,
        false => -y,
    };
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = graphics::drawable_size(ctx);
        let (screen_w_half, screen_h_half) = (screen_w * 0.5, screen_h * 0.5);

        let mut ball_vel = na::Vector2::new(0.0, 0.0);
        randomize_vec(&mut ball_vel, BALL_VEL, BALL_VEL);

        MainState {
            player_1_pos: na::Point2::new(RACKET_WIDTH_HALF + RACKET_PADDING, screen_h_half),
            player_2_pos: na::Point2::new(
                screen_w - RACKET_WIDTH_HALF - RACKET_PADDING,
                screen_h_half,
            ),
            ball_pos: na::Point2::new(screen_w_half, screen_h_half),
            player_1_score: 0,
            player_2_score: 0,
            ball_vel,
            ball_is_inside_racket: false,
            racket_mesh: graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                RACKET,
                graphics::WHITE,
            ),
            ball_mesh: graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                BALL,
                graphics::WHITE,
            ),
        }
    }
}