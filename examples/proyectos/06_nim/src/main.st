// Nim: retirar fichas
// Juega contra una estrategia que deja múltiplos de cuatro.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

mostrar("Hay 7 fichas. Retira entre 1 y 3; gana quien retire la última.");
variable fichas = 7;
mientras fichas > 0 {
    variable quitar = entero(entrada(""));
    si quitar < 1 o quitar > 3 o quitar > fichas { lanzar "Movimiento inválido"; }
    fichas -= quitar;
    si fichas == 0 { mostrar("Ganaste"); romper; }
    variable bot = fichas % 4;
    si bot == 0 { bot = 1; }
    fichas -= bot;
    mostrar("Bot retira: " + bot);
    mostrar("Quedan: " + fichas);
    si fichas == 0 { mostrar("Ganó el bot"); }
}
