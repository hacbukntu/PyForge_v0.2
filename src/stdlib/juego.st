// Módulo seguro de juego para ST.
// No modifica el compilador ni depende de Python; solo añade un runtime mínimo
// para entidades, escenas y motor 2D compatible con la base actual.


/*
    hola
    que tal 
*/


"""

hola
como 
que tal

"""
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

clase Motor2D {
    variable escena_actual: Escena2D;
    variable ejecutando: booleano;
    variable paso: entero;
}

implementar Motor2D {
    funcion iniciar(escena: Escena2D) -> nada {
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

funcion crear_entidad(nombre: cadena, x_inicial: entero, y_inicial: entero) -> Entidad2D {
    retornar nuevo Entidad2D(nombre, x_inicial, y_inicial, 10, 12, verdadero);
}

funcion crear_escena(nombre_escena: cadena) -> Escena2D {
    variable jugador: Entidad2D = crear_entidad("jugador", 10, 18);
    variable enemigo: Entidad2D = crear_entidad("enemigo", 40, 22);
    retornar nuevo Escena2D(nombre_escena, "noche", jugador, enemigo, 3, "cargando");
}

funcion crear_motor() -> Motor2D {
    variable escena: Escena2D = crear_escena("nivel_1");
    retornar nuevo Motor2D(escena, falso, 0);
}

// Culebrita 2D: tablero de 20 por 20. Cada celda se codifica fila * 20 + columna.
// Dirección: 0 derecha, 1 abajo, 2 izquierda, 3 arriba. El motor no dibuja ni lee teclas.
funcion avanzar_serpiente(cuerpo: lista; direccion: entero; comida: entero) -> lista {
    variable siguiente: lista = [];
    si longitud(cuerpo) == 0 o direccion < 0 o direccion > 3 { retornar siguiente; }
    variable cabeza = cuerpo[0];
    variable x = cabeza % 20;
    variable fila = cabeza / 20;
    si direccion == 0 { x = x + 1; }
    si direccion == 1 { fila = fila + 1; }
    si direccion == 2 { x = x - 1; }
    si direccion == 3 { fila = fila - 1; }
    si x < 0 o x >= 20 o fila < 0 o fila >= 20 { retornar siguiente; }
    variable nueva = fila * 20 + x;
    variable limite = longitud(cuerpo) - 1;
    si nueva == comida { limite = longitud(cuerpo); }
    para i en rango(limite) {
        si cuerpo[i] == nueva { retornar siguiente; }
    }
    agregar(siguiente; nueva);
    para i en rango(limite) { agregar(siguiente; cuerpo[i]); }
    retornar siguiente;
}

// Selecciona una celda libre recorriendo el tablero; resultado reproducible para probar partidas.
funcion comida_libre(cuerpo: lista; semilla: entero) -> entero {
    para paso en rango(400) {
        variable celda = (semilla + paso) % 400;
        variable ocupada = falso;
        para i en rango(longitud(cuerpo)) { si cuerpo[i] == celda { ocupada = verdadero; } }
        si no ocupada { retornar celda; }
    }
    retornar -1;
}
