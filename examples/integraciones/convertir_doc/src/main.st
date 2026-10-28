importar "documentos";
// Requiere LibreOffice. Coloca un documento de prueba entrada.doc en esta carpeta.
intentar { documentos.convertir("entrada.doc", "salida.docx"); }
capturar error { mostrar(error); }
