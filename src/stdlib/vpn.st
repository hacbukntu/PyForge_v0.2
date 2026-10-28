// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Detecta herramientas e interfaces locales; no afirma que haya un túnel activo.
funcion detectar() -> cadena {
    retornar integraciones.pedir("vpn.detectar", "[" + "]");
}
// Revisión conservadora de perfil SIN claves; no conecta ni modifica la red.
funcion validar(ruta: cadena, protocolo: cadena) -> cadena {
    retornar integraciones.pedir("vpn.validar", "[" + json.codificar(ruta) + "," + json.codificar(protocolo) + "]");
}
