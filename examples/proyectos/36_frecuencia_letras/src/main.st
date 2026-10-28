// Frecuencia de caracteres
// Agrupa caracteres Unicode y mantiene su orden de aparición.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable texto = "banana";
variable letras: lista[cadena] = [];
variable frecuencias: diccionario = {};
para i en rango(0, longitud(texto)) {
    variable c = texto[i];
    variable existe = falso;
    para j en rango(0, longitud(letras)) { si letras[j] == c { existe = verdadero; } }
    si existe { frecuencias[c] = frecuencias[c] + 1; }
    sino { agregar(letras,c); frecuencias[c] = 1; }
}
para i en rango(0, longitud(letras)) { mostrar(letras[i] + ": " + frecuencias[letras[i]]); }
