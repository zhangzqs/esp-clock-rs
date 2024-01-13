use std::{
    fmt::Debug,
    rc::Rc,
    sync::{
        Arc, Mutex,
    },
};

use color_space::Hsv;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{PixelColor, Rgb888, RgbColor},
};
use embedded_graphics_group::DisplayGroup;

use rand::{rngs::ThreadRng, Rng as _};

mod firework;
use firework::{Context, ParticleSystem};

mod vec2;
use vec2::Vec2f;

use crate::common::{GraphicsAppBase, IGraphicsApp};

#[derive(Debug, Clone, Copy)]
enum FireworkAppEvent {
    Fire,
}

struct FireworkGraphicsAppState {
    particle_system: ParticleSystem,
    rng: ThreadRng,
}

struct FireworkGraphicsApp;

impl IGraphicsApp for FireworkGraphicsApp {
    type Event = FireworkAppEvent;

    type State = FireworkGraphicsAppState;

    fn setup() -> Self::State {
        let ctx = Rc::new(Context {
            gravity: Vec2f::new(0.0, 0.5),
            world_bounds: Vec2f::new(240.0, 240.0),
        });
        let particle_system = ParticleSystem::new(Rc::clone(&ctx));
        let rng = rand::thread_rng();
        Self::State {
            particle_system,
            rng,
        }
    }

    fn render<DisplayColor, DisplayError, Display>(state: &mut Self::State, display: &mut Display)
    where
        DisplayColor: PixelColor + From<Rgb888>,
        DisplayError: Debug,
        Display: DrawTarget<Color = DisplayColor, Error = DisplayError>,
    {
        display.clear(Rgb888::BLACK.into()).unwrap();
        state.particle_system.draw(&mut *display);
        state.particle_system.update(0.4);
    }

    fn event(state: &mut Self::State, e: Self::Event) {
        let rng = &mut state.rng;
        match e {
            FireworkAppEvent::Fire => {
                let hsv = Hsv::new(rng.gen_range(0.0..360.0), 1.0, 1.0);
                // let rgb = Rgb::from(hsl);
                // 随机速度
                let velocity = Vec2f::new(rng.gen_range(-1.0..1.0), rng.gen_range(-16.0..-10.0));
                state.particle_system.add_particle(
                    true,
                    hsv,
                    Vec2f::new(120.0, 240.0),
                    velocity,
                    rng.gen_range(5.0..10.0),
                );
            }
        }
    }

    fn name<'a>() -> &'a str {
        "Firework"
    }
}

pub struct FireworkApp<EGC, EGD, EGE>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static,
    EGE: Debug,
{
    base: GraphicsAppBase<EGD, FireworkAppEvent, FireworkGraphicsApp>,
}

impl<EGC, EGD, EGE> FireworkApp<EGC, EGD, EGE>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
{
    pub fn new(display_group: Arc<Mutex<DisplayGroup<EGD>>>) -> Self {
        let base = GraphicsAppBase::new(display_group);
        Self { base }
    }

    pub fn enter(&mut self) {
        self.base.enter();
    }

    pub fn exit(&mut self) {
        self.base.exit();
    }

    pub fn fire(&mut self) {
        self.base.send_event(FireworkAppEvent::Fire);
    }
}
