// Cambio mínimo con programación dinámica
// Obtiene el menor número de monedas para un importe alcanzable.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable monedas = [1,3,4];
variable objetivo = 6;
variable minimo: lista = [0];
para importe en rango(1, objetivo+1) {
    variable mejor = 1000000;
    para j en rango(0, longitud(monedas)) {
        si monedas[j] <= importe {
            si minimo[importe-monedas[j]] + 1 < mejor { mejor = minimo[importe-monedas[j]] + 1; }
        }
    }
    agregar(minimo,mejor);
}
mostrar("Monedas mínimas: " + minimo[objetivo]);
