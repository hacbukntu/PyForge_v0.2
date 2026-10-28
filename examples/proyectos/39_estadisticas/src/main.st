// Estadísticas descriptivas
// Calcula media, mínimo, máximo y amplitud de una serie.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable datos = [10,20,15,5,30];
si longitud(datos) == 0 { lanzar "Serie vacía"; }
variable suma = 0;
variable minimo = datos[0];
variable maximo = datos[0];
para i en rango(0, longitud(datos)) {
    suma += datos[i];
    si datos[i] < minimo { minimo = datos[i]; }
    si datos[i] > maximo { maximo = datos[i]; }
}
mostrar("Media: " + (flotante(suma)/flotante(longitud(datos))));
mostrar("Mínimo: " + minimo); mostrar("Máximo: " + maximo);
mostrar("Amplitud: " + (maximo-minimo));
