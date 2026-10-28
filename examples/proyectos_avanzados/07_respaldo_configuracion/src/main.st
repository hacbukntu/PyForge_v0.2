importar "archivos";
importar "json";
importar "time";
importar "os";
// Usa una versión de datos explícita; un esquema desconocido no se copia como si fuera válido.
variable contenido = archivos.leer("datos/config.json", "utf-8");
si json.a_entero(json.obtener(contenido, "version")) != 1 { lanzar "Versión de configuración no admitida"; }
archivos.crear_carpeta("copias");
variable destino = "copias/config-" + time.ahora() + ".json";
archivos.copiar("datos/config.json", destino);
si archivos.leer(destino, "utf-8") != contenido { lanzar "Copia inconsistente"; }
// Mueve y retira solamente un marcador temporal creado por esta ejecución.
archivos.escribir_texto("copias/marcador.tmp", "prueba", falso, "utf-8");
archivos.mover("copias/marcador.tmp", "copias/completado.tmp");
archivos.eliminar("copias/completado.tmp", verdadero);
mostrar("Respaldo verificado: " + cadena(os.existe(destino)));
