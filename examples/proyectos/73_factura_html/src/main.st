// Informe HTML de factura
// Transforma líneas de una factura en una tabla con total calculado.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "html.st" como html;

variable productos = ["Cuaderno","Boligrafo"];
variable precios = [300,100];
variable cantidad = [2,5];
variable total = 0;
variable cuerpo = "<h1>Factura de ejemplo</h1><table><tr><th>Producto</th><th>Centavos</th></tr>";
para i en rango(0, longitud(productos)) {
    variable importe = precios[i]*cantidad[i]; total += importe;
    cuerpo += "<tr><td>" + html.escapar(productos[i]) + "</td><td>" + importe + "</td></tr>";
}
cuerpo += "</table><p>Total: " + total + " centavos</p>";

io.escribir_texto("index.html", "<!doctype html><html lang=\"es\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ST</title><body>" + cuerpo + "</body></html>");
mostrar("Generado: index.html");
