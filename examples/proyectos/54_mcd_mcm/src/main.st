// Máximo común divisor y mínimo común múltiplo
// Utiliza el algoritmo de Euclides con comprobación de entradas positivas.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

funcion mcd(a: entero, b: entero) -> entero {
    si a <= 0 o b <= 0 { lanzar "Se requieren enteros positivos"; }
    variable x = a;
    variable z = b;
    mientras z != 0 { variable resto = x % z; x = z; z = resto; }
    retornar x;
}
variable a = 84;
variable b = 30;
variable divisor = mcd(a,b);
mostrar("MCD: " + divisor);
mostrar("MCM: " + (a/divisor*b));
