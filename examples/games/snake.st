// Ejemplo base de Snake escrito en ST.
// Este archivo demuestra una lógica de juego en .st, sin tocar el compilador.

funcion crear_serpiente() -> lista {
    variable cuerpo: lista = [2, 0, 1, 0, 0, 0];
    retornar cuerpo;
}

funcion mostrar_serpiente(cuerpo: lista) -> nada {
    variable i: entero = 0;
    mientras i < longitud(cuerpo) {
        mostrar("segmento " + i + ": (" + cuerpo[i] + ", " + cuerpo[i + 1] + ")");
        i = i + 2;
    }
}

funcion cabeza_choca_con_comida(cuerpo: lista, comida_x: entero, comida_y: entero) -> booleano {
    si cuerpo[0] == comida_x y cuerpo[1] == comida_y {
        retornar verdadero;
    }
    retornar falso;
}

variable serpiente: lista = crear_serpiente();
variable comida_x: entero = 5;
variable comida_y: entero = 5;

mostrar("Serpiente inicial:");
mostrar_serpiente(serpiente);
mostrar("Comida: (" + comida_x + ", " + comida_y + ")");

si cabeza_choca_con_comida(serpiente, comida_x, comida_y) {
    mostrar("La serpiente ya toca la comida");
} sino {
    mostrar("La serpiente aún no toca la comida");
}

mostrar("Base lista para ampliar a un juego completo desde .st");
