// Búsqueda binaria
// Localiza un elemento en una colección ordenada con intervalo semiabierto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

funcion buscar_ordenado(datos: lista, objetivo: entero) -> entero {
    variable izquierda = 0;
    variable derecha = longitud(datos);
    mientras izquierda < derecha {
        variable medio = izquierda + (derecha-izquierda)/2;
        si datos[medio] == objetivo { retornar medio; }
        si datos[medio] < objetivo { izquierda = medio+1; } sino { derecha = medio; }
    }
    retornar -1;
}
mostrar(buscar_ordenado([2,4,6,8,10],8));
mostrar(buscar_ordenado([2,4,6,8,10],3));
