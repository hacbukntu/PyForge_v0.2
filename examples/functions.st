// Los parámetros tienen tipos; las llamadas usan el mismo generador nativo.
funcion sumar(a: entero, b: entero) -> entero {
    retornar a + b;
}
funcion factorial(n: entero) -> entero {
    si n <= 1 { retornar 1; }
    retornar n * factorial(n - 1);
}
mostrar(sumar(20, 22));
mostrar(factorial(5));
