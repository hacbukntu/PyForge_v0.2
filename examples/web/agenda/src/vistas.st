// Lógica nativa de presentación. Django escapa el mensaje al insertarlo en HTML.
// Agrega aquí funciones de tu aplicación; no construyas SQL concatenando entradas.
importar "json";
funcion responder(nombre: cadena) -> cadena {
    retornar "{\"mensaje\":" + json.codificar("Hola, " + nombre + ". Esta respuesta viene de ST nativo.") + "}";
}
