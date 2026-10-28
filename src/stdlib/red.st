// API opcional: ejecutable ST nativo con auxiliar Python 3.11.
// Permisos en PyForge.toml; consulta docs/INTEGRACIONES.md antes de utilizarla.
importar "integraciones";
importar "json";

// Conecta TCP a un host autorizado con timeout de cinco segundos.
funcion tcp_conectar(host: cadena, puerto: entero) -> entero {
    retornar json.a_entero(integraciones.pedir("red.tcp_conectar", "[" + json.codificar(host) + "," + json.de_entero(puerto) + "]"));
}

// Escucha TCP IPv4 con permiso explícito.
funcion tcp_escuchar(host: cadena, puerto: entero) -> entero {
    retornar json.a_entero(integraciones.pedir("red.tcp_escuchar", "[" + json.codificar(host) + "," + json.de_entero(puerto) + "]"));
}

// Acepta una conexión con timeout y devuelve otro identificador.
funcion tcp_aceptar(servidor: entero) -> entero {
    retornar json.a_entero(integraciones.pedir("red.tcp_aceptar", "[" + json.de_entero(servidor) + "]"));
}

// Envía texto UTF-8 completo por TCP/TLS; devuelve bytes enviados.
funcion enviar(conexion: entero, texto: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("red.enviar", "[" + json.de_entero(conexion) + "," + json.codificar(texto) + "]"));
}

// Recibe hasta 1024 bytes en base64; TCP puede fragmentar mensajes.
funcion recibir(conexion: entero) -> cadena {
    retornar json.decodificar(integraciones.pedir("red.recibir", "[" + json.de_entero(conexion) + "]"));
}

// Cierra el recurso de red y prohíbe volver a utilizarlo.
funcion cerrar(conexion: entero) -> booleano {
    retornar json.a_booleano(integraciones.pedir("red.cerrar", "[" + json.de_entero(conexion) + "]"));
}

// Abre UDP IPv4 local; sin garantía de entrega ni cifrado.
funcion udp_abrir(host: cadena, puerto: entero) -> entero {
    retornar json.a_entero(integraciones.pedir("red.udp_abrir", "[" + json.codificar(host) + "," + json.de_entero(puerto) + "]"));
}

// Envía un datagrama al host autorizado.
funcion udp_enviar(conexion: entero, host: cadena, puerto: entero, texto: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("red.udp_enviar", "[" + json.de_entero(conexion) + "," + json.codificar(host) + "," + json.de_entero(puerto) + "," + json.codificar(texto) + "]"));
}

// Devuelve origen y datos base64; rechaza datagramas truncados.
funcion udp_recibir(conexion: entero) -> cadena {
    retornar integraciones.pedir("red.udp_recibir", "[" + json.de_entero(conexion) + "]");
}

// TLS mínimo 1.2 con CA/nombre verificados; ca vacía usa almacén del sistema.
funcion tls_conectar(host: cadena, puerto: entero, ca: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("red.tls_conectar", "[" + json.codificar(host) + "," + json.de_entero(puerto) + "," + json.codificar(ca) + "]"));
}

// SSH verifica known_hosts; contraseña desde variable de entorno autorizada.
funcion ssh_conectar(host: cadena, puerto: entero, usuario: cadena, hosts_conocidos: cadena, variable_clave: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("red.ssh_conectar", "[" + json.codificar(host) + "," + json.de_entero(puerto) + "," + json.codificar(usuario) + "," + json.codificar(hosts_conocidos) + "," + json.codificar(variable_clave) + "]"));
}

// Lista archivos remotos a través de SSH autenticado.
funcion sftp_listar(conexion: entero, ruta: cadena) -> cadena {
    retornar integraciones.pedir("red.sftp_listar", "[" + json.de_entero(conexion) + "," + json.codificar(ruta) + "]");
}

// Sube un archivo sin reemplazar uno remoto existente.
funcion sftp_subir(conexion: entero, local: cadena, remoto: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("red.sftp_subir", "[" + json.de_entero(conexion) + "," + json.codificar(local) + "," + json.codificar(remoto) + "]"));
}

// Descarga como máximo 8 MiB sin sobrescribir el destino.
funcion sftp_descargar(conexion: entero, remoto: cadena, local: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("red.sftp_descargar", "[" + json.de_entero(conexion) + "," + json.codificar(remoto) + "," + json.codificar(local) + "]"));
}

// FTPS explícito con canal de datos cifrado y certificados verificados.
funcion ftps_conectar(host: cadena, puerto: entero, usuario: cadena, variable_clave: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("red.ftps_conectar", "[" + json.codificar(host) + "," + json.de_entero(puerto) + "," + json.codificar(usuario) + "," + json.codificar(variable_clave) + "]"));
}

// Lista sobre una conexión FTPS; rechaza CR/LF en rutas.
funcion ftp_listar(conexion: entero, ruta: cadena) -> cadena {
    retornar integraciones.pedir("red.ftp_listar", "[" + json.de_entero(conexion) + "," + json.codificar(ruta) + "]");
}

// Sube por FTPS usando STOU; servidor asigna nombre único.
funcion ftp_subir(conexion: entero, local: cadena, remoto: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("red.ftp_subir", "[" + json.de_entero(conexion) + "," + json.codificar(local) + "," + json.codificar(remoto) + "]"));
}

// Descarga por FTPS de forma acotada y exclusiva.
funcion ftp_descargar(conexion: entero, remoto: cadena, local: cadena) -> booleano {
    retornar json.a_booleano(integraciones.pedir("red.ftp_descargar", "[" + json.de_entero(conexion) + "," + json.codificar(remoto) + "," + json.codificar(local) + "]"));
}
// Devuelve host/puerto local y si el recurso usa TLS, como JSON.
funcion direccion(conexion: entero) -> cadena {
    retornar integraciones.pedir("red.direccion", "[" + json.de_entero(conexion) + "]");
}
// Envía bytes arbitrarios representados mediante base64 estricto.
funcion enviar_bytes(conexion: entero, contenido: cadena) -> entero {
    retornar json.a_entero(integraciones.pedir("red.enviar_bytes", "[" + json.de_entero(conexion) + "," + json.codificar(contenido) + "]"));
}
