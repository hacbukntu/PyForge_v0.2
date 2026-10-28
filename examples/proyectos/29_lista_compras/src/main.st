// Lista de compras consolidada
// Acumula productos repetidos conservando el orden de primera aparición.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable pedidos = ["pan","leche","pan","arroz","leche"];
variable orden: lista[cadena] = [];
variable cantidades: diccionario = {};
para i en rango(0, longitud(pedidos)) {
    variable producto = pedidos[i];
    variable visto = falso;
    para j en rango(0, longitud(orden)) { si orden[j] == producto { visto = verdadero; } }
    si visto { cantidades[producto] = cantidades[producto] + 1; }
    sino { agregar(orden, producto); cantidades[producto] = 1; }
}
para i en rango(0, longitud(orden)) { mostrar(orden[i] + ": " + cantidades[orden[i]]); }
