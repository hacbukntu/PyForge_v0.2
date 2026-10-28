fn main() -> eframe::Result<()> {
    // El tamaño inicial deja visibles jerarquía, viewport, inspector y consola.
    // Los límites mínimos evitan que una ventana pequeña oculte las herramientas.
    let options = eframe::NativeOptions {
        // Glow evita el fallo observado en Mesa/Vulkan donde cada texto se veía
        // como una barra sólida aunque el viewport se dibujara correctamente.
        renderer: eframe::Renderer::Glow,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 620.0])
            .with_icon(editor_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "PyForge Editor 3D",
        options,
        Box::new(|_| Box::<pyforge_engine3d::editor::EditorApp>::default()),
    )
}

/// Decodifica el mismo rayo amarillo/negro usado por la web y los archivos ST.
/// El PNG queda incrustado en el binario, por lo que el icono también funciona
/// cuando el editor se instala fuera del repositorio.
fn editor_icon() -> eframe::egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/pyforge-editor3d.png"))
        .expect("el icono oficial incluido debe ser un PNG válido")
        .into_rgba8();
    let (width, height) = image.dimensions();
    eframe::egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
