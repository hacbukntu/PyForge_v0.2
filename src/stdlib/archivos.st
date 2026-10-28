// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Lee texto dentro del proyecto con codificación explícita.
funcion leer(ruta: cadena, codificacion: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("archivos.leer", "[" + json.codificar(ruta) + "," + json.codificar(codificacion) + "]"));
}

// Crea texto; sobrescribir exige verdadero explícito.
funcion escribir_texto(ruta: cadena, texto: cadena, sobrescribir: booleano, codificacion: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("archivos.escribir", "[" + json.codificar(ruta) + "," + json.codificar(texto) + "," + json.de_booleano(sobrescribir) + "," + json.codificar(codificacion) + "]"));
}

// Crea carpetas locales; requiere permiso de escritura.
funcion crear_carpeta(ruta: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("archivos.crear_carpeta", "[" + json.codificar(ruta) + "]"));
}

// Devuelve un arreglo JSON de nombres, hasta 128 entradas.
funcion listar(ruta: cadena) -> cadena {
    retornar integraciones.pedir("archivos.listar", "[" + json.codificar(ruta) + "]");
}

// Elimina un archivo o carpeta vacía; exige confirmación.
funcion eliminar(ruta: cadena, confirmar: booleano) -> booleano {
    retornar json.a_booleano(integraciones.pedir("archivos.eliminar", "[" + json.codificar(ruta) + "," + json.de_booleano(confirmar) + "]"));
}

// Copia sin sobrescribir destinos existentes.
funcion copiar(origen: cadena, destino: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("archivos.copiar", "[" + json.codificar(origen) + "," + json.codificar(destino) + "]"));
}

// Copia y elimina el original tras guardar correctamente.
funcion mover(origen: cadena, destino: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("archivos.mover", "[" + json.codificar(origen) + "," + json.codificar(destino) + "]"));
}

// Consulta existencia bajo los mismos permisos de lectura.
funcion existe(ruta: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("archivos.existe", "[" + json.codificar(ruta) + "]"));
}
