// Plan de estudio
// Distribuye temas en días según un presupuesto de minutos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable temas = ["Variables","Funciones","Colecciones","Modulos","Clases"];
variable minutos = [30,50,40,60,30];
variable capacidad = 90;
variable dia = 1;
variable usados = 0;
para i en rango(0, longitud(temas)) {
    si minutos[i] > capacidad { lanzar "Tema demasiado largo"; }
    si usados + minutos[i] > capacidad { dia += 1; usados = 0; }
    usados += minutos[i];
    mostrar("Día " + dia + ": " + temas[i]);
}
