// El tipo de elemento se conserva en variables, parámetros, retornos y escrituras.
funcion primer_nombre(nombres: lista[cadena]) -> cadena {
    retornar nombres[0];
}
variable nombres: lista[cadena] = [];
nombres.agregar("Ana");
agregar(nombres, "Luis");
mostrar(primer_nombre(nombres));

// Los literales no vacíos infieren el tipo de sus elementos.
variable indicadores = [verdadero, falso];
mostrar(indicadores[0]);
variable precios: diccionario[flotante] = {};
precios["pan"] = 1.5;
mostrar(precios["pan"]);
