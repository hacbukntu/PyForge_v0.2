importar "red.udp" como udp;
importar "json";
importar "cifrado";
importar "sistema";
// UDP no garantiza entrega: se valida origen y correlación de la confirmación.
variable local_puerto = entero(sistema.entorno("ST_LOCAL_PORT"));
variable remoto_puerto = entero(sistema.entorno("ST_PORT"));
variable conexion = udp.udp_abrir("127.0.0.1", local_puerto);
intentar {
    variable mensaje = json.objeto(["id", "sensor", "valor"], ["7", json.codificar("temperatura"), "24"]);
    udp.udp_enviar(conexion, "127.0.0.1", remoto_puerto, mensaje);
    variable paquete = udp.udp_recibir(conexion);
    si json.a_entero(json.obtener(paquete, "puerto")) != remoto_puerto { lanzar "Puerto de respuesta inesperado"; }
    si json.decodificar(json.obtener(paquete, "host")) != "127.0.0.1" { lanzar "Origen inesperado"; }
    variable datos = cifrado.desde_base64(json.decodificar(json.obtener(paquete, "datos_base64")));
    si json.a_entero(json.obtener(datos, "ack")) != 7 { lanzar "Confirmación de otro mensaje"; }
    mostrar("Telemetría confirmada: mensaje 7");
} finalmente { udp.cerrar(conexion); }
