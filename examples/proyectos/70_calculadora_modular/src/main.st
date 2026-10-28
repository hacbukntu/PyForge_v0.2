// Calculadora modular
// Separa reglas aritméticas del programa de consola mediante importaciones locales.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "operaciones.st" como calculo;
mostrar("Dos enteros:");
variable a = entero(entrada(""));
variable b = entero(entrada(""));
mostrar("Suma: " + calculo.sumar(a,b));
mostrar("Producto: " + calculo.multiplicar(a,b));
intentar { mostrar("Cociente: " + calculo.dividir(a,b)); } capturar error { mostrar(error); }
