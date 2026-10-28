// Captura nativa de teclado para juegos ejecutados en una terminal Linux.
// Los códigos son enteros para evitar asignaciones y cadenas en el bucle crítico.

// Permite detectar IDE, tuberías o archivos, donde la captura inmediata no está disponible.
funcion es_terminal() -> booleano {
    retornar aux_es_terminal() == 1;
}

// Lee una tecla sin Enter. Devuelve -1 cuando vence la espera sin recibir entrada.
// La espera válida es 0..1000 ms; usar 0 permite controlar el ritmo con time.dormir.
funcion leer(espera_milisegundos: entero) -> entero {
    si espera_milisegundos < 0 o espera_milisegundos > 1000 {
        lanzar "teclado.leer: la espera debe estar entre 0 y 1000 milisegundos";
    }
    variable codigo: entero = aux_leer(espera_milisegundos);
    si codigo < -1 {
        lanzar "teclado.leer: stdin no es una terminal o falló termios";
    }
    retornar codigo;
}

// Constantes como funciones: facilitan comparar teclas sin números mágicos.
funcion ninguna() -> entero { retornar -1; }
funcion escape() -> entero { retornar 27; }
funcion espacio() -> entero { retornar 32; }
funcion enter() -> entero { retornar 10; }
funcion arriba() -> entero { retornar 1001; }
funcion abajo() -> entero { retornar 1002; }
funcion derecha() -> entero { retornar 1003; }
funcion izquierda() -> entero { retornar 1004; }
