// Cierre de caja
// Reconcilia apertura, ventas, devoluciones y efectivo contado.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable apertura = 10000;
variable ventas = [2500,4000,1500];
variable devoluciones = [500];
variable esperado = apertura;
para i en rango(0, longitud(ventas)) { esperado += ventas[i]; }
para i en rango(0, longitud(devoluciones)) { esperado -= devoluciones[i]; }
mostrar("Efectivo contado en centavos:");
variable contado = entero(entrada(""));
mostrar("Esperado: " + esperado);
mostrar("Diferencia: " + (contado - esperado));
si contado == esperado { mostrar("Caja conciliada"); } sino { mostrar("Revisar movimientos"); }
