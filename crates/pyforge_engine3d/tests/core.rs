use glam::{Mat4, Quat, Vec3};
use pyforge_engine3d::{
    animation::{skin_matrices, JointPose},
    audio::SpatialSource,
    scene::{Scene, SCENE_VERSION},
};
#[test]
fn escena_rechaza_version() {
    let s = Scene {
        version: SCENE_VERSION + 1,
        name: "x".into(),
        entities: vec![],
    };
    assert!(s.validate().is_err());
}
#[test]
fn esqueleto_y_audio() {
    let pose = JointPose {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    assert_eq!(
        skin_matrices(&[pose], &[None], &[Mat4::IDENTITY])
            .unwrap()
            .len(),
        1
    );
    let (l, r) = SpatialSource {
        position: Vec3::X,
        volume: 1.0,
        max_distance: 10.0,
    }
    .gains(Vec3::ZERO, Vec3::X);
    assert!(r > l);
}
#[test]
fn escena_guardado_atomico() {
    let root = std::env::temp_dir().join(format!("pyforge-scene-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("nivel.stscene");
    let scene = Scene {
        version: SCENE_VERSION,
        name: "nivel".into(),
        entities: vec![],
    };
    scene.save_atomic(&path).unwrap();
    let loaded = Scene::load(&path).unwrap();
    assert_eq!(loaded.name, "nivel");
    assert!(!root.join(".nivel.stscene.tmp").exists());
    std::fs::remove_dir_all(root).unwrap();
}
