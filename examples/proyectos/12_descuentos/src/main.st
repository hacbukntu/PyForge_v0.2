// Descuentos por volumen
// Aplica descuentos escalonados y obtiene el importe final en centavos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// La escala es comercial ficticia y puede ajustarse sin cambiar el cálculo.
funcion porcentaje(unidades: entero) -> entero {
    si unidades >= 20 { retornar 15; }
    si unidades >= 10 { retornar 8; }
    retornar 0;
}
mostrar("Cantidad de unidades:");
variable cantidad = entero(entrada(""));
si cantidad < 1 { lanzar "La cantidad debe ser positiva"; }
variable subtotal = cantidad * 500;
variable descuento = subtotal * porcentaje(cantidad) / 100;
mostrar("Descuento: " + descuento);
mostrar("Pagar: " + (subtotal - descuento));
