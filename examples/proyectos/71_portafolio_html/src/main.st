// Portafolio HTML
// Genera una página personal con proyectos escapando el nombre del autor.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "html.st" como html;

// El nombre se trata como dato, nunca como una etiqueta HTML.
variable autor = "Ana & Luis";
variable proyectos = ["Inventario","Agenda","Juego ST"];
variable cuerpo = "<h1>Portafolio de " + html.escapar(autor) + "</h1><ul>";
para i en rango(0, longitud(proyectos)) { cuerpo += "<li>" + html.escapar(proyectos[i]) + "</li>"; }
cuerpo += "</ul>";

io.escribir_texto("index.html", "<!doctype html><html lang=\"es\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ST</title><body>" + cuerpo + "</body></html>");
mostrar("Generado: index.html");
