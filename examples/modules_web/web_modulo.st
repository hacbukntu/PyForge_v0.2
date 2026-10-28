importar "json";
funcion cambiar_desde_st(nombre: cadena; tipo: cadena) -> cadena {
    variable payload = json.objeto([
        "titulo",
        "tipo",
        "mensaje",
        "estado"
    ], [
        json.codificar("PyForge Web"),
        json.codificar(tipo),
        json.codificar("Hola, " + nombre + ". Cambiado desde ST."),
        json.codificar("ok")
    ]);
    retornar json.codificar(payload);
}
funcion estado_culebrita() -> cadena {
    variable estado = json.objeto([
        "titulo",
        "puntuacion",
        "velocidad",
        "direccion",
        "serpiente",
        "comida"
    ], [
        json.codificar("Culebrita ST"),
        json.de_entero(240),
        json.de_entero(140),
        json.codificar("derecha"),
        json.arreglo([
            json.arreglo([json.de_entero(8), json.de_entero(8)]),
            json.arreglo([json.de_entero(7), json.de_entero(8)]),
            json.arreglo([json.de_entero(6), json.de_entero(8)])
        ]),
        json.arreglo([json.de_entero(12), json.de_entero(6)])
    ]);
    retornar json.codificar(estado);
}
