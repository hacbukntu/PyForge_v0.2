// Equivalente ST del cálculo de examples/benchmark.py.
funcion sumar_hasta(n: entero) -> entero {
    variable total: entero = 0;
    para i en rango(n) { 
        total += i; 
    }
    ///mostrar(i);
    retornar total;
}
mostrar(sumar_hasta(100000000));
