// Arena de Cristales: recoge cinco objetivos mientras dos drones persiguen al jugador.
importar "motor3d";

funcion bloqueado(x: entero; z: entero) -> booleano {
    retornar (x == -3 y z == 2) o (x == 2 y z == -3) o
             (x == 4 y z == 3) o (x == -5 y z == -2);
}

variable comando = entrada();
variable jugador_x = 0; variable jugador_z = 0;
variable objetivo_x = 3; variable objetivo_z = 3;
variable enemigo_1_x = -6; variable enemigo_1_z = 6;
variable enemigo_2_x = 6; variable enemigo_2_z = -6;
variable nivel = 1; variable puntos = 0; variable vidas = 3; variable turnos = 0;
variable terminado = falso; variable victoria = falso;
variable terminado_num = 0; variable victoria_num = 0;
variable anterior_x = 0; variable anterior_z = 0;
variable mensaje = "Recoge el cristal amarillo";

si comando != "init" {
    jugador_x = entero(entrada()); jugador_z = entero(entrada());
    objetivo_x = entero(entrada()); objetivo_z = entero(entrada());
    enemigo_1_x = entero(entrada()); enemigo_1_z = entero(entrada());
    enemigo_2_x = entero(entrada()); enemigo_2_z = entero(entrada());
    nivel = entero(entrada()); puntos = entero(entrada()); vidas = entero(entrada()); turnos = entero(entrada());
    terminado_num = entero(entrada()); victoria_num = entero(entrada());
    terminado = terminado_num == 1; victoria = victoria_num == 1;
}

si comando == "r" {
    jugador_x = 0; jugador_z = 0; objetivo_x = 3; objetivo_z = 3;
    enemigo_1_x = -6; enemigo_1_z = 6; enemigo_2_x = 6; enemigo_2_z = -6;
    nivel = 1; puntos = 0; vidas = 3; turnos = 0; terminado = falso; victoria = falso;
    mensaje = "Arena reiniciada";
} sino_si comando == "q" {
    terminado = verdadero; mensaje = "Partida cerrada";
} sino_si no terminado {
    anterior_x = jugador_x; anterior_z = jugador_z;
    si comando == "w" { jugador_z = jugador_z + 1; }
    sino_si comando == "s" { jugador_z = jugador_z - 1; }
    sino_si comando == "a" { jugador_x = jugador_x - 1; }
    sino_si comando == "d" { jugador_x = jugador_x + 1; }
    si bloqueado(jugador_x; jugador_z) { jugador_x = anterior_x; jugador_z = anterior_z; mensaje = "Una roca bloquea el paso"; }
    turnos = turnos + 1;

    // Los drones avanzan un eje cada dos turnos. La persecución sigue presionando,
    // pero siempre deja rutas de escape comprobables hacia los cinco cristales.
    si turnos % 2 == 0 {
        si enemigo_1_x < jugador_x { enemigo_1_x = enemigo_1_x + 1; }
        sino_si enemigo_1_x > jugador_x { enemigo_1_x = enemigo_1_x - 1; }
        sino_si enemigo_1_z < jugador_z { enemigo_1_z = enemigo_1_z + 1; }
        sino_si enemigo_1_z > jugador_z { enemigo_1_z = enemigo_1_z - 1; }
        si enemigo_2_x < jugador_x { enemigo_2_x = enemigo_2_x + 1; }
        sino_si enemigo_2_x > jugador_x { enemigo_2_x = enemigo_2_x - 1; }
        sino_si enemigo_2_z < jugador_z { enemigo_2_z = enemigo_2_z + 1; }
        sino_si enemigo_2_z > jugador_z { enemigo_2_z = enemigo_2_z - 1; }
    }
}

si jugador_x < -7 { jugador_x = -7; } si jugador_x > 7 { jugador_x = 7; }
si jugador_z < -7 { jugador_z = -7; } si jugador_z > 7 { jugador_z = 7; }

si no terminado y ((jugador_x == enemigo_1_x y jugador_z == enemigo_1_z) o
                     (jugador_x == enemigo_2_x y jugador_z == enemigo_2_z)) {
    vidas = vidas - 1; jugador_x = 0; jugador_z = 0;
    enemigo_1_x = -6; enemigo_1_z = 6; enemigo_2_x = 6; enemigo_2_z = -6;
    mensaje = "Un dron te alcanzó: pierdes una vida";
    si vidas <= 0 { terminado = verdadero; victoria = falso; mensaje = "Los drones ganaron"; }
}

si no terminado y jugador_x == objetivo_x y jugador_z == objetivo_z {
    puntos = puntos + 75; nivel = nivel + 1; mensaje = "Cristal recogido: +75 puntos";
    si nivel == 2 { objetivo_x = -5; objetivo_z = 5; }
    sino_si nivel == 3 { objetivo_x = 5; objetivo_z = 0; }
    sino_si nivel == 4 { objetivo_x = -4; objetivo_z = -5; }
    sino_si nivel == 5 { objetivo_x = 1; objetivo_z = 6; }
    sino { terminado = verdadero; victoria = verdadero; mensaje = "Arena de Cristales completada"; }
}

variable motor = motor3d.crear_motor("cristales"; 1180; 800); motor.iniciar(); motor.actualizar(0.016);
mostrar("OK"); mostrar(jugador_x); mostrar(jugador_z); mostrar(objetivo_x); mostrar(objetivo_z);
mostrar(enemigo_1_x); mostrar(enemigo_1_z); mostrar(enemigo_2_x); mostrar(enemigo_2_z);
mostrar(nivel); mostrar(puntos); mostrar(vidas); mostrar(turnos);
si terminado { mostrar(1); } sino { mostrar(0); } si victoria { mostrar(1); } sino { mostrar(0); }
mostrar(mensaje);
