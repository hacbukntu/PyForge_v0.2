// Portal Runner: cinco portales, puntuación y victoria.
// Implementa el protocolo común documentado en examples/games/JUEGOS_3D.md.
importar "motor3d";

variable comando: cadena = entrada();
variable jugador_x: entero = 0;
variable jugador_z: entero = 0;
variable objetivo_x: entero = 3;
variable objetivo_z: entero = 2;
variable enemigo_1_x: entero = 0;
variable enemigo_1_z: entero = 0;
variable enemigo_2_x: entero = 0;
variable enemigo_2_z: entero = 0;
variable nivel: entero = 1;
variable puntos: entero = 0;
variable vidas: entero = 3;
variable turnos: entero = 0;
variable terminado: booleano = falso;
variable victoria: booleano = falso;
variable terminado_num: entero = 0;
variable victoria_num: entero = 0;
variable mensaje: cadena = "Encuentra el primer portal";

si comando != "init" {
    jugador_x = entero(entrada()); jugador_z = entero(entrada());
    objetivo_x = entero(entrada()); objetivo_z = entero(entrada());
    enemigo_1_x = entero(entrada()); enemigo_1_z = entero(entrada());
    enemigo_2_x = entero(entrada()); enemigo_2_z = entero(entrada());
    nivel = entero(entrada()); puntos = entero(entrada());
    vidas = entero(entrada()); turnos = entero(entrada());
    terminado_num = entero(entrada()); victoria_num = entero(entrada());
    terminado = terminado_num == 1; victoria = victoria_num == 1;
}

si comando == "r" {
    jugador_x = 0; jugador_z = 0; objetivo_x = 3; objetivo_z = 2;
    nivel = 1; puntos = 0; vidas = 3; turnos = 0;
    terminado = falso; victoria = falso; mensaje = "Partida reiniciada";
} sino_si comando == "q" {
    terminado = verdadero; victoria = falso; mensaje = "Partida cerrada";
} sino_si no terminado {
    si comando == "w" { jugador_z = jugador_z + 1; }
    sino_si comando == "s" { jugador_z = jugador_z - 1; }
    sino_si comando == "a" { jugador_x = jugador_x - 1; }
    sino_si comando == "d" { jugador_x = jugador_x + 1; }
    si comando == "w" o comando == "s" o comando == "a" o comando == "d" {
        turnos = turnos + 1; mensaje = "Movimiento aceptado por ST";
    }
}

si jugador_x < -10 { jugador_x = -10; }
si jugador_x > 10 { jugador_x = 10; }
si jugador_z < -10 { jugador_z = -10; }
si jugador_z > 10 { jugador_z = 10; }

// La transformación pasa por el núcleo motor3d antes de publicar el estado.
variable motor = motor3d.crear_motor("portal_runner"; 1180; 800);
motor.iniciar(); motor.actualizar(0.016);
variable matriz = motor3d.matriz_traslacion(motor3d.vector(flotante(jugador_x); 0.0; flotante(jugador_z)));
variable posicion = matriz.transformar_punto(motor3d.vector(0.0; 0.0; 0.0));
jugador_x = entero(posicion.x); jugador_z = entero(posicion.z);

si no terminado y jugador_x == objetivo_x y jugador_z == objetivo_z {
    puntos = puntos + 100; nivel = nivel + 1; mensaje = "Portal alcanzado: +100 puntos";
    si nivel == 2 { objetivo_x = -4; objetivo_z = 4; }
    sino_si nivel == 3 { objetivo_x = 6; objetivo_z = -3; }
    sino_si nivel == 4 { objetivo_x = -7; objetivo_z = -6; }
    sino_si nivel == 5 { objetivo_x = 1; objetivo_z = 8; }
    sino { terminado = verdadero; victoria = verdadero; mensaje = "Has completado Portal Runner"; }
}

mostrar("OK"); mostrar(jugador_x); mostrar(jugador_z);
mostrar(objetivo_x); mostrar(objetivo_z);
mostrar(enemigo_1_x); mostrar(enemigo_1_z); mostrar(enemigo_2_x); mostrar(enemigo_2_z);
mostrar(nivel); mostrar(puntos); mostrar(vidas); mostrar(turnos);
si terminado { mostrar(1); } sino { mostrar(0); }
si victoria { mostrar(1); } sino { mostrar(0); }
mostrar(mensaje);
