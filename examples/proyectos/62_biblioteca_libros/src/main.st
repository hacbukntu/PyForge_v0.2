// Préstamos de biblioteca
// Gestiona el préstamo y devolución de un libro con transiciones válidas.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

clase Libro {
    variable titulo: cadena;
    variable prestado: booleano;
    // Solo presta ejemplares disponibles.
    funcion prestar() -> nada { si yo.prestado { lanzar "Ya está prestado"; } yo.prestado = verdadero; }
    // Una devolución requiere que exista un préstamo activo.
    funcion devolver() -> nada { si no yo.prestado { lanzar "No estaba prestado"; } yo.prestado = falso; }
}
variable libro = nuevo Libro("Aprender ST",falso);
libro.prestar();
mostrar("Prestado: " + libro.prestado);
intentar { libro.prestar(); } capturar error { mostrar(error); }
libro.devolver();
mostrar("Disponible: " + no libro.prestado);
