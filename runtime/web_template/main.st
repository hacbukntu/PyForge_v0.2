// Despacho nativo: solo operaciones declaradas, sin ejecutar código enviado por el navegador.
importar "json";
importar "vistas.st";
importar "snake.st";
variable solicitud = entrada("");
variable tipo = json.decodificar(json.obtener(solicitud; "tipo"));
variable nombre = "";
si tipo == "snake" {
    mostrar(estado_juego(solicitud));
} sino {
    nombre = json.decodificar(json.obtener(solicitud; "nombre"));
    mostrar(responder(nombre; tipo));
}
