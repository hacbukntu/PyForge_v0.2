importar "cifrado";
importar "archivos";
importar "json";
importar "os";
// El hash detecta cambios respecto a un resumen confiable; no prueba quién creó el archivo.
funcion resumen(ruta: cadena) -> cadena { retornar cifrado.hash(archivos.leer(ruta, "utf-8"), "sha256"); }
si no os.existe("datos/entrega.txt") { lanzar "Falta la entrega de prueba"; }
variable original = resumen("datos/entrega.txt");
archivos.escribir_texto("integridad.json", json.objeto(["archivo", "sha256"], [json.codificar("datos/entrega.txt"), json.codificar(original)]), falso, "utf-8");
mostrar("Entrega intacta: " + cadena(cifrado.comparar(original, resumen("datos/entrega.txt"))));
// La alteración se hace en una copia nueva, no en el documento original.
archivos.escribir_texto("alterado.txt", "Contenido diferente", falso, "utf-8");
mostrar("Cambio detectado: " + cadena(no cifrado.comparar(original, resumen("alterado.txt"))));
