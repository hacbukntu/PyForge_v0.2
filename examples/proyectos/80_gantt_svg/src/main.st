// Cronograma SVG
// Convierte duraciones y posiciones de tareas en un gráfico vectorial autónomo.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "html.st" como xml;
variable nombres = ["Diseno","Codigo","Pruebas"];
variable inicio = [0,2,5];
variable duracion = [2,3,2];
variable salida = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"140\" role=\"img\" aria-label=\"Cronograma\"><rect width=\"400\" height=\"140\" fill=\"#111214\"/>";
para i en rango(0, longitud(nombres)) {
    variable y_pos = i*40+20;
    salida += "<text x=\"5\" y=\"" + y_pos + "\" fill=\"white\">" + xml.escapar(nombres[i]) + "</text>";
    salida += "<rect x=\"" + (100+inicio[i]*35) + "\" y=\"" + (y_pos-12) + "\" width=\"" + (duracion[i]*35) + "\" height=\"18\" fill=\"#e6c566\"/>";
}
salida += "</svg>";
io.escribir_texto("cronograma.svg",salida);
mostrar("Cronograma generado");
