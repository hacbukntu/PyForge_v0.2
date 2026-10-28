// Punto público de la biblioteca. La anotación acepta matrices homogéneas.
funcion sumar_filas(filas: lista[lista[entero]]) -> entero {
    variable total = 0;
    para i en rango(longitud(filas)) {
        para j en rango(longitud(filas[i])) {
            total = total + filas[i][j];
        }
    }
    retornar total;
}
