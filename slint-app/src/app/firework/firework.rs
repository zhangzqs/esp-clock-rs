use std::{
    f32::consts::PI,
    fmt::Debug,
    rc::Rc,
};

use color_space::{Hsv, ToRgb};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{PixelColor, Rgb888, RgbColor},
    primitives::{Circle, Primitive, PrimitiveStyle},
    Drawable,
};


use rand::{
    prelude::{ThreadRng},
    Rng as _,
};

use super::vec2::Vec2f;

pub struct Context {
    // 重力加速度
    pub gravity: Vec2f,
    // 世界的边界
    pub world_bounds: Vec2f,
}

pub struct Particle {
    // 先前的位置
    prev_position: Option<Vec2f>,
    // 粒子的上下文
    ctx: Rc<Context>,
    // 粒子是否会爆炸
    will_explode: bool,
    // 颜色
    color: Hsv,
    // 当前位置
    position: Vec2f,
    // 当前速度
    velocity: Vec2f,
    // 粒子半径
    radius: f32,
    // 待回收
    is_dead: bool,
    // 粒子已存活的时间
    duration: f32,
}

impl Particle {
    pub fn new(
        ctx: Rc<Context>,
        will_explode: bool,
        color: Hsv,
        position: Vec2f,
        velocity: Vec2f,
        radius: f32,
    ) -> Self {
        Self {
            ctx,
            will_explode,
            color,
            position,
            velocity,
            radius,
            is_dead: false,
            duration: 0.0,
            prev_position: None,
        }
    }

    /// 更新粒子状态，返回是否已经爆炸
    fn update(&mut self, dt: f32) -> bool {
        // 速度更新
        let old_velocity = self.velocity;
        self.velocity += self.ctx.gravity * dt;
        // 位置更新
        self.prev_position = Some(self.position);
        self.position += self.velocity * dt;
        // 时间更新
        self.duration += dt;

        // 对于不会爆炸的粒子，需要持续更新颜色
        if !self.will_explode {
            self.color.v *= 0.98;
            if self.color.v < 0.1 {
                self.is_dead = true;
            }
        }

        // 当速度方向与重力方向相反时，说明粒子需要爆炸
        let should_explode = self.will_explode && self.velocity.y() * old_velocity.y() < 0.0;
        if should_explode {
            // 标记粒子已经爆炸，下一次将要回收
            self.is_dead = true;
        }
        should_explode
    }

    /// 绘制粒子
    fn draw<C, D, E>(&self, display: &mut D)
    where
        C: PixelColor + From<Rgb888>,
        D: DrawTarget<Color = C, Error = E>,
        E: Debug,
    {
        let rgb = self.color.to_rgb();
        Circle::new(self.position.into(), (self.radius * 2.0f32) as u32)
            .into_styled(PrimitiveStyle::with_fill(
                Rgb888::new(rgb.r as _, rgb.g as _, rgb.b as _).into(),
            ))
            .draw(display)
            .unwrap();
    }
}

pub struct ParticleSystem {
    /// 粒子系统的上下文
    ctx: Rc<Context>,
    /// 粒子列表
    particles: Vec<Particle>,
    /// 随机数发生器
    rng: ThreadRng,
}

impl ParticleSystem {
    pub fn new(ctx: Rc<Context>) -> Self {
        Self {
            ctx,
            particles: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    /// 更新粒子系统状态，返回是否已经爆炸
    pub fn update(&mut self, dt: f32) {
        // 待添加爆炸后产生的新粒子
        let mut particles_to_add = Vec::new();
        // 更新粒子状态
        for particle in &mut self.particles {
            if particle.update(dt) {
                // 粒子爆炸
                let position = particle.position;
                let velocity = particle.velocity;
                let radius = particle.radius;
                let color = particle.color;
                // 添加新粒子
                particles_to_add.push((position, velocity, radius, color));
            }

            // 不会爆炸的粒子需要被回收
            if !particle.will_explode {
                // 粒子是否超出边界
                let is_out_of_bounds = particle.position.x() < 0.0
                    || particle.position.x() > self.ctx.world_bounds.x()
                    || particle.position.y() < 0.0
                    || particle.position.y() > self.ctx.world_bounds.y();
                if is_out_of_bounds {
                    // 标记粒子需要被回收
                    particle.is_dead = true;
                }
            }
        }
        // 移除待回收的粒子
        self.particles.retain(|x| !x.is_dead);

        // 添加新粒子
        for (position, velocity, radius, color) in particles_to_add {
            // 计算面积
            let area = radius * radius;
            // 需要产生的粒子数量，每个粒子的面积为 0.1
            let count = 10;
            // 计算新粒子的半径
            let new_radius = area / count as f32;
            // 产生新粒子
            for _ in 0..count {
                // 随机角度
                let th = self.rng.gen_range(0.0..2.0 * PI);
                let (x, y) = th.sin_cos();
                // 随机冲量
                let r = self.rng.gen_range(0.0..2.0);
                // 随机速度
                let velocity = velocity + Vec2f::new(x, y) * r;
                // 随机颜色
                let mut c = color;
                c.h += self.rng.gen_range(-20.0..20.0);
                self.add_particle(false, c, position, velocity, new_radius);
            }
        }
    }

    /// 绘制粒子系统
    pub fn draw<C, D, E>(&self, display: &mut D)
    where
        C: PixelColor + From<Rgb888>,
        D: DrawTarget<Color = C, Error = E>,
        E: Debug,
    {
        // 绘制粒子
        for particle in &self.particles {
            particle.draw(display);
        }
    }

    /// 添加粒子
    pub fn add_particle(
        &mut self,
        will_explode: bool,
        color: Hsv,
        position: Vec2f,
        velocity: Vec2f,
        radius: f32,
    ) {
        self.particles.push(Particle::new(
            Rc::clone(&self.ctx),
            will_explode,
            color,
            position,
            velocity,
            radius,
        ));
    }
}
