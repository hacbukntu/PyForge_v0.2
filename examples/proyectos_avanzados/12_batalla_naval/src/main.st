importar "io";
importar "json";
// Un tablero de 4x4 usa índices fila*4+columna; 1 significa parte de un barco.
variable barcos = [0,1,1,0,0,0,0,0,0,0,1,0,0,0,0,0];
variable disparos = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
variable aciertos = 0;
variable turnos = 0;
mientras turnos < 8 y aciertos < 3 {
    variable casilla = entero(io.leer_linea("Disparo 0..15: "));
    turnos = turnos + 1;
    si casilla < 0 o casilla > 15 { mostrar("Fuera del tablero"); continuar; }
    si disparos[casilla] == 1 { mostrar("Ya disparaste ahí"); continuar; }
    disparos[casilla] = 1;
    si barcos[casilla] == 1 { aciertos = aciertos + 1; mostrar("Impacto"); }
    sino { mostrar("Agua"); }
}
mostrar("Victoria: " + cadena(aciertos == 3));
mostrar(json.objeto(["impactos", "turnos"], [json.de_entero(aciertos), json.de_entero(turnos)]));
