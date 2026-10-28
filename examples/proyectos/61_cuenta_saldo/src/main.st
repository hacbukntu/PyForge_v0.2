// Cuenta con saldo controlado
// Modela depósitos y retiros con validación de saldo y errores capturables.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

clase Cuenta {
    variable titular: cadena;
    variable saldo: entero;
    // Un depósito válido conserva el invariante de saldo no negativo.
    funcion depositar(importe: entero) -> nada {
        si importe <= 0 { lanzar "Depósito inválido"; }
        yo.saldo = yo.saldo + importe;
    }
    // Rechaza retirar más saldo del disponible antes de modificar el objeto.
    funcion retirar(importe: entero) -> nada {
        si importe <= 0 o importe > yo.saldo { lanzar "Saldo insuficiente"; }
        yo.saldo = yo.saldo - importe;
    }
}
variable cuenta = nuevo Cuenta("Ana",1000);
cuenta.depositar(500); cuenta.retirar(200);
intentar { cuenta.retirar(2000); } capturar error { mostrar(error); }
mostrar("Saldo: " + cuenta.saldo);
