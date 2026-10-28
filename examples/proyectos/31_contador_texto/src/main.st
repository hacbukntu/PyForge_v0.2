// Contador de texto
// Cuenta caracteres Unicode, palabras y líneas de un texto de entrada.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Escribe una frase:");
variable texto = entrada();
variable palabras = 0;
variable dentro = falso;
para i en rango(0, longitud(texto)) {
    variable separador = texto[i] == " " o texto[i] == "\t";
    si separador { dentro = falso; }
    sino { si no dentro { palabras += 1; } dentro = verdadero; }
}
mostrar("Caracteres: " + longitud(texto));
mostrar("Palabras: " + palabras);
