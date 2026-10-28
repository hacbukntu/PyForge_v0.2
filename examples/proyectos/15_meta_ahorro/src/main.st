// Meta de ahorro
// Estima cuántos meses hacen falta con aportaciones constantes, sin intereses.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// División redondeada hacia arriba para no prometer llegar antes de completar la meta.
funcion meses(meta: entero, actual: entero, aporte: entero) -> entero {
    si meta < 0 o actual < 0 o aporte <= 0 { lanzar "Datos inválidos"; }
    si actual >= meta { retornar 0; }
    variable falta = meta - actual;
    retornar falta / aporte + entero(falta % aporte != 0);
}
mostrar("Meta, ahorro actual y aporte mensual:");
variable meta = entero(entrada(""));
variable actual = entero(entrada(""));
variable aporte = entero(entrada(""));
mostrar("Meses: " + meses(meta, actual, aporte));
