// Laberinto Neón: navegación con colisiones deterministas y límite de turnos.
importar "motor3d";

funcion bloqueado(x: entero; z: entero) -> booleano {
    retornar (z == -4 y x >= -4 y x <= 2) o (x == 2 y z >= -3 y z <= 0) o
             (z == -1 y x >= -4 y x <= -2) o (x == -1 y z >= 0 y z <= 2) o
             (z == 2 y x >= 0 y x <= 4) o (x == 4 y z >= 3 y z <= 5) o
             (z == 4 y x >= -5 y x <= 1);
}

variable comando = entrada();
variable jugador_x = -6; variable jugador_z = -6;
variable objetivo_x = 6; variable objetivo_z = 6;
variable enemigo_1_x = 0; variable enemigo_1_z = 0; variable enemigo_2_x = 0; variable enemigo_2_z = 0;
variable nivel = 1; variable puntos = 0; variable vidas = 1; variable turnos = 0;
variable terminado = falso; variable victoria = falso; variable terminado_num = 0; variable victoria_num = 0;
variable anterior_x = 0; variable anterior_z = 0;
variable mensaje = "Busca la salida púrpura";

si comando != "init" {
    jugador_x = entero(entrada()); jugador_z = entero(entrada()); objetivo_x = entero(entrada()); objetivo_z = entero(entrada());
    enemigo_1_x = entero(entrada()); enemigo_1_z = entero(entrada()); enemigo_2_x = entero(entrada()); enemigo_2_z = entero(entrada());
    nivel = entero(entrada()); puntos = entero(entrada()); vidas = entero(entrada()); turnos = entero(entrada());
    terminado_num = entero(entrada()); victoria_num = entero(entrada()); terminado = terminado_num == 1; victoria = victoria_num == 1;
}

si comando == "r" {
    jugador_x = -6; jugador_z = -6; puntos = 0; turnos = 0; terminado = falso; victoria = falso; mensaje = "Laberinto reiniciado";
} sino_si comando == "q" { terminado = verdadero; mensaje = "Partida cerrada"; }
sino_si no terminado {
    anterior_x = jugador_x; anterior_z = jugador_z;
    si comando == "w" { jugador_z = jugador_z + 1; }
    sino_si comando == "s" { jugador_z = jugador_z - 1; }
    sino_si comando == "a" { jugador_x = jugador_x - 1; }
    sino_si comando == "d" { jugador_x = jugador_x + 1; }
    turnos = turnos + 1;
    si bloqueado(jugador_x; jugador_z) { jugador_x = anterior_x; jugador_z = anterior_z; mensaje = "El muro neón bloquea el camino"; }
    sino { mensaje = "Avanzas por el laberinto"; }
}

si jugador_x < -7 { jugador_x = -7; } si jugador_x > 7 { jugador_x = 7; }
si jugador_z < -7 { jugador_z = -7; } si jugador_z > 7 { jugador_z = 7; }
si no terminado y jugador_x == objetivo_x y jugador_z == objetivo_z {
    terminado = verdadero; victoria = verdadero; puntos = 1000 - turnos * 5;
    si puntos < 100 { puntos = 100; } mensaje = "Salida encontrada: laberinto completado";
}
si no terminado y turnos >= 120 { terminado = verdadero; victoria = falso; mensaje = "Se agotaron los 120 turnos"; }

variable motor = motor3d.crear_motor("laberinto"; 1180; 800); motor.iniciar(); motor.actualizar(0.016);
mostrar("OK"); mostrar(jugador_x); mostrar(jugador_z); mostrar(objetivo_x); mostrar(objetivo_z);
mostrar(enemigo_1_x); mostrar(enemigo_1_z); mostrar(enemigo_2_x); mostrar(enemigo_2_z);
mostrar(nivel); mostrar(puntos); mostrar(vidas); mostrar(turnos);
si terminado { mostrar(1); } sino { mostrar(0); } si victoria { mostrar(1); } sino { mostrar(0); }
mostrar(mensaje);
