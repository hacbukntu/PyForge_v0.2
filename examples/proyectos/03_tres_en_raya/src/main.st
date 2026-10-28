// Tres en raya para dos jugadores
// Alterna jugadores, valida casillas y detecta filas, columnas y diagonales.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Revisa las ocho líneas posibles y devuelve el ganador o cero.
funcion ganador(t: lista) -> entero {
    variable lineas = [0,1,2,3,4,5,6,7,8,0,3,6,1,4,7,2,5,8,0,4,8,2,4,6];
    para i en rango(0, 8) {
        variable a = t[lineas[i*3]];
        si a != 0 y a == t[lineas[i*3+1]] y a == t[lineas[i*3+2]] { retornar a; }
    }
    retornar 0;
}
variable tablero = [0,0,0,0,0,0,0,0,0];
mostrar(tipo(tablero));
variable ganador_final = 0;
mostrar("Dos jugadores: elige casilla 0..8, por filas.");
para turno en rango(0, 9) {
    variable casilla = entero(entrada(""));
    si casilla < 0 o casilla > 8 { lanzar "Casilla fuera de rango"; }
    si tablero[casilla] != 0 { lanzar "Casilla ocupada"; }
    tablero[casilla] = turno % 2 + 1;
    para fila en rango(0, 3) { mostrar(interpolar("{} {} {}", tablero[fila*3], tablero[fila*3+1], tablero[fila*3+2])); }
    ganador_final = ganador(tablero);
    si ganador_final != 0 { romper; }
}
mostrar("Ganador: " + ganador_final); // Cero representa empate.
