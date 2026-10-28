// Catálogo HTML
// Crea tarjetas de productos con etiquetas de disponibilidad.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "html.st" como html;

variable nombres = ["Libro ST","Cable USB"];
variable existencias = [3,0];
variable cuerpo = "<h1>Catálogo</h1>";
para i en rango(0, longitud(nombres)) {
    variable estado = "Disponible";
    si existencias[i] == 0 { estado = "Agotado"; }
    cuerpo += "<article><h2>" + html.escapar(nombres[i]) + "</h2><p>" + estado + "</p></article>";
}

io.escribir_texto("index.html", "<!doctype html><html lang=\"es\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ST</title><body>" + cuerpo + "</body></html>");
mostrar("Generado: index.html");
