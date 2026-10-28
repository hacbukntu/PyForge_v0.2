//! Ejecutor visual común para juegos cuya lógica se compila desde ST.
//!
//! Cada pulsación inicia una transición ST corta. El proceso recibe el estado
//! numérico actual y devuelve un estado completo validado. Esta frontera evita
//! mantener vivo el runtime experimental y permite compartir ventana, cámara,
//! HUD y representación sólida entre varios juegos sin duplicar código nativo.

use pyforge_engine3d::{
    editor::EditorState,
    editor_viewport::show_game_viewport,
    scene::{Entity, Scene, Transform, SCENE_VERSION},
};
use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const MAX_COORDINATE: i64 = 10_000;
const MAX_MESSAGE_BYTES: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameProfile {
    Portales,
    Cristales,
    Laberinto,
    Guardianes,
}

impl GameProfile {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "portales" => Ok(Self::Portales),
            "cristales" => Ok(Self::Cristales),
            "laberinto" => Ok(Self::Laberinto),
            "guardianes" => Ok(Self::Guardianes),
            _ => Err(format!(
                "perfil desconocido {value:?}; usa portales, cristales, laberinto o guardianes"
            )),
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Portales => "Portal Runner 3D",
            Self::Cristales => "Arena de Cristales 3D",
            Self::Laberinto => "Laberinto Neón 3D",
            Self::Guardianes => "Guardianes del Núcleo 3D",
        }
    }

    fn instructions(self) -> &'static str {
        match self {
            Self::Portales => "WASD/flechas: mover · alcanza cinco portales",
            Self::Cristales => "WASD/flechas: mover · recoge cinco cristales · evita drones",
            Self::Laberinto => "WASD/flechas: mover · encuentra la salida sin atravesar muros",
            Self::Guardianes => {
                "WASD/flechas: mover · Espacio/Enter: atacar · sobrevive seis rondas"
            }
        }
    }

    fn enemy_count(self) -> usize {
        match self {
            Self::Cristales | Self::Guardianes => 2,
            _ => 0,
        }
    }

    fn objective_material(self) -> &'static str {
        match self {
            Self::Cristales => "cristal",
            Self::Guardianes => "energia",
            _ => "portal",
        }
    }

    fn decorations(self) -> Vec<Entity> {
        match self {
            Self::Portales => vec![
                cube("_torre NO", -9.0, 1.0, 9.0, 0.8, 2.0, 0.8, "obstaculo"),
                cube("_torre NE", 9.0, 1.0, 9.0, 0.8, 2.0, 0.8, "obstaculo"),
                cube("_torre SO", -9.0, 1.0, -9.0, 0.8, 2.0, 0.8, "obstaculo"),
                cube("_torre SE", 9.0, 1.0, -9.0, 0.8, 2.0, 0.8, "obstaculo"),
            ],
            Self::Cristales => [(-3.0, 2.0), (2.0, -3.0), (4.0, 3.0), (-5.0, -2.0)]
                .into_iter()
                .enumerate()
                .map(|(index, (x, z))| {
                    cube(
                        &format!("_roca {index}"),
                        x,
                        0.65,
                        z,
                        1.1,
                        1.3,
                        1.1,
                        "obstaculo",
                    )
                })
                .collect(),
            Self::Laberinto => maze_walls(),
            Self::Guardianes => (-6..=6)
                .step_by(2)
                .flat_map(|value| {
                    [
                        cube(
                            "_borde norte",
                            value as f32,
                            0.4,
                            7.0,
                            0.35,
                            0.8,
                            0.35,
                            "obstaculo",
                        ),
                        cube(
                            "_borde sur",
                            value as f32,
                            0.4,
                            -7.0,
                            0.35,
                            0.8,
                            0.35,
                            "obstaculo",
                        ),
                    ]
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct GameState {
    player_x: i64,
    player_z: i64,
    target_x: i64,
    target_z: i64,
    enemy_1_x: i64,
    enemy_1_z: i64,
    enemy_2_x: i64,
    enemy_2_z: i64,
    level: i64,
    score: i64,
    lives: i64,
    turns: i64,
    ended: bool,
    won: bool,
    message: String,
}

impl GameState {
    fn validate(&self) -> Result<(), String> {
        for coordinate in [
            self.player_x,
            self.player_z,
            self.target_x,
            self.target_z,
            self.enemy_1_x,
            self.enemy_1_z,
            self.enemy_2_x,
            self.enemy_2_z,
        ] {
            if coordinate.abs() > MAX_COORDINATE {
                return Err("el juego ST devolvió una coordenada fuera de límites".into());
            }
        }
        if !(0..=10_000).contains(&self.level)
            || !(0..=1_000).contains(&self.lives)
            || !(0..=10_000_000).contains(&self.turns)
            || self.score.abs() > 1_000_000_000
        {
            return Err("el juego ST devolvió contadores fuera de límites".into());
        }
        if self.message.len() > MAX_MESSAGE_BYTES {
            return Err("el mensaje del juego ST supera 2048 bytes".into());
        }
        Ok(())
    }
}

struct GameBridge {
    program: String,
}

impl GameBridge {
    fn spawn(program: &str) -> Result<(Self, GameState), String> {
        let bridge = Self {
            program: program.to_string(),
        };
        let state = bridge.command("init", None)?;
        Ok((bridge, state))
    }

    fn command(&self, command: &str, current: Option<&GameState>) -> Result<GameState, String> {
        if !matches!(command, "init" | "w" | "a" | "s" | "d" | "f" | "r" | "q") {
            return Err(format!("comando de juego no permitido: {command}"));
        }
        let mut child = Command::new(&self.program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("iniciar transición ST: {error}"))?;

        {
            let mut input = child.stdin.take().ok_or("stdin de ST no disponible")?;
            writeln!(input, "{command}")
                .map_err(|error| format!("escribir comando ST: {error}"))?;
            if let Some(state) = current {
                for value in [
                    state.player_x,
                    state.player_z,
                    state.target_x,
                    state.target_z,
                    state.enemy_1_x,
                    state.enemy_1_z,
                    state.enemy_2_x,
                    state.enemy_2_z,
                    state.level,
                    state.score,
                    state.lives,
                    state.turns,
                    i64::from(state.ended),
                    i64::from(state.won),
                ] {
                    writeln!(input, "{value}")
                        .map_err(|error| format!("escribir estado ST: {error}"))?;
                }
            }
            input
                .flush()
                .map_err(|error| format!("vaciar stdin ST: {error}"))?;
        }

        // Una transición defectuosa no puede congelar indefinidamente la ventana.
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match child
                .try_wait()
                .map_err(|error| format!("consultar ST: {error}"))?
            {
                Some(_) => break,
                None if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("la transición ST superó el límite de 3 segundos".into());
                }
                None => thread::sleep(Duration::from_millis(5)),
            }
        }
        let output = child
            .wait_with_output()
            .map_err(|error| format!("esperar ST: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            return Err(format!(
                "ST terminó con error: {}",
                if detail.is_empty() {
                    "sin diagnóstico"
                } else {
                    detail
                }
            ));
        }
        if output.stdout.len() > 65_536 || output.stderr.len() > 65_536 {
            return Err("la transición ST produjo más de 64 KiB de salida".into());
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|error| format!("stdout ST no es UTF-8: {error}"))?;
        Self::parse_state(&stdout)
    }

    fn parse_i64(lines: &[&str], index: usize, name: &str) -> Result<i64, String> {
        let raw = lines
            .get(index)
            .ok_or_else(|| format!("falta campo ST: {name}"))?;
        raw.trim()
            .parse::<i64>()
            .map_err(|error| format!("campo ST {name} inválido ({raw}): {error}"))
    }

    fn parse_flag(lines: &[&str], index: usize, name: &str) -> Result<bool, String> {
        match Self::parse_i64(lines, index, name)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(format!("campo ST {name} debe ser 0 o 1; llegó {value}")),
        }
    }

    fn parse_state(stdout: &str) -> Result<GameState, String> {
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() < 16 {
            return Err(format!(
                "respuesta ST incompleta: se esperaban 16 líneas y llegaron {}",
                lines.len()
            ));
        }
        if lines[0].trim() != "OK" {
            return Err(format!("cabecera ST inesperada: {:?}", lines[0]));
        }
        let state = GameState {
            player_x: Self::parse_i64(&lines, 1, "player_x")?,
            player_z: Self::parse_i64(&lines, 2, "player_z")?,
            target_x: Self::parse_i64(&lines, 3, "target_x")?,
            target_z: Self::parse_i64(&lines, 4, "target_z")?,
            enemy_1_x: Self::parse_i64(&lines, 5, "enemy_1_x")?,
            enemy_1_z: Self::parse_i64(&lines, 6, "enemy_1_z")?,
            enemy_2_x: Self::parse_i64(&lines, 7, "enemy_2_x")?,
            enemy_2_z: Self::parse_i64(&lines, 8, "enemy_2_z")?,
            level: Self::parse_i64(&lines, 9, "level")?,
            score: Self::parse_i64(&lines, 10, "score")?,
            lives: Self::parse_i64(&lines, 11, "lives")?,
            turns: Self::parse_i64(&lines, 12, "turns")?,
            ended: Self::parse_flag(&lines, 13, "ended")?,
            won: Self::parse_flag(&lines, 14, "won")?,
            message: lines[15..].join("\n"),
        };
        state.validate()?;
        Ok(state)
    }
}

struct GameApp {
    profile: GameProfile,
    bridge: GameBridge,
    game: GameState,
    scene: Scene,
    editor: EditorState,
    status: String,
}

impl GameApp {
    fn new(profile: GameProfile, bridge: GameBridge, game: GameState) -> Self {
        let scene = build_scene(profile, &game);
        let mut editor = EditorState::default();
        editor.selected = None;
        editor.viewport.camera.distance = 20.0;
        editor.viewport.camera.pitch = 48.0_f32.to_radians();
        Self {
            profile,
            bridge,
            status: game.message.clone(),
            game,
            scene,
            editor,
        }
    }

    fn send(&mut self, command: &str) {
        if self.game.ended && command != "r" && command != "q" {
            self.status = "La partida terminó; pulsa R para reiniciar".into();
            return;
        }
        match self.bridge.command(command, Some(&self.game)) {
            Ok(game) => {
                self.status = game.message.clone();
                self.game = game;
                self.scene = build_scene(self.profile, &self.game);
                self.editor.viewport.camera.target =
                    glam::Vec3::new(self.game.player_x as f32, 0.0, self.game.player_z as f32);
            }
            Err(error) => self.status = error,
        }
    }
}

impl eframe::App for GameApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        use eframe::egui::{self, Key};
        let mut command = None;
        ctx.input(|input| {
            if input.key_pressed(Key::W) || input.key_pressed(Key::ArrowUp) {
                command = Some("w");
            } else if input.key_pressed(Key::S) || input.key_pressed(Key::ArrowDown) {
                command = Some("s");
            } else if input.key_pressed(Key::A) || input.key_pressed(Key::ArrowLeft) {
                command = Some("a");
            } else if input.key_pressed(Key::D) || input.key_pressed(Key::ArrowRight) {
                command = Some("d");
            } else if input.key_pressed(Key::Space) || input.key_pressed(Key::Enter) {
                command = Some("f");
            } else if input.key_pressed(Key::R) {
                command = Some("r");
            } else if input.key_pressed(Key::Q) || input.key_pressed(Key::Escape) {
                command = Some("q");
            }
        });
        if let Some(command) = command {
            self.send(command);
            if command == "q" {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        egui::TopBottomPanel::top("hud_juego").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(format!("{} · ST", self.profile.title()));
                ui.separator();
                ui.strong(format!("Nivel {}", self.game.level));
                ui.label(format!("Puntos {}", self.game.score));
                ui.label(format!("Vidas {}", self.game.lives));
                ui.label(format!("Turnos {}", self.game.turns));
                if ui.button("Reiniciar").clicked() {
                    self.send("r");
                }
                if ui.button("Acción").clicked() {
                    self.send("f");
                }
            });
            ui.label(self.profile.instructions());
            ui.label("R: reiniciar · Q/Escape: salir · botón derecho/central/rueda: cámara");
            ui.label(format!("ST: {}", self.status));
            if self.game.ended {
                let (text, color) = if self.game.won {
                    (
                        "¡VICTORIA! Pulsa R para jugar otra vez",
                        egui::Color32::LIGHT_GREEN,
                    )
                } else {
                    (
                        "PARTIDA TERMINADA. Pulsa R para reintentar",
                        egui::Color32::LIGHT_RED,
                    )
                };
                ui.colored_label(color, text);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor.selected = None;
            let output = show_game_viewport(ui, &mut self.scene, &mut self.editor);
            self.editor.selected = None;
            if let Some(status) = output.status {
                self.status = status;
            }
        });
        ctx.request_repaint();
    }
}

fn build_scene(profile: GameProfile, game: &GameState) -> Scene {
    let mut entities = vec![
        cube(
            "JUGADOR",
            game.player_x as f32,
            0.6,
            game.player_z as f32,
            0.8,
            1.2,
            0.8,
            "jugador",
        ),
        cube(
            "OBJETIVO",
            game.target_x as f32,
            0.8,
            game.target_z as f32,
            0.65,
            1.6,
            0.65,
            profile.objective_material(),
        ),
    ];
    if profile.enemy_count() >= 1 {
        entities.push(cube(
            "DRON A",
            game.enemy_1_x as f32,
            0.55,
            game.enemy_1_z as f32,
            0.9,
            1.1,
            0.9,
            "enemigo",
        ));
    }
    if profile.enemy_count() >= 2 {
        entities.push(cube(
            "DRON B",
            game.enemy_2_x as f32,
            0.55,
            game.enemy_2_z as f32,
            0.9,
            1.1,
            0.9,
            "enemigo",
        ));
    }
    entities.extend(profile.decorations());
    Scene {
        version: SCENE_VERSION,
        name: profile.title().into(),
        entities,
    }
}

fn cube(name: &str, x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, material: &str) -> Entity {
    Entity {
        name: name.into(),
        transform: Transform {
            position: [x, y, z],
            scale: [sx, sy, sz],
            ..Default::default()
        },
        mesh: Some("cubo".into()),
        material: Some(material.into()),
    }
}

fn maze_walls() -> Vec<Entity> {
    // Estas posiciones coinciden con la función `bloqueado` del juego ST.
    let positions = [
        (-4, -4),
        (-3, -4),
        (-2, -4),
        (-1, -4),
        (0, -4),
        (1, -4),
        (2, -4),
        (2, -3),
        (2, -2),
        (2, -1),
        (2, 0),
        (-4, -1),
        (-3, -1),
        (-2, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (-1, 2),
        (0, 2),
        (1, 2),
        (2, 2),
        (3, 2),
        (4, 2),
        (4, 3),
        (4, 4),
        (4, 5),
        (-5, 4),
        (-4, 4),
        (-3, 4),
        (-2, 4),
        (-1, 4),
        (0, 4),
        (1, 4),
    ];
    positions
        .into_iter()
        .enumerate()
        .map(|(index, (x, z))| {
            cube(
                &format!("_muro {index}"),
                x as f32,
                0.75,
                z as f32,
                0.92,
                1.5,
                0.92,
                "muro",
            )
        })
        .collect()
}

fn parse_arguments() -> Result<(GameProfile, String), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|arg| arg == "--version") {
        println!("pyforge-game3d 0.2.0");
        std::process::exit(0);
    }
    if arguments
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!(
            "USO: pyforge-game3d [--perfil portales|cristales|laberinto|guardianes] <binario-st>"
        );
        std::process::exit(0);
    }
    match arguments.as_slice() {
        [program] => Ok((GameProfile::Portales, program.clone())),
        [flag, profile, program] if flag == "--perfil" => {
            Ok((GameProfile::parse(profile)?, program.clone()))
        }
        _ => Err(
            "USO: pyforge-game3d [--perfil portales|cristales|laberinto|guardianes] <binario-st>"
                .into(),
        ),
    }
}

fn run() -> Result<(), String> {
    let (profile, logic) = parse_arguments()?;
    let (bridge, state) = GameBridge::spawn(&logic)?;
    let title = format!("{} · PyForge ST", profile.title());
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([1180.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        &title.clone(),
        options,
        Box::new(move |_| Box::new(GameApp::new(profile, bridge, state))),
    )
    .map_err(|error| error.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("pyforge-game3d: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "OK\n3\n2\n-4\n4\n5\n6\n-5\n-6\n2\n100\n3\n9\n0\n0\nEstado correcto\n";

    #[test]
    fn respuesta_st_valida_se_convierte_en_estado() {
        let state = GameBridge::parse_state(VALID).expect("la respuesta válida debe aceptarse");
        assert_eq!((state.player_x, state.player_z), (3, 2));
        assert_eq!((state.enemy_2_x, state.enemy_2_z), (-5, -6));
        assert_eq!((state.level, state.score, state.lives), (2, 100, 3));
    }

    #[test]
    fn respuesta_st_incompleta_y_banderas_invalidas_se_rechazan() {
        assert!(GameBridge::parse_state("OK\n0\n")
            .unwrap_err()
            .contains("incompleta"));
        let invalid = VALID.replacen("\n0\n0\nEstado", "\n2\n0\nEstado", 1);
        assert!(GameBridge::parse_state(&invalid)
            .unwrap_err()
            .contains("0 o 1"));
    }

    #[test]
    fn perfiles_publicos_se_reconocen() {
        for name in ["portales", "cristales", "laberinto", "guardianes"] {
            assert!(GameProfile::parse(name).is_ok());
        }
        assert!(GameProfile::parse("desconocido").is_err());
    }

    #[test]
    fn los_cuatro_perfiles_generan_escenas_validas() {
        let state = GameBridge::parse_state(VALID).unwrap();
        for profile in [
            GameProfile::Portales,
            GameProfile::Cristales,
            GameProfile::Laberinto,
            GameProfile::Guardianes,
        ] {
            let scene = build_scene(profile, &state);
            scene.validate().unwrap();
            assert!(scene.entities.len() >= 2 + profile.enemy_count());
        }
    }
}
