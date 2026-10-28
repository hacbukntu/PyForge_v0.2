// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Crea TXT, RTF, DOCX o PDF simple; requiere biblioteca del formato.
funcion crear(ruta: cadena, texto: cadena, sobrescribir: booleano) -> booleano {
    retornar json.a_booleano(integraciones.pedir("documentos.crear", "[" + json.codificar(ruta) + "," + json.codificar(texto) + "," + json.de_booleano(sobrescribir) + "]"));
}

// Extrae texto TXT, DOCX o PDF; no realiza OCR.
funcion leer(ruta: cadena) -> cadena {
    retornar json.decodificar(integraciones.pedir("documentos.leer", "[" + json.codificar(ruta) + "]"));
}

// Edita TXT o runs DOCX; no edita posiciones arbitrarias de un PDF.
funcion sustituir(ruta: cadena, anterior: cadena, nuevo_texto: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("documentos.reemplazar", "[" + json.codificar(ruta) + "," + json.codificar(anterior) + "," + json.codificar(nuevo_texto) + "]"));
}

// Convierte mediante LibreOffice autorizado sin sobrescribir el destino.
funcion convertir(origen: cadena, destino: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("documentos.convertir", "[" + json.codificar(origen) + "," + json.codificar(destino) + "]"));
}

// Une PDF; origenes es un arreglo JSON de rutas.
funcion unir_pdf(origenes: cadena, destino: cadena) -> booleano {
    json.comprobar(origenes);
    retornar json.a_booleano(integraciones.pedir("documentos.unir_pdf", "[" + origenes + "," + json.codificar(destino) + "]"));
}

// Extrae una página usando índice desde cero.
funcion extraer_pdf(origen: cadena, destino: cadena, pagina: entero) -> booleano {
    retornar json.a_booleano(integraciones.pedir("documentos.extraer_pdf", "[" + json.codificar(origen) + "," + json.codificar(destino) + "," + json.de_entero(pagina) + "]"));
}
