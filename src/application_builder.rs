use legion::{
    systems::{Builder, ParallelRunnable},
    Resources, World,
};

use crate::application::Application;

pub struct ApplicationBuilder {
    scheduler_builder: Builder,
}

impl ApplicationBuilder {
    pub fn add_system<T: ParallelRunnable + 'static>(&mut self, system: T) {
        self.scheduler_builder.add_system(system);
    }

    pub fn build(&mut self) -> Application {
        let schedule = self.scheduler_builder.build();

        Application {
            world: World::default(),
            resources: Resources::default(),
            schedule,
            data: None,
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        let scheduler_builder: Builder = Builder::default();

        ApplicationBuilder { scheduler_builder }
    }
}
