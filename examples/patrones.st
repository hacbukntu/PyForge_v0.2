// El valor se evalúa una vez. Se ejecuta el primer caso coincidente, sin continuar al siguiente.
variable estado = "listo";
coincidir estado {
    caso "pendiente" { mostrar("Todavía no"); }
    caso "listo" { mostrar("Puedes continuar"); }
    sino { mostrar("Estado desconocido"); }
}

funcion describir(numero: entero) -> cadena {
    coincidir numero {
        caso -1 { retornar "negativo"; }
        caso 0 { retornar "cero"; }
        sino { retornar "otro"; }
    }
}
mostrar(describir(0));
