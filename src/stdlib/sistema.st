// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Lee una variable de entorno autorizada por su nombre exacto.
funcion entorno(nombre: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("sistema.entorno", "[" + json.codificar(nombre) + "]"));
}
