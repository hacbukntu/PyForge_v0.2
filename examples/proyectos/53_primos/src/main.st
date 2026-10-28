// Criba de números primos
// Marca múltiplos para enumerar primos hasta un límite pequeño.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable limite = 30;
variable multiplo = 0;
variable primo: lista[booleano] = [];
para i en rango(0, limite+1) { agregar(primo,verdadero); }
primo[0] = falso; primo[1] = falso;
para p en rango(2, limite+1) {
    si primo[p] {
        mostrar(p);
        multiplo = p*p;
        mientras multiplo <= limite { primo[multiplo] = falso; multiplo += p; }
    }
}
