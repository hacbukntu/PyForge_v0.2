//! Editor visual 3D y conexión bidireccional con escenas declaradas en ST.
//!
//! La interfaz conserva el documento [`Scene`] como única fuente de verdad. La
//! jerarquía, el inspector y los gizmos mutan ese documento con historial. Un
//! archivo `.st` puede reconstruirlo mediante [`crate::st_scene`] y el editor
//! puede exportar cualquier cambio visual de vuelta a llamadas ST legibles.

#[cfg(any(feature = "editor", test))]
use crate::scene::{Transform, SCENE_VERSION};
use crate::{
    editor_viewport::ViewportState,
    scene::{Entity, Scene},
};

/// Estado independiente de la interfaz: selección, historial y cámara.
#[derive(Default)]
pub struct EditorState {
    pub selected: Option<usize>,
    undo: Vec<Scene>,
    redo: Vec<Scene>,
    pub viewport: ViewportState,
}

impl EditorState {
    /// Guarda una instantánea previa. El límite evita crecimiento ilimitado de memoria.
    pub fn checkpoint(&mut self, scene: &Scene) {
        self.undo.push(scene.clone());
        self.redo.clear();
        if self.undo.len() > 128 {
            self.undo.remove(0);
        }
    }

    /// Añade una entidad solo después de validar el documento propuesto.
    pub fn add(&mut self, scene: &mut Scene, entity: Entity) -> Result<(), String> {
        let mut proposed = scene.clone();
        proposed.entities.push(entity);
        proposed.validate()?;
        self.checkpoint(scene);
        *scene = proposed;
        self.selected = Some(scene.entities.len() - 1);
        Ok(())
    }

    /// Duplica la selección y desplaza la copia para que no quede oculta.
    pub fn duplicate_selected(&mut self, scene: &mut Scene) -> Result<bool, String> {
        let Some(index) = self.selected else {
            return Ok(false);
        };
        let Some(source) = scene.entities.get(index) else {
            self.selected = None;
            return Ok(false);
        };
        let mut duplicate = source.clone();
        duplicate.name = unique_name(scene, &format!("{} copia", source.name));
        duplicate.transform.position[0] += 1.0;
        self.add(scene, duplicate)?;
        Ok(true)
    }

    /// Elimina la selección y elige un vecino válido.
    pub fn delete_selected(&mut self, scene: &mut Scene) -> bool {
        let Some(index) = self.selected.filter(|index| *index < scene.entities.len()) else {
            self.selected = None;
            return false;
        };
        self.checkpoint(scene);
        scene.entities.remove(index);
        self.selected = if scene.entities.is_empty() {
            None
        } else {
            Some(index.min(scene.entities.len() - 1))
        };
        true
    }

    /// Reemplaza el documento como una sola operación reversible.
    pub fn replace(&mut self, scene: &mut Scene, replacement: Scene) -> Result<(), String> {
        replacement.validate()?;
        self.checkpoint(scene);
        *scene = replacement;
        self.selected = (!scene.entities.is_empty()).then_some(0);
        Ok(())
    }

    /// Recupera el documento anterior y ajusta la selección.
    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        if let Some(old) = self.undo.pop() {
            self.redo.push(scene.clone());
            *scene = old;
            self.normalize_selection(scene);
            true
        } else {
            false
        }
    }

    /// Reaplica el documento retirado por `undo`.
    pub fn redo(&mut self, scene: &mut Scene) -> bool {
        if let Some(new) = self.redo.pop() {
            self.undo.push(scene.clone());
            *scene = new;
            self.normalize_selection(scene);
            true
        } else {
            false
        }
    }

    pub fn history_counts(&self) -> (usize, usize) {
        (self.undo.len(), self.redo.len())
    }

    fn normalize_selection(&mut self, scene: &Scene) {
        if self
            .selected
            .is_some_and(|index| index >= scene.entities.len())
        {
            self.selected = scene.entities.len().checked_sub(1);
        }
    }
}

fn unique_name(scene: &Scene, base: &str) -> String {
    if !scene.entities.iter().any(|entity| entity.name == base) {
        return base.to_string();
    }
    for suffix in 2..=10_000 {
        let candidate = format!("{base} {suffix}");
        if !scene.entities.iter().any(|entity| entity.name == candidate) {
            return candidate;
        }
    }
    format!("{base} {}", scene.entities.len() + 1)
}

#[cfg(feature = "editor")]
mod app {
    use super::*;
    use crate::{
        editor_viewport::show_viewport,
        st_scene::{compile_and_run_scene_script, save_source_atomic, scene_to_st},
    };
    use eframe::egui::{self, Color32, DragValue, FontId, Key, RichText, ScrollArea, TextEdit};
    use glam::{EulerRot, Quat};
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::mpsc::{self, Receiver, TryRecvError},
        thread,
        time::{Duration, Instant, SystemTime},
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum BottomTab {
        Console,
        Script,
        Help,
    }

    /// Aplicación completa del editor. Los campos de ruta son visibles para que
    /// funcione igual desde el repositorio, un `.deb` o una instalación manual.
    pub struct EditorApp {
        pub scene: Scene,
        pub state: EditorState,
        status: String,
        scene_path: String,
        script_path: String,
        compiler_path: String,
        search: String,
        console: Vec<String>,
        source: String,
        source_dirty: bool,
        scene_dirty: bool,
        show_hierarchy: bool,
        show_inspector: bool,
        show_bottom: bool,
        bottom_tab: BottomTab,
        auto_reload: bool,
        last_script_modified: Option<SystemTime>,
        next_auto_check: Instant,
        script_job: Option<Receiver<Result<Scene, String>>>,
        style_ready: bool,
    }

    impl Default for EditorApp {
        fn default() -> Self {
            let cube = Entity {
                name: "Cubo".into(),
                transform: Transform {
                    position: [0.0, 0.5, 0.0],
                    ..Default::default()
                },
                mesh: Some("cubo".into()),
                material: Some("material_base".into()),
            };
            let scene = Scene {
                version: SCENE_VERSION,
                name: "escena_nueva".into(),
                entities: vec![cube],
            };
            let source = scene_to_st(&scene).unwrap_or_default();
            Self {
                scene,
                state: EditorState {
                    selected: Some(0),
                    ..Default::default()
                },
                status: "Guía abierta: identifica cada panel y después carga tu escena ST".into(),
                scene_path: "escena.stscene".into(),
                script_path: "escena_editor.st".into(),
                compiler_path: find_compiler(),
                search: String::new(),
                console: vec![
                    "PyForge Editor 3D iniciado".into(),
                    "La escena visual puede exportarse y reconstruirse desde ST.".into(),
                ],
                source,
                source_dirty: true,
                scene_dirty: false,
                show_hierarchy: true,
                show_inspector: true,
                show_bottom: true,
                // La primera pantalla explica los controles; después el usuario
                // puede cambiar libremente a Consola o Script de escena.
                bottom_tab: BottomTab::Help,
                auto_reload: false,
                last_script_modified: None,
                next_auto_check: Instant::now(),
                script_job: None,
                style_ready: false,
            }
        }
    }

    impl EditorApp {
        fn configure_style(&mut self, ctx: &egui::Context) {
            if self.style_ready {
                return;
            }
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Color32::from_rgb(19, 21, 26);
            visuals.window_fill = Color32::from_rgb(22, 24, 30);
            visuals.extreme_bg_color = Color32::from_rgb(12, 14, 18);
            visuals.selection.bg_fill = Color32::from_rgb(184, 137, 16);
            visuals.selection.stroke.color = Color32::from_rgb(255, 211, 62);
            visuals.widgets.active.bg_fill = Color32::from_rgb(93, 72, 18);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(53, 56, 65);
            ctx.set_visuals(visuals);
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(10.0, 5.0);
            ctx.set_style(style);
            self.style_ready = true;
        }

        fn log(&mut self, message: impl Into<String>) {
            let message = message.into();
            self.status = message.clone();
            self.console.push(message);
            if self.console.len() > 300 {
                self.console.drain(0..self.console.len() - 300);
            }
        }

        fn new_scene(&mut self) {
            let scene = Scene {
                version: SCENE_VERSION,
                name: "escena_nueva".into(),
                entities: Vec::new(),
            };
            if let Err(error) = self.state.replace(&mut self.scene, scene) {
                self.log(error);
                return;
            }
            self.scene_dirty = true;
            self.log("Nueva escena creada");
        }

        fn open_scene(&mut self) {
            match Scene::load(Path::new(&self.scene_path)) {
                Ok(scene) => {
                    if let Err(error) = self.state.replace(&mut self.scene, scene) {
                        self.log(error);
                    } else {
                        self.scene_dirty = false;
                        self.log(format!("Escena abierta: {}", self.scene_path));
                    }
                }
                Err(error) => self.log(error),
            }
        }

        fn save_scene(&mut self) {
            match self.scene.save_atomic(Path::new(&self.scene_path)) {
                Ok(()) => {
                    self.scene_dirty = false;
                    self.log(format!("Escena guardada: {}", self.scene_path));
                }
                Err(error) => self.log(error),
            }
        }

        fn add_primitive(&mut self, kind: &str) {
            let number = self.scene.entities.len() + 1;
            let (label, material, position, scale) = match kind {
                "plano" => ("Plano", "suelo", [0.0, -0.1, 0.0], [6.0, 0.2, 6.0]),
                "luz" => ("Luz", "energia", [2.0, 3.0, 2.0], [0.25, 0.25, 0.25]),
                "camara" => ("Cámara", "cristal", [0.0, 2.0, 5.0], [0.5, 0.35, 0.8]),
                _ => (
                    "Cubo",
                    "material_base",
                    [number as f32 * 1.25, 0.5, 0.0],
                    [1.0, 1.0, 1.0],
                ),
            };
            let entity = Entity {
                name: unique_name(&self.scene, label),
                transform: Transform {
                    position,
                    scale,
                    ..Default::default()
                },
                mesh: Some(kind.into()),
                material: Some(material.into()),
            };
            match self.state.add(&mut self.scene, entity) {
                Ok(()) => {
                    self.scene_dirty = true;
                    self.log(format!("{label} añadido"));
                }
                Err(error) => self.log(error),
            }
        }

        fn duplicate(&mut self) {
            match self.state.duplicate_selected(&mut self.scene) {
                Ok(true) => {
                    self.scene_dirty = true;
                    self.log("Entidad duplicada");
                }
                Ok(false) => self.log("Selecciona una entidad para duplicarla"),
                Err(error) => self.log(error),
            }
        }

        fn delete(&mut self) {
            if self.state.delete_selected(&mut self.scene) {
                self.scene_dirty = true;
                self.log("Entidad eliminada");
            } else {
                self.log("Selecciona una entidad para eliminarla");
            }
        }

        fn undo(&mut self) {
            if self.state.undo(&mut self.scene) {
                self.scene_dirty = true;
                self.log("Cambio deshecho");
            }
        }

        fn redo(&mut self) {
            if self.state.redo(&mut self.scene) {
                self.scene_dirty = true;
                self.log("Cambio rehecho");
            }
        }

        fn frame_selected(&mut self) {
            if let Some(entity) = self
                .state
                .selected
                .and_then(|index| self.scene.entities.get(index))
            {
                self.state.viewport.camera.frame(&entity.transform);
                self.log(format!("Vista centrada en {}", entity.name));
            }
        }

        fn load_source(&mut self) {
            let path = Path::new(&self.script_path);
            match fs::read_to_string(path) {
                Ok(source) if source.len() <= 4 * 1024 * 1024 => {
                    self.source = source;
                    self.source_dirty = false;
                    self.last_script_modified = modified(path);
                    self.bottom_tab = BottomTab::Script;
                    self.show_bottom = true;
                    self.log(format!("Script abierto: {}", self.script_path));
                }
                Ok(_) => self.log("El script ST supera 4 MiB"),
                Err(error) => self.log(format!("Abrir script ST: {error}")),
            }
        }

        fn save_source(&mut self) -> bool {
            match save_source_atomic(Path::new(&self.script_path), &self.source) {
                Ok(()) => {
                    self.source_dirty = false;
                    self.last_script_modified = modified(Path::new(&self.script_path));
                    self.log(format!("Script guardado: {}", self.script_path));
                    true
                }
                Err(error) => {
                    self.log(error);
                    false
                }
            }
        }

        fn export_scene(&mut self) {
            match scene_to_st(&self.scene) {
                Ok(source) => {
                    self.source = source;
                    self.source_dirty = true;
                    self.bottom_tab = BottomTab::Script;
                    self.show_bottom = true;
                    if self.save_source() {
                        self.log("Escena visual exportada a ST");
                    }
                }
                Err(error) => self.log(error),
            }
        }

        fn apply_source(&mut self) {
            if self.script_job.is_some() {
                self.log("Ya hay una compilación ST en curso");
                return;
            }
            if !self.save_source() {
                return;
            }
            let compiler = PathBuf::from(&self.compiler_path);
            let source = PathBuf::from(&self.script_path);
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || {
                let _ = sender.send(compile_and_run_scene_script(&compiler, &source));
            });
            self.script_job = Some(receiver);
            self.log("Compilando y ejecutando la escena ST…");
        }

        fn poll_script_job(&mut self) {
            let result = match self.script_job.as_ref().map(Receiver::try_recv) {
                Some(Ok(result)) => Some(result),
                Some(Err(TryRecvError::Disconnected)) => {
                    Some(Err("el trabajo ST terminó sin respuesta".into()))
                }
                _ => None,
            };
            let Some(result) = result else {
                return;
            };
            self.script_job = None;
            match result {
                Ok(scene) => match self.state.replace(&mut self.scene, scene) {
                    Ok(()) => {
                        self.scene_dirty = true;
                        self.log("Escena reconstruida correctamente desde ST");
                    }
                    Err(error) => self.log(error),
                },
                Err(error) => self.log(error),
            }
        }

        fn poll_auto_reload(&mut self) {
            if !self.auto_reload || self.source_dirty || self.script_job.is_some() {
                return;
            }
            if Instant::now() < self.next_auto_check {
                return;
            }
            self.next_auto_check = Instant::now() + Duration::from_millis(750);
            let current = modified(Path::new(&self.script_path));
            if current.is_some()
                && self.last_script_modified.is_some()
                && current != self.last_script_modified
            {
                self.load_source();
                self.apply_source();
            } else if self.last_script_modified.is_none() {
                self.last_script_modified = current;
            }
        }

        fn handle_shortcuts(&mut self, ctx: &egui::Context) {
            let typing = ctx.wants_keyboard_input();
            let (save, undo, redo, duplicate, delete, focus) = ctx.input(|input| {
                let command = input.modifiers.command;
                (
                    command && input.key_pressed(Key::S),
                    command && input.key_pressed(Key::Z) && !input.modifiers.shift,
                    (command && input.key_pressed(Key::Y))
                        || (command && input.modifiers.shift && input.key_pressed(Key::Z)),
                    command && input.key_pressed(Key::D),
                    input.key_pressed(Key::Delete),
                    input.key_pressed(Key::F),
                )
            });
            if save {
                if self.bottom_tab == BottomTab::Script && self.show_bottom {
                    self.save_source();
                } else {
                    self.save_scene();
                }
            }
            if !typing {
                if undo {
                    self.undo();
                }
                if redo {
                    self.redo();
                }
                if duplicate {
                    self.duplicate();
                }
                if delete {
                    self.delete();
                }
                if focus {
                    self.frame_selected();
                }
            }
        }

        fn show_menu(&mut self, ctx: &egui::Context) {
            egui::TopBottomPanel::top("menu_principal")
                .exact_height(30.0)
                .show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("Archivo", |ui| {
                            if ui.button("＋ Nueva escena").clicked() {
                                self.new_scene();
                                ui.close_menu();
                            }
                            if ui.button("▣ Abrir .stscene").clicked() {
                                self.open_scene();
                                ui.close_menu();
                            }
                            if ui.button("◆ Guardar .stscene    Ctrl+S").clicked() {
                                self.save_scene();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("ST Abrir script").clicked() {
                                self.load_source();
                                ui.close_menu();
                            }
                            if ui.button("ST Exportar escena").clicked() {
                                self.export_scene();
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Editar", |ui| {
                            if ui.button("↶ Deshacer    Ctrl+Z").clicked() {
                                self.undo();
                                ui.close_menu();
                            }
                            if ui.button("↷ Rehacer    Ctrl+Y").clicked() {
                                self.redo();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("⧉ Duplicar    Ctrl+D").clicked() {
                                self.duplicate();
                                ui.close_menu();
                            }
                            if ui.button("⌫ Eliminar    Supr").clicked() {
                                self.delete();
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Crear", |ui| {
                            for (icon, label, kind) in [
                                ("◼", "Cubo", "cubo"),
                                ("▱", "Plano", "plano"),
                                ("☀", "Luz", "luz"),
                                ("◉", "Cámara", "camara"),
                            ] {
                                if ui.button(format!("{icon} {label}")).clicked() {
                                    self.add_primitive(kind);
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.menu_button("Ventana", |ui| {
                            ui.checkbox(&mut self.show_hierarchy, "Jerarquía");
                            ui.checkbox(&mut self.show_inspector, "Inspector");
                            ui.checkbox(&mut self.show_bottom, "Consola y ST");
                        });
                        ui.menu_button("Ayuda", |ui| {
                            if ui.button("Controles del editor").clicked() {
                                self.bottom_tab = BottomTab::Help;
                                self.show_bottom = true;
                                ui.close_menu();
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("PYFORGE · ST")
                                    .color(Color32::from_rgb(255, 205, 52))
                                    .strong(),
                            );
                        });
                    });
                });
        }

        fn show_toolbar(&mut self, ctx: &egui::Context) {
            egui::TopBottomPanel::top("barra_herramientas")
                .exact_height(44.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if icon_button(ui, "[+] Nuevo", "Crear una escena vacía").clicked() {
                            self.new_scene();
                        }
                        if icon_button(ui, "[O] Abrir", "Abrir el archivo .stscene indicado")
                            .clicked()
                        {
                            self.open_scene();
                        }
                        if icon_button(ui, "[S] Guardar", "Guardar el archivo .stscene").clicked() {
                            self.save_scene();
                        }
                        ui.separator();
                        if icon_button(ui, "< Deshacer", "Volver al cambio anterior").clicked() {
                            self.undo();
                        }
                        if icon_button(ui, "> Rehacer", "Recuperar el cambio deshecho").clicked() {
                            self.redo();
                        }
                        if icon_button(ui, "[D] Duplicar", "Duplicar la entidad seleccionada")
                            .clicked()
                        {
                            self.duplicate();
                        }
                        if icon_button(ui, "[X] Eliminar", "Eliminar la entidad seleccionada")
                            .clicked()
                        {
                            self.delete();
                        }
                        ui.separator();
                        if ui
                            .add_enabled(
                                self.script_job.is_none(),
                                egui::Button::new("▶ Aplicar ST"),
                            )
                            .on_hover_text(
                                "Compilar el script con pyforgec y reconstruir la escena",
                            )
                            .clicked()
                        {
                            self.apply_source();
                        }
                        if ui
                            .button("⇄ Exportar ST")
                            .on_hover_text("Guardar el estado visual como código ST")
                            .clicked()
                        {
                            self.export_scene();
                        }
                        if self.script_job.is_some() {
                            ui.spinner();
                        }
                        ui.separator();
                        ui.label("Escena:");
                        ui.add(TextEdit::singleline(&mut self.scene_path).desired_width(170.0));
                        let dirty = if self.scene_dirty {
                            "● sin guardar"
                        } else {
                            "✓ guardada"
                        };
                        ui.label(RichText::new(dirty).color(if self.scene_dirty {
                            Color32::from_rgb(255, 195, 55)
                        } else {
                            Color32::from_rgb(92, 205, 138)
                        }));
                    });
                });
        }

        fn show_hierarchy(&mut self, ctx: &egui::Context) {
            if !self.show_hierarchy {
                return;
            }
            egui::SidePanel::left("jerarquia")
                .resizable(true)
                .default_width(230.0)
                .min_width(170.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Jerarquía");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.menu_button("+ Agregar", |ui| {
                                for (label, kind) in [
                                    ("◼ Cubo", "cubo"),
                                    ("▱ Plano", "plano"),
                                    ("☀ Luz", "luz"),
                                    ("◉ Cámara", "camara"),
                                ] {
                                    if ui.button(label).clicked() {
                                        self.add_primitive(kind);
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                    });
                    let before = self.scene.clone();
                    if ui
                        .add(
                            TextEdit::singleline(&mut self.scene.name)
                                .hint_text("Nombre de escena"),
                        )
                        .changed()
                    {
                        self.state.checkpoint(&before);
                        self.scene_dirty = true;
                    }
                    ui.add(
                        TextEdit::singleline(&mut self.search)
                            .hint_text("⌕ Buscar entidad…")
                            .desired_width(f32::INFINITY),
                    );
                    ui.separator();
                    let needle = self.search.to_lowercase();
                    ScrollArea::vertical().show(ui, |ui| {
                        for (index, entity) in self.scene.entities.iter().enumerate() {
                            if !needle.is_empty() && !entity.name.to_lowercase().contains(&needle) {
                                continue;
                            }
                            let icon = match entity.mesh.as_deref() {
                                Some("plano") => "▱",
                                Some("luz") => "☀",
                                Some("camara") => "◉",
                                _ => "◼",
                            };
                            let response = ui.selectable_label(
                                self.state.selected == Some(index),
                                format!("{icon}  {}", entity.name),
                            );
                            if response.clicked() {
                                self.state.selected = Some(index);
                            }
                            if response.double_clicked() {
                                self.state.selected = Some(index);
                                self.state.viewport.camera.frame(&entity.transform);
                            }
                        }
                    });
                    ui.separator();
                    ui.small(format!("{} entidades", self.scene.entities.len()));
                });
        }

        fn show_inspector(&mut self, ctx: &egui::Context) {
            if !self.show_inspector {
                return;
            }
            let before = self.scene.clone();
            let mut changed = false;
            let mut duplicate = false;
            let mut delete = false;
            egui::SidePanel::right("inspector").resizable(true).default_width(300.0).min_width(235.0).show(ctx, |ui| {
                ui.heading("Inspector");
                let Some(index) = self.state.selected else {
                    ui.separator();
                    ui.label("Selecciona una entidad en la jerarquía o en el viewport.");
                    return;
                };
                let Some(entity) = self.scene.entities.get_mut(index) else { return; };
                ui.separator();
                ui.label("Nombre");
                changed |= ui.text_edit_singleline(&mut entity.name).changed();
                ui.horizontal(|ui| {
                    if ui.button("⧉ Duplicar").clicked() { duplicate = true; }
                    if ui.button("⌫ Eliminar").clicked() { delete = true; }
                    if ui.button("F Enfocar").clicked() { self.state.viewport.camera.frame(&entity.transform); }
                });

                egui::CollapsingHeader::new("↔ Transformación").default_open(true).show(ui, |ui| {
                    ui.label("Posición");
                    changed |= vec3_control(ui, &mut entity.transform.position, 0.1, None);
                    let rotation = Quat::from_array(entity.transform.rotation).normalize();
                    let (x, y, z) = rotation.to_euler(EulerRot::XYZ);
                    let mut degrees = [x.to_degrees(), y.to_degrees(), z.to_degrees()];
                    ui.label("Rotación XYZ (grados)");
                    if vec3_control(ui, &mut degrees, 0.5, None) {
                        entity.transform.rotation = Quat::from_euler(
                            EulerRot::XYZ, degrees[0].to_radians(), degrees[1].to_radians(), degrees[2].to_radians(),
                        ).normalize().to_array();
                        changed = true;
                    }
                    ui.label("Escala");
                    changed |= vec3_control(ui, &mut entity.transform.scale, 0.05, Some((0.001, 100_000.0)));
                    if ui.button("Restablecer transformación").clicked() {
                        entity.transform = Transform::default();
                        changed = true;
                    }
                });

                egui::CollapsingHeader::new("▦ Recursos").default_open(true).show(ui, |ui| {
                    let mut mesh = entity.mesh.clone().unwrap_or_default();
                    let mut material = entity.material.clone().unwrap_or_default();
                    ui.label("Malla o tipo visual");
                    if ui.text_edit_singleline(&mut mesh).changed() {
                        entity.mesh = (!mesh.trim().is_empty()).then_some(mesh);
                        changed = true;
                    }
                    ui.label("Material");
                    if ui.text_edit_singleline(&mut material).changed() {
                        entity.material = (!material.trim().is_empty()).then_some(material);
                        changed = true;
                    }
                    ui.small("Paleta: material_base, jugador, portal, cristal, enemigo, obstaculo, suelo y energia.");
                });

                egui::CollapsingHeader::new("ST · fuente de la entidad").show(ui, |ui| {
                    ui.small("Exporta la escena para obtener esta transformación como llamada editor3d.entidad(...). Después puedes modificarla y aplicar ST.");
                    if ui.button("⇄ Abrir script ST").clicked() {
                        self.bottom_tab = BottomTab::Script;
                        self.show_bottom = true;
                    }
                });
            });
            if changed {
                self.state.checkpoint(&before);
                self.scene_dirty = true;
            }
            if duplicate {
                self.duplicate();
            }
            if delete {
                self.delete();
            }
        }

        fn show_bottom(&mut self, ctx: &egui::Context) {
            if !self.show_bottom {
                return;
            }
            #[derive(Clone, Copy)]
            enum Action {
                None,
                Load,
                Save,
                Apply,
                Export,
                Clear,
            }
            let mut action = Action::None;
            egui::TopBottomPanel::bottom("panel_inferior").resizable(true).default_height(220.0).min_height(105.0).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Console, "Consola");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Script, "ST Script de escena");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Help, "? Controles");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("×").on_hover_text("Ocultar panel").clicked() { self.show_bottom = false; }
                    });
                });
                ui.separator();
                match self.bottom_tab {
                    BottomTab::Console => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Salida del editor").strong());
                            if ui.small_button("Limpiar").clicked() { action = Action::Clear; }
                        });
                        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            for line in &self.console { ui.label(RichText::new(line).monospace()); }
                        });
                    }
                    BottomTab::Script => {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Archivo .st:");
                            ui.add(TextEdit::singleline(&mut self.script_path).desired_width(260.0));
                            if ui.button("▣ Abrir").clicked() { action = Action::Load; }
                            if ui.button("◆ Guardar").clicked() { action = Action::Save; }
                            if ui.add_enabled(self.script_job.is_none(), egui::Button::new("▶ Aplicar ST")).clicked() { action = Action::Apply; }
                            if ui.button("⇄ Exportar visual").clicked() { action = Action::Export; }
                            ui.checkbox(&mut self.auto_reload, "Recarga automática");
                            if self.source_dirty { ui.colored_label(Color32::from_rgb(255, 195, 55), "● modificado"); }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Compilador:");
                            ui.add(TextEdit::singleline(&mut self.compiler_path).desired_width(360.0));
                            ui.small("La ejecución solo ocurre al pulsar Aplicar ST o activar recarga.");
                        });
                        let response = ui.add_sized(
                            ui.available_size(),
                            TextEdit::multiline(&mut self.source).font(FontId::monospace(13.0)).code_editor().desired_width(f32::INFINITY),
                        );
                        if response.changed() { self.source_dirty = true; }
                    }
                        BottomTab::Help => {
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.label(RichText::new("Guía rápida del editor").strong().size(16.0));
                                ui.label("Jerarquía (izquierda): selecciona y busca los objetos de la escena.");
                                ui.label("Viewport 3D (centro): muestra la escena y permite manipular objetos.");
                                ui.label("Inspector (derecha): cambia nombre, posición, rotación, escala, malla y material.");
                                ui.label("Panel inferior: alterna entre esta ayuda, la consola y el archivo ST.");
                                ui.separator();
                                ui.label(RichText::new("Botones superiores").strong());
                                ui.label("Nuevo crea una escena vacía. Abrir y Guardar trabajan con archivos .stscene.");
                                ui.label("Deshacer y Rehacer recorren el historial. Duplicar y Eliminar actúan sobre la selección.");
                                ui.label("Aplicar ST compila el script visible. Exportar ST guarda los cambios visuales como código.");
                                ui.separator();
                                ui.label(RichText::new("Controles del viewport").strong());
                            ui.label("W: mover · E: rotar · R: escalar · arrastra los ejes X/Y/Z.");
                            ui.label("Botón derecho: orbitar · botón central: desplazar · rueda: zoom · F: enfocar selección.");
                            ui.label("Ctrl+Z/Ctrl+Y: historial · Ctrl+D: duplicar · Supr: eliminar.");
                            ui.separator();
                            ui.label(RichText::new("Flujo ST").strong());
                            ui.label("1. Exporta la escena visual a un archivo .st.");
                            ui.label("2. Modifica llamadas editor3d.entidad(...) con sintaxis ST.");
                            ui.label("3. Pulsa Aplicar ST. pyforgec compila y el editor valida la salida.");
                            ui.label("4. Mueve objetos con los gizmos y vuelve a exportar cuando quieras conservar los cambios en ST.");
                        });
                    }
                }
            });
            match action {
                Action::None => {}
                Action::Load => self.load_source(),
                Action::Save => {
                    self.save_source();
                }
                Action::Apply => self.apply_source(),
                Action::Export => self.export_scene(),
                Action::Clear => self.console.clear(),
            }
        }

        fn show_status(&mut self, ctx: &egui::Context) {
            egui::TopBottomPanel::bottom("estado")
                .exact_height(25.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("●").color(Color32::from_rgb(255, 205, 52)));
                        ui.label(&self.status);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (undo, redo) = self.state.history_counts();
                            ui.small(format!(
                                "LLVM · {} entidades · historial {undo}/{redo}",
                                self.scene.entities.len()
                            ));
                        });
                    });
                });
        }
    }

    impl eframe::App for EditorApp {
        fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
            self.configure_style(ctx);
            self.poll_script_job();
            self.poll_auto_reload();
            self.handle_shortcuts(ctx);
            self.show_menu(ctx);
            self.show_toolbar(ctx);
            self.show_status(ctx);
            self.show_bottom(ctx);
            self.show_hierarchy(ctx);
            self.show_inspector(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                let output = show_viewport(ui, &mut self.scene, &mut self.state);
                if let Some(status) = output.status {
                    self.status = status;
                }
                if output.changed {
                    self.scene_dirty = true;
                }
            });
            if self.script_job.is_some() || self.auto_reload {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
    }

    fn icon_button(ui: &mut egui::Ui, label: &str, help: &str) -> egui::Response {
        ui.add_sized(
            [88.0, 28.0],
            egui::Button::new(RichText::new(label).size(13.0)),
        )
        .on_hover_text(help)
    }

    fn vec3_control(
        ui: &mut egui::Ui,
        values: &mut [f32; 3],
        speed: f64,
        range: Option<(f32, f32)>,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            for (index, label) in ["X", "Y", "Z"].into_iter().enumerate() {
                let color = [
                    Color32::from_rgb(239, 76, 76),
                    Color32::from_rgb(78, 211, 117),
                    Color32::from_rgb(73, 137, 242),
                ][index];
                ui.label(RichText::new(label).color(color).strong());
                let mut drag = DragValue::new(&mut values[index])
                    .speed(speed)
                    .max_decimals(3);
                if let Some((minimum, maximum)) = range {
                    drag = drag.clamp_range(minimum..=maximum);
                }
                changed |= ui.add(drag).changed();
            }
        });
        changed
    }

    fn modified(path: &Path) -> Option<SystemTime> {
        path.metadata().ok()?.modified().ok()
    }

    fn find_compiler() -> String {
        if let Ok(path) = std::env::var("PYFORGE_COMPILER") {
            if !path.trim().is_empty() {
                return path;
            }
        }
        if let Ok(current) = std::env::current_exe() {
            if let Some(parent) = current.parent() {
                let sibling = parent.join(if cfg!(windows) {
                    "pyforgec.exe"
                } else {
                    "pyforgec"
                });
                if sibling.is_file() {
                    return sibling.to_string_lossy().into_owned();
                }
            }
        }
        let development = Path::new("target/debug").join(if cfg!(windows) {
            "pyforgec.exe"
        } else {
            "pyforgec"
        });
        if development.is_file() {
            development.to_string_lossy().into_owned()
        } else {
            "pyforgec".into()
        }
    }

    pub use EditorApp as PublicEditorApp;
}

#[cfg(feature = "editor")]
pub use app::PublicEditorApp as EditorApp;

#[cfg(test)]
mod tests {
    use super::*;

    fn scene() -> Scene {
        Scene {
            version: SCENE_VERSION,
            name: "prueba".into(),
            entities: vec![Entity {
                name: "Cubo".into(),
                transform: Transform::default(),
                mesh: Some("cubo".into()),
                material: None,
            }],
        }
    }

    #[test]
    fn duplicate_delete_and_history_are_reversible() {
        let mut scene = scene();
        let mut state = EditorState {
            selected: Some(0),
            ..Default::default()
        };
        assert!(state.duplicate_selected(&mut scene).unwrap());
        assert_eq!(scene.entities.len(), 2);
        assert!(state.delete_selected(&mut scene));
        assert_eq!(scene.entities.len(), 1);
        assert!(state.undo(&mut scene));
        assert_eq!(scene.entities.len(), 2);
        assert!(state.undo(&mut scene));
        assert_eq!(scene.entities.len(), 1);
    }
}
