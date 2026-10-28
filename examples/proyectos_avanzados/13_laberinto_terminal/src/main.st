importar "io";
// Tablero rectangular: # pared, . camino, S inicio y E salida.
variable mapa = ["#####", "#S..#", "###.#", "#..E#", "#####"];
variable fila = 1;
variable columna = 1;
variable pasos = 0;
mientras mapa[fila][columna] != "E" y pasos < 30 {
    para yfila en rango(0, 5) {
        variable linea = "";
        para xcol en rango(0, 5) {
            si fila == yfila y columna == xcol { linea = linea + "@"; }
            sino { linea = linea + mapa[yfila][xcol]; }
        }
        mostrar(linea);
    }
    variable orden = io.leer_linea("Mover w/a/s/d, q para salir: ");
    si orden == "q" o orden == "" { romper; }
    variable nf = fila;
    variable nc = columna;
    si orden == "w" { nf = nf - 1; }
    sino_si orden == "s" { nf = nf + 1; }
    sino_si orden == "a" { nc = nc - 1; }
    sino_si orden == "d" { nc = nc + 1; }
    sino { mostrar("Orden desconocida"); continuar; }
    pasos = pasos + 1;
    si nf >= 0 y nf < 5 y nc >= 0 y nc < 5 {
        si mapa[nf][nc] != "#" { fila = nf; columna = nc; }
    }
}
mostrar("Salida alcanzada: " + cadena(mapa[fila][columna] == "E"));
