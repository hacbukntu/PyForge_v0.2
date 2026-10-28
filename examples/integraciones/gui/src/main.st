importar "gui";
importar "json";
importar "time";
// Requiere PySide6 y una sesión gráfica Linux/WSLg. Qt vive en un proceso dedicado.
variable ventana = gui.crear("{\"titulo\": \"Hola ST\", \"tema\": \"oscuro\", \"controles\": [{\"tipo\": \"etiqueta\", \"id\": \"saludo\", \"texto\": \"Pulsa el botón\"}, {\"tipo\": \"boton\", \"id\": \"boton\", \"texto\": \"Saludar\"}]}");
variable abierta = verdadero;
intentar {
    mientras abierta {
        variable evento = gui.evento(ventana);
        variable clase_evento = json.decodificar(json.obtener(evento, "evento"));
        si clase_evento == "click" { gui.cambiar_texto(ventana, "saludo", "Hola desde ST"); }
        si clase_evento == "cerrada" o clase_evento == "error" { abierta = falso; }
        time.dormir(10000);
    }
} finalmente { gui.cerrar(ventana); }
