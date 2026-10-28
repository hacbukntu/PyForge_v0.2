// Punto de equilibrio
// Calcula unidades necesarias para cubrir costes fijos con un margen positivo.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

funcion equilibrio(fijo: entero, precio: entero, costo: entero) -> entero {
    si fijo < 0 o costo < 0 o precio <= costo { lanzar "Se requiere margen positivo"; }
    variable margen = precio - costo;
    retornar fijo / margen + entero(fijo % margen != 0);
}
mostrar("Unidades: " + equilibrio(100000, 2500, 1500));
mostrar("Con coste fijo 100001: " + equilibrio(100001, 2500, 1500));
