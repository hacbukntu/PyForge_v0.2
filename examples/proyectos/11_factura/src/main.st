// Factura en centavos
// Calcula subtotal, impuesto configurable y total evitando redondeo binario.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Los importes son centavos; la tasa es un porcentaje entero de demostración.
funcion total_lineas(precios: lista, cantidades: lista) -> entero {
    si longitud(precios) != longitud(cantidades) { lanzar "Listas incompatibles"; }
    variable suma = 0;
    para i en rango(0, longitud(precios)) {
        si precios[i] < 0 o cantidades[i] < 0 { lanzar "Importe negativo"; }
        suma += precios[i] * cantidades[i];
    }
    retornar suma;
}
variable subtotal = total_lineas([1250,750], [2,3]);
variable impuesto = (subtotal * 10 + 50) / 100;
mostrar("Subtotal: " + subtotal);
mostrar("Impuesto: " + impuesto);
mostrar("Total: " + (subtotal + impuesto));
