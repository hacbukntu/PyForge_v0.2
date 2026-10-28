// Demo de la culebrita: una página de juego profesional en Django y lógica ST.
importar "json";
funcion estado_inicial() -> cadena {
    variable tablero = json.objeto([
        "titulo",
        "puntuacion",
        "velocidad",
        "direccion",
        "serpiente",
        "comida"
    ], [
        json.codificar("Culebrita ST"),
        json.de_entero(220),
        json.de_entero(140),
        json.codificar("derecha"),
        json.arreglo([
            json.arreglo([json.de_entero(5), json.de_entero(5)]),
            json.arreglo([json.de_entero(4), json.de_entero(5)]),
            json.arreglo([json.de_entero(3), json.de_entero(5)])
        ]),
        json.arreglo([json.de_entero(9), json.de_entero(7)])
    ]);
    retornar json.codificar(tablero);
}
