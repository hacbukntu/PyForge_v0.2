// Pila de tareas
// Implementa apilar y desapilar con un contador lógico sobre una lista.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

clase Pila {
    variable datos: lista;
    variable usados: entero;
    // Reutiliza posiciones disponibles o amplía la lista si hace falta.
    funcion apilar(valor: entero) -> nada {
        si yo.usados == longitud(yo.datos) { agregar(yo.datos,valor); }
        sino { yo.datos[yo.usados] = valor; }
        yo.usados = yo.usados+1;
    }
    // El contador separa capacidad reservada y elementos activos.
    funcion sacar() -> entero {
        si yo.usados == 0 { lanzar "Pila vacía"; }
        yo.usados = yo.usados-1;
        retornar yo.datos[yo.usados];
    }
}
variable pila = nuevo Pila([],0);
pila.apilar(10); pila.apilar(20);
mostrar(pila.sacar()); mostrar(pila.sacar());
intentar { mostrar(pila.sacar()); } capturar error { mostrar(error); }
