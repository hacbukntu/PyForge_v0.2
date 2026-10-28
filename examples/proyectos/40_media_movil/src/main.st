// Media móvil de ventas
// Suaviza una serie usando una ventana deslizante de tres muestras.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable ventas = [10,20,30,40,50];
variable ventana = 3;
variable suma = 0;
para i en rango(0, longitud(ventas)) {
    suma += ventas[i];
    si i >= ventana { suma -= ventas[i-ventana]; }
    si i >= ventana - 1 { mostrar("Media " + i + ": " + (flotante(suma)/flotante(ventana))); }
}
