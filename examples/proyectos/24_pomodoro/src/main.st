// Planificador de bloques de concentración
// Genera una secuencia de trabajo y descansos sin bloquear minutos durante la demostración.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Cantidad de bloques (1..8):");
variable bloques = entero(entrada(""));
si bloques < 1 o bloques > 8 { lanzar "Cantidad inválida"; }
variable minutos = 0;
variable descanso = 5;
para i en rango(0, bloques) {
    mostrar("Trabajar 25 minutos"); minutos += 25;
    si i < bloques - 1 {
        descanso = 5;
        si (i+1) % 4 == 0 { descanso = 15; }
        mostrar("Descansar " + descanso + " minutos"); minutos += descanso;
    }
}
mostrar("Duración total: " + minutos);
