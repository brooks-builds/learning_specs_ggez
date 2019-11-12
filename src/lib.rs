use bbggez::ggez::{
	Context, 
	GameResult, 
	graphics,
	nalgebra::{Vector2, Point2},
};
use bbggez::ggez::event::EventHandler;
use specs::{Builder, Component, ReadStorage, System, VecStorage, World, WorldExt, RunNow, WriteStorage};

pub struct Game {
	world: World,
}

impl Game {
	pub fn new(context: &mut Context) -> Game {
		let mut world = World::new();

		world.register::<Position>();
		world.register::<Size>();
		world.register::<Circle>();

		world.create_entity()
			.with(Position::new(100.0, 100.0))
			.with(Size::new(0.0))
			.with(Circle::new(context))
			.build();

			Game {
				world,
			}
		}
		
		fn handle_window_size_change(&self, context: &mut Context, (width, height): (f32, f32)) -> GameResult<()> {
			graphics::set_screen_coordinates(context, graphics::Rect::new(0.0, 0.0, width, height))
		}
	}
	
	impl EventHandler for Game {
		fn update(&mut self, context: &mut Context) -> GameResult<()> {
			let arena_size = graphics::drawable_size(context);
			self.handle_window_size_change(context, arena_size)?;

			let mut grow_circle = GrowCircleSystem;
			grow_circle.run_now(&self.world);
			
			self.world.maintain();		

		Ok(())
	}

	fn draw(&mut self, context: &mut Context) -> GameResult<()> {
		graphics::clear(context, graphics::BLACK);

		let mut draw_circle = DrawCircleSystem (context);
		draw_circle.run_now(&self.world);

		graphics::present(context)
	}
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Position(Vector2<f32>);

impl Position {
	pub fn new(x: f32, y: f32) -> Position {
		Position (Vector2::new(x, y))
	}
}

#[derive(Component)]
#[storage(VecStorage)]
struct Size(f32);

impl Size {
	pub fn new(size: f32) -> Size {
		Size (size)
	}
}

#[derive(Component)]
#[storage(VecStorage)]
struct Circle(graphics::Mesh);

impl Circle {
	pub fn new(context: &mut Context) -> Circle {
		let mesh = graphics::MeshBuilder::new()
			.circle(
				graphics::DrawMode::fill(), 
				Point2::new(0.0, 0.0), 
				1.0, 
				0.00000000001, 
				graphics::WHITE,
			)
			.build(context)
			.unwrap();
		
			Circle (mesh)
	}
}

struct DrawCircleSystem<'a>(&'a mut Context);

impl<'a> System<'a> for DrawCircleSystem<'a> {
    type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, Size>,
		ReadStorage<'a, Circle>
	);

    fn run(&mut self, (position, size, mesh): Self::SystemData) {
		use specs::Join;
		
		
        for (position, size, mesh) in (&position, &size, &mesh).join() {
			graphics::draw(
				self.0, 
				&mesh.0, 
				graphics::DrawParam::default()
					.dest(Point2::from(position.0))
					.scale([size.0, size.0])
			).unwrap();
        }
    }
}

struct GrowCircleSystem;

impl<'a> System<'a> for GrowCircleSystem {
	type SystemData = (
		WriteStorage<'a, Size>
	);

	fn run(&mut self, mut size: Self::SystemData) {
		use specs::Join;

		for size in (&mut size).join() {
			size.0 += 0.1;
		}
	}
}