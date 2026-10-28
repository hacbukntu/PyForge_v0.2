// Laberinto de cuadrícula
// Navega con WASD, respeta paredes y llega a una salida.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Mapa de 3 por 3, con una pared central; índice = fila * ancho + columna.
variable mapa = [0,0,0,0,1,0,0,0,0];
variable x = 0;
variable y_pos = 0;
mostrar("Llega a 2,2 con w/a/s/d. Máximo 12 movimientos.");
para turno en rango(0, 12) {
    variable orden = entrada("");
    variable nx = x;
    variable ny = y_pos;
    coincidir orden {
        caso "w" { ny -= 1; } caso "s" { ny += 1; }
        caso "a" { nx -= 1; } caso "d" { nx += 1; }
        sino { lanzar "Usa w/a/s/d"; }
    }
    si nx >= 0 y nx < 3 y ny >= 0 y ny < 3 {
        si mapa[ny*3+nx] == 0 { x = nx; y_pos = ny; }
    }
    mostrar(interpolar("Posición: {},{}", x, y_pos));
    si x == 2 y y_pos == 2 { mostrar("Salida encontrada"); romper; }
}
