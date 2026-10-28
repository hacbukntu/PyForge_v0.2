// Orden de dependencias
// Planifica tareas de un grafo acíclico y detecta ciclos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// A precede B y C; B y C preceden D.
variable nombres = ["A","B","C","D"];
variable aristas = [0,1,1,0,0,0,0,1,0,0,0,1,0,0,0,0];
variable pendientes = [0,1,1,2];
variable cola: lista = [];
para i en rango(0, 4) { si pendientes[i] == 0 { agregar(cola,i); } }
variable cabeza = 0;
mientras cabeza < longitud(cola) {
    variable actual = cola[cabeza]; cabeza += 1; mostrar(nombres[actual]);
    para j en rango(0, 4) {
        si aristas[actual*4+j] == 1 {
            pendientes[j] = pendientes[j]-1;
            si pendientes[j] == 0 { agregar(cola,j); }
        }
    }
}
si cabeza != 4 { lanzar "El grafo contiene un ciclo"; }
