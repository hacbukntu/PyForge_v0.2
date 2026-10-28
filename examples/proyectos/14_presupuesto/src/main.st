// Control de presupuesto
// Clasifica gastos y avisa si una categoría supera su presupuesto.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable categorias = ["comida","transporte","ocio"];
variable limites = {"comida":15000,"transporte":5000,"ocio":4000};
variable gastado = {"comida":12000,"transporte":6200,"ocio":2500};
variable disponible = 0;
para i en rango(0, longitud(categorias)) {
    variable clave = categorias[i];
    variable saldo = limites[clave] - gastado[clave];
    disponible += saldo;
    mostrar(clave + ": " + saldo);
    si saldo < 0 { mostrar("Revisar " + clave); }
}
mostrar("Saldo global: " + disponible);
