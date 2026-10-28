// Sensor con umbral
// Valida lecturas y emite un estado según límites configurables.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

estructura Sensor {
    variable nombre: cadena;
    variable minimo: entero;
    variable maximo: entero;
    // Clasifica una muestra sin modificarla ni conectarse a hardware.
    funcion estado(valor: entero) -> cadena {
        si valor < yo.minimo { retornar "bajo"; }
        si valor > yo.maximo { retornar "alto"; }
        retornar "normal";
    }
}
variable sensor = nuevo Sensor("Humedad",30,70);
variable lecturas = [20,50,80];
para i en rango(0, longitud(lecturas)) { mostrar(sensor.nombre + ": " + sensor.estado(lecturas[i])); }
