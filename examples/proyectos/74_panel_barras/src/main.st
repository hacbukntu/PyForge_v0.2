// Panel HTML de actividad
// Muestra barras proporcionales y datos accesibles como texto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "html.st" como html;

variable dias = ["Lunes","Martes","Miercoles"];
variable completadas = [4,8,6];
variable meta = 10;
variable cuerpo = "<h1>Actividad semanal</h1>";
para i en rango(0, longitud(dias)) {
    cuerpo += "<p>" + html.escapar(dias[i]) + ": " + completadas[i] + "/" + meta + "</p>";
    cuerpo += "<meter min=\"0\" max=\"10\" value=\"" + completadas[i] + "\">" + completadas[i] + "</meter>";
}

io.escribir_texto("index.html", "<!doctype html><html lang=\"es\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ST</title><body>" + cuerpo + "</body></html>");
mostrar("Generado: index.html");
