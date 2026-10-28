// Motor mínimo 2D para ST.
// Está pensado como una base segura para escenas, entidades y bucles del juego.

clase Entidad {
    variable nombre: cadena;
    variable x: entero;
    variable pos_y: entero;
    variable ancho: entero;
    variable alto: entero;
    variable activo: booleano;
}

implementar Entidad {
    funcion mover(dx: entero, dy: entero) -> nada {
        yo.x = yo.x + dx;
        yo.pos_y = yo.pos_y + dy;
    }

    funcion colisiona_con(otra: Entidad) -> booleano {
        si yo.x < otra.x + otra.ancho y yo.x + yo.ancho > otra.x y yo.pos_y < otra.pos_y + otra.alto y yo.pos_y + yo.alto > otra.pos_y {
            retornar verdadero;
        }
        retornar falso;
    }
}

clase Escena {
    variable nombre: cadena;
    variable jugador: Entidad;
    variable enemigo: Entidad;
    variable fondo: cadena;
    variable puntuacion: entero;
}

implementar Escena {
    funcion actualizar() -> nada {
        yo.jugador.mover(1, 0);
        yo.enemigo.mover(-1, 1);
        si yo.jugador.colisiona_con(yo.enemigo) {
            yo.puntuacion = yo.puntuacion + 10;
        }
    }
}

clase Motor2D {
    variable escena_actual: Escena;
    variable ejecutando: booleano;
    variable paso: entero;
}

implementar Motor2D {
    funcion iniciar(escena: Escena) -> nada {
        yo.escena_actual = escena;
        yo.ejecutando = verdadero;
        yo.paso = 0;
    }

    funcion frame() -> nada {
        si yo.ejecutando {
            yo.escena_actual.actualizar();
            yo.paso = yo.paso + 1;
        }
    }

    funcion detener() -> nada {
        yo.ejecutando = falso;
    }
}

funcion crear_entidad(nombre: cadena, x_inicial: entero, y_inicial: entero) -> Entidad {
    retornar nuevo Entidad(nombre, x_inicial, y_inicial, 10, 10, verdadero);
}

funcion crear_escena(nombre_escena: cadena) -> Escena {
    variable jugador: Entidad = crear_entidad("jugador", 10, 10);
    variable enemigo: Entidad = crear_entidad("enemigo", 40, 20);
    retornar nuevo Escena(nombre_escena, jugador, enemigo, "negro", 0);
}

funcion crear_motor() -> Motor2D {
    variable escena: Escena = crear_escena("inicio");
    retornar nuevo Motor2D(escena, falso, 0);
}

variable jugador: Entidad = crear_entidad("jugador", 10, 10);
variable enemigo: Entidad = crear_entidad("enemigo", 40, 20);
variable escena: Escena = crear_escena("nivel_1");
variable motor: Motor2D = crear_motor();

motor.iniciar(escena);

mostrar("Escena: " + escena.nombre);
mostrar("Puntuacion inicial: " + escena.puntuacion);

jugador.mover(3, 0);
enemigo.mover(1, 1);

motor.frame();

si jugador.colisiona_con(enemigo) {
    mostrar("Colision detectada");
} sino {
    mostrar("Sin colision");
}

mostrar("Paso actual: " + motor.paso);
mostrar("Puntuacion final: " + escena.puntuacion);
