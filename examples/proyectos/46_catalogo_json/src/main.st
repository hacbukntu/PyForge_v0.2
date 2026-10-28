// Consultar catálogo JSON
// Recorre registros anidados y filtra productos disponibles.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "json";
importar "io";
variable catalogo = io.leer_texto("catalogo.json");
variable cantidad = json.a_entero(json.obtener(catalogo,"cantidad"));
variable productos = json.obtener(catalogo,"productos");
para i en rango(0, cantidad) {
    variable producto = json.elemento(productos,i);
    si json.a_booleano(json.obtener(producto,"disponible")) {
        mostrar(json.decodificar(json.obtener(producto,"nombre")));
    }
}
