// Validador de documentos JSON
// Informa del primer problema de sintaxis mediante excepciones capturables.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "json";
importar "io";
variable texto = io.leer_texto("documento.json");
intentar {
    json.comprobar(texto);
    mostrar("Documento válido");
    mostrar("Tipo: " + json.tipo(texto));
} capturar error {
    mostrar("Documento rechazado");
    mostrar(error);
}
