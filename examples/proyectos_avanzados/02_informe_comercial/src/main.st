importar "documentos";
importar "archivos";
importar "json";
// Compone un informe sencillo con datos validados, sin confundirlo con maquetación avanzada.
funcion informe(datos: cadena) -> cadena {
    json.comprobar(datos);
    variable cliente = json.decodificar(json.obtener(datos, "cliente"));
    variable total = json.a_entero(json.obtener(datos, "total_centavos"));
    si total < 0 { lanzar "El total no puede ser negativo"; }
    retornar "INFORME COMERCIAL\nCliente: " + cliente + "\nTotal en centavos: " + total + "\nEstado: pendiente de pago";
}
variable texto = informe(archivos.leer("datos/venta.json", "utf-8"));
archivos.crear_carpeta("salida");
documentos.crear("salida/informe.docx", texto, falso);
documentos.crear("salida/informe.pdf", texto, falso);
documentos.crear("salida/informe.rtf", texto, falso);
documentos.crear("salida/informe.txt", texto, falso);
mostrar("Informes creados: DOCX, PDF, RTF y TXT");
mostrar(documentos.leer("salida/informe.docx"));
