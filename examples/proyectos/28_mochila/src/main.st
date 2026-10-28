// Lista de equipaje por peso
// Selecciona elementos prioritarios que caben en un límite de peso.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable nombres = ["Portatil","Cargador","Libro","Chaqueta"];
variable pesos = [1800,300,900,700];
variable limite = 3000;
variable cargado = 0;
para i en rango(0, longitud(nombres)) {
    si cargado + pesos[i] <= limite { cargado += pesos[i]; mostrar("Llevar: " + nombres[i]); }
    sino { mostrar("Dejar: " + nombres[i]); }
}
mostrar("Gramos: " + cargado);
