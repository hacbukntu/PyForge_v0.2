// Agenda de contactos
// Busca teléfonos por nombre sin consultar claves inexistentes.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable nombres = ["Ana","Luis","Eva"];
variable telefonos = {"Ana":"555-0101","Luis":"555-0102","Eva":"555-0103"};
mostrar("Nombre exacto:");
variable consulta = entrada("");
variable encontrado = falso;
para i en rango(0, longitud(nombres)) {
    si nombres[i] == consulta { mostrar(telefonos[consulta]); encontrado = verdadero; romper; }
}
si no encontrado { mostrar("Contacto no encontrado"); }
