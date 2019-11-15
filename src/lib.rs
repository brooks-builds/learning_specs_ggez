use bbggez::ggez::event::EventHandler;
use bbggez::ggez::{
	graphics,
	nalgebra::{Point2, Vector2},
	timer, Context, GameResult,
};
use bbggez::rand;
use bbggez::rand::prelude::*;
use specs::{
	Builder, Component, DenseVecStorage, Entities, Read, ReadStorage, RunNow, System, World,
	WorldExt, Write, WriteStorage,
};

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
		world.register::<Mesh>();

		world.insert(EntitiesCount::new());

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

		if timer::fps(context) > 60.0 {
			self.world
				.create_entity()
				.with(Position::new(arena_size.0 / 2.0, arena_size.1 / 2.0))
				.with(Size::new(3.0))
				.with(Color::new(1.0, 1.0, 1.0, 0.5))
				.with(Velocity::new())
				.with(Mesh::new(context))
				.build();
			let mut increment_entities = IncrementEntityCountSystem;
			increment_entities.run_now(&self.world);
		}

		let mut move_system = MoveSystem(timer::delta(context).as_secs_f32());
		let mut bounce_system = BounceOffWallsSystem(arena_size.0, arena_size.1);

		move_system.run_now(&self.world);
		bounce_system.run_now(&self.world);

		self.world.maintain();

		Ok(())
	}

	fn draw(&mut self, context: &mut Context) -> GameResult<()> {
		graphics::clear(context, graphics::BLACK);

		let mut render_system = RenderSystem(context, timer::fps(context));
		render_system.run_now(&self.world);

		graphics::present(context)
	}
}

#[derive(Component, Debug)]
struct Position(Vector2<f32>);

impl Position {
	pub fn new(x: f32, y: f32) -> Position {
		Position(Vector2::new(x, y))
	}
}

#[derive(Component)]
struct Size(f32);

impl Size {
	pub fn new(size: f32) -> Size {
		Size(size)
	}
}

#[derive(Component)]
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
struct Velocity {
	pub velocity: Vector2<f32>,
}

impl Velocity {
	pub fn new() -> Velocity {
		let mut rng = rand::thread_rng();

		Velocity {
			velocity: Vector2::new(rng.gen_range(-20.0, 20.0), rng.gen_range(-20.0, 20.0)),
		}
	}

	pub fn get(&self) -> &Vector2<f32> {
		&self.velocity
	}
}

#[derive(Component)]
struct GrowBy(f32);

impl GrowBy {
	pub fn new(amount: f32) -> GrowBy {
		GrowBy(amount)
	}

	pub fn reset(&mut self) {
		self.0 = 0.0;
	}
}

#[derive(Default)]
struct EntitiesCount(usize);

impl EntitiesCount {
	pub fn new() -> EntitiesCount {
		EntitiesCount(0)
	}
}

struct RenderSystem<'a>(&'a mut Context, f64);

impl<'a> System<'a> for RenderSystem<'a> {
	type SystemData = (
		ReadStorage<'a, Position>,
		Read<'a, EntitiesCount>,
		ReadStorage<'a, Mesh>,
	);

	fn run(&mut self, (position, entities_count, mesh): Self::SystemData) {
		use specs::Join;

		for (position, mesh) in (&position, &mesh).join() {
			graphics::draw(
				self.0,
				&mesh.0,
				graphics::DrawParam::default().dest(Point2::from(position.0)),
			)
			.unwrap();
		}
		let text = graphics::Text::new(format!("Entities: {} FPS: {}", entities_count.0, self.1));

		graphics::draw(
			self.0,
			&text,
			graphics::DrawParam::default().dest(Point2::new(10.0, 10.0)),
		)
		.unwrap();
	}
}

#[derive(Component)]
struct Mesh(graphics::Mesh);

impl Mesh {
	pub fn new(context: &mut Context) -> Mesh {
		let mesh = graphics::MeshBuilder::new()
			.circle(
				graphics::DrawMode::fill(),
				Point2::new(0.0, 0.0),
				3.0,
				0.001,
				graphics::WHITE,
			)
			.build(context)
			.unwrap();

		Mesh(mesh)
	}
}

struct MoveSystem(f32);

impl<'a> System<'a> for MoveSystem {
	type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

	fn run(&mut self, (velocity, mut position): Self::SystemData) {
		use specs::rayon::prelude::*;
		use specs::ParJoin;

		(&velocity, &mut position)
			.par_join()
			.for_each(|(velocity, position)| {
				position.0 += velocity.velocity * self.0;
			});
	}
}

struct EatSystem;

impl<'a> System<'a> for EatSystem {
	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, Size>,
		Entities<'a>,
		WriteStorage<'a, GrowBy>,
	);

	fn run(&mut self, (position, size, entities, mut grow_by): Self::SystemData) {
		use specs::Join;

		for (my_position, my_size, me, grow_by) in
			(&position, &size, &entities, &mut grow_by).join()
		{
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
	type SystemData = (WriteStorage<'a, GrowBy>, WriteStorage<'a, Size>);

	fn run(&mut self, (mut grow_by, mut size): Self::SystemData) {
		use specs::Join;

		for (grow_by, size) in (&mut grow_by, &mut size).join() {
			size.0 += grow_by.0;
			grow_by.reset();
		}
	}
}

struct IncrementEntityCountSystem;

impl<'a> System<'a> for IncrementEntityCountSystem {
	type SystemData = (Write<'a, EntitiesCount>);

	fn run(&mut self, mut entities_count: Self::SystemData) {
		entities_count.0 += 1;
	}
}

struct BounceOffWallsSystem(f32, f32);

impl<'a> System<'a> for BounceOffWallsSystem {
	type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Velocity>);

	fn run(&mut self, (mut position, mut velocity): Self::SystemData) {
		use specs::rayon::prelude::*;
		use specs::ParJoin;

		(&mut position, &mut velocity)
			.par_join()
			.for_each(|(position, velocity)| {
				if position.0.x > self.0 {
					velocity.velocity.x *= -1.0;
					position.0.x = self.0
				} else if position.0.x < 0.0 {
					velocity.velocity.x *= -1.0;
					position.0.x = 0.0
				}

				if position.0.y > self.1 {
					velocity.velocity.y *= -1.0;
					position.0.y = self.1;
				} else if position.0.y < 0.0 {
					velocity.velocity.y *= -1.0;
					position.0.y = 0.0
				}
			});
	}
}
