// Ejemplo interactivo: ejecútalo en una terminal real, no con entrada redirigida.
importar "teclado";
importar "time";

variable jugando = verdadero;
variable tecla = teclado.ninguna();
si no teclado.es_terminal() {
    mostrar("Este ejemplo necesita una terminal interactiva.");
} sino {
    mostrar("Usa las flechas o WASD. Pulsa Q o Escape para salir.");
    mientras jugando {
        tecla = teclado.leer(50);
        si tecla == teclado.arriba() o tecla == 119 { mostrar("arriba"); }
        sino_si tecla == teclado.abajo() o tecla == 115 { mostrar("abajo"); }
        sino_si tecla == teclado.izquierda() o tecla == 97 { mostrar("izquierda"); }
        sino_si tecla == teclado.derecha() o tecla == 100 { mostrar("derecha"); }
        sino_si tecla == 113 o tecla == teclado.escape() { jugando = falso; }
        // Aquí se actualizarían posición, colisiones y dibujo del juego.
        time.dormir(16);
    }
    mostrar("Juego terminado; el terminal fue restaurado.");
}
