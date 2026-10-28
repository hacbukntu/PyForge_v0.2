// Lector de CSV sencillo
// Separa una fila en campos conservando los vacíos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// CSV limitado sin comillas, campos multilínea ni separadores escapados.
funcion separar(fila: cadena) -> lista[cadena] {
    variable campos: lista[cadena] = [];
    variable campo = "";
    para i en rango(0, longitud(fila)) {
        si fila[i] == "," { agregar(campos,campo); campo = ""; }
        sino { campo += fila[i]; }
    }
    agregar(campos,campo);
    retornar campos;
}
variable campos = separar("Ana,20,,ST");
para i en rango(0, longitud(campos)) { mostrar(i + ": [" + campos[i] + "]"); }
