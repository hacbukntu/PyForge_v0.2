// Una tupla conserva su orden y permite leer índices negativos, pero no mutar.
variable coordenadas: tupla = tupla(10, 20);
mostrar(coordenadas[-1]);

// Un conjunto conserva una sola entrada por entero, también al usar el método.
variable unicos: conjunto = conjunto(1, 1, 2);
unicos.agregar(3);
agregar(unicos, 3);
mostrar(longitud(unicos));

// es compara referencias: copiar la variable conserva la misma colección.
variable referencia = coordenadas;
mostrar(referencia es coordenadas);

// nulo es un valor almacenable; nada continúa indicando una función sin retorno.
funcion sin_dato() -> nulo {
    retornar nulo;
}
variable dato: nulo = sin_dato();
mostrar(dato);
mostrar(dato es nulo);
