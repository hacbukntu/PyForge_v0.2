// Aventura de decisiones
// Explora dos rutas y combina una llave con una decisión final.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Elige bosque o cueva:");
variable ruta = entrada("");
variable llave = falso;
coincidir ruta {
    caso "bosque" { llave = verdadero; mostrar("Encontraste una llave"); }
    caso "cueva" { mostrar("Encontraste un mapa"); }
    sino { lanzar "Ruta desconocida"; }
}
mostrar("Elige puerta o regreso:");
variable accion = entrada("");
si accion == "puerta" y llave { mostrar("Tesoro encontrado"); }
sino_si accion == "puerta" { mostrar("La puerta está cerrada"); }
sino_si accion == "regreso" { mostrar("Volviste a casa"); }
sino { lanzar "Acción desconocida"; }
