// Declara las variables antes de los bloques para evitar la limitación de locales en if.
variable total: entero = 0;
para i en rango(1, 11) {
    si i == 3 { continuar; }
    si i == 8 { romper; }
    total += i;
}
mostrar(total);
variable n = 3;
mientras n > 0 {
    mostrar(n);
    n -= 1;
}
si total == 25 {
    mostrar("correcto");
} sino si total == 0 {
    mostrar("vacio");
} sino {
    mostrar("otro");
}
mostrar(no falso y verdadero);
