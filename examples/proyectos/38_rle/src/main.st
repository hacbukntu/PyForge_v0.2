// Compresor RLE de texto
// Codifica secuencias de caracteres iguales como símbolo y cantidad.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable texto = "aaabbccccd";
variable resultado = "";
variable inicio = 0;
mientras inicio < longitud(texto) {
    variable fin = inicio + 1;
    mientras fin < longitud(texto) y texto[fin] == texto[inicio] { fin += 1; }
    resultado += texto[inicio] + ":" + (fin-inicio) + ";";
    inicio = fin;
}
mostrar(resultado);
