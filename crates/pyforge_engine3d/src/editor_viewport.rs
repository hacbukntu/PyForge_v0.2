//! Viewport 3D interactivo del editor.
//!
//! Este módulo mantiene separadas dos responsabilidades:
//! - la cámara, el estado del gizmo y sus operaciones matemáticas no dependen de
//!   `egui`, por lo que pueden probarse sin abrir una ventana;
//! - la capa activada por la característica `editor` proyecta la escena y la
//!   dibuja dentro del mismo render pass que usa `eframe`.
//!
//! El desacoplamiento permite sustituir el dibujo alámbrico por el renderer PBR
//! sin cambiar la lógica de selección o manipulación del editor.

use crate::scene::Transform;
use glam::{Mat4, Quat, Vec2, Vec3};

/// Operación activa del gizmo.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GizmoMode {
    /// Desplaza una entidad sobre un eje mundial.
    #[default]
    Translate,
    /// Gira una entidad alrededor de un eje mundial.
    Rotate,
    /// Modifica un componente de la escala.
    Scale,
}

/// Eje mundial escogido por el usuario.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

impl GizmoAxis {
    /// Devuelve el vector unitario que representa este eje.
    pub fn vector(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }

    /// Devuelve el índice del componente correspondiente en un arreglo XYZ.
    pub fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}

/// Captura inmutable del comienzo de un arrastre.
///
/// Guardar el origen evita acumular errores de punto flotante en cada fotograma:
/// el resultado siempre se calcula desde la transformación inicial.
#[derive(Clone, Debug)]
pub struct GizmoDrag {
    pub axis: GizmoAxis,
    pub mode: GizmoMode,
    pub origin: Transform,
    pub pointer_origin: [f32; 2],
}

impl GizmoDrag {
    /// Aplica el desplazamiento escalar del puntero a la transformación original.
    ///
    /// `movement_pixels` ya está proyectado sobre el eje visible. Para trasladar
    /// y escalar, `world_units_per_pixel` convierte ese recorrido a unidades del
    /// mundo. Para rotar se utiliza una velocidad fija y predecible.
    pub fn apply(&self, movement_pixels: f32, world_units_per_pixel: f32) -> Transform {
        let mut result = self.origin.clone();
        let component = self.axis.index();

        match self.mode {
            GizmoMode::Translate => {
                result.position[component] += movement_pixels * world_units_per_pixel;
            }
            GizmoMode::Scale => {
                result.scale[component] =
                    (result.scale[component] + movement_pixels * world_units_per_pixel).max(0.001);
            }
            GizmoMode::Rotate => {
                let original = Quat::from_array(result.rotation).normalize();
                let delta = Quat::from_axis_angle(self.axis.vector(), movement_pixels * 0.01);
                result.rotation = (delta * original).normalize().to_array();
            }
        }

        result
    }
}

/// Cámara orbital usada por el viewport.
#[derive(Clone, Debug)]
pub struct ViewportCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov_y_radians: f32,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: 35.0_f32.to_radians(),
            pitch: 25.0_f32.to_radians(),
            distance: 12.0,
            fov_y_radians: 55.0_f32.to_radians(),
        }
    }
}

impl ViewportCamera {
    /// Calcula la posición cartesiana a partir de los ángulos orbitales.
    pub fn position(&self) -> Vec3 {
        let horizontal = self.pitch.cos() * self.distance;
        self.target
            + Vec3::new(
                self.yaw.sin() * horizontal,
                self.pitch.sin() * self.distance,
                self.yaw.cos() * horizontal,
            )
    }

    /// Crea la matriz usada para proyectar posiciones mundiales.
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let projection = Mat4::perspective_rh(self.fov_y_radians, aspect.max(0.01), 0.05, 10_000.0);
        let view = Mat4::look_at_rh(self.position(), self.target, Vec3::Y);
        projection * view
    }

    /// Convierte una posición mundial a coordenadas locales del viewport.
    ///
    /// Devuelve `None` para puntos detrás de la cámara o fuera del rango de
    /// profundidad. Esto impide que una coordenada inválida llegue al pintor.
    pub fn project(&self, point: Vec3, size: [f32; 2]) -> Option<[f32; 2]> {
        let clip = self.view_projection(size[0] / size[1].max(1.0)) * point.extend(1.0);
        if !clip.is_finite() || clip.w <= 0.0001 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        if ndc.z < 0.0 || ndc.z > 1.0 {
            return None;
        }
        Some([(ndc.x * 0.5 + 0.5) * size[0], (0.5 - ndc.y * 0.5) * size[1]])
    }

    /// Orbita la cámara con límites que evitan invertirla en los polos.
    pub fn orbit(&mut self, delta_pixels: [f32; 2]) {
        self.yaw -= delta_pixels[0] * 0.008;
        self.pitch = (self.pitch + delta_pixels[1] * 0.008).clamp(-1.50, 1.50);
    }

    /// Acerca o aleja la cámara de forma exponencial y acotada.
    pub fn zoom(&mut self, wheel_delta: f32) {
        self.distance = (self.distance * (-wheel_delta * 0.0015).exp()).clamp(0.1, 50_000.0);
    }

    /// Desplaza el objetivo en el plano de la cámara.
    pub fn pan(&mut self, delta_pixels: [f32; 2]) {
        let forward = (self.target - self.position()).normalize_or_zero();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        let speed = self.distance * 0.0015;
        self.target += right * (-delta_pixels[0] * speed) + up * (delta_pixels[1] * speed);
    }

    /// Centra la cámara en una entidad y conserva una distancia cómoda según su tamaño.
    pub fn frame(&mut self, transform: &Transform) {
        self.target = Vec3::from_array(transform.position);
        let radius = Vec3::from_array(transform.scale)
            .abs()
            .max_element()
            .max(0.1);
        self.distance = (radius * 5.0).clamp(1.5, 50_000.0);
    }

    /// Restaura una perspectiva predecible para recuperar una vista perdida.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Estado persistente de la interacción del viewport.
#[derive(Clone, Debug)]
pub struct ViewportState {
    pub camera: ViewportCamera,
    pub gizmo_mode: GizmoMode,
    pub drag: Option<GizmoDrag>,
    /// Permite ocultar la rejilla sin cambiar la escena.
    pub show_grid: bool,
    /// Permite ocultar nombres cuando una escena contiene muchas entidades.
    pub show_labels: bool,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            camera: ViewportCamera::default(),
            gizmo_mode: GizmoMode::default(),
            drag: None,
            show_grid: true,
            show_labels: true,
        }
    }
}

impl ViewportState {
    /// Inicia una operación que posteriormente puede deshacerse como un solo paso.
    pub fn begin_drag(&mut self, axis: GizmoAxis, transform: Transform, pointer_origin: [f32; 2]) {
        self.drag = Some(GizmoDrag {
            axis,
            mode: self.gizmo_mode,
            origin: transform,
            pointer_origin,
        });
    }

    /// Finaliza cualquier manipulación activa.
    pub fn end_drag(&mut self) {
        self.drag = None;
    }
}

/// Distancia de un punto a un segmento, útil para detectar un eje del gizmo.
pub fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared <= f32::EPSILON {
        return point.distance(start);
    }
    let t = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}

/// Proyección escalar del movimiento del ratón sobre un eje visible.
pub fn projected_movement(pointer_delta: Vec2, axis_screen_delta: Vec2) -> f32 {
    pointer_delta.dot(axis_screen_delta.normalize_or_zero())
}

#[cfg(feature = "editor")]
mod ui {
    use super::*;
    use crate::{
        editor::EditorState,
        scene::{Entity, Scene},
    };
    use eframe::egui::{
        self, Color32, FontId, Painter, PointerButton, Pos2, Rect, Response, Sense, Stroke, Ui,
        Vec2 as EguiVec2,
    };

    const CUBE_CORNERS: [Vec3; 8] = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ];
    const CUBE_EDGES: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    const CUBE_FACES: [[usize; 4]; 6] = [
        [0, 1, 2, 3],
        [4, 7, 6, 5],
        [0, 4, 5, 1],
        [3, 2, 6, 7],
        [1, 5, 6, 2],
        [0, 3, 7, 4],
    ];

    /// Resultado de dibujar el viewport durante un fotograma.
    pub struct ViewportOutput {
        pub response: Response,
        pub status: Option<String>,
        /// Indica que un gizmo modificó una transformación durante este cuadro.
        pub changed: bool,
    }

    /// Dibuja y controla el viewport en la región central del editor.
    pub fn show_viewport(
        ui: &mut Ui,
        scene: &mut Scene,
        state: &mut EditorState,
    ) -> ViewportOutput {
        show_viewport_mode(ui, scene, state, true)
    }

    /// Dibuja la misma escena sólida para un juego, sin selección ni gizmos que
    /// puedan interceptar las teclas W/E/R reservadas por la jugabilidad.
    pub fn show_game_viewport(
        ui: &mut Ui,
        scene: &mut Scene,
        state: &mut EditorState,
    ) -> ViewportOutput {
        show_viewport_mode(ui, scene, state, false)
    }

    fn show_viewport_mode(
        ui: &mut Ui,
        scene: &mut Scene,
        state: &mut EditorState,
        editing: bool,
    ) -> ViewportOutput {
        if editing {
            show_toolbar(ui, &mut state.viewport);
        } else {
            ui.horizontal(|ui| {
                ui.strong("Escena 3D");
                ui.separator();
                ui.label("caras sólidas · cámara orbital");
            });
        }

        let available = ui.available_size().max(EguiVec2::new(160.0, 120.0));
        let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());
        let rect = response.rect;
        let mut status = None;
        let mut changed = false;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 17, 22));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(48, 52, 62)),
        );

        handle_camera(ui, &response, &mut state.viewport.camera);
        if state.viewport.show_grid {
            draw_grid(&painter, rect, &state.viewport.camera);
        }
        draw_entities(
            &painter,
            rect,
            scene,
            state.selected,
            &state.viewport.camera,
            state.viewport.show_labels,
        );

        let pointer = response.interact_pointer_pos();
        let hot_axis = editing
            .then_some(())
            .and_then(|_| state.selected)
            .and_then(|index| {
                scene.entities.get(index).and_then(|entity| {
                    pointer.and_then(|position| {
                        hit_test_gizmo(
                            rect,
                            &state.viewport.camera,
                            entity,
                            Vec2::new(position.x, position.y),
                        )
                    })
                })
            });

        if editing {
            if let Some(index) = state.selected {
                if let Some(entity) = scene.entities.get(index) {
                    draw_gizmo(
                        &painter,
                        rect,
                        &state.viewport.camera,
                        entity,
                        state.viewport.gizmo_mode,
                        hot_axis.or_else(|| state.viewport.drag.as_ref().map(|drag| drag.axis)),
                    );
                }
            }
        }

        if editing && response.drag_started_by(PointerButton::Primary) {
            if let (Some(axis), Some(index), Some(position)) = (hot_axis, state.selected, pointer) {
                if let Some(transform) = scene
                    .entities
                    .get(index)
                    .map(|entity| entity.transform.clone())
                {
                    // Un único punto de control hace que todo el arrastre se deshaga de una vez.
                    state.checkpoint(scene);
                    state
                        .viewport
                        .begin_drag(axis, transform, [position.x, position.y]);
                    status = Some(format!(
                        "Gizmo {:?} · eje {:?}",
                        state.viewport.gizmo_mode, axis
                    ));
                }
            } else if let Some(position) = pointer {
                state.selected = select_entity(
                    rect,
                    &state.viewport.camera,
                    &scene.entities,
                    Vec2::new(position.x, position.y),
                );
            }
        }

        if editing && response.dragged_by(PointerButton::Primary) {
            apply_active_drag(rect, scene, state, pointer);
            changed = state.viewport.drag.is_some();
        }
        if editing && response.drag_stopped_by(PointerButton::Primary) {
            state.viewport.end_drag();
        }

        painter.text(
            rect.left_bottom() + EguiVec2::new(10.0, -10.0),
            egui::Align2::LEFT_BOTTOM,
            if editing {
                "RMB orbitar · MMB desplazar · rueda zoom · W/E/R gizmo"
            } else {
                "RMB orbitar · MMB desplazar · rueda zoom"
            },
            FontId::monospace(11.0),
            Color32::from_gray(145),
        );

        ViewportOutput {
            response,
            status,
            changed,
        }
    }

    fn show_toolbar(ui: &mut Ui, viewport: &mut ViewportState) {
        ui.horizontal(|ui| {
            ui.strong("Viewport 3D");
            ui.separator();
            ui.selectable_value(&mut viewport.gizmo_mode, GizmoMode::Translate, "W · Mover")
                .on_hover_text("Mover la entidad seleccionada sobre X, Y o Z");
            ui.selectable_value(&mut viewport.gizmo_mode, GizmoMode::Rotate, "E · Rotar")
                .on_hover_text("Rotar la entidad seleccionada");
            ui.selectable_value(&mut viewport.gizmo_mode, GizmoMode::Scale, "R · Escalar")
                .on_hover_text("Cambiar la escala de la entidad seleccionada");
            ui.separator();
            ui.toggle_value(&mut viewport.show_grid, "Rejilla");
            ui.toggle_value(&mut viewport.show_labels, "Nombres");
            if ui
                .button("Restaurar vista")
                .on_hover_text("Restaurar cámara")
                .clicked()
            {
                viewport.camera.reset();
            }
        });

        let accepts_shortcuts = !ui.ctx().wants_keyboard_input();
        if accepts_shortcuts && ui.input(|input| input.key_pressed(egui::Key::W)) {
            viewport.gizmo_mode = GizmoMode::Translate;
        }
        if accepts_shortcuts && ui.input(|input| input.key_pressed(egui::Key::E)) {
            viewport.gizmo_mode = GizmoMode::Rotate;
        }
        if accepts_shortcuts && ui.input(|input| input.key_pressed(egui::Key::R)) {
            viewport.gizmo_mode = GizmoMode::Scale;
        }
    }

    fn handle_camera(ui: &Ui, response: &Response, camera: &mut ViewportCamera) {
        if response.dragged_by(PointerButton::Secondary) {
            let delta = ui.input(|input| input.pointer.delta());
            camera.orbit([delta.x, delta.y]);
        }
        if response.dragged_by(PointerButton::Middle) {
            let delta = ui.input(|input| input.pointer.delta());
            camera.pan([delta.x, delta.y]);
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.raw_scroll_delta.y);
            if scroll != 0.0 {
                camera.zoom(scroll);
            }
        }
    }

    fn project(camera: &ViewportCamera, rect: Rect, point: Vec3) -> Option<Pos2> {
        camera
            .project(point, [rect.width(), rect.height()])
            .map(|point| Pos2::new(rect.left() + point[0], rect.top() + point[1]))
    }

    fn draw_grid(painter: &Painter, rect: Rect, camera: &ViewportCamera) {
        for value in -20..=20 {
            let color = if value == 0 {
                Color32::from_gray(100)
            } else if value % 5 == 0 {
                Color32::from_gray(54)
            } else {
                Color32::from_gray(35)
            };
            draw_world_line(
                painter,
                rect,
                camera,
                Vec3::new(value as f32, 0.0, -20.0),
                Vec3::new(value as f32, 0.0, 20.0),
                Stroke::new(1.0_f32, color),
            );
            draw_world_line(
                painter,
                rect,
                camera,
                Vec3::new(-20.0, 0.0, value as f32),
                Vec3::new(20.0, 0.0, value as f32),
                Stroke::new(1.0_f32, color),
            );
        }
    }

    fn draw_entities(
        painter: &Painter,
        rect: Rect,
        scene: &Scene,
        selected: Option<usize>,
        camera: &ViewportCamera,
        show_labels: bool,
    ) {
        // Las caras de todas las entidades se ordenan desde la más lejana. Este
        // painter no sustituye un depth buffer de GPU, pero proporciona volúmenes
        // sólidos coherentes para juegos y conserva el alambre como contorno.
        let mut faces: Vec<(f32, Vec<Pos2>, Color32, Color32)> = Vec::new();
        let camera_position = camera.position();
        for (index, entity) in scene.entities.iter().enumerate() {
            let matrix = entity.transform.matrix();
            let corners = CUBE_CORNERS.map(|corner| matrix.transform_point3(corner));
            let base = material_color(entity.material.as_deref(), selected == Some(index));
            for (face_index, indices) in CUBE_FACES.iter().enumerate() {
                let world = indices.map(|corner| corners[corner]);
                let mut screen = Vec::with_capacity(4);
                for point in world {
                    let Some(point) = project(camera, rect, point) else {
                        screen.clear();
                        break;
                    };
                    screen.push(point);
                }
                if screen.len() != 4 {
                    continue;
                }
                let center = world.into_iter().sum::<Vec3>() * 0.25;
                let depth = center.distance_squared(camera_position);
                let shade = [0.58, 0.92, 0.72, 1.05, 0.82, 0.66][face_index];
                faces.push((depth, screen, shade_color(base, shade), base));
            }
        }
        faces.sort_by(|left, right| right.0.total_cmp(&left.0));
        for (_, points, fill, outline) in faces {
            painter.add(egui::Shape::convex_polygon(
                points,
                fill,
                Stroke::new(0.75_f32, outline),
            ));
        }

        for (index, entity) in scene.entities.iter().enumerate() {
            let matrix = entity.transform.matrix();
            let corners = CUBE_CORNERS.map(|corner| matrix.transform_point3(corner));
            let color = if selected == Some(index) {
                Color32::from_rgb(255, 205, 52)
            } else {
                material_color(entity.material.as_deref(), false)
            };
            for (start, end) in CUBE_EDGES {
                draw_world_line(
                    painter,
                    rect,
                    camera,
                    corners[start],
                    corners[end],
                    Stroke::new(
                        if selected == Some(index) {
                            2.0_f32
                        } else {
                            1.25_f32
                        },
                        color,
                    ),
                );
            }
            if show_labels && !entity.name.starts_with('_') {
                if let Some(position) =
                    project(camera, rect, Vec3::from_array(entity.transform.position))
                {
                    painter.text(
                        position + EguiVec2::new(6.0, -6.0),
                        egui::Align2::LEFT_BOTTOM,
                        &entity.name,
                        FontId::monospace(11.0),
                        color,
                    );
                }
            }
        }
    }

    /// Paleta común del editor y de los juegos; un material desconocido conserva
    /// el azul del editor en vez de fallar o ejecutar contenido del recurso.
    fn material_color(material: Option<&str>, selected: bool) -> Color32 {
        if selected {
            return Color32::from_rgb(255, 205, 52);
        }
        match material.unwrap_or("base") {
            "jugador" => Color32::from_rgb(54, 196, 255),
            "portal" | "meta" => Color32::from_rgb(183, 77, 255),
            "cristal" => Color32::from_rgb(255, 211, 54),
            "enemigo" => Color32::from_rgb(244, 74, 91),
            "muro" | "obstaculo" => Color32::from_rgb(87, 96, 112),
            "suelo" => Color32::from_rgb(44, 54, 67),
            "energia" => Color32::from_rgb(67, 224, 145),
            _ => Color32::from_rgb(115, 169, 230),
        }
    }

    fn shade_color(color: Color32, factor: f32) -> Color32 {
        let scale = |component: u8| (component as f32 * factor).clamp(0.0, 255.0) as u8;
        Color32::from_rgb(scale(color.r()), scale(color.g()), scale(color.b()))
    }

    fn draw_world_line(
        painter: &Painter,
        rect: Rect,
        camera: &ViewportCamera,
        start: Vec3,
        end: Vec3,
        stroke: Stroke,
    ) {
        if let (Some(start), Some(end)) = (project(camera, rect, start), project(camera, rect, end))
        {
            painter.line_segment([start, end], stroke);
        }
    }

    fn gizmo_points(camera: &ViewportCamera, entity: &Entity) -> [(GizmoAxis, Vec3, Vec3); 3] {
        let origin = Vec3::from_array(entity.transform.position);
        let length = (camera.distance * 0.12).clamp(0.35, 25.0);
        [
            (GizmoAxis::X, origin, origin + Vec3::X * length),
            (GizmoAxis::Y, origin, origin + Vec3::Y * length),
            (GizmoAxis::Z, origin, origin + Vec3::Z * length),
        ]
    }

    fn axis_color(axis: GizmoAxis, active: bool) -> Color32 {
        if active {
            return Color32::from_rgb(255, 220, 70);
        }
        match axis {
            GizmoAxis::X => Color32::from_rgb(239, 76, 76),
            GizmoAxis::Y => Color32::from_rgb(78, 211, 117),
            GizmoAxis::Z => Color32::from_rgb(73, 137, 242),
        }
    }

    fn draw_gizmo(
        painter: &Painter,
        rect: Rect,
        camera: &ViewportCamera,
        entity: &Entity,
        mode: GizmoMode,
        active_axis: Option<GizmoAxis>,
    ) {
        for (axis, start_world, end_world) in gizmo_points(camera, entity) {
            let (Some(start), Some(end)) = (
                project(camera, rect, start_world),
                project(camera, rect, end_world),
            ) else {
                continue;
            };
            let active = active_axis == Some(axis);
            let color = axis_color(axis, active);
            painter.line_segment(
                [start, end],
                Stroke::new(if active { 4.0_f32 } else { 3.0_f32 }, color),
            );
            match mode {
                GizmoMode::Translate => {
                    let direction = (end - start).normalized();
                    let normal = EguiVec2::new(-direction.y, direction.x);
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            end,
                            end - direction * 11.0 + normal * 5.0,
                            end - direction * 11.0 - normal * 5.0,
                        ],
                        color,
                        Stroke::NONE,
                    ));
                }
                GizmoMode::Rotate => {
                    painter.circle_stroke(end, 6.0, Stroke::new(2.0_f32, color));
                }
                GizmoMode::Scale => {
                    painter.rect_filled(
                        Rect::from_center_size(end, EguiVec2::splat(9.0)),
                        1.0,
                        color,
                    );
                }
            }
        }
    }

    fn hit_test_gizmo(
        rect: Rect,
        camera: &ViewportCamera,
        entity: &Entity,
        pointer: Vec2,
    ) -> Option<GizmoAxis> {
        gizmo_points(camera, entity)
            .into_iter()
            .filter_map(|(axis, start, end)| {
                let start = project(camera, rect, start)?;
                let end = project(camera, rect, end)?;
                let distance = distance_to_segment(
                    pointer,
                    Vec2::new(start.x, start.y),
                    Vec2::new(end.x, end.y),
                );
                (distance <= 9.0).then_some((axis, distance))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(axis, _)| axis)
    }

    fn select_entity(
        rect: Rect,
        camera: &ViewportCamera,
        entities: &[Entity],
        pointer: Vec2,
    ) -> Option<usize> {
        entities
            .iter()
            .enumerate()
            .filter_map(|(index, entity)| {
                let center = project(camera, rect, Vec3::from_array(entity.transform.position))?;
                let distance = pointer.distance(Vec2::new(center.x, center.y));
                (distance <= 24.0).then_some((index, distance))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(index, _)| index)
    }

    fn apply_active_drag(
        rect: Rect,
        scene: &mut Scene,
        state: &mut EditorState,
        pointer: Option<Pos2>,
    ) {
        let (Some(index), Some(pointer), Some(drag)) =
            (state.selected, pointer, state.viewport.drag.as_ref())
        else {
            return;
        };

        let origin = Vec3::from_array(drag.origin.position);
        let length = (state.viewport.camera.distance * 0.12).clamp(0.35, 25.0);
        let Some(axis_start) = project(&state.viewport.camera, rect, origin) else {
            return;
        };
        let Some(axis_end) = project(
            &state.viewport.camera,
            rect,
            origin + drag.axis.vector() * length,
        ) else {
            return;
        };

        let pointer_delta = Vec2::new(
            pointer.x - drag.pointer_origin[0],
            pointer.y - drag.pointer_origin[1],
        );
        let axis_delta = Vec2::new(axis_end.x - axis_start.x, axis_end.y - axis_start.y);
        let movement = projected_movement(pointer_delta, axis_delta);
        let pixels_per_world_unit = axis_delta.length() / length;
        let world_units_per_pixel = 1.0 / pixels_per_world_unit.max(0.001);

        if let Some(entity) = scene.entities.get_mut(index) {
            entity.transform = drag.apply(movement, world_units_per_pixel);
        }
    }
}

#[cfg(feature = "editor")]
pub use ui::{show_game_viewport, show_viewport, ViewportOutput};

#[cfg(test)]
mod tests {
    use super::*;

    fn base_transform() -> Transform {
        Transform {
            position: [1.0, 2.0, 3.0],
            rotation: Quat::IDENTITY.to_array(),
            scale: [1.0, 2.0, 3.0],
        }
    }

    #[test]
    fn trasladar_solo_modifica_el_eje_elegido() {
        let drag = GizmoDrag {
            axis: GizmoAxis::Y,
            mode: GizmoMode::Translate,
            origin: base_transform(),
            pointer_origin: [0.0, 0.0],
        };
        let result = drag.apply(20.0, 0.05);
        assert_eq!(result.position, [1.0, 3.0, 3.0]);
        assert_eq!(drag.origin.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn escala_nunca_llega_a_cero() {
        let drag = GizmoDrag {
            axis: GizmoAxis::X,
            mode: GizmoMode::Scale,
            origin: base_transform(),
            pointer_origin: [0.0, 0.0],
        };
        let result = drag.apply(-10_000.0, 1.0);
        assert_eq!(result.scale[0], 0.001);
        assert_eq!(result.scale[1..], [2.0, 3.0]);
    }

    #[test]
    fn rotacion_del_gizmo_permanece_normalizada() {
        let drag = GizmoDrag {
            axis: GizmoAxis::Z,
            mode: GizmoMode::Rotate,
            origin: base_transform(),
            pointer_origin: [0.0, 0.0],
        };
        let result = drag.apply(50.0, 0.0);
        let rotation = Quat::from_array(result.rotation);
        assert!((rotation.length() - 1.0).abs() < 1e-5);
        assert_ne!(rotation, Quat::IDENTITY);
    }

    #[test]
    fn camara_proyecta_su_objetivo_al_centro() {
        let camera = ViewportCamera::default();
        let projected = camera.project(camera.target, [800.0, 600.0]).unwrap();
        assert!((projected[0] - 400.0).abs() < 0.01);
        assert!((projected[1] - 300.0).abs() < 0.01);
    }

    #[test]
    fn distancia_al_segmento_se_acota_a_los_extremos() {
        let distance = distance_to_segment(Vec2::new(12.0, 3.0), Vec2::ZERO, Vec2::X * 10.0);
        assert!((distance - 3.605_551_2).abs() < 1e-5);
    }
}
