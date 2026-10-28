importar "documentos";
// TXT/RTF no requieren bibliotecas adicionales. Los destinos deben ser nuevos.
documentos.crear("nota.txt", "Título de prueba\nTexto en español", falso);
documentos.sustituir("nota.txt", "prueba", "ejemplo");
mostrar(documentos.leer("nota.txt"));
documentos.crear("nota.rtf", "Texto enriquecido: español ñ", falso);
