// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Genera clave AES-256 base64 con el CSPRNG; no la imprimas ni guardes en el repositorio.
funcion clave() -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.clave", "[" + "]"));
}

// AES-256-GCM: nonce aleatorio, versión autenticada y datos asociados.
funcion cifrar(texto: cadena, clave_texto: cadena, asociados: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.cifrar", "[" + json.codificar(texto) + "," + json.codificar(clave_texto) + "," + json.codificar(asociados) + "]"));
}

// Autentica antes de devolver texto; rechaza modificación y clave incorrecta.
funcion descifrar(sobre: cadena, clave_texto: cadena, asociados: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.descifrar", "[" + json.codificar(sobre) + "," + json.codificar(clave_texto) + "," + json.codificar(asociados) + "]"));
}

// SHA256 o SHA512; no sustituye un hash de contraseñas.
funcion hash(texto: cadena, algoritmo: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.hash", "[" + json.codificar(texto) + "," + json.codificar(algoritmo) + "]"));
}

// HMAC-SHA256 con clave base64.
funcion hmac(texto: cadena, clave_texto: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.hmac", "[" + json.codificar(texto) + "," + json.codificar(clave_texto) + "]"));
}

// Compara bytes sin diferencias de tiempo dependientes de su contenido.
funcion comparar(izquierda: cadena, derecha: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("cifrado.comparar", "[" + json.codificar(izquierda) + "," + json.codificar(derecha) + "]"));
}
// Hash lento PBKDF2-HMAC-SHA256 con sal aleatoria; guarda el resultado, nunca la contraseña.
funcion hash_clave(clave_texto: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.hash_clave", "[" + json.codificar(clave_texto) + "]"));
}
// Comprueba contraseña con el coste y formato STP1; la comparación no filtra su contenido.
funcion verificar_clave(clave_texto: cadena, resumen: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("cifrado.verificar_clave", "[" + json.codificar(clave_texto) + "," + json.codificar(resumen) + "]"));
}
// Base64 es codificación, no cifrado. Facilita transporte de texto sobre APIs binarias.
funcion base64(texto: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.base64", "[" + json.codificar(texto) + "]"));
}
// Convierte base64 a UTF-8; los datos que no sean texto producen una excepción.
funcion desde_base64(texto: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("cifrado.desde_base64", "[" + json.codificar(texto) + "]"));
}
