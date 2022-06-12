use application_builder::ApplicationBuilder;
use legion::system;
use winit::window::WindowBuilder;

mod application;
mod application_builder;

pub fn run() {
    let mut builder = ApplicationBuilder::default();
    builder.add_system(test_system_system());

    let mut app = builder.build();
    app.world.push((Test { value: 0 }, ()));

    app.start(WindowBuilder::new().with_title("Test"));
}

#[system(for_each)]
fn test_system(test: &mut Test) {
    test.value += 1;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Test {
    pub value: i32,
}
