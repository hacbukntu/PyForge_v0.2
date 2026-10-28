use pyforge_engine3d::{BackendPreference, Renderer};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
fn main() {
    if let Err(e) = run() {
        eprintln!("pyforge-engine3d: {e}");
        std::process::exit(1)
    }
}
fn run() -> Result<(), String> {
    if let Some(argument) = std::env::args().nth(1) {
        if argument == "--version" {
            println!("pyforge-engine3d 0.1.0");
            return Ok(());
        }
        if argument == "--help" || argument == "-h" {
            println!("USO: pyforge-engine3d [--vulkan|--opengl]\nAbre la demostración GPU nativa.");
            return Ok(());
        }
    }
    let preference = match std::env::args().nth(1).as_deref() {
        Some("--vulkan") => BackendPreference::Vulkan,
        Some("--opengl") => BackendPreference::OpenGl,
        _ => BackendPreference::Auto,
    };
    let events = EventLoop::new().map_err(|e| e.to_string())?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("PyForge ST — Motor 3D")
            .build(&events)
            .map_err(|e| e.to_string())?,
    );
    // Algunos controladores antiguos hacen panic dentro de wgpu al crear la superficie GL.
    // Lo convertimos en diagnóstico: un backend opcional nunca debe tumbar el IDE sin contexto.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let creation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pollster::block_on(Renderer::new(window.clone(), preference))
    }));
    std::panic::set_hook(previous_hook);
    let mut renderer = creation.map_err(|payload| {
        let detail = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|x| x.to_string()))
            .unwrap_or_else(|| "fallo interno sin mensaje".into());
        format!("falló la inicialización GPU: {detail}")
    })??;
    eprintln!("Backend activo: {:?}", renderer.backend);
    events
        .run(move |event, target| match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(s) => renderer.resize(s.width, s.height),
                WindowEvent::RedrawRequested => {
                    if let Err(e) = renderer.render() {
                        eprintln!("{e}");
                        target.exit()
                    }
                }
                _ => {}
            },
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        })
        .map_err(|e| e.to_string())
}
