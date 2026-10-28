// Generador de identificadores URL
// Transforma títulos ASCII a minúsculas con guiones y elimina símbolos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Título ASCII:");
variable titulo = entrada("");
variable mayus = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
variable minus = "abcdefghijklmnopqrstuvwxyz";
variable permitidos = minus + "0123456789";
variable salida = "";
variable separador = falso;
para i en rango(0, longitud(titulo)) {
    variable c = titulo[i];
    variable indice = buscar(mayus,c);
    si indice >= 0 { c = minus[indice]; }
    si buscar(permitidos,c) >= 0 {
        si separador y longitud(salida) > 0 { salida += "-"; }
        salida += c; separador = falso;
    } sino { separador = verdadero; }
}
mostrar(salida);
