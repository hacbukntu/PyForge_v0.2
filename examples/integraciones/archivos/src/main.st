importar "archivos";
// Todos los nombres son relativos al proyecto; el destino nunca se sobrescribe implícitamente.
archivos.crear_carpeta("datos");
archivos.escribir_texto("datos/nota.txt", "Hola desde ST: español ñ", falso, "utf-8");
mostrar(archivos.leer("datos/nota.txt", "utf-8"));
archivos.copiar("datos/nota.txt", "datos/copia.txt");
mostrar(archivos.listar("datos"));
// Solo se borra la copia creada por este ejemplo, con confirmación explícita.
archivos.eliminar("datos/copia.txt", verdadero);
