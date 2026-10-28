// Entrada compilada para cada petición de la vista ST; recibe JSON acotado por stdin.
importar "json";
importar "vistas.st";
variable solicitud = entrada("");
variable nombre = json.decodificar(json.obtener(solicitud; "nombre"));
mostrar(responder(nombre));
