// Verificador de palíndromos
// Compara caracteres extremos ignorando espacios y signos elegidos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Normalización explícita: el ejemplo distingue mayúsculas y acentos.
funcion normalizar(texto: cadena) -> cadena {
    retornar reemplazar(reemplazar(reemplazar(texto," ",""),",",""),".","");
}
mostrar("Texto en minúsculas:");
variable limpio = normalizar(entrada());
variable coincide = verdadero;
para i en rango(0, longitud(limpio) / 2) { si limpio[i] != limpio[-i-1] { coincide = falso; } }
mostrar("Palíndromo: " + coincide);
