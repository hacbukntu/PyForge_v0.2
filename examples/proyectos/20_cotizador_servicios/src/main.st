// Cotizador de servicios
// Combina horas, tarifa, materiales y recargo por urgencia.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Devuelve una cotización entera en centavos, sin impuestos implícitos.
funcion cotizar(horas: entero, tarifa: entero, materiales: entero, urgente: booleano) -> entero {
    si horas < 0 o tarifa < 0 o materiales < 0 { lanzar "No se admiten negativos"; }
    variable base = horas * tarifa + materiales;
    si urgente { base += base / 4; }
    retornar base;
}
mostrar("Normal: " + cotizar(4,2500,1000,falso));
mostrar("Urgente: " + cotizar(4,2500,1000,verdadero));
