importar "math";
importar "time";
importar "io";
importar "json";
// Rondas reproducibles: permite comparar puntuaciones sin presentar un PRNG como criptografía.
variable aciertos = 0;
variable inicio = time.monotono();
para ronda en rango(1, 4) {
    variable a = ronda * 3;
    variable b = ronda + 2;
    variable respuesta = entero(io.leer_linea(cadena(a) + " x " + b + " = "));
    si math.absoluto_entero(respuesta - a*b) == 0 { aciertos = aciertos + 1; mostrar("Correcto"); }
    sino { mostrar("Respuesta: " + a*b); }
}
mostrar("Puntuación: " + aciertos + "/3");
mostrar(json.objeto(["aciertos", "rondas"], [json.de_entero(aciertos), "3"]));
si time.monotono() < inicio { lanzar "El reloj monotónico retrocedió"; }
