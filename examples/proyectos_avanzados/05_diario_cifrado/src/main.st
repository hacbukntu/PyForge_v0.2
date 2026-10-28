importar "cifrado";
importar "archivos";
importar "io";
importar "sistema";
// La clave procede del entorno autorizado, nunca de una constante en el código.
variable clave = sistema.entorno("ST_DIARIO_CLAVE");
si clave == "" { lanzar "Configura ST_DIARIO_CLAVE con 32 bytes aleatorios en base64"; }
variable nota = io.leer_linea("Nota de prueba (la terminal puede mostrarla): ");
si nota == "" { lanzar "No se guardan notas vacías"; }
variable sobre = cifrado.cifrar(nota, clave, "diario-ST-v1");
archivos.escribir_texto("diario.stenc", sobre, falso, "utf-8");
variable recuperado = cifrado.descifrar(archivos.leer("diario.stenc", "utf-8"), clave, "diario-ST-v1");
si recuperado != nota { lanzar "Falló la verificación del diario"; }
mostrar("Diario cifrado y recuperación verificada");
