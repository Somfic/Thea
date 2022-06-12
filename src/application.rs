use std::sync::Arc;

use legion::{
    storage::IntoComponentSource,
    systems::{Builder, ParallelRunnable},
    Entity, Resources, Schedule, World,
};
use winit::window::WindowBuilder;

pub struct Application {
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
    pub data: Option<ApplicationData>,
}

pub struct ApplicationData {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    imgui_routine: rend3_imgui::ImguiRenderRoutine,
    frame_start: instant::Instant,
}

impl Application {
    pub fn start(self, window_builder: WindowBuilder) {
        rend3_framework::start(self, window_builder);
    }
}

impl rend3_framework::App for Application {
    const HANDEDNESS: rend3::types::Handedness = rend3::types::Handedness::Left;

    fn sample_count(&self) -> rend3::types::SampleCount {
        rend3::types::SampleCount::One
    }

    fn setup(
        &mut self,
        window: &winit::window::Window,
        renderer: &std::sync::Arc<rend3::Renderer>,
        routines: &std::sync::Arc<rend3_framework::DefaultRoutines>,
        surface_format: rend3::types::TextureFormat,
    ) {
        println!("setup");

        // Set up imgui
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Create the imgui render routine
        let imgui_routine =
            rend3_imgui::ImguiRenderRoutine::new(renderer, &mut imgui, surface_format);

        self.data = Some(ApplicationData {
            imgui,
            platform,
            imgui_routine,
            frame_start: instant::Instant::now(),
        });
    }

    fn handle_event(
        &mut self,
        window: &winit::window::Window,
        renderer: &std::sync::Arc<rend3::Renderer>,
        routines: &std::sync::Arc<rend3_framework::DefaultRoutines>,
        base_rendergraph: &rend3_routine::base::BaseRenderGraph,
        surface: Option<&std::sync::Arc<rend3::types::Surface>>,
        resolution: glam::UVec2,
        event: rend3_framework::Event<'_, ()>,
        control_flow: impl FnOnce(winit::event_loop::ControlFlow),
    ) {
        let data = self.data.as_mut().unwrap();
        data.platform
            .handle_event(data.imgui.io_mut(), window, &event);

        match event {
            // Close button was clicked, we should close.
            rend3_framework::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                control_flow(winit::event_loop::ControlFlow::Exit);
            }

            winit::event::Event::MainEventsCleared => {
                let now = instant::Instant::now();
                let delta = now - data.frame_start;

                // todo: push delta into resources

                data.frame_start = now;
                data.imgui.io_mut().update_delta_time(delta);

                window.request_redraw();
            }

            winit::event::Event::RedrawRequested(..) => {
                // Prepare UI
                data.platform
                    .prepare_frame(data.imgui.io_mut(), window)
                    .expect("Could not prepare UI frame");
                let ui = data.imgui.frame();

                // todo: put ui into resources
                ui.show_demo_window(&mut true);

                // Run ECS updates
                self.schedule.execute(&mut self.world, &mut self.resources);

                // Prepare for rendering
                data.platform.prepare_render(&ui, window);

                // Get a frame
                let frame = rend3::util::output::OutputFrame::Surface {
                    surface: Arc::clone(surface.unwrap()),
                };
                // Ready up the renderer
                let (cmd_bufs, ready) = renderer.ready();

                // Lock the routines
                let pbr_routine = rend3_framework::lock(&routines.pbr);
                let tonemapping_routine = rend3_framework::lock(&routines.tonemapping);

                // Build a rendergraph
                let mut graph = rend3::graph::RenderGraph::new();

                // Add the default rendergraph without a skybox
                base_rendergraph.add_to_graph(
                    &mut graph,
                    &ready,
                    &pbr_routine,
                    None,
                    &tonemapping_routine,
                    resolution,
                    rend3::types::SampleCount::One,
                    glam::Vec4::ZERO,
                );

                // Add UI to the graph
                let surface = graph.add_surface_texture();
                data.imgui_routine
                    .add_to_graph(&mut graph, ui.render(), surface);

                // Dispatch a render using the built up rendergraph
                graph.execute(renderer, frame, cmd_bufs, &ready);

                control_flow(winit::event_loop::ControlFlow::Poll);
            }

            _ => {}
        }
    }
}
