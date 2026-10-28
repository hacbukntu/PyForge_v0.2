// Reservas con aforo
// Reserva y libera plazas sin superar un aforo fijo.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

clase Sala {
    variable capacidad: entero;
    variable ocupadas: entero;
    // Devuelve falso sin cambiar el estado cuando no quedan plazas suficientes.
    funcion reservar(plazas: entero) -> booleano {
        si plazas <= 0 { lanzar "Cantidad inválida"; }
        si yo.ocupadas + plazas > yo.capacidad { retornar falso; }
        yo.ocupadas = yo.ocupadas + plazas; retornar verdadero;
    }
    // Libera únicamente plazas que ya estaban ocupadas.
    funcion liberar(plazas: entero) -> nada {
        si plazas < 0 o plazas > yo.ocupadas { lanzar "Liberación inválida"; }
        yo.ocupadas = yo.ocupadas - plazas;
    }
}
variable sala = nuevo Sala(10,0);
mostrar(sala.reservar(7)); mostrar(sala.reservar(4)); sala.liberar(2);
mostrar("Disponibles: " + (sala.capacidad-sala.ocupadas));
