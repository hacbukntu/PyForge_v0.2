// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Ejecuta ruta autorizada sin shell, timeout y salida limitada; devuelve JSON.
funcion ejecutar(programa: cadena, argumentos: cadena, segundos: entero) -> cadena {
    json.comprobar(argumentos);
    retornar integraciones.pedir("comandos.ejecutar", "[" + json.codificar(programa) + "," + argumentos + "," + json.de_entero(segundos) + "]");
}
