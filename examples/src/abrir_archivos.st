importar "math";
importar "time" como reloj;
desde "json" importar codificar como a_json;
importar "io";
importar "os";

// Este ejemplo reemplaza saludo.txt en el directorio actual.
variable nombre: cadena = io.leer_linea("Tu nombre: ");
io.escribir_texto("saludo.txt", "Hola, " + nombre + "\n");
io.imprimir(io.leer_texto("saludo.txt"));
mostrar(os.existe("saludo.txt"));





