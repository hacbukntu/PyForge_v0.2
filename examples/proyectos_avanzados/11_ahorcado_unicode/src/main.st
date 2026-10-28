importar "io";
importar "json";
// Construye una vista sin revelar letras pendientes; ST indexa caracteres Unicode.
funcion vista(palabra: cadena, usadas: cadena) -> cadena {
    variable resultado = "";
    para i en rango(0, longitud(palabra)) {
        si buscar(usadas, palabra[i]) >= 0 { resultado = resultado + palabra[i]; }
        sino { resultado = resultado + "_"; }
    }
    retornar resultado;
}
variable palabra = "cañon";
variable usadas = "";
variable vidas = 6;
variable intentos = 0;
mientras vidas > 0 y vista(palabra, usadas) != palabra y intentos < 20 {
    mostrar(vista(palabra, usadas));
    variable letra = io.leer_linea("Una letra: ");
    intentos = intentos + 1;
    si longitud(letra) != 1 { mostrar("Introduce exactamente un carácter"); continuar; }
    si buscar(usadas, letra) >= 0 { mostrar("Letra repetida"); continuar; }
    usadas = usadas + letra;
    si buscar(palabra, letra) < 0 { vidas = vidas - 1; }
}
mostrar("Palabra: " + palabra);
mostrar("Ganaste: " + cadena(vista(palabra, usadas) == palabra));
mostrar(json.objeto(["vidas"], [json.de_entero(vidas)]));
