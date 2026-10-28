// Pong en ST: base mínima para entender el patrón de juego.
// Sin romper el compilador ni la compatibilidad con Python.

clase Pelota {
    variable x: entero;
    variable pos_y: entero;
    variable vel_x: entero;
    variable vel_y: entero;
    variable radio: entero;
}

implementar Pelota {
    funcion mover() -> nada {
        yo.x = yo.x + yo.vel_x;
        yo.pos_y = yo.pos_y + yo.vel_y;
    }

    funcion rebotar_horizontal() -> nada {
        yo.vel_x = -yo.vel_x;
    }

    funcion rebotar_vertical() -> nada {
        yo.vel_y = -yo.vel_y;
    }
}

clase Raqueta {
    variable x: entero;
    variable pos_y: entero;
    variable ancho: entero;
    variable alto: entero;
}

implementar Raqueta {
    funcion mover(arriba: booleano) -> nada {
        si arriba {
            yo.pos_y = yo.pos_y - 4;
        } sino {
            yo.pos_y = yo.pos_y + 4;
        }
    }
}

funcion crear_pelota() -> Pelota {
    retornar nuevo Pelota(40, 30, 2, 2, 4);
}

funcion crear_raqueta(x_inicio: entero, y_inicio: entero) -> Raqueta {
    retornar nuevo Raqueta(x_inicio, y_inicio, 12, 50);
}

funcion actualizar_pong(pelota: Pelota, raq_izquierda: Raqueta, raq_derecha: Raqueta) -> nada {
    pelota.mover();

    si pelota.x <= 0 o pelota.x >= 100 {
        pelota.rebotar_horizontal();
    }

    si pelota.pos_y <= 0 o pelota.pos_y >= 60 {
        pelota.rebotar_vertical();
    }

    si pelota.x < raq_izquierda.x + raq_izquierda.ancho y pelota.pos_y >= raq_izquierda.pos_y y pelota.pos_y <= raq_izquierda.pos_y + raq_izquierda.alto {
        pelota.rebotar_horizontal();
    }

    si pelota.x > raq_derecha.x y pelota.pos_y >= raq_derecha.pos_y y pelota.pos_y <= raq_derecha.pos_y + raq_derecha.alto {
        pelota.rebotar_horizontal();
    }
}

variable pelota: Pelota = crear_pelota();
variable izquierda: Raqueta = crear_raqueta(2, 20);
variable derecha: Raqueta = crear_raqueta(96, 20);

mostrar("PONG en ST");
mostrar("Pelota: (" + pelota.x + ", " + pelota.pos_y + ")");
actualizar_pong(pelota, izquierda, derecha);
mostrar("Despues del update: (" + pelota.x + ", " + pelota.pos_y + ")");
