// Transporte opcional: JSON local al auxiliar Python, sin shell ni conexión TCP.
// Esta función común valida la respuesta y transforma errores en excepciones ST.
importar "json";
funcion pedir(operacion: cadena, argumentos: cadena) -> cadena {
    json.comprobar(argumentos);
    variable solicitud = "{\"operacion\":" + json.codificar(operacion) + ",\"argumentos\":" + argumentos + "}";
    variable respuesta = aux_peticion(solicitud);
    si json.obtener(respuesta, "ok") != "true" {
        lanzar json.decodificar(json.obtener(respuesta, "error"));
    }
    retornar json.obtener(respuesta, "resultado");
}
// Informa de versión, plataforma y dependencias realmente detectadas, como JSON.
funcion informacion() -> cadena { retornar pedir("api.informacion", "[]"); }
// Enumera únicamente los permisos declarados en PyForge.toml del directorio actual.
funcion permisos() -> cadena { retornar pedir("api.permisos", "[]"); }
