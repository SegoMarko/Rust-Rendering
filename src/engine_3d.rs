use ggez;
use ggez::event;
use ggez::graphics::{self, MeshBuilder};
use ggez::input::keyboard::{self, KeyCode};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

const LINE_WIDTH: f32 = 1.0;

struct Triangle
{
    p: [na::Point3<f32>; 3]
}

impl Clone for Triangle
{
    fn clone(&self) -> Triangle{
        Triangle
        {
            p: [self.p[0], self.p[1], self.p[2]]
        }
    }
}

impl Default for Triangle
{
    fn default() -> Triangle {
        Triangle
        {
            p: [na::Point3::new(0.0, 0.0, 0.0), na::Point3::new(1.0, 1.0, 1.0), na::Point3::new(2.0, 2.0, 2.0)],
        }
    }
}

pub struct MainState
{
    mesh: Vec<Triangle>,
    matrix: [[f32; 4]; 4],
    m_rotate_x: [[f32; 4]; 4],
    m_rotate_z: [[f32; 4]; 4],
    f_theta: f32,
    paused: bool,
    camera: na::Point3<f32>
}

impl event::EventHandler for MainState
{
    fn update(&mut self, ctx: &mut Context) -> GameResult
    {
        if keyboard::is_key_pressed(ctx, KeyCode::K) {
            self.paused = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::L) {
            self.paused = false;
        }

        if !self.paused
        {
            self.f_theta += 0.02;

            self.m_rotate_z[0][0] = self.f_theta.cos();
            self.m_rotate_z[0][1] = self.f_theta.sin();
            self.m_rotate_z[1][0] = -self.f_theta.sin();
            self.m_rotate_z[1][1] = self.f_theta.cos();

            self.m_rotate_x[1][1] = (self.f_theta * 0.5).cos();
            self.m_rotate_x[1][2] = (self.f_theta * 0.5).sin();
            self.m_rotate_x[2][1] = -(self.f_theta * 0.5).sin();
            self.m_rotate_x[2][2] = (self.f_theta * 0.5).cos();
        }
        Ok(())
    }


    fn draw(&mut self, ctx: &mut Context) -> GameResult
    {
        if !self.paused
        {
            graphics::clear(ctx, graphics::BLACK);

            let (screen_w, screen_h) = graphics::drawable_size(ctx);
            let (screen_w_half, screen_h_half) = (screen_w / 2.0, screen_h / 2.0);

            let mut mesh_builder = graphics::MeshBuilder::new();
            for triangle in &mut self.mesh
            {
                // projected triangle
                let mut tri: Triangle = Triangle::default();

                // rotation (before translation!)
                let mut tri_z: Triangle = Triangle::default();
                multiply_matrix_vector(&triangle.p[0], &mut tri_z.p[0], &self.m_rotate_z);
                multiply_matrix_vector(&triangle.p[1], &mut tri_z.p[1], &self.m_rotate_z);
                multiply_matrix_vector(&triangle.p[2], &mut tri_z.p[2], &self.m_rotate_z);

                let mut tri_x: Triangle = Triangle::default();
                multiply_matrix_vector(&tri_z.p[0], &mut tri_x.p[0], &self.m_rotate_x);
                multiply_matrix_vector(&tri_z.p[1], &mut tri_x.p[1], &self.m_rotate_x);
                multiply_matrix_vector(&tri_z.p[2], &mut tri_x.p[2], &self.m_rotate_x);

                // translation
                let mut tri_translated = tri_x.clone();
                tri_translated.p[0].z = tri_x.p[0].z + 2.0;
                tri_translated.p[1].z = tri_x.p[1].z + 2.0;
                tri_translated.p[2].z = tri_x.p[2].z + 2.0;

                // calculating normal
                let mut normal = na::Point3::new(0.0, 0.0, 0.0);
                let mut line1 = na::Point3::new(0.0, 0.0, 0.0);
                let mut line2 = na::Point3::new(0.0, 0.0, 0.0);

                line1.x = tri_translated.p[1].x - tri_translated.p[0].x;
                line1.y = tri_translated.p[1].y - tri_translated.p[0].y;
                line1.z = tri_translated.p[1].z - tri_translated.p[0].z;
                line2.x = tri_translated.p[2].x - tri_translated.p[0].x;
                line2.y = tri_translated.p[2].y - tri_translated.p[0].y;
                line2.z = tri_translated.p[2].z - tri_translated.p[0].z;
                normal.x = line1.y * line2.z - line1.z * line2.y;
                normal.y = line1.z * line2.x - line1.x * line2.z;
                normal.z = line1.x * line2.y - line1.y * line2.x;

                // normalising normal
                let normal_len = (normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
                normal.x /= normal_len; normal.y /= normal_len; normal.z /= normal_len;

                // if triangle will be seen
                if normal.x * (tri_translated.p[0].x - self.camera.x)
                    + normal.y * (tri_translated.p[0].y - self.camera.y)
                    + normal.z * (tri_translated.p[0].z - self.camera.z) < 0.0
                {
                    // projection
                    multiply_matrix_vector(&tri_translated.p[0], &mut tri.p[0], &self.matrix);
                    multiply_matrix_vector(&tri_translated.p[1], &mut tri.p[1], &self.matrix);
                    multiply_matrix_vector(&tri_translated.p[2], &mut tri.p[2], &self.matrix);

                    // scaling
                    tri.p[0].x += 1.0; tri.p[0].y += 1.0;
                    tri.p[1].x += 1.0; tri.p[1].y += 1.0;
                    tri.p[2].x += 1.0; tri.p[2].y += 1.0;
                    tri.p[0].x *= 0.5 * screen_w; tri.p[0].y *= 0.5 * screen_h;
                    tri.p[1].x *= 0.5 * screen_w; tri.p[1].y *= 0.5 * screen_h;
                    tri.p[2].x *= 0.5 * screen_w; tri.p[2].y *= 0.5 * screen_h;

                    let mut points = [na::Point2::new(0.0, 0.0); 4];
                    triangle_to_points(&tri, &mut points);
                    //mesh_builder.line(&points, LINE_WIDTH, graphics::WHITE)?;
                    mesh_builder.triangles(&points, graphics::WHITE)?;
                }
            }

            let mesh = mesh_builder.build(ctx)?;
            let mut draw_param = graphics::DrawParam::default();
            //draw_param.dest = na::Point2::new(screen_w_half, screen_h_half).into();
            draw_param.dest = na::Point2::new(0.0, 0.0).into();
            graphics::draw(ctx, &mesh, draw_param)?;

            graphics::present(ctx)?;
        }
        Ok(())
    }
}

impl MainState
{
    pub fn new(ctx: &mut Context) -> Self
    {
        let cube = vec!
            [
                Triangle{p:[na::Point3::new(0.0, 0.0, 0.0), na::Point3::new(0.0, 1.0, 0.0), na::Point3::new(1.0, 1.0, 0.0)]},
                Triangle{p:[na::Point3::new(0.0, 0.0, 0.0), na::Point3::new(1.0, 1.0, 0.0), na::Point3::new(1.0, 0.0, 0.0)]},

                Triangle{p:[na::Point3::new(1.0, 0.0, 0.0), na::Point3::new(1.0, 1.0, 0.0), na::Point3::new(1.0, 1.0, 1.0)]},
                Triangle{p:[na::Point3::new(1.0, 0.0, 0.0), na::Point3::new(1.0, 1.0, 1.0), na::Point3::new(1.0, 0.0, 1.0)]},

                Triangle{p:[na::Point3::new(1.0, 0.0, 1.0), na::Point3::new(1.0, 1.0, 1.0), na::Point3::new(0.0, 1.0, 1.0)]},
                Triangle{p:[na::Point3::new(1.0, 0.0, 1.0), na::Point3::new(0.0, 1.0, 1.0), na::Point3::new(0.0, 0.0, 1.0)]},

                Triangle{p:[na::Point3::new(0.0, 0.0, 1.0), na::Point3::new(0.0, 1.0, 1.0), na::Point3::new(0.0, 1.0, 0.0)]},
                Triangle{p:[na::Point3::new(0.0, 0.0, 1.0), na::Point3::new(0.0, 1.0, 0.0), na::Point3::new(0.0, 0.0, 0.0)]},

                Triangle{p:[na::Point3::new(0.0, 1.0, 0.0), na::Point3::new(0.0, 1.0, 1.0), na::Point3::new(1.0, 1.0, 1.0)]},
                Triangle{p:[na::Point3::new(0.0, 1.0, 0.0), na::Point3::new(1.0, 1.0, 1.0), na::Point3::new(1.0, 1.0, 0.0)]},

                Triangle{p:[na::Point3::new(1.0, 0.0, 1.0), na::Point3::new(0.0, 0.0, 1.0), na::Point3::new(0.0, 0.0, 0.0)]},
                Triangle{p:[na::Point3::new(1.0, 0.0, 1.0), na::Point3::new(0.0, 0.0, 0.0), na::Point3::new(1.0, 0.0, 0.0)]},
            ];

        let f_near = 0.1;
        let f_far = 1000.0;
        let f_fov = 90.0;
        let (screen_w, screen_h) = graphics::drawable_size(ctx);
        let f_aspect = screen_h / screen_w;
        let f_fov_rad = 1.0 / (f_fov * 0.5 / 180.0 * (std::f64::consts::PI as f32)).tan();
        
        let mut projection_matrix = [[0.0; 4]; 4];
        projection_matrix[0][0] = f_aspect * f_fov_rad;
        projection_matrix[1][1] = f_fov_rad;
        projection_matrix[2][2] = f_far / (f_far - f_near);
        projection_matrix[3][2] = (-f_far * f_near) / (f_far - f_near);
        projection_matrix[2][3] = 1.0;
        projection_matrix[3][3] = 0.0;

        let theta: f32 = 0.0;
        let mut rotate_z_matrix = [[0.0; 4]; 4];
        rotate_z_matrix[0][0] = theta.cos();
        rotate_z_matrix[0][1] = theta.sin();
        rotate_z_matrix[1][0] = -theta.sin();
        rotate_z_matrix[1][1] = theta.cos();
        rotate_z_matrix[2][2] = 1.0;
        rotate_z_matrix[3][3] = 1.0;

        let mut rotate_x_matrix = [[0.0; 4]; 4];
        rotate_x_matrix[0][0] = 1.0;
        rotate_x_matrix[1][1] = (theta * 0.5).cos();
        rotate_x_matrix[1][2] = (theta * 0.5).sin();
        rotate_x_matrix[2][1] = -(theta * 0.5).sin();
        rotate_x_matrix[2][2] = (theta * 0.5).cos();
        rotate_x_matrix[3][3] = 1.0;

        MainState
        {
            mesh: cube,
            matrix: projection_matrix,
            f_theta: theta,
            m_rotate_x: rotate_x_matrix,
            m_rotate_z: rotate_z_matrix,
            paused: false,
            camera: na::Point3::new(0.0, 0.0, 0.0)
        }
    }
}

fn multiply_matrix_vector(i: &na::Point3<f32>, o: &mut na::Point3<f32>, m: &[[f32; 4]; 4])
{
    o.x = i.x * m[0][0] + i.y * m[1][0] + i.z * m[2][0] + m[3][0];
    o.y = i.x * m[0][1] + i.y * m[1][1] + i.z * m[2][1] + m[3][1];
    o.z = i.x * m[0][2] + i.y * m[1][2] + i.z * m[2][2] + m[3][2];

    let w = i.x * m[0][3] + i.y * m[1][3] + i.z * m[2][3] + m[3][3];

    if w != 0.0
    {
        o.x /= w; o.y /= w; o.z /= w;
    }
}

fn triangle_to_points(tri: &Triangle, o: &mut [na::Point2<f32>; 4])
{
    o[0] =  na::Point2::new(tri.p[0].x, tri.p[0].y);
    o[1] =  na::Point2::new(tri.p[1].x, tri.p[1].y);
    o[2] =  na::Point2::new(tri.p[2].x, tri.p[2].y);
    o[3] =  na::Point2::new(tri.p[0].x, tri.p[0].y);
}