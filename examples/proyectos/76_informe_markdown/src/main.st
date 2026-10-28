// Informe de avances Markdown
// Produce una tabla de seguimiento con porcentajes calculados.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
variable tareas = ["Editor","Documentacion","Pruebas"];
variable completado = [8,6,10];
variable total = [10,10,10];
variable salida = "# Avance del proyecto\n\n| Area | Porcentaje |\n| --- | --- |\n";
para i en rango(0, longitud(tareas)) {
    si total[i] <= 0 { lanzar "Total inválido"; }
    salida += "| " + tareas[i] + " | " + (completado[i]*100/total[i]) + "% |\n";
}
io.escribir_texto("avance.md",salida);
mostrar("Informe generado");
