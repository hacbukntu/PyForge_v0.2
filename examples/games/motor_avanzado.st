// Motor 2D avanzado en ST.
// Este ejemplo extiende la base mínima de entidades y escenas con:
// - físicas simples
// - cámara
// - lista de entidades por escena
// - bucle principal del juego
// Todo sigue escrito en .st y sin tocar el compilador base.

clase Entidad2D {
    variable nombre: cadena;
    variable x: entero;
    variable pos_y: entero;
    variable ancho: entero;
    variable alto: entero;
    variable activo: booleano;
}

implementar Entidad2D {
    funcion mover(dx: entero, dy: entero) -> nada {
        yo.x = yo.x + dx;
        yo.pos_y = yo.pos_y + dy;
    }

    funcion aplicar_gravedad() -> nada {
        si yo.activo {
            yo.pos_y = yo.pos_y + 1;
            si yo.pos_y > 80 {
                yo.pos_y = 80;
            }
        }
    }

    funcion saltar() -> nada {
        si yo.activo y yo.pos_y >= 70 {
            yo.pos_y = yo.pos_y - 7;
        }
    }

    funcion colisiona_con(otra: Entidad2D) -> booleano {
        si yo.x < otra.x + otra.ancho y yo.x + yo.ancho > otra.x y yo.pos_y < otra.pos_y + otra.alto y yo.pos_y + yo.alto > otra.pos_y {
            retornar verdadero;
        }
        retornar falso;
    }
}

clase Camara {
    variable x: entero;
    variable pos_y: entero;
    variable ancho: entero;
    variable alto: entero;
}

implementar Camara {
    funcion seguir(entidad: Entidad2D) -> nada {
        yo.x = entidad.x - 10;
        yo.pos_y = entidad.pos_y - 5;

        si yo.x < 0 {
            yo.x = 0;
        }
        si yo.pos_y < 0 {
            yo.pos_y = 0;
        }
    }
}

clase Escena2D {
    variable nombre: cadena;
    variable fondo: cadena;
    variable jugador: Entidad2D;
    variable enemigo: Entidad2D;
    variable objetivo: entero;
    variable estado: cadena;
}

implementar Escena2D {
    funcion preparar_juego(jugador_local: Entidad2D, enemigo_local: Entidad2D) -> nada {
        yo.jugador = jugador_local;
        yo.enemigo = enemigo_local;
    }

    funcion contar_activos() -> entero {
        variable total: entero = 0;

        si yo.jugador.activo {
            total = total + 1;
        }
        si yo.enemigo.activo {
            total = total + 1;
        }

        retornar total;
    }

    funcion actualizar() -> nada {
        si yo.jugador.activo {
            yo.jugador.aplicar_gravedad();
        }
        si yo.enemigo.activo {
            yo.enemigo.aplicar_gravedad();
        }

        si yo.objetivo > 0 {
            yo.estado = "jugando";
        } sino {
            yo.estado = "completado";
        }
    }
}

clase Motor2DPro {
    variable escena_actual: Escena2D;
    variable camara: Camara;
    variable ejecutando: booleano;
    variable paso: entero;
}

implementar Motor2DPro {
    funcion iniciar(escena: Escena2D) -> nada {
        yo.escena_actual = escena;
        yo.ejecutando = verdadero;
        yo.paso = 0;
        yo.camara.x = 0;
        yo.camara.pos_y = 0;
    }

    funcion frame() -> nada {
        si yo.ejecutando {
            yo.escena_actual.actualizar();
            yo.paso = yo.paso + 1;
            yo.camara.seguir(yo.escena_actual.jugador);
        }
    }

    funcion detener() -> nada {
        yo.ejecutando = falso;
    }
}

funcion crear_entidad(nombre: cadena, x_inicial: entero, y_inicial: entero) -> Entidad2D {
    retornar nuevo Entidad2D(nombre, x_inicial, y_inicial, 10, 12, verdadero);
}

funcion crear_escena(nombre_escena: cadena) -> Escena2D {
    variable escena: Escena2D = nuevo Escena2D(nombre_escena, "noche", nuevo Entidad2D("jugador", 0, 0, 10, 12, verdadero), nuevo Entidad2D("enemigo", 0, 0, 10, 12, verdadero), 3, "cargando");
    retornar escena;
}

funcion crear_motor() -> Motor2DPro {
    variable escena: Escena2D = crear_escena("nivel_1");
    variable camara: Camara = nuevo Camara(0, 0, 80, 50);
    retornar nuevo Motor2DPro(escena, camara, falso, 0);
}

variable jugador: Entidad2D = crear_entidad("jugador", 12, 18);
variable enemigo: Entidad2D = crear_entidad("enemigo", 48, 22);
variable escena: Escena2D = crear_escena("nivel_1");
variable motor: Motor2DPro = crear_motor();

escena.preparar_juego(jugador, enemigo);
motor.iniciar(escena);

jugador.saltar();
mostrar("Escena: " + escena.nombre);
mostrar("Estado inicial: " + escena.estado);

motor.frame();

si jugador.colisiona_con(enemigo) {
    mostrar("Colision detectada");
} sino {
    mostrar("Sin colision en este frame");
}

mostrar("Activos en escena: " + escena.contar_activos());
mostrar("Paso del motor: " + motor.paso);
mostrar("Camara: (" + motor.camara.x + ", " + motor.camara.pos_y + ")");
mostrar("Prototipo de motor 2D listo para continuar");
