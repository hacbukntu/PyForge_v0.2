// Este módulo publica funciones en el espacio de nombres del programa.
funcion cuadrado(n: entero) -> entero { retornar n * n; }
funcion sumar_lista(valores: lista) -> entero {
    variable total = 0;
    para i en rango(longitud(valores)) { total += valores[i]; }
    retornar total;
}
