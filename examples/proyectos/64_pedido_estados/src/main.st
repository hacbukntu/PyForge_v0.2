// Estado de un pedido
// Aplica una máquina de estados para pagar, enviar y entregar pedidos.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

clase Pedido {
    variable codigo: cadena;
    variable estado: cadena;
    // Solo acepta la transición desde el estado esperado.
    funcion avanzar(esperado: cadena, destino: cadena) -> nada {
        si yo.estado != esperado { lanzar "Transición inválida"; }
        yo.estado = destino;
    }
}
variable pedido = nuevo Pedido("P-001","creado");
pedido.avanzar("creado","pagado");
pedido.avanzar("pagado","enviado");
intentar { pedido.avanzar("creado","cancelado"); } capturar error { mostrar(error); }
pedido.avanzar("enviado","entregado");
mostrar(pedido.codigo + ": " + pedido.estado);
