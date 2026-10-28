// Vista ST compilada. Django presenta el mensaje escapado y nunca interpreta HTML del visitante.
importar "json";
funcion responder(nombre: cadena; tipo: cadena) -> cadena {
    retornar json.objeto(["mensaje"]; [json.codificar("Hola, " + nombre + ". Esta respuesta viene de ST nativo.")]);
}
