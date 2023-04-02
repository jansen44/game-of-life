use egui_winit::winit;

pub struct State {
    pub running: bool,
    pub cell_scale_factor: f32,
    pub cell_offset: f32,
    pub clear_color_r: f64,
    pub clear_color_g: f64,
    pub clear_color_b: f64,
}

pub struct GuiCtx {
    ctx: egui::Context,
    renderer: egui_wgpu::renderer::Renderer,
    state: egui_winit::State,
    screen_descriptor: egui_wgpu::renderer::ScreenDescriptor,
}

impl GuiCtx {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        device: &wgpu::Device,
        surface_cfg: &wgpu::SurfaceConfiguration,
        window: &winit::window::Window,
    ) -> Self {
        let size = window.inner_size();
        Self {
            ctx: egui::Context::default(),
            renderer: egui_wgpu::renderer::Renderer::new(&device, surface_cfg.format, None, 1),
            state: egui_winit::State::new(event_loop),
            screen_descriptor: egui_wgpu::renderer::ScreenDescriptor {
                pixels_per_point: window.scale_factor() as f32,
                size_in_pixels: [size.width, size.height],
            },
        }
    }

    pub fn build_ui(
        &mut self,
        state: &mut State,
        window: &winit::window::Window,
    ) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);

        let full_output = self.ctx.run(raw_input, |ctx| {
            egui::Window::new("Clear Color").show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut state.clear_color_r, 0.0..=1.0));
                ui.add(egui::Slider::new(&mut state.clear_color_g, 0.0..=1.0));
                ui.add(egui::Slider::new(&mut state.clear_color_b, 0.0..=1.0));
            });

            egui::Window::new("Cell").show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut state.cell_scale_factor, 5.0..=100.0));
                ui.add(egui::Slider::new(&mut state.cell_offset, 1.0..=100.0));
            });
        });

        let platform_output = full_output.platform_output.clone();

        self.state
            .handle_platform_output(window, &self.ctx, platform_output);

        full_output
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        output: egui::FullOutput,
    ) -> Vec<egui::ClippedPrimitive> {
        let clipped_primitives = self.ctx.tessellate(output.shapes);

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &clipped_primitives,
            &self.screen_descriptor,
        );

        for (tex_id, img_delta) in output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, tex_id, &img_delta);
        }

        for tex_id in output.textures_delta.free {
            self.renderer.free_texture(&tex_id);
        }

        clipped_primitives
    }

    pub fn renderer(&self) -> &egui_wgpu::Renderer {
        &self.renderer
    }

    pub fn screen_descriptor(&self) -> &egui_wgpu::renderer::ScreenDescriptor {
        &self.screen_descriptor
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        let r = self.state.on_event(&self.ctx, event);
        r.consumed
    }
}
