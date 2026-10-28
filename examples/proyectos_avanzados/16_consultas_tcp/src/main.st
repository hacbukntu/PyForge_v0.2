importar "red.tcp" como tcp;
importar "red.socket" como sockets;
importar "sistema";
importar "cifrado";
// Protocolo de laboratorio ASCII terminado en LF: TCP puede dividir la respuesta.
funcion recibir_linea(conexion: entero) -> cadena {
    variable texto = "";
    mientras buscar(texto, "\n") < 0 {
        variable bloque = tcp.recibir(conexion);
        si bloque == "" { lanzar "Conexión cerrada antes del delimitador"; }
        texto = texto + cifrado.desde_base64(bloque);
        si longitud(texto) > 1024 { lanzar "Respuesta excesiva"; }
    }
    retornar texto;
}
variable puerto = entero(sistema.entorno("ST_PORT"));
variable conexion = tcp.tcp_conectar("127.0.0.1", puerto);
intentar {
    tcp.enviar(conexion, "consulta\n");
    variable respuesta = recibir_linea(conexion);
    si respuesta != "OK tickets=3\n" { lanzar "Respuesta inesperada del servidor"; }
    mostrar("Consulta TCP verificada");
    // Consulta de socket sin revelar puertos efímeros en el resultado reproducible.
    variable direccion = sockets.direccion(conexion);
} finalmente { tcp.cerrar(conexion); }
