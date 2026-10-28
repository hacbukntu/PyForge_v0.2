// Ruta mínima en un grafo
// Recorre un grafo no ponderado con cola e identifica distancias.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

// Matriz plana de adyacencia: A-B, A-C, B-D y C-D.
variable enlaces = [0,1,1,0,1,0,0,1,1,0,0,1,0,1,1,0];
variable distancia = [0,-1,-1,-1];
variable cola = [0];
variable cabeza = 0;
mientras cabeza < longitud(cola) {
    variable actual = cola[cabeza]; cabeza += 1;
    para vecino en rango(0, 4) {
        si enlaces[actual*4+vecino] == 1 y distancia[vecino] == -1 {
            distancia[vecino] = distancia[actual]+1;
            agregar(cola,vecino);
        }
    }
}
para i en rango(0, 4) { mostrar("Nodo " + i + ": " + distancia[i]); }
