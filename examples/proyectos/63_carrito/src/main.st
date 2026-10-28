// Carrito de compra
// Encapsula cantidades y calcula un subtotal al agregar productos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

clase Carrito {
    variable unidades: entero;
    variable subtotal: entero;
    // No almacena objetos anidados: acumula valores escalares comprobados.
    funcion incluir(precio: entero, cantidad: entero) -> nada {
        si precio < 0 o cantidad <= 0 { lanzar "Línea inválida"; }
        yo.unidades = yo.unidades + cantidad;
        yo.subtotal = yo.subtotal + precio*cantidad;
    }
}
variable carrito = nuevo Carrito(0,0);
carrito.incluir(500,2); carrito.incluir(1200,1);
mostrar("Unidades: " + carrito.unidades);
mostrar("Subtotal: " + carrito.subtotal);
