importar "time";
importar "io";
importar "archivos";
importar "json";
// Espera sin consumir CPU activamente y mide duración monotónica, no hora de calendario.
variable segundos = entero(io.leer_linea("Duración en segundos (1..3600): "));
si segundos < 1 o segundos > 3600 { lanzar "Duración fuera de rango"; }
variable inicio = time.monotono();
time.dormir(segundos * 1000);
variable duracion = time.monotono() - inicio;
archivos.escribir_texto("sesion.json", json.objeto(["plan_segundos", "real_ms"], [json.de_entero(segundos), json.de_entero(duracion)]), falso, "utf-8");
mostrar("Sesión completada y registrada");
