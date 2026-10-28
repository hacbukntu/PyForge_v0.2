importar "red.ssh" como ssh;
importar "red.sftp" como sftp;
importar "red.ftp" como ftp;
importar "sistema";
importar "archivos";
importar "io";
// Solo laboratorio local; la contraseña se lee mediante su nombre de variable autorizada.
variable modo = io.leer_linea("Protocolo sftp / ftps: ");
variable usuario = sistema.entorno("ST_USER");
si usuario == "" { lanzar "Configura ST_USER y ST_PASSWORD fuera del código"; }
variable puerto = entero(sistema.entorno("ST_PORT"));
// Un único identificador fuera de las ramas mantiene el ámbito estático del compilador.
variable conexion = -1;
si modo == "sftp" {
    conexion = ssh.ssh_conectar("localhost", puerto, usuario, "datos/known_hosts", "ST_PASSWORD");
    intentar {
        mostrar(sftp.sftp_listar(conexion, "."));
        sftp.sftp_subir(conexion, "datos/entrega.txt", "demo-st.txt");
        sftp.sftp_descargar(conexion, "demo-st.txt", "descarga.txt");
        si archivos.leer("descarga.txt", "utf-8") != archivos.leer("datos/entrega.txt", "utf-8") { lanzar "Contenido SFTP distinto"; }
        mostrar("Entrega SFTP verificada");
    } finalmente { ssh.cerrar(conexion); }
} sino_si modo == "ftps" {
    conexion = ftp.ftps_conectar("localhost", puerto, usuario, "ST_PASSWORD");
    intentar {
        mostrar(ftp.ftp_listar(conexion, "."));
        ftp.ftp_subir(conexion, "datos/entrega.txt", "demo-st.txt");
        // STOU asigna el nombre remoto; esta API aún no lo devuelve, por eso no se inventa.
        mostrar("Subida FTPS aceptada; nombre único asignado por el servidor");
    } finalmente { ftp.cerrar(conexion); }
} sino { lanzar "Elige sftp o ftps"; }
