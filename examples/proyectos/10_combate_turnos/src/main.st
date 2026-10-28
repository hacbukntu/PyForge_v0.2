// Combate por turnos
// Administra salud, pociones y ataques en una batalla de consola.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable salud = 20;
variable enemigo = 18;
variable pociones = 1;
mostrar("Acciones: atacar o curar.");
para turno en rango(0, 10) {
    variable orden = entrada("");
    si orden == "atacar" { enemigo -= 6; }
    sino_si orden == "curar" y pociones > 0 { salud += 5; pociones -= 1; }
    sino { lanzar "Acción inválida o sin pociones"; }
    si enemigo <= 0 { mostrar("Victoria"); romper; }
    salud -= 4;
    mostrar(interpolar("Salud: {}, rival: {}", salud, enemigo));
    si salud <= 0 { mostrar("Derrota"); romper; }
}
