// Las anotaciones son opcionales: el compilador deduce el tipo del valor inicial.
variable nombre = "Ana";
variable edad_texto = "20";
variable edad = entero(edad_texto);
variable precio = flotante("12.50");
variable activo = booleano(edad);
mostrar(tipo(nombre)); // cadena
mostrar(tipo(edad)); // entero
mostrar(tipo(precio)); // flotante
mostrar(tipo(activo)); // booleano
mostrar("El próximo año: " + (edad + 1));

// La conversión crea un valor nuevo; no cambia el tipo estático de edad_texto.
variable edad_como_cadena = cadena(edad);
mostrar(edad_como_cadena);
intentar {
    mostrar(entero("veinte"));
} capturar error {
    mostrar("No se pudo convertir: " + error);
}
