// Cola de prioridades de tareas
// Ordena tareas por urgencia usando listas paralelas e intercambio estable.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable tareas = ["Documentar","Incidente","Revision"];
variable prioridad = [3,1,2];
// Temporales declarados fuera de las ramas para el análisis estático actual.
variable numero = 0;
variable nombre = "";
para pasada en rango(0, longitud(tareas)) {
    para i en rango(0, longitud(tareas)-1) {
        si prioridad[i] > prioridad[i+1] {
            numero = prioridad[i]; prioridad[i] = prioridad[i+1]; prioridad[i+1] = numero;
            nombre = tareas[i]; tareas[i] = tareas[i+1]; tareas[i+1] = nombre;
        }
    }
}
para i en rango(0, longitud(tareas)) { mostrar(prioridad[i] + ": " + tareas[i]); }
