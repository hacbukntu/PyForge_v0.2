// Paso 1: crear entidades. El constructor del módulo define tamaño 10 por 12.
importar "juego";
variable jugador = juego.crear_entidad("jugador"; 10; 70);
variable enemigo = juego.crear_entidad("enemigo"; 12; 70);
mostrar(jugador.colisiona_con(enemigo)); // True: sus rectángulos se solapan.

// Paso 2: mover y saltar. No hay renderizado oculto: son reglas comprobables.
jugador.mover(30; 0);
mostrar(jugador.colisiona_con(enemigo)); // False.
jugador.saltar();
mostrar(jugador.pos_y); // 63.

// Paso 3: conectar los mismos objetos a una escena y avanzar exactamente un frame.
variable escena = juego.crear_escena("tutorial");
escena.preparar_juego(jugador; enemigo);
variable motor = juego.crear_motor();
motor.iniciar(escena);
motor.frame();
mostrar(jugador.pos_y); // 64: la escena aplicó gravedad al objeto compartido.
mostrar(motor.paso); // 1.
motor.detener();
motor.frame();
mostrar(motor.paso); // Sigue siendo 1.
