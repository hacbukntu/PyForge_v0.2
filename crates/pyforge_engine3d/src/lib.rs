//! Backend gráfico opcional. No forma parte del ejecutable ST salvo elección explícita.
pub mod animation;
pub mod assets;
pub mod audio;
pub mod editor;
pub mod editor_viewport;
pub mod renderer;
pub mod scene;
pub mod st_scene;

pub use renderer::{BackendPreference, Renderer};
pub use scene::Scene;
