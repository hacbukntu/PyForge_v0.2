// Plan de cuotas sin intereses
// Genera un calendario de cuotas iguales y ajusta la última por el resto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Total en centavos y cantidad de cuotas:");
variable total = entero(entrada(""));
variable cuotas = entero(entrada(""));
si total < 0 o cuotas < 1 o cuotas > 60 { lanzar "Plan inválido"; }
variable pagado = 0;
para i en rango(0, cuotas) {
    variable pago = total / cuotas;
    si i == cuotas - 1 { pago += total % cuotas; }
    pagado += pago;
    mostrar("Cuota " + (i+1) + ": " + pago);
}
mostrar("Comprobación: " + pagado);
