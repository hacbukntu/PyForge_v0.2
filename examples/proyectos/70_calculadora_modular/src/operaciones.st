// Funciones exportadas; las operaciones enteras conservan comprobación de desbordamiento.
// Suma dos operandos sin cambiar sus tipos.
funcion sumar(a: entero, b: entero) -> entero { retornar a+b; }
// Multiplica dos operandos i64.
funcion multiplicar(a: entero, b: entero) -> entero { retornar a*b; }
// Comprueba el divisor para ofrecer un error capturable.
funcion dividir(a: entero, b: entero) -> entero { si b == 0 { lanzar "Divisor cero"; } retornar a/b; }
