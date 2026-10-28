// Escena completamente definida en ST para PyForge Editor 3D.
// Abre este archivo desde la pestaña «Script ST» y pulsa «Aplicar ST».
importar "editor3d";

editor3d.iniciar("Estudio ST"; 5);

// Cada entidad usa cuatro llamadas para respetar el ABI de ST: metadatos,
// posición XYZ, rotación en cuaternión XYZW y escala XYZ.
editor3d.entidad("Suelo"; "plano"; "suelo");
editor3d.posicion(0.0; -0.15; 0.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(12.0; 0.2; 12.0);

editor3d.entidad("Jugador"; "cubo"; "jugador");
editor3d.posicion(0.0; 0.75; 0.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(1.0; 1.5; 1.0);

editor3d.entidad("Portal"; "cubo"; "portal");
editor3d.posicion(4.0; 1.25; -2.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(0.3; 2.5; 2.0);

editor3d.entidad("Obstáculo"; "cubo"; "obstaculo");
editor3d.posicion(-3.0; 1.0; 2.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(2.0; 2.0; 2.0);

editor3d.entidad("Cristal"; "cubo"; "cristal");
editor3d.posicion(2.0; 0.5; 3.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(0.6; 0.6; 0.6);

editor3d.terminar();
