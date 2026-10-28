// Multiplicación de matrices
// Multiplica dos matrices 2x2 usando almacenamiento plano.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable a = [1,2,3,4];
variable b = [5,6,7,8];
variable resultado = [0,0,0,0];
para fila en rango(0, 2) {
    para columna en rango(0, 2) {
        variable suma = 0;
        para k en rango(0, 2) { suma += a[fila*2+k] * b[k*2+columna]; }
        resultado[fila*2+columna] = suma;
    }
}
para fila en rango(0, 2) { mostrar(interpolar("{} {}",resultado[fila*2],resultado[fila*2+1])); }
