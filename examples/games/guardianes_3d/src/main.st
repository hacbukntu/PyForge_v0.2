// Guardianes del Núcleo: combate por turnos, dos drones y seis rondas.
importar "motor3d";

variable comando = entrada();
variable jugador_x = 0; variable jugador_z = -3;
variable objetivo_x = 0; variable objetivo_z = 0;
variable enemigo_1_x = -5; variable enemigo_1_z = 5;
variable enemigo_2_x = 5; variable enemigo_2_z = 5;
variable nivel = 1; variable puntos = 0; variable vidas = 15; variable turnos = 0;
variable terminado = falso; variable victoria = falso; variable terminado_num = 0; variable victoria_num = 0;
variable golpe = falso;
variable mensaje = "Defiende el núcleo verde";

si comando != "init" {
    jugador_x = entero(entrada()); jugador_z = entero(entrada()); objetivo_x = entero(entrada()); objetivo_z = entero(entrada());
    enemigo_1_x = entero(entrada()); enemigo_1_z = entero(entrada()); enemigo_2_x = entero(entrada()); enemigo_2_z = entero(entrada());
    nivel = entero(entrada()); puntos = entero(entrada()); vidas = entero(entrada()); turnos = entero(entrada());
    terminado_num = entero(entrada()); victoria_num = entero(entrada()); terminado = terminado_num == 1; victoria = victoria_num == 1;
}

si comando == "r" {
    jugador_x = 0; jugador_z = -3; enemigo_1_x = -5; enemigo_1_z = 5; enemigo_2_x = 5; enemigo_2_z = 5;
    nivel = 1; puntos = 0; vidas = 15; turnos = 0; terminado = falso; victoria = falso; mensaje = "Defensa reiniciada";
} sino_si comando == "q" { terminado = verdadero; mensaje = "Partida cerrada"; }
sino_si no terminado {
    si comando == "w" { jugador_z = jugador_z + 1; }
    sino_si comando == "s" { jugador_z = jugador_z - 1; }
    sino_si comando == "a" { jugador_x = jugador_x - 1; }
    sino_si comando == "d" { jugador_x = jugador_x + 1; }

    si comando == "f" {
        golpe = falso;
        si (enemigo_1_x == jugador_x y (enemigo_1_z == jugador_z + 1 o enemigo_1_z == jugador_z - 1)) o
           (enemigo_1_z == jugador_z y (enemigo_1_x == jugador_x + 1 o enemigo_1_x == jugador_x - 1)) {
            enemigo_1_x = -6; enemigo_1_z = 6; puntos = puntos + 50; golpe = verdadero;
        }
        si (enemigo_2_x == jugador_x y (enemigo_2_z == jugador_z + 1 o enemigo_2_z == jugador_z - 1)) o
           (enemigo_2_z == jugador_z y (enemigo_2_x == jugador_x + 1 o enemigo_2_x == jugador_x - 1)) {
            enemigo_2_x = 6; enemigo_2_z = 6; puntos = puntos + 50; golpe = verdadero;
        }
        si golpe { mensaje = "Ataque acertado: +50 puntos"; } sino { mensaje = "Ataque sin objetivo adyacente"; }
    }

    turnos = turnos + 1;
    si enemigo_1_x < 0 { enemigo_1_x = enemigo_1_x + 1; } sino_si enemigo_1_x > 0 { enemigo_1_x = enemigo_1_x - 1; }
    si enemigo_1_z < 0 { enemigo_1_z = enemigo_1_z + 1; } sino_si enemigo_1_z > 0 { enemigo_1_z = enemigo_1_z - 1; }
    si enemigo_2_x < 0 { enemigo_2_x = enemigo_2_x + 1; } sino_si enemigo_2_x > 0 { enemigo_2_x = enemigo_2_x - 1; }
    si enemigo_2_z < 0 { enemigo_2_z = enemigo_2_z + 1; } sino_si enemigo_2_z > 0 { enemigo_2_z = enemigo_2_z - 1; }
}

si jugador_x < -6 { jugador_x = -6; } si jugador_x > 6 { jugador_x = 6; }
si jugador_z < -6 { jugador_z = -6; } si jugador_z > 6 { jugador_z = 6; }

si no terminado y enemigo_1_x == 0 y enemigo_1_z == 0 {
    vidas = vidas - 1; enemigo_1_x = -6; enemigo_1_z = 6; mensaje = "El dron A dañó el núcleo";
}
si no terminado y enemigo_2_x == 0 y enemigo_2_z == 0 {
    vidas = vidas - 1; enemigo_2_x = 6; enemigo_2_z = 6; mensaje = "El dron B dañó el núcleo";
}
si vidas <= 0 { terminado = verdadero; victoria = falso; mensaje = "El núcleo fue destruido"; }
si no terminado y turnos >= nivel * 8 {
    nivel = nivel + 1; puntos = puntos + 100; mensaje = "Ronda superada: +100 puntos";
    enemigo_1_x = -6; enemigo_1_z = 6; enemigo_2_x = 6; enemigo_2_z = 6;
    si nivel > 6 { terminado = verdadero; victoria = verdadero; mensaje = "Has defendido las seis rondas"; }
}

variable motor = motor3d.crear_motor("guardianes"; 1180; 800); motor.iniciar(); motor.actualizar(0.016);
mostrar("OK"); mostrar(jugador_x); mostrar(jugador_z); mostrar(objetivo_x); mostrar(objetivo_z);
mostrar(enemigo_1_x); mostrar(enemigo_1_z); mostrar(enemigo_2_x); mostrar(enemigo_2_z);
mostrar(nivel); mostrar(puntos); mostrar(vidas); mostrar(turnos);
si terminado { mostrar(1); } sino { mostrar(0); } si victoria { mostrar(1); } sino { mostrar(0); }
mostrar(mensaje);
