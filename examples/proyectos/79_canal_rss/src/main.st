// Canal RSS estático
// Construye un documento XML con títulos escapados y enlaces de ejemplo.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "html.st" como xml;
variable titulos = ["ST & herramientas","Curso disponible"];
variable salida = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>Noticias ST</title><link>https://example.test/</link><description>Novedades</description>";
para i en rango(0, longitud(titulos)) {
    salida += "<item><title>" + xml.escapar(titulos[i]) + "</title><link>https://example.test/noticia/" + i + "</link></item>";
}
salida += "</channel></rss>";
io.escribir_texto("noticias.xml",salida);
mostrar("RSS generado");
