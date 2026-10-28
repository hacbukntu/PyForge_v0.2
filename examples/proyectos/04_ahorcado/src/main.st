// Ahorcado
// Descubre una palabra conservando letras acertadas y controlando seis fallos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable palabra = "rust";
variable descubiertas = [falso,falso,falso,falso];
variable errores = 0;
variable pendientes = longitud(palabra);
mostrar("Descubre cuatro letras; seis fallos disponibles.");
mientras pendientes > 0 y errores < 6 {
    variable letra = entrada("");
    si longitud(letra) != 1 { lanzar "Escribe una sola letra"; }
    variable existe = falso;
    para i en rango(0, longitud(palabra)) {
        si palabra[i] == letra {
            existe = verdadero;
            si no descubiertas[i] { descubiertas[i] = verdadero; pendientes -= 1; }
        }
    }
    si no existe { errores += 1; }
    variable vista = "";
    para i en rango(0, longitud(palabra)) { si descubiertas[i] { vista += palabra[i]; } sino { vista += "_"; } }
    mostrar(vista);
}
si pendientes == 0 { mostrar("Ganaste"); } sino { mostrar("Perdiste: " + palabra); }
