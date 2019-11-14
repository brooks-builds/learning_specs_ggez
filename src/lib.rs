use bbggez::ggez::event::EventHandler;
use bbggez::ggez::{
    graphics,
    nalgebra::{Point2, Vector2},
    Context, GameResult,
};
use specs::{Builder, Component, ReadStorage, RunNow, System, VecStorage, World, WorldExt, WriteStorage, Entities};

pub struct Game {
    world: World,
}

impl Game {
    pub fn new(_context: &mut Context) -> Game {
        let mut world = World::new();

        world.register::<Position>();
        world.register::<Size>();
		world.register::<Color>();
		world.register::<Velocity>();
		world.register::<GrowBy>();

        world
            .create_entity()
            .with(Position::new(100.0, 100.0))
            .with(Size::new(50.0))
			.with(Color::new(1.0, 0.0, 0.0, 1.0))
			.with(Velocity::new(0.1, 0.0))
			.with(GrowBy::new(0.0))
            .build();

        world
            .create_entity()
            .with(Position::new(100.0, 200.0))
            .with(Size::new(50.0))
			.with(Color::new(0.0, 0.0, 1.0, 1.0))
			.with(GrowBy::new(0.0))
			.build();
		
		world
            .create_entity()
            .with(Position::new(500.0, 100.0))
            .with(Size::new(20.0))
			.with(Color::new(0.0, 1.0, 1.0, 1.0))
			.with(GrowBy::new(0.0))
            .build();

        Game { world }
    }

    fn handle_window_size_change(
        &self,
        context: &mut Context,
        (width, height): (f32, f32),
    ) -> GameResult<()> {
        graphics::set_screen_coordinates(context, graphics::Rect::new(0.0, 0.0, width, height))
    }
}

impl EventHandler for Game {
    fn update(&mut self, context: &mut Context) -> GameResult<()> {
        let arena_size = graphics::drawable_size(context);
        self.handle_window_size_change(context, arena_size)?;

		let mut move_system = MoveSystem;
		let mut eat_system = EatSystem;
		let mut grow_system = GrowSystem;

		move_system.run_now(&self.world);
		eat_system.run_now(&self.world);
		grow_system.run_now(&self.world);
        self.world.maintain();

        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult<()> {
        graphics::clear(context, graphics::BLACK);

        let mut render_system = RenderSystem(context);
        render_system.run_now(&self.world);

        graphics::present(context)
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Position(Vector2<f32>);

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position(Vector2::new(x, y))
    }
}

#[derive(Component)]
#[storage(VecStorage)]
struct Size(f32);

impl Size {
    pub fn new(size: f32) -> Size {
        Size(size)
    }
}

#[derive(Component)]
#[storage(VecStorage)]
struct Color {
    pub value: graphics::Color,
}

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color {
            value: graphics::Color::new(red, green, blue, alpha),
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
struct Velocity {
	velocity: Vector2<f32>
}

impl Velocity {
	pub fn new(x: f32, y: f32) -> Velocity {
		Velocity {
			velocity: Vector2::new(x, y)
		}
	}

	pub fn get(&self) -> &Vector2<f32> {
		&self.velocity
	}
}

#[derive(Component)]
#[storage(VecStorage)]
struct GrowBy(f32);

impl GrowBy {
	pub fn new(amount: f32) -> GrowBy {
		GrowBy (amount)
	}

	pub fn reset(&mut self) {
		self.0 = 0.0;
	}
}

struct RenderSystem<'a>(&'a mut Context);

impl<'a> System<'a> for RenderSystem<'a> {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Color>,
    );

    fn run(&mut self, (position, size, color): Self::SystemData) {
        use specs::Join;

        for (position, size, color) in (&position, &size, &color).join() {
            let mesh = graphics::MeshBuilder::new()
                .circle(
                    graphics::DrawMode::fill(),
                    Point2::from(position.0),
                    size.0,
                    0.1,
                    color.value,
                )
                .build(self.0)
                .unwrap();

            graphics::draw(self.0, &mesh, graphics::DrawParam::default()).unwrap();
        }
    }
}

struct MoveSystem;

impl<'a> System<'a> for MoveSystem {
	type SystemData = (
		ReadStorage<'a, Velocity>,
		WriteStorage<'a, Position>
	);

	fn run(&mut self, (velocity, mut position): Self::SystemData) {
		use specs::Join;

		for (velocity, position) in (&velocity, &mut position).join() {
			position.0 += velocity.get();
		}
	}
}

struct EatSystem;

impl<'a> System<'a> for EatSystem {
	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, Size>,
		Entities<'a>,
		WriteStorage<'a, GrowBy>
	);

	fn run(&mut self, (position, size, entities, mut grow_by): Self::SystemData) {
		use specs::Join;

		
		for(my_position, my_size, me, grow_by) in (&position, &size, &entities, &mut grow_by).join() {
			for (other_position, other_size, other_entity) in (&position, &size, &entities).join() {
				if me != other_entity {
					let distance = my_position.0 - other_position.0;
	
					let distance = distance.magnitude();

					if my_size.0 > other_size.0 && distance + other_size.0 < my_size.0 {
						entities.delete(other_entity).unwrap();
						grow_by.0 += other_size.0;
					}
				}
			}


		}


	}
}

struct GrowSystem;

impl<'a> System<'a> for GrowSystem {
	type SystemData = (
		WriteStorage<'a, GrowBy>,
		WriteStorage<'a, Size>
	);

	fn run(&mut self, (mut grow_by, mut size): Self::SystemData) {
		use specs::Join;

		for (grow_by, size) in (&mut grow_by, &mut size).join() {
			size.0 += grow_by.0;
			grow_by.reset();
		}
	}
}