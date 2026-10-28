// Repartir una cuenta
// Distribuye centavos y asigna el resto sin perder dinero en el reparto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Total en centavos y número de personas:");
variable total = entero(entrada(""));
variable personas = entero(entrada(""));
si total < 0 o personas < 1 o personas > 100 { lanzar "Datos fuera de rango"; }
variable base = total / personas;
variable resto = total % personas;
para i en rango(0, personas) {
    variable pago = base;
    si i < resto { pago += 1; }
    mostrar("Persona " + (i+1) + ": " + pago);
}
