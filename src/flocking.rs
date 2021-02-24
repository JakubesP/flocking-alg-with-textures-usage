use swgl::camera2d::interface::CameraType;
use swgl::gl_wrapper::texture::texture_2d::Texture2D;
use swgl::global_tools::helpers::random_numbers::get_random;
use swgl::global_tools::vector2::Vector2;
use swgl::graphics_2d::color::Color;
use swgl::graphics_2d::renderer::rectangle_renderer::RectangleRenderer;
use swgl::graphics_2d::vertex_2d::predefined::single_tex_vertex2d::SingleTexVertex2D;
use swgl::runtime_error::SWGLResult;

// -----------------------------------------------------------------------------------------

pub const BOID_SIZE_X: f32 = 21.0;
pub const BOID_SIZE_Y: f32 = 35.0;
pub const MAX_SPEED: f32 = 200.0;
pub const MAX_FORCE: f32 = 0.05;

pub const FLOCK_NEIGHBORHOOD_ZONE: f32 = 360.0;
pub const BOID_SEPARATION_ZONE: f32 = 65.0;

pub const COHESION_WEIGHT: f32 = 1.0;
pub const SEPARATE_WEIGHT: f32 = 1.5;
pub const CURSOR_SEPARATE_WEIGHT: f32 = 4.0;
pub const ALIGN_WEIGHT: f32 = 1.0;

// -----------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Boid {
    pub acceleration: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub position: Vector2<f32>,
    pub color: Color,
}

impl Boid {
    pub fn new(position: &Vector2<f32>, color: &Color) -> Self {
        let direction = get_random(0.0, 360.0) as f32;
        Self {
            velocity: Vector2::from_angle(direction),
            position: *position,
            color: *color,
            acceleration: Vector2::zero(),
        }
    }

    pub fn seek(&mut self, target: &Vector2<f32>) -> Vector2<f32> {
        let mut desired = *target - self.position;
        desired.normalize();
        desired *= MAX_SPEED;
        let mut steer = desired - self.velocity;
        steer.limit(MAX_FORCE);
        steer
    }
}

// -----------------------------------------------------------------------------------------

pub struct Flock {
    boids: Vec<Boid>,
}

impl Flock {
    pub fn new(count: usize, area_relative_size: f32) -> SWGLResult<Self> {
        let mut boids = vec![];

        for _ in 0..count {
            let position = Vector2::new(area_relative_size / 2.0, area_relative_size / 2.0);
            boids.push(Boid::new(&position, &Color::from_hex(0x79e095ff)));
        }

        Ok(Flock { boids })
    }

    pub fn update_model(&mut self, renderer: &mut RectangleRenderer<SingleTexVertex2D>) {
        for boid in self.boids.iter() {
            let boid_vertices = SingleTexVertex2D::new_general(0.0, 1.0);
            let angle = boid.velocity.heading().to_degrees() - 90.0;
            renderer.add_sprite_with_trans(
                boid_vertices,
                &boid.position,
                &Vector2::new(BOID_SIZE_X, BOID_SIZE_Y),
                &Vector2::new(BOID_SIZE_X / 2.0, BOID_SIZE_Y / 2.0),
                angle.to_radians(),
            );
        }
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        area_relative_size: f32,
        border_thick: f32,
        mouse_cursor: &Vector2<f32>,
    ) {
        for boid_index in 0..self.boids.len() {
            // flock
            let flock_force = self.flock(boid_index, mouse_cursor);
            self.boids[boid_index].acceleration += flock_force;

            let current_boid = &mut self.boids[boid_index];

            // update
            current_boid.velocity += current_boid.acceleration;
            current_boid.velocity.limit(MAX_SPEED);
            current_boid.position += current_boid.velocity * delta_time;
            current_boid.acceleration *= 0.0;

            // borders
            if current_boid.position.x > area_relative_size - border_thick / 2.0 {
                current_boid.position.x = border_thick / 2.0;
            } else if current_boid.position.x < border_thick / 2.0 {
                current_boid.position.x = area_relative_size - border_thick / 2.0;
            }

            if current_boid.position.y > area_relative_size - border_thick / 2.0 {
                current_boid.position.y = border_thick / 2.0;
            } else if current_boid.position.y < border_thick / 2.0 {
                current_boid.position.y = area_relative_size - border_thick / 2.0;
            }
        }
    }

    pub fn draw(
        &mut self,
        context: &swgl::AppContext,
        renderer: &mut RectangleRenderer<SingleTexVertex2D>,
        camera: &dyn CameraType,
        tex: &Texture2D,
    ) {
        self.update_model(renderer);
        renderer.flush(context, camera, Some(tex)).unwrap();
    }
}

// -----------------------------------------------------------------------------------------

impl Flock {
    pub fn cohesion(&mut self, current_boid_index: usize) -> Vector2<f32> {
        let current_boid = &self.boids[current_boid_index];

        let mut center_of_mas = Vector2::zero();
        let mut neighbours_count = 0;

        for (index, boid) in self.boids.iter().enumerate() {
            let distance = boid.position.distance_to(&current_boid.position);
            if index != current_boid_index && distance < FLOCK_NEIGHBORHOOD_ZONE && distance > 0.0 {
                center_of_mas += boid.position;
                neighbours_count += 1;
            }
        }

        if neighbours_count > 0 {
            center_of_mas /= neighbours_count as f32;
            return self.boids[current_boid_index].seek(&center_of_mas);
        } else {
            return Vector2::zero();
        }
    }

    pub fn separate(&self, current_boid_index: usize) -> Vector2<f32> {
        let current_boid = &self.boids[current_boid_index];

        let mut steer_vector = Vector2::zero();
        let mut neighbours_count = 0;

        for (index, boid) in self.boids.iter().enumerate() {
            let distance = boid.position.distance_to(&current_boid.position);
            if index != current_boid_index && distance < BOID_SEPARATION_ZONE && distance > 0.0 {
                let mut diff = current_boid.position - boid.position;
                diff.normalize();
                diff /= distance;
                steer_vector += diff;
                neighbours_count += 1;
            }
        }

        if neighbours_count > 0 {
            steer_vector /= neighbours_count as f32;
        }

        if steer_vector.mag() > 0.0 {
            steer_vector.normalize();
            steer_vector *= MAX_SPEED;
            steer_vector -= current_boid.velocity;
            steer_vector.limit(MAX_FORCE);
        }

        steer_vector
    }

    pub fn separate_from_cursor(
        &self,
        current_boid_index: usize,
        mouse_cursor: &Vector2<f32>,
    ) -> Vector2<f32> {
        let current_boid = &self.boids[current_boid_index];
        let mut steer_vector = Vector2::zero();
        let distance = current_boid.position.distance_to(mouse_cursor);

        if distance < BOID_SEPARATION_ZONE && distance > 0.0 {
            let mut diff = current_boid.position - *mouse_cursor;
            diff.normalize();
            diff /= distance;
            steer_vector += diff;

            if steer_vector.mag() > 0.0 {
                steer_vector.normalize();
                steer_vector *= MAX_SPEED;
                steer_vector -= current_boid.velocity;
                steer_vector.limit(MAX_FORCE);
            }
        }

        steer_vector
    }

    pub fn align(&self, current_boid_index: usize) -> Vector2<f32> {
        let current_boid = &self.boids[current_boid_index];

        let mut average_velocity = Vector2::zero();
        let mut neighbours_count = 0;

        for (index, boid) in self.boids.iter().enumerate() {
            let distance = boid.position.distance_to(&current_boid.position);
            if index != current_boid_index && distance < FLOCK_NEIGHBORHOOD_ZONE && distance > 0.0 {
                average_velocity += boid.velocity;
                neighbours_count += 1;
            }
        }

        if neighbours_count > 0 {
            average_velocity /= neighbours_count as f32;
            average_velocity.normalize();
            average_velocity *= MAX_SPEED;
            let mut steer = average_velocity - current_boid.velocity;
            steer.limit(MAX_FORCE);
            return steer;
        } else {
            return Vector2::zero();
        }
    }

    pub fn flock(
        &mut self,
        current_boid_index: usize,
        mouse_cursor: &Vector2<f32>,
    ) -> Vector2<f32> {
        let mut result = Vector2::zero();
        result += self.cohesion(current_boid_index) * COHESION_WEIGHT;
        result += self.separate(current_boid_index) * SEPARATE_WEIGHT;
        result += self.align(current_boid_index) * ALIGN_WEIGHT;
        result +=
            self.separate_from_cursor(current_boid_index, mouse_cursor) * CURSOR_SEPARATE_WEIGHT;
        return result;
    }
}
