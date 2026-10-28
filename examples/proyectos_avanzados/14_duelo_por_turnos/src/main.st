importar "io";
importar "json";
// Cada instancia conserva sus propios puntos de vida y pociones.
clase Heroe {
    variable vida: entero;
    variable pociones: entero;
    // Reduce daño sin permitir vida negativa.
    funcion recibir(danio: entero) -> nada {
        yo.vida = yo.vida - danio;
        si yo.vida < 0 { yo.vida = 0; }
    }
    // Una poción no permite superar el máximo inicial.
    funcion curar() -> nada {
        si yo.pociones > 0 {
            yo.pociones = yo.pociones - 1;
            yo.vida = yo.vida + 8;
            si yo.vida > 30 { yo.vida = 30; }
        }
    }
}
variable jugador = nuevo Heroe(30, 2);
variable enemigo = nuevo Heroe(24, 0);
variable ronda = 0;
mientras jugador.vida > 0 y enemigo.vida > 0 y ronda < 10 {
    variable accion = io.leer_linea("a atacar / c curar / q salir: ");
    si accion == "q" o accion == "" { romper; }
    si accion != "a" y accion != "c" { mostrar("Acción inválida"); continuar; }
    ronda = ronda + 1;
    si accion == "a" { enemigo.recibir(8); } sino { jugador.curar(); }
    si enemigo.vida > 0 { jugador.recibir(5); }
    mostrar("Tu vida: " + jugador.vida + "; rival: " + enemigo.vida);
}
mostrar("Victoria del héroe: " + cadena(enemigo.vida == 0));
mostrar(json.objeto(["vida", "pociones"], [json.de_entero(jugador.vida), json.de_entero(jugador.pociones)]));
