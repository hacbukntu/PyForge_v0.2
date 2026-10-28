// Breakout en ST: una versión simple y entendible del patrón "bloques + pelota + raqueta".

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

    funcion rebotar() -> nada {
        yo.vel_x = -yo.vel_x;
        yo.vel_y = -yo.vel_y;
    }
}

clase Raqueta {
    variable x: entero;
    variable pos_y: entero;
    variable ancho: entero;
    variable alto: entero;
}

funcion crear_pelota() -> Pelota {
    retornar nuevo Pelota(20, 20, 2, 2, 3);
}

funcion crear_raqueta() -> Raqueta {
    retornar nuevo Raqueta(40, 50, 20, 4);
}

funcion actualizar_breakout(pelota: Pelota, raqueta: Raqueta) -> nada {
    pelota.mover();

    si pelota.x <= 0 o pelota.x >= 100 {
        pelota.vel_x = -pelota.vel_x;
    }

    si pelota.pos_y <= 0 {
        pelota.vel_y = -pelota.vel_y;
    }

    si pelota.pos_y >= 60 {
        pelota.vel_y = -pelota.vel_y;
    }

    si pelota.x >= raqueta.x y pelota.x <= raqueta.x + raqueta.ancho y pelota.pos_y >= raqueta.pos_y y pelota.pos_y <= raqueta.pos_y + raqueta.alto {
        pelota.vel_y = -pelota.vel_y;
    }
}

variable pelota: Pelota = crear_pelota();
variable raqueta: Raqueta = crear_raqueta();

mostrar("BREAKOUT en ST");
mostrar("Inicio: (" + pelota.x + ", " + pelota.pos_y + ")");
actualizar_breakout(pelota, raqueta);
mostrar("Despues del update: (" + pelota.x + ", " + pelota.pos_y + ")");
