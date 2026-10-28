// Operaciones entre conjuntos
// Calcula unión e intersección eliminando repetidos con conjuntos nativos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Las listas permiten recorrer candidatos; los conjuntos eliminan repetidos.
// El lenguaje actual no permite indexar directamente un conjunto.
variable a = [1,2,2,3];
variable b = [3,4,5];
variable union_valores = conjunto();
variable interseccion = conjunto();
para i en rango(0, longitud(a)) {
    agregar(union_valores,a[i]);
    para j en rango(0, longitud(b)) { si a[i] == b[j] { agregar(interseccion,a[i]); } }
}
para i en rango(0, longitud(b)) { agregar(union_valores,b[i]); }
mostrar("Unión: " + longitud(union_valores));
mostrar("Intersección: " + longitud(interseccion));

