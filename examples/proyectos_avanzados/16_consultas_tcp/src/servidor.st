importar "red.tcp" como tcp;
importar "sistema";
importar "cifrado";
// Servidor de una consulta: publicar en otra interfaz exige permisos y diseño adicionales.
variable servidor = tcp.tcp_escuchar("127.0.0.1", entero(sistema.entorno("ST_PORT")));
intentar {
    mostrar("LISTO");
    variable cliente = tcp.tcp_aceptar(servidor);
    intentar {
        variable mensaje = "";
        mientras buscar(mensaje, "\n") < 0 {
            variable parte = tcp.recibir(cliente);
            si parte == "" { lanzar "Cliente desconectado"; }
            mensaje = mensaje + cifrado.desde_base64(parte);
            si longitud(mensaje) > 128 { lanzar "Petición excesiva"; }
        }
        si mensaje == "consulta\n" { tcp.enviar(cliente, "OK tickets=3\n"); }
        sino { tcp.enviar(cliente, "ERROR peticion\n"); }
    } finalmente { tcp.cerrar(cliente); }
} finalmente { tcp.cerrar(servidor); }
