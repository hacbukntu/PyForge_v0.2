// Detector de reuniones superpuestas
// Compara intervalos de minutos y detecta conflictos de agenda.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Intervalos semiabiertos: una reunión que termina a las 10 no choca con otra que empieza a las 10.
funcion chocan(a: entero, b: entero, c: entero, d: entero) -> booleano { retornar a < d y c < b; }
variable nombres = ["Equipo","Cliente","Revision"];
variable inicio = [540,570,600];
variable fin = [600,630,660];
para i en rango(0, longitud(nombres)) {
    para j en rango(i+1, longitud(nombres)) {
        si chocan(inicio[i],fin[i],inicio[j],fin[j]) { mostrar(nombres[i] + " / " + nombres[j]); }
    }
}
