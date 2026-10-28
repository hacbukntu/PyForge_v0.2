// Matemática con tipos explícitos: las funciones flotantes reciben flotante, no entero.
// Constantes como funciones: los módulos actuales exportan funciones, no variables globales.
funcion pi() -> flotante { retornar 3.141592653589793; }
funcion euler() -> flotante { retornar 2.718281828459045; }
// Valor absoluto flotante; convierte también -0.0 en +0.0.
funcion absoluto(valor: flotante) -> flotante {
    si valor == 0.0 { retornar 0.0; }
    si valor < 0.0 { retornar -valor; }
    retornar valor;
}
// El mínimo entero no tiene opuesto i64: el runtime conserva su control de desbordamiento.
funcion absoluto_entero(valor: entero) -> entero {
    si valor < 0 { retornar -valor; }
    retornar valor;
}
// Selecciona el menor de dos flotantes.
funcion minimo(a: flotante, b: flotante) -> flotante { si a < b { retornar a; } retornar b; }
// Selecciona el mayor de dos flotantes.
funcion maximo(a: flotante, b: flotante) -> flotante { si a > b { retornar a; } retornar b; }
// La comprobación de dominio produce una excepción ST capturable antes de usar sqrtsd.
funcion raiz(valor: flotante) -> flotante {
    si valor < 0.0 { lanzar "math.raiz: el argumento debe ser no negativo"; }
    retornar aux_raiz(valor);
}
// Exponenciación por cuadrados, O(log |exponente|); admite exponentes enteros negativos.
funcion potencia(base: flotante, exponente: entero) -> flotante {
    variable factor: flotante = base;
    variable n: entero = exponente;
    variable resultado: flotante = 1.0;
    si n < 0 {
        si base == 0.0 { lanzar "math.potencia: cero no admite exponente negativo"; }
        factor = 1.0 / factor;
        // Evita negar el mínimo i64: se retira primero una unidad del exponente.
        resultado = factor;
        n = -(n + 1);
    }
    mientras n > 0 {
        si n % 2 == 1 { resultado = resultado * factor; }
        n = n / 2;
        si n > 0 { factor = factor * factor; }
    }
    retornar resultado;
}
