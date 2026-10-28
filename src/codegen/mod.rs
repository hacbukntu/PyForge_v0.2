//! Punto de acceso al único generador de código público de PyForge: LLVM 21.
//! `llvm_compat` queda privado porque contiene el runtime ensamblador compartido y
//! la transición temporal de excepciones; no puede seleccionarse como backend.
/// Runtime Linux y puente privado para la ABI histórica de excepciones.
pub(crate) mod llvm_compat;

/// Backend obligatorio basado en LLVM 21 mediante Inkwell.
pub mod llvm;
