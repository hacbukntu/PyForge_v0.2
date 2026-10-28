importar "red.ssl" como tls;
importar "cifrado";
importar "sistema";
// Laboratorio HTTPS ASCII: CA y nombre localhost se verifican obligatoriamente.
variable conexion = tls.tls_conectar("localhost", entero(sistema.entorno("ST_PORT")), "datos/ca.pem");
intentar {
    tls.enviar(conexion, "GET /estado HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    variable respuesta = "";
    mientras buscar(respuesta, "\r\n\r\n") < 0 {
        variable bloque = tls.recibir(conexion);
        si bloque == "" { lanzar "Respuesta HTTP incompleta"; }
        respuesta = respuesta + cifrado.desde_base64(bloque);
        si longitud(respuesta) > 2048 { lanzar "Cabeceras demasiado grandes para este ejemplo"; }
    }
    si buscar(respuesta, "HTTP/1.1 200 ") != 0 { lanzar "Servicio no disponible"; }
    mostrar("HTTPS: certificado y estado 200 verificados");
} finalmente { tls.cerrar(conexion); }
