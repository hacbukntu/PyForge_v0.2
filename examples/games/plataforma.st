// Plataforma 2D simple en ST.
// Sirve como base para crear un prototipo de salto, gravedad y movimiento.

clase Jugador {
    variable x: entero;
    variable pos_y: entero;
    variable ancho: entero;
    variable alto: entero;
    variable vy: entero;
    variable saltando: booleano;
}

implementar Jugador {
    funcion aplicar_gravedad() -> nada {
        si yo.saltando {
            yo.pos_y = yo.pos_y + yo.vy;
            yo.vy = yo.vy + 1;
            si yo.pos_y >= 40 {
                yo.pos_y = 40;
                yo.vy = 0;
                yo.saltando = falso;
            }
        }
    }

    funcion saltar() -> nada {
        si no yo.saltando {
            yo.vy = -8;
            yo.saltando = verdadero;
        }
    }
}

funcion crear_jugador() -> Jugador {
    retornar nuevo Jugador(10, 40, 5, 8, 0, falso);
}

funcion actualizar_plataforma(jugador: Jugador) -> nada {
    jugador.aplicar_gravedad();
    si jugador.x < 0 { jugador.x = 0; }
    si jugador.x > 90 { jugador.x = 90; }
}

variable jugador: Jugador = crear_jugador();
variable pasos: entero = 0;

mostrar("PLATAFORMA en ST");
mostrar("Jugador en: (" + jugador.x + ", " + jugador.pos_y + ")");
jugador.saltar();
actualizar_plataforma(jugador);
mostrar("Despues del salto: (" + jugador.x + ", " + jugador.pos_y + ")");

mientras pasos < 3 {
    jugador.x = jugador.x + 5;
    actualizar_plataforma(jugador);
    pasos = pasos + 1;
}

mostrar("Fin de la prueba de plataforma");
